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

fn drive(source: &str) -> (erl_parse::SyntaxTree, Vec<erl_parse::NodeId>) {
    let tokens = scan_all(source);
    let tree = erl_parse::parse(tokens, erl_parse::ParseMode::TermList);
    let roots: Vec<_> = tree.roots().map(|v| v.node_id()).collect();
    (tree, roots)
}

fn kind_of(tree: &erl_parse::SyntaxTree, id: erl_parse::NodeId) -> erl_parse::SyntaxKind {
    tree.view(id).expect("entry exists").kind()
}

fn hand_tied_ids(tree: &erl_parse::SyntaxTree) -> Vec<erl_parse::NodeId> {
    tree.roots()
        .flat_map(|root| std::iter::once(root).chain(root.descendants()))
        .map(|v| v.node_id())
        .collect()
}

fn node_ids(tree: &erl_parse::SyntaxTree) -> Vec<erl_parse::NodeId> {
    tree.nodes().map(|v| v.node_id()).collect()
}

#[test]
fn nodes_matches_hand_tied_walk_for_single_root() {
    let (tree, _roots) = drive("{ok, 1}.");
    assert_eq!(node_ids(&tree), hand_tied_ids(&tree));
    assert!(!node_ids(&tree).is_empty());
}

#[test]
fn nodes_matches_hand_tied_walk_for_multiple_roots() {
    let (tree, _roots) = drive("{ok, 1}.\n{error, notfound}.\n[a, b, c].\n");
    assert_eq!(node_ids(&tree), hand_tied_ids(&tree));
    assert!(node_ids(&tree).len() > tree.roots().count());
}

#[test]
fn nodes_is_empty_for_empty_tree() {
    let (tree, _roots) = drive("");
    assert_eq!(node_ids(&tree), hand_tied_ids(&tree));
    assert!(node_ids(&tree).is_empty());
}

#[test]
fn empty_term_list_emits_no_units_and_no_errors() {
    let (tree, roots) = drive("");
    assert!(roots.is_empty());
    assert!(tree.roots().next().is_none());
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
