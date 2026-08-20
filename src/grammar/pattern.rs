//! Pattern grammar.
//!
//! Shares the token cursor, precedence table, and marker primitives with
//! [`crate::grammar::expr`], but switches the parser into
//! [`ParseContext::Pattern`] so that expression-only constructs (calls,
//! blocks, funs, comprehensions, remote qualifiers, sends, and
//! maybe-matches) push a [`ParseError`] rather than pass silently. The
//! shape of the syntax tree is the same as for expressions; downstream
//! consumers see a structured node in every position, plus errors that
//! flag positions where restrictions were violated.

use crate::grammar::expr::parse_expr;
use crate::parser::{CompletedMarker, ParseContext, Parser};

/// Parses a pattern at the current cursor position.
///
/// Delegates to [`parse_expr`][crate::grammar::expr::parse_expr] with
/// [`ParseContext::Pattern`] active for the span; restrictions are
/// enforced through structured [`ParseError`][crate::ParseError]s
/// appended to the parser's error list.
pub(crate) fn parse_pattern(p: &mut Parser) -> CompletedMarker {
    let prev = p.set_context(ParseContext::Pattern);
    let completed = parse_expr(p);
    p.set_context(prev);
    completed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{NodeId, SyntaxKind};
    use crate::{ParseMode, Parser};
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

    fn drive_pattern(source: &str) -> Parser {
        let mut p = Parser::new(ParseMode::Module);
        for t in scan_all(source) {
            p.push_token_without_grammar_for_test(t);
        }
        p.reset_for_test();
        let outer = p.start();
        parse_pattern(&mut p);
        outer.complete(&mut p, SyntaxKind::Error);
        p.finalize_pending_units_for_test();
        p
    }

    fn first_child_kind(p: &Parser, root: NodeId) -> SyntaxKind {
        p.syntax_tree()
            .syntax()
            .entry(NodeId::new(root.get() + 1))
            .expect("first child")
            .kind()
    }

    #[test]
    fn accepts_atom_variable_and_literal_patterns() {
        for (source, kind) in [
            ("foo", SyntaxKind::AtomExpr),
            ("X", SyntaxKind::VarExpr),
            ("42", SyntaxKind::IntegerExpr),
            ("-1", SyntaxKind::UnaryOpExpr),
        ] {
            let mut p = drive_pattern(source);
            let root = p.next_top_node().expect("unit");
            assert_eq!(first_child_kind(&p, root), kind, "source {source}");
            assert!(
                p.syntax_tree().errors().is_empty(),
                "source {source} produced unexpected errors"
            );
        }
    }

    #[test]
    fn accepts_tuple_and_list_and_map_and_record_and_bitstring_patterns() {
        for source in [
            "{X, Y, 1}",
            "[H | T]",
            "#{key := V}",
            "#user{name = N}",
            "<<A:8, B/binary>>",
        ] {
            let mut p = drive_pattern(source);
            let _ = p.next_top_node().expect("unit");
            assert!(
                p.syntax_tree().errors().is_empty(),
                "source {source} produced unexpected errors"
            );
        }
    }

    #[test]
    fn accepts_match_pattern_x_equals_pat() {
        // `X = {a, B}` is a valid match pattern binding X to the tuple.
        let mut p = drive_pattern("X = {a, B}");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::MatchExpr);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn rejects_call_in_pattern_position() {
        let mut p = drive_pattern("f(1)");
        let _ = p.next_top_node().expect("unit");
        assert!(!p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn rejects_block_expressions_in_pattern_position() {
        for source in [
            "begin 1 end",
            "case X of a -> 1 end",
            "if true -> 1 end",
            "fun (X) -> X end",
        ] {
            let mut p = drive_pattern(source);
            let _ = p.next_top_node().expect("unit");
            assert!(
                !p.syntax_tree().errors().is_empty(),
                "source {source} should have produced an error"
            );
        }
    }

    #[test]
    fn rejects_comprehension_in_pattern_position() {
        let mut p = drive_pattern("[X || X <- L]");
        let _ = p.next_top_node().expect("unit");
        assert!(!p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn rejects_remote_qualifier_and_send_in_pattern_position() {
        let mut p = drive_pattern("mod:foo");
        let _ = p.next_top_node().expect("unit");
        assert!(!p.syntax_tree().errors().is_empty());

        let mut p = drive_pattern("Pid ! msg");
        let _ = p.next_top_node().expect("unit");
        assert!(!p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn rejects_catch_prefix_in_pattern_position() {
        let mut p = drive_pattern("catch 1");
        let _ = p.next_top_node().expect("unit");
        assert!(!p.syntax_tree().errors().is_empty());
    }
}
