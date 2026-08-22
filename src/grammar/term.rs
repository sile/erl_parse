//! Term grammar.
//!
//! Recognises the token structure that Erlang treats as a term literal:
//! atoms, numbers, chars, strings, binary literals, tuples, proper /
//! improper lists, map literals, and unary sign prefixes. Variables,
//! calls, blocks, record literals, and bound identifiers are rejected,
//! aligning with what `file:consult/1` accepts via
//! `erl_parse:parse_term/1` → `erl_parse:normalise/1`.
//!
//! Rust-value conversion, Erlang term evaluation, and OTP Abstract
//! Format normalisation are out of scope.
//!
//! Implementation: reuses [`crate::grammar::expr::parse_expr`] with the
//! parser switched into [`ParseContext::Term`]. Restrictions specific
//! to term position (no variables, no `=` match, no records) are
//! enforced by the shared expression parser through structured
//! [`Diagnostic`][crate::Diagnostic]s.

use crate::grammar::expr::parse_expr;
use crate::parser::{CompletedMarker, ParseContext, Parser};

/// Parses a single term at the current cursor position.
pub(crate) fn parse_term(p: &mut Parser) -> CompletedMarker {
    let prev = p.set_context(ParseContext::Term);
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

    fn drive_term(source: &str) -> Parser {
        let mut p = Parser::new(ParseMode::Module);
        for t in scan_all(source) {
            p.push_token_without_grammar_for_test(t);
        }
        p.reset_for_test();
        let outer = p.start();
        parse_term(&mut p);
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
    fn accepts_literal_terms() {
        for (source, kind) in [
            ("foo", SyntaxKind::AtomExpr),
            ("42", SyntaxKind::IntegerExpr),
            ("1.5", SyntaxKind::FloatExpr),
            ("$a", SyntaxKind::CharExpr),
            ("\"hi\"", SyntaxKind::StringExpr),
            ("{a, 1}", SyntaxKind::TupleExpr),
            ("[a, b, c]", SyntaxKind::ListExpr),
            ("[1 | []]", SyntaxKind::ConsExpr),
            ("#{a => 1}", SyntaxKind::MapExpr),
            ("<<1, 2>>", SyntaxKind::BitstringExpr),
            ("-1", SyntaxKind::UnaryOpExpr),
        ] {
            let mut p = drive_term(source);
            let root = p.next_node().expect("unit");
            assert_eq!(first_child_kind(&p, root), kind, "source {source}");
            assert!(
                p.syntax_tree().diagnostics().is_empty(),
                "source {source} produced unexpected errors"
            );
        }
    }

    #[test]
    fn rejects_variables_in_term_position() {
        let mut p = drive_term("X");
        let _ = p.next_node().expect("unit");
        assert!(!p.syntax_tree().diagnostics().is_empty());
    }

    #[test]
    fn rejects_match_in_term_position() {
        let mut p = drive_term("X = 1");
        let _ = p.next_node().expect("unit");
        assert!(!p.syntax_tree().diagnostics().is_empty());
    }

    #[test]
    fn rejects_call_and_block_and_fun_in_term_position() {
        for source in [
            "foo(1)",
            "begin 1 end",
            "case a of a -> 1 end",
            "fun (X) -> X end",
        ] {
            let mut p = drive_term(source);
            let _ = p.next_node().expect("unit");
            assert!(
                !p.syntax_tree().diagnostics().is_empty(),
                "source {source} should have produced an error"
            );
        }
    }

    #[test]
    fn rejects_remote_qualifier_in_term_position() {
        let mut p = drive_term("mod:foo");
        let _ = p.next_node().expect("unit");
        assert!(!p.syntax_tree().diagnostics().is_empty());
    }
}
