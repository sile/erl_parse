//! Integration tests for the `erl_parse::ParseMode::Expression` top-level.
//! Exercises the public surface as external consumers see it.

fn scan_all(source: &str) -> Vec<erl_tokenize::Token> {
    let mut out = Vec::new();
    let mut pos = erl_tokenize::Position::new();
    while let Some(t) = erl_tokenize::scan_token(source, pos).expect("valid source") {
        out.push(t);
        pos = t.end();
    }
    out
}

fn kind_of(tree: &erl_parse::SyntaxTree, id: erl_parse::NodeId) -> erl_parse::SyntaxKind {
    tree.view(id).expect("entry exists").kind()
}

fn drive(source: &str) -> (erl_parse::SyntaxTree, Vec<erl_parse::NodeId>) {
    let tokens = scan_all(source);
    let tree = erl_parse::parse(&tokens, erl_parse::ParseMode::Expression);
    let roots: Vec<_> = tree.roots().map(|v| v.node_id()).collect();
    (tree, roots)
}

#[test]
fn expression_mode_emits_unit_on_dot() {
    let (tree, roots) = drive("1 + 2.");
    assert_eq!(roots.len(), 1);
    assert_eq!(
        kind_of(&tree, roots[0]),
        erl_parse::SyntaxKind::BinaryOpExpr
    );
    assert!(tree.diagnostics().is_empty());
    // Root plus its two integer operands.
    let root = tree.view(roots[0]).expect("root");
    assert!(1 + root.descendants().count() >= 3);
}

#[test]
fn expression_mode_parses_input_without_trailing_dot() {
    let (tree, roots) = drive("foo(1, 2)");
    assert!(tree.diagnostics().is_empty());
    assert_eq!(roots.len(), 1);
    assert_eq!(kind_of(&tree, roots[0]), erl_parse::SyntaxKind::CallExpr);
}

#[test]
fn expression_mode_emits_multiple_units_across_dots() {
    let (tree, roots) = drive("1. 2.");
    assert_eq!(roots.len(), 2);
    assert_eq!(kind_of(&tree, roots[0]), erl_parse::SyntaxKind::IntegerExpr);
    assert_eq!(kind_of(&tree, roots[1]), erl_parse::SyntaxKind::IntegerExpr);
    assert!(tree.diagnostics().is_empty());
}

#[test]
fn token_index_matches_input_order() {
    // Include a comment so hidden tokens participate in the index
    // stream on the same footing as lexical tokens.
    let source = "foo % note\n bar";
    let scanned = scan_all(source);
    let tree = erl_parse::parse(&scanned, erl_parse::ParseMode::Module);
    assert_eq!(tree.tokens().len(), scanned.len());
    for (i, expected) in scanned.iter().enumerate() {
        let index = erl_parse::TokenIndex::new(i);
        let got = tree
            .tokens()
            .get(index.get())
            .copied()
            .expect("input index recovers the same token");
        assert_eq!(got, *expected, "get({index:?}) mismatch");
    }
}
