//! Guard grammar.
//!
//! Parses guard sequences (`Guard1 ; Guard2 ; ...`), each guard being a
//! comma-separated list of guard expressions. Structure only: whether a
//! given call is a guard BIF, whether a remote call is legal, and
//! whether operator operands are valid are semantic concerns handled by
//! an `erl_lint`-style pass, not by this parser.
//!
//! The grammar reuses [`crate::grammar::expr::parse_expr`] for
//! individual guard expressions; the parser stays in
//! [`ParseContext::Expression`] because the syntactic surface of a
//! guard expression is the same as a general expression at this level.
//! Legality of the operations that appear (guard BIF vs arbitrary call,
//! for example) is left to the semantic phase. The productions live in
//! [`crate::grammar::clause`] so block expressions and function
//! declarations share one implementation.

#[cfg(test)]
mod tests {
    use crate::grammar::clause::parse_guard_sequence;
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

    fn drive_guard(source: &str) -> Parser {
        let mut p = Parser::new(ParseMode::Module);
        for t in scan_all(source) {
            p.feed_token_without_grammar_for_test(t);
        }
        p.reset_for_test();
        let outer = p.start();
        parse_guard_sequence(&mut p);
        outer.complete(&mut p, SyntaxKind::Error);
        p.finalize_pending_units_for_test();
        p
    }

    fn tree_contains_kind(p: &Parser, kind: SyntaxKind) -> bool {
        let syntax = p.syntax_tree().syntax();
        (0..syntax.len())
            .any(|i| syntax.entry(NodeId::new(i)).expect("entry exists").kind() == kind)
    }

    #[test]
    fn parses_simple_guard_sequence() {
        let mut p = drive_guard("X > 0, X < 10");
        let _ = p.next_node().expect("unit");
        assert!(tree_contains_kind(&p, SyntaxKind::GuardSequence));
        assert!(tree_contains_kind(&p, SyntaxKind::Guard));
    }

    #[test]
    fn parses_semicolon_separated_guards() {
        // `X > 0, X < 10 ; Y == a` — one sequence containing two guards.
        let mut p = drive_guard("X > 0, X < 10 ; Y == a");
        let _ = p.next_node().expect("unit");
        let syntax = p.syntax_tree().syntax();
        let guard_count = (0..syntax.len())
            .filter(|i| {
                syntax.entry(NodeId::new(*i)).expect("entry exists").kind() == SyntaxKind::Guard
            })
            .count();
        assert_eq!(guard_count, 2);
    }

    #[test]
    fn accepts_calls_and_remote_calls_in_guard_position() {
        // Guard grammar is purely structural; semantic BIF checks live
        // elsewhere. Both `is_atom(X)` and `erlang:is_atom(X)` parse
        // without errors here.
        let mut p = drive_guard("is_atom(X)");
        let _ = p.next_node().expect("unit");
        assert!(p.syntax_tree().diagnostics().is_empty());

        let mut p = drive_guard("erlang:is_atom(X)");
        let _ = p.next_node().expect("unit");
        assert!(p.syntax_tree().diagnostics().is_empty());
    }
}
