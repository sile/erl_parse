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
