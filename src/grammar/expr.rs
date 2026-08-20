//! Expression grammar.
//!
//! Pratt-style expression parser layered on the parser core's cursor and
//! marker primitives. The same core is shared by [`crate::grammar::pattern`],
//! [`crate::grammar::guard`], and [`crate::grammar::term`] with position
//! -specific allowlists layered on top.
//!
//! This module currently implements the "core" surface — literals,
//! parenthesized and container expressions (tuple, list, cons), unary /
//! binary operator applications with proper precedence and associativity,
//! call suffix and remote qualifier. Block expressions, funs,
//! comprehensions, records, bitstrings, and maps are added in follow-up
//! commits.
//!
//! Grammar shape follows OTP 29's `lib/stdlib/src/erl_parse.yrl`.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Exercised by the in-module tests today; wired into the parser and public aux entry points by follow-up commits"
    )
)]

use erl_tokenize::{Symbol, TokenKind};

use crate::error::{Expected, ParseError, ParseErrorKind};
use crate::grammar::operator::{self, Assoc, CALL_LBP, REMOTE_LBP};
use crate::parser::{CompletedMarker, Parser};
use crate::syntax::SyntaxKind;
use crate::token_range::TokenRange;

/// Parses a single expression at the current cursor position.
///
/// The resulting node is always completed (with [`SyntaxKind::Error`] when
/// the input at the cursor cannot be recognized as an expression start).
/// A [`ParseError`] is pushed for each failing site; the cursor still
/// advances at least one lexical token when possible so callers do not
/// spin.
pub(crate) fn parse_expr(p: &mut Parser) -> CompletedMarker {
    parse_expr_bp(p, 0)
}

/// Parses an expression using the Pratt precedence-climbing loop.
///
/// `min_bp` is the minimum left binding power that an infix / suffix
/// operator must exceed to be consumed by this call; a lower-priority
/// operator ends the loop and hands control back to the caller.
fn parse_expr_bp(p: &mut Parser, min_bp: u16) -> CompletedMarker {
    let mut lhs = parse_expr_max(p);
    let mut last_nonassoc_bp: Option<u16> = None;

    while let Some((_, token)) = p.peek_lexical(0) {
        // Infix operator (from the precedence table)?
        if let Some(bp) = operator::infix_binding_power(token) {
            if bp.lbp <= min_bp {
                break;
            }
            if last_nonassoc_bp == Some(bp.lbp) {
                // Two non-associative operators of equal precedence in a
                // row (for example `1 == 2 == 3`); record the error and
                // keep parsing so downstream consumers still see a tree.
                p.push_error(ParseError::new(
                    ParseErrorKind::UnexpectedToken,
                    TokenRange::empty_at(p.cursor_position()),
                    Expected::Category("non-associative operator used twice"),
                    Some(token),
                ));
            }
            let kind = binary_op_kind(token);
            let m = lhs.precede(p);
            p.consume_lexical();
            parse_expr_bp(p, bp.rbp);
            lhs = m.complete(p, kind);
            last_nonassoc_bp = matches!(bp.assoc, Assoc::Nonassoc).then_some(bp.lbp);
            continue;
        }

        // Call suffix `(...)`. `Left 750 '('` in the yrl.
        if is_symbol(token, Symbol::OpenParen) && CALL_LBP > min_bp {
            let m = lhs.precede(p);
            parse_argument_list(p);
            lhs = m.complete(p, SyntaxKind::CallExpr);
            last_nonassoc_bp = None;
            continue;
        }

        // Remote qualifier `Mod : Fun`. `Nonassoc 800 ':'` in the yrl.
        if is_symbol(token, Symbol::Colon) && REMOTE_LBP > min_bp {
            let m = lhs.precede(p);
            p.consume_lexical();
            parse_expr_max(p);
            lhs = m.complete(p, SyntaxKind::RemoteExpr);
            last_nonassoc_bp = None;
            continue;
        }

        break;
    }

    lhs
}

/// Parses a single "maximal" expression: an atomic, prefix-op application,
/// parenthesized expression, or container literal. Corresponds roughly to
/// `expr_max` in OTP 29's yrl and is used wherever the grammar wants a
/// self-delimited expression form (for example on either side of the
/// remote qualifier `:`).
fn parse_expr_max(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    let Some((idx, token)) = p.peek_lexical(0) else {
        p.push_error(ParseError::new(
            ParseErrorKind::UnexpectedEof,
            TokenRange::empty_at(p.cursor_position()),
            Expected::Category("expression"),
            None,
        ));
        return m.complete(p, SyntaxKind::Error);
    };

    // Prefix / unary operator (including `catch` at precedence 0).
    if let Some(rbp) = operator::prefix_binding_power(token) {
        p.consume_lexical();
        parse_expr_bp(p, rbp);
        let kind = if is_keyword(token, erl_tokenize::Keyword::Catch) {
            SyntaxKind::CatchExpr
        } else {
            SyntaxKind::UnaryOpExpr
        };
        return m.complete(p, kind);
    }

    match token.kind() {
        TokenKind::Atom => atomic(p, m, SyntaxKind::AtomExpr),
        TokenKind::Variable => atomic(p, m, SyntaxKind::VarExpr),
        TokenKind::Integer => atomic(p, m, SyntaxKind::IntegerExpr),
        TokenKind::Float => atomic(p, m, SyntaxKind::FloatExpr),
        TokenKind::Char => atomic(p, m, SyntaxKind::CharExpr),
        TokenKind::String => {
            p.consume_lexical();
            while matches!(
                p.peek_lexical(0).map(|(_, t)| t.kind()),
                Some(TokenKind::String)
            ) {
                p.consume_lexical();
            }
            m.complete(p, SyntaxKind::StringExpr)
        }
        TokenKind::SigilString => {
            p.consume_lexical();
            m.complete(p, SyntaxKind::SigilStringExpr)
        }
        TokenKind::Symbol(Symbol::OpenParen) => parse_paren(p, m),
        TokenKind::Symbol(Symbol::OpenBrace) => parse_tuple(p, m),
        TokenKind::Symbol(Symbol::OpenSquare) => parse_list(p, m),
        _ => {
            let _ = idx;
            p.push_error(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                TokenRange::empty_at(p.cursor_position()),
                Expected::Category("expression"),
                Some(token),
            ));
            // Consume one lexical token to guarantee forward progress on
            // unrecognized input.
            let _ = p.consume_lexical();
            m.complete(p, SyntaxKind::Error)
        }
    }
}

fn atomic(p: &mut Parser, m: crate::parser::Marker, kind: SyntaxKind) -> CompletedMarker {
    p.consume_lexical();
    m.complete(p, kind)
}

fn parse_paren(p: &mut Parser, m: crate::parser::Marker) -> CompletedMarker {
    p.consume_lexical(); // `(`
    parse_expr_bp(p, 0);
    expect_symbol(
        p,
        Symbol::CloseParen,
        "`)` to close parenthesized expression",
    );
    m.complete(p, SyntaxKind::ParenExpr)
}

fn parse_tuple(p: &mut Parser, m: crate::parser::Marker) -> CompletedMarker {
    p.consume_lexical(); // `{`
    if at_symbol(p, Symbol::CloseBrace) {
        p.consume_lexical();
        return m.complete(p, SyntaxKind::TupleExpr);
    }
    parse_comma_separated_exprs(p, Symbol::CloseBrace);
    expect_symbol(p, Symbol::CloseBrace, "`}` to close tuple");
    m.complete(p, SyntaxKind::TupleExpr)
}

/// Parses list body starting at the opening `[`. Distinguishes proper
/// lists (`[a, b, c]` and `[]`) from cons form (`[H1, H2, ... | Tail]`)
/// by whether a `|` appears before the closing `]`.
fn parse_list(p: &mut Parser, m: crate::parser::Marker) -> CompletedMarker {
    p.consume_lexical(); // `[`
    if at_symbol(p, Symbol::CloseSquare) {
        p.consume_lexical();
        return m.complete(p, SyntaxKind::ListExpr);
    }
    parse_expr_bp(p, 0);
    let mut has_tail = false;
    loop {
        if at_symbol(p, Symbol::Comma) {
            p.consume_lexical();
            parse_expr_bp(p, 0);
            continue;
        }
        if at_symbol(p, Symbol::VerticalBar) {
            p.consume_lexical();
            has_tail = true;
            parse_expr_bp(p, 0);
            break;
        }
        break;
    }
    expect_symbol(p, Symbol::CloseSquare, "`]` to close list");
    let kind = if has_tail {
        SyntaxKind::ConsExpr
    } else {
        SyntaxKind::ListExpr
    };
    m.complete(p, kind)
}

/// Parses `(Expr, Expr, ...)` including the opening `(` and closing `)`
/// as an [`SyntaxKind::ArgumentList`] node.
fn parse_argument_list(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.consume_lexical(); // `(`
    if at_symbol(p, Symbol::CloseParen) {
        p.consume_lexical();
        return m.complete(p, SyntaxKind::ArgumentList);
    }
    parse_comma_separated_exprs(p, Symbol::CloseParen);
    expect_symbol(p, Symbol::CloseParen, "`)` to close argument list");
    m.complete(p, SyntaxKind::ArgumentList)
}

/// Parses `Expr, Expr, ...` up to but not including `close`. Stops on
/// end-of-input as well; the caller is responsible for the closing
/// delimiter.
fn parse_comma_separated_exprs(p: &mut Parser, close: Symbol) {
    loop {
        parse_expr_bp(p, 0);
        if !at_symbol(p, Symbol::Comma) {
            break;
        }
        p.consume_lexical();
        if at_symbol(p, close) {
            // Trailing comma before the closing delimiter is a syntax
            // error at the yrl level (Erlang does not accept it); record
            // and let the caller close the group.
            p.push_error(ParseError::new(
                ParseErrorKind::UnexpectedToken,
                TokenRange::empty_at(p.cursor_position()),
                Expected::Category("expression after `,`"),
                p.peek_lexical(0).map(|(_, t)| t),
            ));
            break;
        }
    }
}

fn binary_op_kind(token: erl_tokenize::Token) -> SyntaxKind {
    match token.kind() {
        TokenKind::Symbol(Symbol::Match) => SyntaxKind::MatchExpr,
        TokenKind::Symbol(Symbol::Bang) => SyntaxKind::SendExpr,
        _ => SyntaxKind::BinaryOpExpr,
    }
}

// ---------------------------------------------------------------------
// Cursor helpers.
// ---------------------------------------------------------------------

fn at_symbol(p: &Parser, sym: Symbol) -> bool {
    matches!(
        p.peek_lexical(0).map(|(_, t)| t.kind()),
        Some(TokenKind::Symbol(s)) if s == sym
    )
}

fn is_symbol(token: erl_tokenize::Token, sym: Symbol) -> bool {
    matches!(token.kind(), TokenKind::Symbol(s) if s == sym)
}

fn is_keyword(token: erl_tokenize::Token, kw: erl_tokenize::Keyword) -> bool {
    matches!(token.kind(), TokenKind::Keyword(k) if k == kw)
}

fn expect_symbol(p: &mut Parser, sym: Symbol, msg: &'static str) {
    if at_symbol(p, sym) {
        p.consume_lexical();
        return;
    }
    let found = p.peek_lexical(0).map(|(_, t)| t);
    p.push_error(ParseError::new(
        if found.is_some() {
            ParseErrorKind::UnexpectedToken
        } else {
            ParseErrorKind::UnexpectedEof
        },
        TokenRange::empty_at(p.cursor_position()),
        Expected::Category(msg),
        found,
    ));
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

    /// Constructs a parser, pushes every token in `source`, resets the
    /// stub-grammar state so `parse_expr` can drive the cursor from the
    /// beginning, then wraps the result in a synthetic top-level unit so
    /// `next_top_node` returns it.
    ///
    /// Used only by the expression-grammar unit tests until the real
    /// expression-mode wiring is in place.
    fn drive(source: &str) -> Parser {
        let mut p = Parser::new(ParseMode::Expression);
        for t in scan_all(source) {
            p.push_token(t);
        }
        // Reset stub-grammar state; the stub will have consumed the whole
        // buffer as a single Error unit ending at the first `.`.
        p.reset_for_test();
        let outer = p.start();
        parse_expr(&mut p);
        outer.complete(&mut p, SyntaxKind::Error);
        p.finalize_pending_units_for_test();
        p
    }

    fn root_kind(p: &Parser, id: NodeId) -> SyntaxKind {
        p.syntax_tree()
            .syntax()
            .entry(id)
            .expect("entry exists")
            .kind()
    }

    fn first_child_kind(p: &Parser, root: NodeId) -> SyntaxKind {
        // The wrapping `Error` node produced by `drive` has exactly one
        // child: the expression `parse_expr` returned.
        p.syntax_tree()
            .syntax()
            .entry(NodeId::new(root.get() + 1))
            .expect("first child")
            .kind()
    }

    #[test]
    fn parses_atomic_expressions() {
        for (source, kind) in [
            ("foo", SyntaxKind::AtomExpr),
            ("Xyz", SyntaxKind::VarExpr),
            ("123", SyntaxKind::IntegerExpr),
            ("1.5", SyntaxKind::FloatExpr),
            ("$a", SyntaxKind::CharExpr),
            ("\"hi\"", SyntaxKind::StringExpr),
        ] {
            let mut p = drive(source);
            let root = p.next_top_node().expect("unit");
            assert_eq!(first_child_kind(&p, root), kind, "source {source}");
            assert!(p.syntax_tree().errors().is_empty(), "source {source}");
        }
    }

    #[test]
    fn concatenates_adjacent_string_tokens() {
        let mut p = drive("\"foo\" \"bar\"");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::StringExpr);
    }

    #[test]
    fn parses_parenthesized_expression() {
        let mut p = drive("(1)");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::ParenExpr);
    }

    #[test]
    fn parses_empty_and_populated_tuples() {
        let mut p = drive("{}");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::TupleExpr);

        let mut p = drive("{a, 1, X}");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::TupleExpr);
    }

    #[test]
    fn parses_proper_list_and_cons() {
        let mut p = drive("[]");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::ListExpr);

        let mut p = drive("[a, b, c]");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::ListExpr);

        let mut p = drive("[H | T]");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::ConsExpr);
    }

    #[test]
    fn precedence_matches_yrl() {
        // 1 + 2 * 3 → BinaryOp(1, +, BinaryOp(2, *, 3))
        let mut p = drive("1 + 2 * 3");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::BinaryOpExpr);
        // Second child = 1 (IntegerExpr), third = another BinaryOp.
        let syntax = p.syntax_tree().syntax();
        let outer = syntax
            .entry(NodeId::new(root.get() + 1))
            .expect("outer binop");
        // Outer binop's inner children start at root+2.
        let left = syntax
            .entry(NodeId::new(root.get() + 2))
            .expect("left operand");
        assert_eq!(left.kind(), SyntaxKind::IntegerExpr);
        let right = syntax
            .entry(NodeId::new(outer.subtree_end().get() - 3))
            .expect("right operand (inner binop)");
        assert_eq!(right.kind(), SyntaxKind::BinaryOpExpr);
    }

    #[test]
    fn match_operator_uses_match_expr_kind() {
        let mut p = drive("X = 1");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::MatchExpr);
    }

    #[test]
    fn send_operator_uses_send_expr_kind() {
        let mut p = drive("Pid ! msg");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::SendExpr);
    }

    #[test]
    fn call_wraps_target_expression() {
        let mut p = drive("f(1, 2)");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::CallExpr);
    }

    #[test]
    fn remote_call_chain_wraps_remote_then_call() {
        let mut p = drive("mod:fun(1)");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::CallExpr);
        // Second entry after the CallExpr is the RemoteExpr target.
        let syntax = p.syntax_tree().syntax();
        let inside = syntax
            .entry(NodeId::new(root.get() + 2))
            .expect("call target");
        assert_eq!(inside.kind(), SyntaxKind::RemoteExpr);
    }

    #[test]
    fn unary_prefix_operator_produces_unary_op_expr() {
        let mut p = drive("-1");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::UnaryOpExpr);
    }

    #[test]
    fn catch_prefix_operator_produces_catch_expr() {
        let mut p = drive("catch 1");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::CatchExpr);
    }

    #[test]
    fn nonassoc_comparison_chain_records_error_but_keeps_progress() {
        let mut p = drive("1 == 2 == 3");
        let _ = p.next_top_node().expect("unit");
        assert!(
            !p.syntax_tree().errors().is_empty(),
            "expected an error for `1 == 2 == 3`"
        );
    }

    #[test]
    fn unexpected_token_produces_error_node_and_still_returns_unit() {
        // A bare `)` cannot start an expression.
        let mut p = drive(")");
        let root = p.next_top_node().expect("unit");
        assert_eq!(root_kind(&p, root), SyntaxKind::Error);
        assert!(!p.syntax_tree().errors().is_empty());
    }
}
