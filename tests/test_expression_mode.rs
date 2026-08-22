//! Integration tests for the `erl_parse::ParseMode::Expression` top-level and the
//! auxiliary entry point methods on `erl_parse::Parser`. Exercises the public
//! surface as external consumers see it.

fn scan_all(source: &str) -> Vec<erl_tokenize::Token> {
    let mut out = Vec::new();
    let mut pos = erl_tokenize::Position::new();
    while let Some(t) = erl_tokenize::scan_token(source, pos).expect("valid source") {
        out.push(t);
        pos = t.end();
    }
    out
}

fn push_all(parser: &mut erl_parse::Parser, source: &str) {
    for t in scan_all(source) {
        parser.push_token(t);
    }
}

fn kind_of(tree: &erl_parse::SyntaxTree, id: erl_parse::NodeId) -> erl_parse::SyntaxKind {
    tree.syntax().entry(id).expect("entry exists").kind()
}

#[test]
fn expression_mode_emits_unit_on_dot() {
    let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Expression);
    push_all(&mut parser, "1 + 2.");
    let node = parser.next_top_node().expect("unit completed at `.`");
    let tree = parser.finish();
    assert_eq!(kind_of(&tree, node), erl_parse::SyntaxKind::BinaryOpExpr);
    assert!(tree.errors().is_empty());
    // Root plus its two integer operands.
    assert!(tree.syntax().len() >= 3);
}

#[test]
fn expression_mode_finish_flushes_input_without_trailing_dot() {
    let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Expression);
    push_all(&mut parser, "foo(1, 2)");
    // No unit before finish because no `.` has been seen.
    assert!(parser.next_top_node().is_none());
    let tree = parser.finish();
    assert!(tree.errors().is_empty());
    assert!(!tree.syntax().is_empty());
    assert_eq!(
        kind_of(&tree, erl_parse::NodeId::new(0)),
        erl_parse::SyntaxKind::CallExpr
    );
}

#[test]
fn expression_mode_emits_multiple_units_across_dots() {
    let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Expression);
    push_all(&mut parser, "1. 2.");
    let first = parser.next_top_node().expect("first unit");
    let second = parser.next_top_node().expect("second unit");
    let tree = parser.finish();
    assert_eq!(kind_of(&tree, first), erl_parse::SyntaxKind::IntegerExpr);
    assert_eq!(kind_of(&tree, second), erl_parse::SyntaxKind::IntegerExpr);
    assert!(tree.errors().is_empty());
}

#[test]
fn parse_expression_range_returns_new_top_level_unit() {
    let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Expression);
    push_all(&mut parser, "1 + 2");
    // Aux entry point requires no in-progress unit and no `.` boundary
    // has been reached yet, so `unit_in_progress` is false and the aux
    // parse can run against the full pushed range.
    let range = erl_parse::TokenRange::new(
        erl_parse::TokenIndex::new(0),
        parser.syntax_tree().tokens().end_index(),
    );
    let id = parser
        .parse_expression_range(range)
        .expect("aux entry point succeeds when no unit is in progress");
    let tree = parser.finish();
    assert_eq!(kind_of(&tree, id), erl_parse::SyntaxKind::BinaryOpExpr);
    assert!(tree.errors().is_empty());
}

#[test]
fn parse_pattern_range_rejects_expression_only_constructs() {
    let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Expression);
    push_all(&mut parser, "foo(1)");
    let range = erl_parse::TokenRange::new(
        erl_parse::TokenIndex::new(0),
        parser.syntax_tree().tokens().end_index(),
    );
    let _ = parser
        .parse_pattern_range(range)
        .expect("no in-progress unit");
    let tree = parser.finish();
    // The call expression is rejected in pattern position, so we see
    // an error while the tree still holds the structural node.
    assert!(!tree.errors().is_empty());
}

#[test]
fn parse_guard_range_returns_guard_sequence_node() {
    let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Expression);
    push_all(&mut parser, "X > 0, X < 10");
    let range = erl_parse::TokenRange::new(
        erl_parse::TokenIndex::new(0),
        parser.syntax_tree().tokens().end_index(),
    );
    let id = parser
        .parse_guard_range(range)
        .expect("no in-progress unit");
    let tree = parser.finish();
    assert_eq!(kind_of(&tree, id), erl_parse::SyntaxKind::GuardSequence);
    assert!(tree.errors().is_empty());
}

#[test]
fn parse_term_range_rejects_variables_and_calls() {
    let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Expression);
    push_all(&mut parser, "X");
    let range = erl_parse::TokenRange::new(
        erl_parse::TokenIndex::new(0),
        parser.syntax_tree().tokens().end_index(),
    );
    let _ = parser.parse_term_range(range).expect("no in-progress unit");
    let tree = parser.finish();
    assert!(!tree.errors().is_empty());
}

#[test]
fn parse_term_range_accepts_literal_tuple() {
    let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Expression);
    push_all(&mut parser, "{ok, 1}");
    let range = erl_parse::TokenRange::new(
        erl_parse::TokenIndex::new(0),
        parser.syntax_tree().tokens().end_index(),
    );
    let id = parser.parse_term_range(range).expect("no in-progress unit");
    let tree = parser.finish();
    assert_eq!(kind_of(&tree, id), erl_parse::SyntaxKind::TupleExpr);
    assert!(tree.errors().is_empty());
}

#[test]
fn push_token_returns_index_of_added_token() {
    // Include a comment so hidden tokens participate in the index
    // stream on the same footing as lexical tokens.
    let source = "foo % note\n bar";
    let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Module);
    let scanned = scan_all(source);
    let mut returned = Vec::new();
    for t in &scanned {
        returned.push(parser.push_token(*t));
    }
    let tree = parser.finish();
    for (i, (index, expected)) in returned.iter().zip(scanned.iter()).enumerate() {
        assert_eq!(
            *index,
            erl_parse::TokenIndex::new(i),
            "push_token {i} returned unexpected index"
        );
        let got = tree
            .tokens()
            .get(*index)
            .expect("returned index recovers the pushed token");
        assert_eq!(got, *expected, "get({index:?}) mismatch");
    }
}

// The `erl_parse::ProtocolError` precondition on the aux-entry-point methods
// (`parse_expression_range` / `parse_pattern_range` /
// `parse_guard_range` / `parse_term_range` / `parse_type_range`)
// covers the case where a top-level unit is still in progress when
// the caller invokes one of them. None of the mode-level top-level
// drivers currently in this crate leave a unit half-open across
// `push_token` / `finish` calls, so the precondition is a contract
// preserved for future error-recovery grammars rather than something
// integration tests can trigger through the public API.
