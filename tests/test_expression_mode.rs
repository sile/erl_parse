//! Integration tests for the `ParseMode::Expression` top-level and the
//! auxiliary entry point methods on `Parser`. Exercises the public
//! surface as external consumers see it.

use erl_parse::{ParseMode, Parser, ProtocolError, SyntaxKind, TokenIndex, TokenRange};
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

fn kind_of(tree: &erl_parse::SyntaxTree, id: erl_parse::NodeId) -> SyntaxKind {
    tree.syntax().entry(id).expect("entry exists").kind()
}

#[test]
fn expression_mode_emits_unit_on_dot() {
    let mut parser = Parser::new(ParseMode::Expression);
    push_all(&mut parser, "1 + 2.");
    let node = parser.next_top_node().expect("unit completed at `.`");
    let tree = parser.finish();
    assert_eq!(kind_of(&tree, node), SyntaxKind::BinaryOpExpr);
    assert!(tree.errors().is_empty());
    // Root plus its two integer operands.
    assert!(tree.syntax().len() >= 3);
}

#[test]
fn expression_mode_finish_flushes_input_without_trailing_dot() {
    let mut parser = Parser::new(ParseMode::Expression);
    push_all(&mut parser, "foo(1, 2)");
    // No unit before finish because no `.` has been seen.
    assert!(parser.next_top_node().is_none());
    let tree = parser.finish();
    assert!(tree.errors().is_empty());
    assert!(!tree.syntax().is_empty());
    assert_eq!(
        kind_of(&tree, erl_parse::NodeId::new(0)),
        SyntaxKind::CallExpr
    );
}

#[test]
fn expression_mode_emits_multiple_units_across_dots() {
    let mut parser = Parser::new(ParseMode::Expression);
    push_all(&mut parser, "1. 2.");
    let first = parser.next_top_node().expect("first unit");
    let second = parser.next_top_node().expect("second unit");
    let tree = parser.finish();
    assert_eq!(kind_of(&tree, first), SyntaxKind::IntegerExpr);
    assert_eq!(kind_of(&tree, second), SyntaxKind::IntegerExpr);
    assert!(tree.errors().is_empty());
}

#[test]
fn parse_expression_range_returns_new_top_level_unit() {
    let mut parser = Parser::new(ParseMode::Expression);
    push_all(&mut parser, "1 + 2");
    // Aux entry point requires no in-progress unit and no `.` boundary
    // has been reached yet, so `unit_in_progress` is false and the aux
    // parse can run against the full pushed range.
    let range = TokenRange::new(
        TokenIndex::new(0),
        parser.syntax_tree().tokens().end_index(),
    );
    let id = parser
        .parse_expression_range(range)
        .expect("aux entry point succeeds when no unit is in progress");
    let tree = parser.finish();
    assert_eq!(kind_of(&tree, id), SyntaxKind::BinaryOpExpr);
    assert!(tree.errors().is_empty());
}

#[test]
fn parse_pattern_range_rejects_expression_only_constructs() {
    let mut parser = Parser::new(ParseMode::Expression);
    push_all(&mut parser, "foo(1)");
    let range = TokenRange::new(
        TokenIndex::new(0),
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
    let mut parser = Parser::new(ParseMode::Expression);
    push_all(&mut parser, "X > 0, X < 10");
    let range = TokenRange::new(
        TokenIndex::new(0),
        parser.syntax_tree().tokens().end_index(),
    );
    let id = parser
        .parse_guard_range(range)
        .expect("no in-progress unit");
    let tree = parser.finish();
    assert_eq!(kind_of(&tree, id), SyntaxKind::GuardSequence);
    assert!(tree.errors().is_empty());
}

#[test]
fn parse_term_range_rejects_variables_and_calls() {
    let mut parser = Parser::new(ParseMode::Expression);
    push_all(&mut parser, "X");
    let range = TokenRange::new(
        TokenIndex::new(0),
        parser.syntax_tree().tokens().end_index(),
    );
    let _ = parser.parse_term_range(range).expect("no in-progress unit");
    let tree = parser.finish();
    assert!(!tree.errors().is_empty());
}

#[test]
fn parse_term_range_accepts_literal_tuple() {
    let mut parser = Parser::new(ParseMode::Expression);
    push_all(&mut parser, "{ok, 1}");
    let range = TokenRange::new(
        TokenIndex::new(0),
        parser.syntax_tree().tokens().end_index(),
    );
    let id = parser.parse_term_range(range).expect("no in-progress unit");
    let tree = parser.finish();
    assert_eq!(kind_of(&tree, id), SyntaxKind::TupleExpr);
    assert!(tree.errors().is_empty());
}

#[test]
fn push_token_returns_index_of_added_token() {
    // Include a comment so hidden tokens participate in the index
    // stream on the same footing as lexical tokens.
    let source = "foo % note\n bar";
    let mut parser = Parser::new(ParseMode::Module);
    let scanned = scan_all(source);
    let mut returned = Vec::new();
    for t in &scanned {
        returned.push(parser.push_token(*t));
    }
    let tree = parser.finish();
    for (i, (index, expected)) in returned.iter().zip(scanned.iter()).enumerate() {
        assert_eq!(
            *index,
            TokenIndex::new(i),
            "push_token {i} returned unexpected index"
        );
        let got = tree
            .tokens()
            .get(*index)
            .expect("returned index recovers the pushed token");
        assert_eq!(got, *expected, "get({index:?}) mismatch");
    }
}

#[test]
fn aux_entry_point_rejects_when_unit_in_progress() {
    // Module mode's stub grammar keeps a unit open while unterminated
    // input is buffered, which lets us construct the "in-progress"
    // precondition without depending on grammar internals.
    let mut parser = Parser::new(ParseMode::Module);
    push_all(&mut parser, "foo");
    let range = TokenRange::new(
        TokenIndex::new(0),
        parser.syntax_tree().tokens().end_index(),
    );
    let err = parser
        .parse_expression_range(range)
        .expect_err("aux entry point should reject in-progress unit");
    assert_eq!(err, ProtocolError::AuxEntryPointWithUnitInProgress);
}
