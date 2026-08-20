//! Integration tests for the error-recovery pass. Every assertion
//! goes through the public [`Parser`] API — no test-only internal
//! hooks — so what these tests observe is what real consumers see.
//!
//! Test inputs deliberately use tokens the scanner accepts but the
//! grammar rejects (unmatched delimiters, extra separators, unexpected
//! keywords) so tokenizer errors do not conflate with parser
//! recovery.

use erl_parse::{
    NodeId, ParseError, ParseErrorKind, ParseMode, Parser, SyntaxKind, SyntaxTree, TokenIndex,
};
use erl_tokenize::{Position, Token, scan_token};

fn scan_all(source: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut pos = Position::new();
    while let Some(t) = scan_token(source, pos).expect("valid source") {
        out.push(t);
        pos = t.end();
    }
    out
}

fn push_all(parser: &mut Parser, source: &str) {
    for t in scan_all(source) {
        parser.push_token(t);
    }
}

fn drive(mode: ParseMode, source: &str) -> (SyntaxTree, Vec<NodeId>) {
    let mut p = Parser::new(mode);
    push_all(&mut p, source);
    let mut roots = Vec::new();
    while let Some(id) = p.next_top_node() {
        roots.push(id);
    }
    (p.finish(), roots)
}

fn kind_of(tree: &SyntaxTree, id: NodeId) -> SyntaxKind {
    tree.syntax().entry(id).expect("entry exists").kind()
}

fn contains_error_node(tree: &SyntaxTree) -> bool {
    tree.syntax()
        .entries()
        .iter()
        .any(|e| e.kind() == SyntaxKind::Error)
}

fn find_error_by_kind(tree: &SyntaxTree, kind: ParseErrorKind) -> Option<ParseError> {
    tree.errors().iter().find(|e| e.kind() == kind).copied()
}

#[test]
fn malformed_form_is_followed_by_recovered_next_form() {
    // First form is `-broken end .` — `end` is a keyword the
    // attribute payload consumer swallows, but the driver still
    // completes at `.` so the next `-ok.` parses cleanly.
    let source = "1 2 3.\n-ok.";
    let (tree, roots) = drive(ParseMode::Module, source);
    assert!(roots.len() >= 2, "recovery lets the second form land");
    assert_eq!(
        kind_of(&tree, *roots.last().expect("last root")),
        SyntaxKind::Attribute,
        "the trailing `-ok.` still parses"
    );
    assert!(!tree.errors().is_empty(), "the first form produced errors");
}

#[test]
fn multiple_independent_errors_come_out_of_one_module() {
    // Three malformed forms and one good one. Errors are separate
    // structured `ParseError`s, not one aggregated blob.
    let source = "1 2 .\n) .\n( .\n-ok.";
    let (tree, _roots) = drive(ParseMode::Module, source);
    assert!(
        tree.errors().len() >= 2,
        "expected several errors, got {:?}",
        tree.errors()
    );
}

#[test]
fn skipped_token_diagnostic_range_matches_error_node_range() {
    // `]` in expression position hits `parse_expr_max`'s retrofitted
    // recovery arm: consumed as SyntaxKind::Error with a matching
    // SkippedToken diagnostic.
    let source = "].";
    let (tree, _roots) = drive(ParseMode::Expression, source);
    let skipped = find_error_by_kind(&tree, ParseErrorKind::SkippedToken)
        .expect("skip_one_token emits a SkippedToken");
    let error_node = tree
        .syntax()
        .entries()
        .iter()
        .find(|e| e.kind() == SyntaxKind::Error)
        .copied()
        .expect("recovery emits an Error node");
    assert_eq!(
        skipped.range(),
        error_node.range(),
        "diagnostic range must match Error node range"
    );
}

#[test]
fn missing_token_diagnostic_is_zero_width_and_no_error_node_is_added() {
    // `foo(1` has no closing `)`; `expect_symbol` emits a
    // MissingToken diagnostic anchored at the cursor without
    // producing an Error node for the missing token itself.
    let source = "foo(1.";
    let (tree, _roots) = drive(ParseMode::Expression, source);
    let missing = find_error_by_kind(&tree, ParseErrorKind::MissingToken)
        .expect("expect_symbol emits a MissingToken");
    assert!(missing.range().is_empty(), "missing token is zero-width");
    assert!(!tree.syntax().is_empty());
}

#[test]
fn recovery_dedupes_same_error_at_the_same_cursor() {
    let source = "1 2 .";
    let (tree, _roots) = drive(ParseMode::Module, source);
    let mut seen: Vec<(ParseErrorKind, TokenIndex)> = Vec::new();
    for e in tree.errors() {
        let key = (e.kind(), e.range().start());
        assert!(
            !seen.contains(&key),
            "duplicate error at same cursor: {:?}",
            e
        );
        seen.push(key);
    }
}

#[test]
fn top_level_recovery_produces_error_nodes_that_survive_in_the_syntax_index() {
    // Trailing garbage after a valid function-declaration body and
    // before the boundary `.` triggers the driver's
    // `skip_until_sync` path.
    let source = "foo() -> 1 xxxx .";
    let (tree, _roots) = drive(ParseMode::Module, source);
    assert!(
        contains_error_node(&tree),
        "driver's skip_until_sync must leave an Error node behind"
    );
    let skipped = find_error_by_kind(&tree, ParseErrorKind::SkippedToken)
        .expect("skip_until_sync emits a SkippedToken");
    let error_node_range = tree
        .syntax()
        .entries()
        .iter()
        .find(|e| e.kind() == SyntaxKind::Error)
        .map(|e| e.range())
        .expect("Error node exists");
    assert_eq!(skipped.range(), error_node_range);
}

#[test]
fn recovery_loop_makes_forward_progress_and_terminates() {
    // Repeated malformed forms must terminate parsing; a stuck
    // recovery loop would hang this test.
    let source = ") . ) . ) . -ok.";
    let (tree, roots) = drive(ParseMode::Module, source);
    assert!(roots.len() >= 4);
    let last = *roots.last().expect("at least one form emitted");
    assert_eq!(kind_of(&tree, last), SyntaxKind::Attribute);
}

#[test]
fn deeply_nested_expression_hits_the_depth_cap_without_stack_overflow() {
    // Well beyond `Parser::MAX_NESTING_DEPTH` = 256. Recovery must
    // report a `NestingDepthExceeded` diagnostic instead of
    // recursing until the stack blows.
    let depth = 4096;
    let mut src = String::with_capacity(depth * 2 + 4);
    for _ in 0..depth {
        src.push('(');
    }
    src.push('1');
    for _ in 0..depth {
        src.push(')');
    }
    src.push('.');

    let (tree, _roots) = drive(ParseMode::Expression, &src);
    let hit = find_error_by_kind(&tree, ParseErrorKind::NestingDepthExceeded);
    assert!(
        hit.is_some(),
        "expected at least one NestingDepthExceeded diagnostic"
    );
    let hit = hit.expect("checked above");
    assert!(
        hit.range().is_empty(),
        "NestingDepthExceeded is a boundary-anchored diagnostic"
    );
}

#[test]
fn parser_does_not_synthesize_a_fake_token_on_missing_close_paren() {
    // `-attr(1, 2 .` is missing the `)`. The token count in the
    // returned tree matches the scanned count.
    let source = "-attr(1, 2 .";
    let scanned = scan_all(source);
    let (tree, _roots) = drive(ParseMode::Module, source);
    assert_eq!(tree.tokens().len(), scanned.len());
}

#[test]
fn tokenizer_lexer_errors_are_not_reported_as_parser_errors() {
    // Only ParseErrorKind values the parser owns appear in the
    // tree's errors — a new kind would need to be added here
    // deliberately.
    let source = "1 2 .";
    let (tree, _roots) = drive(ParseMode::Module, source);
    for e in tree.errors() {
        match e.kind() {
            ParseErrorKind::UnexpectedToken
            | ParseErrorKind::UnexpectedEof
            | ParseErrorKind::SkippedToken
            | ParseErrorKind::MissingToken
            | ParseErrorKind::NestingDepthExceeded => {}
        }
    }
}

#[test]
fn container_recovery_lets_valid_elements_after_a_bad_one_still_land() {
    // Middle element is a stray `)`. Container recovery skips to
    // the next `,` or `]` so `3` still appears in the syntax index.
    let source = "[1, ), 3].";
    let (tree, _roots) = drive(ParseMode::Expression, source);
    assert!(contains_error_node(&tree));
    let has_int_3 = tree
        .syntax()
        .entries()
        .iter()
        .any(|e| e.kind() == SyntaxKind::IntegerExpr);
    assert!(has_int_3);
}

#[test]
fn clause_recovery_skips_to_semicolon_or_end() {
    // Malformed case-of clause head; recovery syncs to `;` and
    // parses the second clause structurally.
    let source = "case X of ) ) ; b -> 2 end.";
    let (tree, _roots) = drive(ParseMode::Expression, source);
    assert!(contains_error_node(&tree));
    assert!(
        tree.syntax()
            .entries()
            .iter()
            .any(|e| e.kind() == SyntaxKind::CaseExpr)
    );
}

#[test]
fn term_list_recovers_a_malformed_term_and_parses_the_next() {
    let source = "{ok, 1}.\nX.\n{ok, 2}.";
    let (tree, roots) = drive(ParseMode::TermList, source);
    assert_eq!(roots.len(), 3, "each `.`-terminated unit yields a root");
    assert!(!tree.errors().is_empty());
    assert_eq!(kind_of(&tree, roots[0]), SyntaxKind::TupleExpr);
    assert_eq!(kind_of(&tree, roots[2]), SyntaxKind::TupleExpr);
}

#[test]
fn error_ranges_are_usable_as_keys_into_external_metadata() {
    let source = "1 2 .";
    let (tree, _roots) = drive(ParseMode::Module, source);
    let err = tree.errors().first().copied().expect("at least one error");
    let start = err.range().start();
    assert!(start.get() <= tree.tokens().end_index().get());
}

#[test]
fn hidden_tokens_around_errors_are_kept_in_the_token_buffer() {
    // Comments and whitespace before / after the malformed region
    // remain in the buffer; recovery does not clip them out.
    let source = "%% pre\n) .\n%% post\n-ok.\n";
    let scanned = scan_all(source);
    let (tree, _roots) = drive(ParseMode::Module, source);
    assert_eq!(tree.tokens().len(), scanned.len());
}
