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

use erl_tokenize::{Keyword, Symbol, TokenKind};

use crate::error::{Expected, ParseError, ParseErrorKind};
use crate::grammar::clause::{
    parse_argument_list, parse_arrow_body, parse_body, parse_case_clause, parse_clause_guard_opt,
    parse_if_clause, parse_semicolon_separated, parse_try_clause,
};
use crate::grammar::operator::{self, Assoc, CALL_LBP, REMOTE_LBP};
use crate::grammar::util::{
    at_keyword, at_symbol, expect_keyword, expect_symbol, is_keyword, is_symbol,
};
use crate::parser::{CompletedMarker, Marker, Parser};
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
        TokenKind::Keyword(Keyword::Begin) => parse_begin(p, m),
        TokenKind::Keyword(Keyword::Case) => parse_case(p, m),
        TokenKind::Keyword(Keyword::If) => parse_if(p, m),
        TokenKind::Keyword(Keyword::Receive) => parse_receive(p, m),
        TokenKind::Keyword(Keyword::Try) => parse_try(p, m),
        TokenKind::Keyword(Keyword::Maybe) => parse_maybe(p, m),
        TokenKind::Keyword(Keyword::Fun) => parse_fun(p, m),
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

/// Parses `Expr, Expr, ...` up to but not including `close`. Stops on
/// end-of-input as well; the caller is responsible for the closing
/// delimiter.
pub(crate) fn parse_comma_separated_exprs(p: &mut Parser, close: Symbol) {
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
        TokenKind::Symbol(Symbol::MaybeMatch) => SyntaxKind::MaybeMatchExpr,
        _ => SyntaxKind::BinaryOpExpr,
    }
}

// ---------------------------------------------------------------------
// Block expressions.
// ---------------------------------------------------------------------

/// `begin Exprs end`.
fn parse_begin(p: &mut Parser, m: Marker) -> CompletedMarker {
    p.consume_lexical(); // `begin`
    parse_body(p);
    expect_keyword(p, Keyword::End, "`end` to close `begin`");
    m.complete(p, SyntaxKind::BeginExpr)
}

/// `case Expr of Clause; Clause; ... end`.
fn parse_case(p: &mut Parser, m: Marker) -> CompletedMarker {
    p.consume_lexical(); // `case`
    parse_expr(p);
    expect_keyword(p, Keyword::Of, "`of` in `case` expression");
    parse_semicolon_separated(p, parse_case_clause);
    expect_keyword(p, Keyword::End, "`end` to close `case`");
    m.complete(p, SyntaxKind::CaseExpr)
}

/// `if Guard -> Body ; Guard -> Body ; ... end`.
fn parse_if(p: &mut Parser, m: Marker) -> CompletedMarker {
    p.consume_lexical(); // `if`
    parse_semicolon_separated(p, parse_if_clause);
    expect_keyword(p, Keyword::End, "`end` to close `if`");
    m.complete(p, SyntaxKind::IfExpr)
}

/// `receive Clauses [after Expr -> Body] end` or `receive after Expr ->
/// Body end` when no message clauses are given.
fn parse_receive(p: &mut Parser, m: Marker) -> CompletedMarker {
    p.consume_lexical(); // `receive`
    if !at_keyword(p, Keyword::After) && !at_keyword(p, Keyword::End) {
        parse_semicolon_separated(p, parse_case_clause);
    }
    if at_keyword(p, Keyword::After) {
        p.consume_lexical();
        parse_expr(p);
        parse_arrow_body(p);
    }
    expect_keyword(p, Keyword::End, "`end` to close `receive`");
    m.complete(p, SyntaxKind::ReceiveExpr)
}

/// `try Body [of Clauses] [catch CatchClauses] [after AfterBody] end`.
///
/// The `catch` and `after` sections are individually optional but at
/// least one of the two must appear in valid Erlang. This parser
/// accepts either or both without enforcing that either is present;
/// error-recovery contracts tighten this in a later change.
fn parse_try(p: &mut Parser, m: Marker) -> CompletedMarker {
    p.consume_lexical(); // `try`
    parse_body(p);
    if at_keyword(p, Keyword::Of) {
        p.consume_lexical();
        parse_semicolon_separated(p, parse_case_clause);
    }
    if at_keyword(p, Keyword::Catch) {
        p.consume_lexical();
        parse_semicolon_separated(p, parse_try_clause);
    }
    if at_keyword(p, Keyword::After) {
        p.consume_lexical();
        parse_body(p);
    }
    expect_keyword(p, Keyword::End, "`end` to close `try`");
    m.complete(p, SyntaxKind::TryExpr)
}

/// `maybe Body [else Clauses] end`. `?=` inside the body is handled by
/// the shared infix table (see [`crate::grammar::operator`]) and
/// materialises as [`SyntaxKind::MaybeMatchExpr`].
fn parse_maybe(p: &mut Parser, m: Marker) -> CompletedMarker {
    p.consume_lexical(); // `maybe`
    parse_body(p);
    if at_keyword(p, Keyword::Else) {
        p.consume_lexical();
        parse_semicolon_separated(p, parse_case_clause);
    }
    expect_keyword(p, Keyword::End, "`end` to close `maybe`");
    m.complete(p, SyntaxKind::MaybeExpr)
}

// ---------------------------------------------------------------------
// Fun expressions and references.
// ---------------------------------------------------------------------

/// Dispatches on the shape of the tokens following `fun`:
///
/// - `fun (` → anonymous fun with clauses.
/// - `fun Name /` → [`SyntaxKind::LocalFunRef`] (`Name` is an atom in
///   valid Erlang; a variable is accepted at the syntax layer and left
///   for a semantic pass to reject).
/// - `fun Mod :` → [`SyntaxKind::RemoteFunRef`] (module, name, and arity
///   may each be an atom / var / integer).
/// - `fun Name (` → named fun (`Name` is a variable in valid Erlang;
///   atom accepted at the syntax layer for the same reason).
fn parse_fun(p: &mut Parser, m: Marker) -> CompletedMarker {
    p.consume_lexical(); // `fun`

    if at_symbol(p, Symbol::OpenParen) {
        parse_semicolon_separated(p, parse_fun_clause);
        expect_keyword(p, Keyword::End, "`end` to close `fun`");
        return m.complete(p, SyntaxKind::AnonymousFun);
    }

    // Decide between LocalFunRef / RemoteFunRef / NamedFun by the token
    // that follows the head atom / variable.
    let first = p.peek_lexical(0);
    let second = p.peek_lexical(1);
    match (first.map(|(_, t)| t.kind()), second.map(|(_, t)| t.kind())) {
        (Some(TokenKind::Atom | TokenKind::Variable), Some(TokenKind::Symbol(Symbol::Slash))) => {
            parse_local_fun_ref(p, m)
        }
        (Some(TokenKind::Atom | TokenKind::Variable), Some(TokenKind::Symbol(Symbol::Colon))) => {
            parse_remote_fun_ref(p, m)
        }
        (
            Some(TokenKind::Atom | TokenKind::Variable),
            Some(TokenKind::Symbol(Symbol::OpenParen)),
        ) => parse_named_fun(p, m),
        _ => {
            let found = first.map(|(_, t)| t);
            p.push_error(ParseError::new(
                if found.is_some() {
                    ParseErrorKind::UnexpectedToken
                } else {
                    ParseErrorKind::UnexpectedEof
                },
                TokenRange::empty_at(p.cursor_position()),
                Expected::Category("`(`, fun reference, or named fun after `fun`"),
                found,
            ));
            m.complete(p, SyntaxKind::Error)
        }
    }
}

fn parse_local_fun_ref(p: &mut Parser, m: Marker) -> CompletedMarker {
    consume_atom_or_var(p, "fun name (atom or variable)");
    expect_symbol(p, Symbol::Slash, "`/` in fun reference");
    consume_integer_or_var(p, "arity (integer or variable)");
    m.complete(p, SyntaxKind::LocalFunRef)
}

fn parse_remote_fun_ref(p: &mut Parser, m: Marker) -> CompletedMarker {
    consume_atom_or_var(p, "module name (atom or variable)");
    expect_symbol(p, Symbol::Colon, "`:` in remote fun reference");
    consume_atom_or_var(p, "function name (atom or variable)");
    expect_symbol(p, Symbol::Slash, "`/` in remote fun reference");
    consume_integer_or_var(p, "arity (integer or variable)");
    m.complete(p, SyntaxKind::RemoteFunRef)
}

fn parse_named_fun(p: &mut Parser, m: Marker) -> CompletedMarker {
    parse_semicolon_separated(p, parse_named_fun_clause);
    expect_keyword(p, Keyword::End, "`end` to close named `fun`");
    m.complete(p, SyntaxKind::NamedFun)
}

/// Anonymous fun clause: `(Args) [when Guard] -> Body`. Wraps as a
/// [`SyntaxKind::Clause`] node (same shape as case/receive clauses).
fn parse_fun_clause(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    parse_argument_list(p);
    parse_clause_guard_opt(p);
    parse_arrow_body(p);
    m.complete(p, SyntaxKind::Clause)
}

/// Named fun clause: `Name (Args) [when Guard] -> Body`.
fn parse_named_fun_clause(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    consume_atom_or_var(p, "named-fun name (variable or atom)");
    parse_argument_list(p);
    parse_clause_guard_opt(p);
    parse_arrow_body(p);
    m.complete(p, SyntaxKind::Clause)
}

fn consume_atom_or_var(p: &mut Parser, msg: &'static str) {
    match p.peek_lexical(0).map(|(_, t)| t.kind()) {
        Some(TokenKind::Atom | TokenKind::Variable) => {
            p.consume_lexical();
        }
        _ => {
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
    }
}

fn consume_integer_or_var(p: &mut Parser, msg: &'static str) {
    match p.peek_lexical(0).map(|(_, t)| t.kind()) {
        Some(TokenKind::Integer | TokenKind::Variable) => {
            p.consume_lexical();
        }
        _ => {
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
    }
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
        // `mod:foo(1)`; `foo` is a plain atom, not the `fun` keyword.
        let mut p = drive("mod:foo(1)");
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

    // -----------------------------------------------------------------
    // Block expressions.
    // -----------------------------------------------------------------

    #[test]
    fn parses_begin_block() {
        let mut p = drive("begin 1, 2, 3 end");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::BeginExpr);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn parses_case_expression_with_two_clauses() {
        let mut p = drive("case X of a -> 1; b -> 2 end");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::CaseExpr);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn parses_if_expression_with_guard_sequence() {
        let mut p = drive("if X > 0 -> pos; X < 0 -> neg; true -> zero end");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::IfExpr);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn parses_receive_with_after() {
        let mut p = drive("receive msg -> ok after 1000 -> timeout end");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::ReceiveExpr);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn parses_receive_after_only() {
        let mut p = drive("receive after 0 -> ok end");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::ReceiveExpr);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn parses_try_with_catch_class_reason_stack() {
        let mut p = drive("try foo() of X -> X catch error:Reason:Stack -> Stack end");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::TryExpr);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn parses_try_with_after_only() {
        let mut p = drive("try do_thing() after cleanup() end");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::TryExpr);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn parses_maybe_with_maybe_match_and_else() {
        let mut p = drive("maybe {ok, X} ?= foo() else error:E -> E end");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::MaybeExpr);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn maybe_match_operator_uses_maybe_match_expr_kind() {
        let mut p = drive("maybe X ?= foo() end");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::MaybeExpr);
        // Search for the MaybeMatchExpr node inside the tree.
        let syntax = p.syntax_tree().syntax();
        let has_maybe_match = (0..syntax.len()).any(|i| {
            syntax.entry(NodeId::new(i)).expect("entry exists").kind() == SyntaxKind::MaybeMatchExpr
        });
        assert!(has_maybe_match, "expected a MaybeMatchExpr node");
    }

    // -----------------------------------------------------------------
    // Fun expressions and references.
    // -----------------------------------------------------------------

    #[test]
    fn parses_anonymous_fun_single_clause() {
        let mut p = drive("fun (X) -> X + 1 end");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::AnonymousFun);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn parses_anonymous_fun_with_multiple_clauses_and_guard() {
        let mut p = drive("fun (0) -> zero; (N) when N > 0 -> N end");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::AnonymousFun);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn parses_named_fun() {
        let mut p = drive("fun Loop(0) -> ok; Loop(N) -> Loop(N - 1) end");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::NamedFun);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn parses_local_fun_ref() {
        let mut p = drive("fun foo/2");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::LocalFunRef);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn parses_remote_fun_ref_with_concrete_names() {
        let mut p = drive("fun mod:fun_name/3");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::RemoteFunRef);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn parses_remote_fun_ref_with_variables() {
        let mut p = drive("fun M:F/N");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::RemoteFunRef);
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn empty_argument_list_in_fun_clause_is_accepted() {
        let mut p = drive("fun () -> ok end");
        let root = p.next_top_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::AnonymousFun);
        assert!(p.syntax_tree().errors().is_empty());
    }
}
