//! Integration tests for `erl_parse::ParseMode::Module`. Exercises the module-
//! mode top-level driver and the form / attribute / function
//! declaration grammar as external consumers see them.

fn scan_all(source: &str) -> Vec<erl_tokenize::Token> {
    let mut out = Vec::new();
    let mut pos = erl_tokenize::Position::new();
    while let Some(t) = erl_tokenize::scan_token(source, pos).expect("valid source") {
        out.push(t);
        pos = t.end();
    }
    out
}

fn feed_all(parser: &mut erl_parse::Parser, source: &str) {
    for t in scan_all(source) {
        parser.feed_token(t);
    }
}

fn kind_of(tree: &erl_parse::SyntaxTree, id: erl_parse::NodeId) -> erl_parse::SyntaxKind {
    tree.syntax().entry(id).expect("entry exists").kind()
}

fn drive(source: &str) -> (erl_parse::SyntaxTree, Vec<erl_parse::NodeId>) {
    let mut p = erl_parse::Parser::new(erl_parse::ParseMode::Module);
    feed_all(&mut p, source);
    let mut roots = Vec::new();
    while let Some(id) = p.next_node() {
        roots.push(id);
    }
    (p.finish(), roots)
}

fn direct_children(
    tree: &erl_parse::SyntaxTree,
    root: erl_parse::NodeId,
) -> Vec<erl_parse::NodeId> {
    tree.view(root)
        .expect("root entry")
        .children()
        .map(|c| c.node_id())
        .collect()
}

#[test]
fn empty_module_emits_no_units_and_no_errors() {
    let (tree, roots) = drive("");
    assert!(roots.is_empty());
    assert!(tree.syntax().is_empty());
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn hidden_only_module_stores_tokens_but_emits_no_units() {
    let (tree, roots) = drive("   \n%% comment\n\n");
    assert!(roots.is_empty());
    assert!(!tree.tokens().is_empty());
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn single_bare_attribute_form() {
    let (tree, roots) = drive("-something.");
    assert_eq!(roots.len(), 1);
    assert_eq!(kind_of(&tree, roots[0]), erl_parse::SyntaxKind::Attribute);
    let children = direct_children(&tree, roots[0]);
    let child_kinds: Vec<_> = children.iter().map(|&c| kind_of(&tree, c)).collect();
    assert_eq!(
        child_kinds,
        vec![
            erl_parse::SyntaxKind::AttributeName,
            erl_parse::SyntaxKind::AttributePayload
        ]
    );
    // The payload node is zero-width because the form has no `(...)`
    // section.
    let payload = tree.syntax().entry(children[1]).expect("payload");
    assert!(payload.range().is_empty());
    assert!(tree.diagnostics().is_empty());
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
        vec![
            erl_parse::SyntaxKind::AttributeName,
            erl_parse::SyntaxKind::AttributePayload
        ]
    );
    // Callers pull the attribute name from the token buffer using the
    // erl_parse::AttributeName child's range; the parser does not interpret
    // `module`.
    let name_entry = tree.syntax().entry(children[0]).expect("name");
    let payload_entry = tree.syntax().entry(children[1]).expect("payload");
    assert!(!name_entry.range().is_empty());
    assert!(!payload_entry.range().is_empty());
    assert!(tree.diagnostics().is_empty());
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
            erl_parse::SyntaxKind::Attribute,
            "source: {source}"
        );
        assert!(
            tree.diagnostics().is_empty(),
            "source: {source} produced unexpected errors: {:?}",
            tree.diagnostics()
        );
    }
}

#[test]
fn record_field_dot_does_not_end_the_form_mid_push() {
    // The field-access `.` is the same token as a form terminator.
    // Incremental `feed_token` must not start `parse_one` when that
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
            erl_parse::SyntaxKind::FunctionDecl,
            "source: {source}"
        );
        assert!(
            tree.diagnostics().is_empty(),
            "source: {source} produced unexpected errors: {:?}",
            tree.diagnostics()
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
            erl_parse::SyntaxKind::Attribute,
            "source: {source}"
        );
        assert!(tree.diagnostics().is_empty(), "source: {source}");
    }
}

#[test]
fn file_attribute_is_treated_as_ordinary_attribute() {
    // `-file` in real Erlang can be treated as a source-position hint
    // by the compiler. This parser does not: the form is just an
    // attribute whose payload is preserved.
    let (tree, roots) = drive("-file(\"other.erl\", 42).");
    assert_eq!(roots.len(), 1);
    assert_eq!(kind_of(&tree, roots[0]), erl_parse::SyntaxKind::Attribute);
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn function_declaration_single_clause() {
    let (tree, roots) = drive("foo() -> ok.");
    assert_eq!(roots.len(), 1);
    assert_eq!(
        kind_of(&tree, roots[0]),
        erl_parse::SyntaxKind::FunctionDecl
    );
    let clauses = direct_children(&tree, roots[0]);
    assert_eq!(clauses.len(), 1);
    assert_eq!(
        kind_of(&tree, clauses[0]),
        erl_parse::SyntaxKind::FunctionClause
    );
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn function_declaration_multiple_clauses_separated_by_semicolon() {
    let (tree, roots) = drive("foo(1) -> one; foo(2) -> two; foo(_) -> other.");
    assert_eq!(roots.len(), 1);
    assert_eq!(
        kind_of(&tree, roots[0]),
        erl_parse::SyntaxKind::FunctionDecl
    );
    let clauses = direct_children(&tree, roots[0]);
    assert_eq!(clauses.len(), 3);
    for c in &clauses {
        assert_eq!(kind_of(&tree, *c), erl_parse::SyntaxKind::FunctionClause);
    }
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn function_declaration_with_guard_and_body() {
    let (tree, roots) = drive("abs(X) when X < 0 -> -X; abs(X) -> X.");
    assert_eq!(roots.len(), 1);
    assert!(tree.diagnostics().is_empty());
    assert_eq!(
        kind_of(&tree, roots[0]),
        erl_parse::SyntaxKind::FunctionDecl
    );
}

#[test]
fn function_clause_arity_mismatch_is_not_a_syntax_error() {
    // The parser does not check same-name / same-arity consistency
    // across clauses in a form. That is a semantic concern.
    let (tree, roots) = drive("foo(1) -> ok; foo(1, 2) -> ok.");
    assert_eq!(roots.len(), 1);
    assert!(tree.diagnostics().is_empty());
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
            erl_parse::SyntaxKind::Attribute,
            erl_parse::SyntaxKind::Attribute,
            erl_parse::SyntaxKind::FunctionDecl
        ]
    );
    assert!(tree.diagnostics().is_empty());
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
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn malformed_attribute_missing_close_paren_emits_error() {
    // Missing `)`: attribute payload takes tokens up to the terminating
    // `.` and the missing close-paren surfaces as a erl_parse::Diagnostic.
    let (tree, roots) = drive("-module(mymod.");
    assert_eq!(roots.len(), 1);
    assert_eq!(kind_of(&tree, roots[0]), erl_parse::SyntaxKind::Attribute);
    assert!(!tree.diagnostics().is_empty());
}

#[test]
fn malformed_function_clause_missing_arrow_emits_error() {
    let (tree, roots) = drive("foo() ok.");
    assert_eq!(roots.len(), 1);
    assert!(!tree.diagnostics().is_empty());
}

#[test]
fn missing_form_terminating_dot_flushes_via_finish() {
    // `-module(m)` never sees a boundary `.`, so no unit completes
    // during push; `finish` force-parses the trailing input as one
    // final unit whose contents include the parsed attribute plus
    // whatever the mode's grammar can extract.
    let mut p = erl_parse::Parser::new(erl_parse::ParseMode::Module);
    feed_all(&mut p, "-module(m)");
    assert!(p.next_node().is_none());
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
    assert_eq!(tree_a.diagnostics().len(), tree_b.diagnostics().len());
}
