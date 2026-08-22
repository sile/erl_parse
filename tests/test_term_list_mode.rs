//! Integration tests for `erl_parse::ParseMode::TermList`. Exercises the
//! `file:consult/1`-style term sequence grammar as external consumers
//! see it.

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
    let mut p = erl_parse::Parser::new(erl_parse::ParseMode::TermList);
    feed_all(&mut p, source);
    let mut roots = Vec::new();
    while let Some(id) = p.next_node() {
        roots.push(id);
    }
    (p.finish(), roots)
}

#[test]
fn empty_term_list_emits_no_units_and_no_errors() {
    let (tree, roots) = drive("");
    assert!(roots.is_empty());
    assert!(tree.syntax().is_empty());
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn sequence_of_literal_terms_yields_one_unit_per_term() {
    // `file:consult/1` / `rebar.config` style: `.`-terminated Erlang
    // terms.
    let (tree, roots) = drive("{ok, 1}.\n{error, notfound}.\n[a, b, c].\n");
    assert_eq!(roots.len(), 3);
    assert_eq!(kind_of(&tree, roots[0]), erl_parse::SyntaxKind::TupleExpr);
    assert_eq!(kind_of(&tree, roots[1]), erl_parse::SyntaxKind::TupleExpr);
    assert_eq!(kind_of(&tree, roots[2]), erl_parse::SyntaxKind::ListExpr);
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn variables_are_rejected_in_term_position() {
    let (tree, _roots) = drive("X.");
    assert!(!tree.diagnostics().is_empty());
}

#[test]
fn calls_are_rejected_in_term_position() {
    let (tree, _roots) = drive("foo(1).");
    assert!(!tree.diagnostics().is_empty());
}

#[test]
fn blocks_are_rejected_in_term_position() {
    let (tree, _roots) = drive("begin 1 end.");
    assert!(!tree.diagnostics().is_empty());
}

#[test]
fn hidden_tokens_between_terms_are_preserved_in_buffer() {
    let source = "{a, 1}.\n%% between terms\n{b, 2}.\n";
    let (tree, roots) = drive(source);
    assert_eq!(roots.len(), 2);
    let scanned = scan_all(source);
    assert_eq!(tree.tokens().len(), scanned.len());
    assert!(tree.diagnostics().is_empty());
}
