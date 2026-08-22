//! Integration tests for `ParseMode::Module`. Exercises the module-
//! mode top-level driver and the form / attribute / function
//! declaration grammar as external consumers see them.

use erl_parse::{FormKind, NodeId, ParseMode, Parser, SyntaxKind, SyntaxTree};
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

fn kind_of(tree: &SyntaxTree, id: NodeId) -> SyntaxKind {
    tree.syntax().entry(id).expect("entry exists").kind()
}

fn drive(source: &str) -> (SyntaxTree, Vec<NodeId>) {
    let mut p = Parser::new(ParseMode::Module);
    push_all(&mut p, source);
    let mut roots = Vec::new();
    while let Some(id) = p.next_top_node() {
        roots.push(id);
    }
    (p.finish(), roots)
}

fn direct_children(tree: &SyntaxTree, root: NodeId) -> Vec<NodeId> {
    let end = tree
        .syntax()
        .entry(root)
        .expect("root entry")
        .subtree_end()
        .get();
    let mut children = Vec::new();
    let mut i = root.get() + 1;
    while i < end {
        let child_id = NodeId::new(i);
        children.push(child_id);
        i = tree
            .syntax()
            .entry(child_id)
            .expect("child")
            .subtree_end()
            .get();
    }
    children
}

#[test]
fn empty_module_emits_no_units_and_no_errors() {
    let (tree, roots) = drive("");
    assert!(roots.is_empty());
    assert!(tree.syntax().is_empty());
    assert!(tree.errors().is_empty());
}

#[test]
fn hidden_only_module_stores_tokens_but_emits_no_units() {
    let (tree, roots) = drive("   \n%% comment\n\n");
    assert!(roots.is_empty());
    assert!(!tree.tokens().is_empty());
    assert!(tree.errors().is_empty());
}

#[test]
fn single_bare_attribute_form() {
    let (tree, roots) = drive("-something.");
    assert_eq!(roots.len(), 1);
    assert_eq!(kind_of(&tree, roots[0]), SyntaxKind::Attribute);
    let children = direct_children(&tree, roots[0]);
    let child_kinds: Vec<_> = children.iter().map(|&c| kind_of(&tree, c)).collect();
    assert_eq!(
        child_kinds,
        vec![SyntaxKind::AttributeName, SyntaxKind::AttributePayload]
    );
    // The payload node is zero-width because the form has no `(...)`
    // section.
    let payload = tree.syntax().entry(children[1]).expect("payload");
    assert!(payload.range().is_empty());
    assert!(tree.errors().is_empty());
}

#[test]
fn parenthesized_attribute_records_name_and_payload_ranges() {
    let (tree, roots) = drive("-module(mymod).");
    assert_eq!(roots.len(), 1);
    let children = direct_children(&tree, roots[0]);
    assert_eq!(
        children
            .iter()
            .map(|&c| kind_of(&tree, c))
            .collect::<Vec<_>>(),
        vec![SyntaxKind::AttributeName, SyntaxKind::AttributePayload]
    );
    // Callers pull the attribute name from the token buffer using the
    // AttributeName child's range; the parser does not interpret
    // `module`.
    let name_entry = tree.syntax().entry(children[0]).expect("name");
    let payload_entry = tree.syntax().entry(children[1]).expect("payload");
    assert!(!name_entry.range().is_empty());
    assert!(!payload_entry.range().is_empty());
    assert!(tree.errors().is_empty());
}

#[test]
fn spec_type_record_and_export_are_uniform_attributes() {
    // The parser does not specialize on the attribute name. All four
    // land on the same `Attribute` shape.
    for source in [
        "-spec foo(X) -> integer().",
        "-type maybe(T) :: undefined | T.",
        "-record(user, {name = \"a\", age = 0}).",
        "-export([foo/1, bar/2]).",
        "-callback handle(term()) -> term().",
        "-opaque handle() :: reference().",
    ] {
        let (tree, roots) = drive(source);
        assert_eq!(roots.len(), 1, "source: {source}");
        assert_eq!(
            kind_of(&tree, roots[0]),
            SyntaxKind::Attribute,
            "source: {source}"
        );
        assert!(
            tree.errors().is_empty(),
            "source: {source} produced unexpected errors: {:?}",
            tree.errors()
        );
    }
}

#[test]
fn record_field_dot_does_not_end_the_form_mid_push() {
    // The field-access `.` is the same token as a form terminator.
    // Incremental `push_token` must not start `parse_one` when that
    // `.` is pushed, because the field name is not in the buffer yet.
    for source in [
        "f(X) -> X#r.f.",
        "f() -> #r.f.",
        "f(X) -> X#_.f.",
        "f(X) -> X#r.f + 1.",
    ] {
        let (tree, roots) = drive(source);
        assert_eq!(roots.len(), 1, "source: {source}");
        assert_eq!(
            kind_of(&tree, roots[0]),
            SyntaxKind::FunctionDecl,
            "source: {source}"
        );
        assert!(
            tree.errors().is_empty(),
            "source: {source} produced unexpected errors: {:?}",
            tree.errors()
        );
    }
}

#[test]
fn unpreprocessed_directives_are_kept_as_unknown_attributes() {
    // `-define`, `-undef`, `-include`, `-ifdef` etc. would normally be
    // consumed by preprocessing. The parser makes no attempt to
    // execute them and treats them as ordinary attribute forms whose
    // payload is preserved verbatim.
    for source in [
        "-define(FOO, 1).",
        "-undef(FOO).",
        "-include(\"header.hrl\").",
        "-ifdef(FOO).",
    ] {
        let (tree, roots) = drive(source);
        assert_eq!(roots.len(), 1, "source: {source}");
        assert_eq!(
            kind_of(&tree, roots[0]),
            SyntaxKind::Attribute,
            "source: {source}"
        );
        assert!(tree.errors().is_empty(), "source: {source}");
    }
}

#[test]
fn file_attribute_is_treated_as_ordinary_attribute() {
    // `-file` in real Erlang can be treated as a source-position hint
    // by the compiler. This parser does not: the form is just an
    // attribute whose payload is preserved.
    let (tree, roots) = drive("-file(\"other.erl\", 42).");
    assert_eq!(roots.len(), 1);
    assert_eq!(kind_of(&tree, roots[0]), SyntaxKind::Attribute);
    assert!(tree.errors().is_empty());
}

#[test]
fn function_declaration_single_clause() {
    let (tree, roots) = drive("foo() -> ok.");
    assert_eq!(roots.len(), 1);
    assert_eq!(kind_of(&tree, roots[0]), SyntaxKind::FunctionDecl);
    let clauses = direct_children(&tree, roots[0]);
    assert_eq!(clauses.len(), 1);
    assert_eq!(kind_of(&tree, clauses[0]), SyntaxKind::FunctionClause);
    assert!(tree.errors().is_empty());
}

#[test]
fn function_declaration_multiple_clauses_separated_by_semicolon() {
    let (tree, roots) = drive("foo(1) -> one; foo(2) -> two; foo(_) -> other.");
    assert_eq!(roots.len(), 1);
    assert_eq!(kind_of(&tree, roots[0]), SyntaxKind::FunctionDecl);
    let clauses = direct_children(&tree, roots[0]);
    assert_eq!(clauses.len(), 3);
    for c in &clauses {
        assert_eq!(kind_of(&tree, *c), SyntaxKind::FunctionClause);
    }
    assert!(tree.errors().is_empty());
}

#[test]
fn function_declaration_with_guard_and_body() {
    let (tree, roots) = drive("abs(X) when X < 0 -> -X; abs(X) -> X.");
    assert_eq!(roots.len(), 1);
    assert!(tree.errors().is_empty());
    assert_eq!(kind_of(&tree, roots[0]), SyntaxKind::FunctionDecl);
}

#[test]
fn function_clause_arity_mismatch_is_not_a_syntax_error() {
    // The parser does not check same-name / same-arity consistency
    // across clauses in a form. That is a semantic concern.
    let (tree, roots) = drive("foo(1) -> ok; foo(1, 2) -> ok.");
    assert_eq!(roots.len(), 1);
    assert!(tree.errors().is_empty());
}

#[test]
fn multiple_forms_yield_one_top_level_unit_per_form() {
    let source = "-module(m). -export([f/0]). f() -> ok.";
    let (tree, roots) = drive(source);
    assert_eq!(roots.len(), 3);
    let kinds: Vec<_> = roots.iter().map(|&r| kind_of(&tree, r)).collect();
    assert_eq!(
        kinds,
        vec![
            SyntaxKind::Attribute,
            SyntaxKind::Attribute,
            SyntaxKind::FunctionDecl
        ]
    );
    assert!(tree.errors().is_empty());
}

#[test]
fn hidden_tokens_between_forms_are_preserved_in_buffer() {
    let source = "-module(a).\n\n%% between forms\n\nf() -> ok.\n";
    let (tree, roots) = drive(source);
    assert_eq!(roots.len(), 2);
    // Every scanned token, including comments and whitespace, lands
    // in the token buffer even though the form nodes do not cover the
    // between-form region.
    let scanned = scan_all(source);
    assert_eq!(tree.tokens().len(), scanned.len());
    assert!(tree.errors().is_empty());
}

#[test]
fn malformed_attribute_missing_close_paren_emits_error() {
    // Missing `)`: attribute payload takes tokens up to the terminating
    // `.` and the missing close-paren surfaces as a ParseError.
    let (tree, roots) = drive("-module(mymod.");
    assert_eq!(roots.len(), 1);
    assert_eq!(kind_of(&tree, roots[0]), SyntaxKind::Attribute);
    assert!(!tree.errors().is_empty());
}

#[test]
fn malformed_function_clause_missing_arrow_emits_error() {
    let (tree, roots) = drive("foo() ok.");
    assert_eq!(roots.len(), 1);
    assert!(!tree.errors().is_empty());
}

#[test]
fn missing_form_terminating_dot_flushes_via_finish() {
    // `-module(m)` never sees a boundary `.`, so no unit completes
    // during push; `finish` force-parses the trailing input as one
    // final unit whose contents include the parsed attribute plus
    // whatever the mode's grammar can extract.
    let mut p = Parser::new(ParseMode::Module);
    push_all(&mut p, "-module(m)");
    assert!(p.next_top_node().is_none());
    let tree = p.finish();
    assert!(!tree.syntax().is_empty());
}

#[test]
fn independent_parser_instances_do_not_share_state() {
    let source = "-module(a). foo() -> 1.";
    let (tree_a, roots_a) = drive(source);
    let (tree_b, roots_b) = drive(source);
    assert_eq!(roots_a.len(), roots_b.len());
    assert_eq!(tree_a.syntax().len(), tree_b.syntax().len());
    assert_eq!(tree_a.errors().len(), tree_b.errors().len());
}

#[test]
fn in_progress_state_is_default_before_between_and_after_forms() {
    // The dot-driven top-level driver consumes an entire form within
    // a single `push_token` call (the one that added the terminating
    // `.`), so `state()` is only observable at form boundaries.
    // Between and outside forms all form-level fields read back as
    // their default (`None`), and no `Position` value is exposed on
    // any range field.
    let mut p = Parser::new(ParseMode::Module);
    let expect_default = |s: erl_parse::InProgressState| {
        assert_eq!(s.form_kind, None);
        assert_eq!(s.attribute_name, None);
        assert_eq!(s.function_name, None);
        assert_eq!(s.function_arity, None);
        assert_eq!(s.current_clause, None);
    };
    expect_default(p.state());

    push_all(&mut p, "-attr(payload).");
    let _ = p.next_top_node().expect("first form");
    expect_default(p.state());

    push_all(&mut p, " foo(1, 2) -> ok.");
    let _ = p.next_top_node().expect("second form");
    expect_default(p.state());
}

#[test]
fn in_progress_state_exposes_form_kind_variants() {
    // Compile-time surface check: the public `FormKind` variants and
    // `InProgressState` fields are reachable through the crate root
    // and are shaped as the API design requires. A caller that adds
    // its own grammar (or a future recovery pass) reads these fields
    // through `Parser::state()` — the field types are what matter
    // here, not the runtime values.
    let sample = erl_parse::InProgressState::default();
    let _: Option<FormKind> = sample.form_kind;
    let _: Option<erl_parse::TokenRange> = sample.attribute_name;
    let _: Option<erl_parse::TokenRange> = sample.function_name;
    let _: Option<usize> = sample.function_arity;
    let _: Option<usize> = sample.current_clause;
    let _ = (FormKind::Attribute, FormKind::FunctionDecl);
}
