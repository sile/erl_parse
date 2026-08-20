//! Clause / body / guard grammar shared by block expressions and
//! function declarations.
//!
//! Block expressions (`case`, `if`, `receive`, `try`, `fun`, `maybe`)
//! and future function-declaration grammar (in the form / module family)
//! all group work into clauses whose shape is `Args [when Guard] ->
//! Body`. This module owns the shared implementation and exposes
//! `pub(crate)` helpers so both the block-expression parser and the
//! form / module grammar can reuse it without duplicating clause / body
//! / guard parsing.
//!
//! Pattern positions inside clause heads route through
//! [`crate::grammar::pattern::parse_pattern`] so illegal expression
//! forms (calls, blocks, and other general expressions) are rejected
//! in pattern position while sharing the same node shape.

use erl_tokenize::{Keyword, Symbol, TokenKind};

use crate::grammar::expr::{parse_comma_separated_exprs, parse_expr, parse_expr_max};
use crate::grammar::pattern::parse_pattern;
use crate::grammar::util::{at_keyword, at_symbol, consume_atom_or_var, expect_symbol};
use crate::parser::{CompletedMarker, Parser};
use crate::syntax::SyntaxKind;

/// Parses a comma-separated list of expressions as a
/// [`SyntaxKind::Body`] node. Corresponds to `exprs` in OTP 29's yrl.
pub(crate) fn parse_body(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    parse_exprs_comma(p);
    m.complete(p, SyntaxKind::Body)
}

/// Parses a `-> Body` sequence: consumes the arrow, then parses a
/// body. Used by every clause form.
pub(crate) fn parse_arrow_body(p: &mut Parser) -> CompletedMarker {
    expect_symbol(p, Symbol::RightArrow, "`->` before clause body");
    parse_body(p)
}

/// Parses a single guard (comma-separated guard expressions) as a
/// [`SyntaxKind::Guard`] node.
pub(crate) fn parse_guard(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    parse_exprs_comma(p);
    m.complete(p, SyntaxKind::Guard)
}

/// Parses a full guard sequence: one or more guards separated by `;`.
/// Wraps the whole thing as a [`SyntaxKind::GuardSequence`] node.
pub(crate) fn parse_guard_sequence(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    parse_guard(p);
    while at_symbol(p, Symbol::Semicolon) {
        p.consume_lexical();
        parse_guard(p);
    }
    m.complete(p, SyntaxKind::GuardSequence)
}

/// Parses `[when GuardSequence]`, or nothing when `when` is not next.
pub(crate) fn parse_clause_guard_opt(p: &mut Parser) {
    if at_keyword(p, Keyword::When) {
        p.consume_lexical();
        parse_guard_sequence(p);
    }
}

/// Parses a `case`- / `receive`- style clause: `Pattern [when Guard]
/// -> Body`. Wraps the result as a [`SyntaxKind::Clause`] node.
pub(crate) fn parse_case_clause(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    parse_pattern(p);
    parse_clause_guard_opt(p);
    parse_arrow_body(p);
    m.complete(p, SyntaxKind::Clause)
}

/// Parses an `if`-clause: `Guard -> Body`. Wraps the result as a
/// [`SyntaxKind::IfClause`] node.
pub(crate) fn parse_if_clause(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    parse_guard_sequence(p);
    parse_arrow_body(p);
    m.complete(p, SyntaxKind::IfClause)
}

/// Parses a `try`-block catch-clause. Accepts both the pattern-only
/// form (`Pat [when Guard] -> Body`, wrapped as [`SyntaxKind::Clause`])
/// and the class-qualified form
/// (`Class : Reason [: Stack] [when Guard] -> Body`, wrapped as
/// [`SyntaxKind::CatchClause`]).
///
/// The class-qualified form is disambiguated by looking for `:` after
/// the first expression, which mirrors how OTP's yrl distinguishes
/// `try_clause` productions.
pub(crate) fn parse_try_clause(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    // Distinguish the class-qualified form `Class : Reason [: Stack]`
    // from a plain pattern by peeking: the yrl rule requires an atom
    // or variable followed immediately by `:` before it commits to
    // the class-qualified production.
    if is_class_qualified_try_clause_head(p) {
        // Class : Reason [: Stack]. Class is an atom or variable per
        // the yrl. Reason is a pat_expr — parsed via `parse_expr_max`
        // under Pattern context so it does not consume the following
        // `:` as a remote qualifier. Stack, when present, is a plain
        // variable.
        consume_atom_or_var(p, "class name in catch clause");
        p.consume_lexical(); // `:`
        let prev = p.set_context(crate::parser::ParseContext::Pattern);
        parse_expr_max(p);
        p.set_context(prev);
        if at_symbol(p, Symbol::Colon) {
            p.consume_lexical();
            consume_atom_or_var(p, "stack-trace variable");
        }
        parse_clause_guard_opt(p);
        parse_arrow_body(p);
        m.complete(p, SyntaxKind::CatchClause)
    } else {
        parse_pattern(p);
        parse_clause_guard_opt(p);
        parse_arrow_body(p);
        m.complete(p, SyntaxKind::Clause)
    }
}

fn is_class_qualified_try_clause_head(p: &Parser) -> bool {
    matches!(
        p.peek_lexical(0).map(|(_, t)| t.kind()),
        Some(TokenKind::Atom | TokenKind::Variable)
    ) && matches!(
        p.peek_lexical(1).map(|(_, t)| t.kind()),
        Some(TokenKind::Symbol(Symbol::Colon))
    )
}

/// Parses one or more of `production` separated by `;`.
///
/// Between iterations, if the cursor is not on `;` or on one of the
/// clause-block terminator keywords (`end` / `after` / `catch` /
/// `else`) and is not at the outer `.` boundary, recovery kicks in
/// under [`crate::parser::RecoveryContext::Clause`] via
/// [`crate::grammar::recovery::skip_until_sync`]: the skipped span
/// becomes a [`SyntaxKind::Error`] node with a matching
/// [`crate::error::ParseErrorKind::SkippedToken`] diagnostic and
/// the loop resumes from the sync token. This keeps a garbled
/// clause head from swallowing the rest of the block.
pub(crate) fn parse_semicolon_separated<F>(p: &mut Parser, mut production: F)
where
    F: FnMut(&mut Parser) -> CompletedMarker,
{
    production(p);
    loop {
        if !at_symbol(p, Symbol::Semicolon) && !at_clause_boundary(p) {
            let _ = crate::grammar::recovery::skip_until_sync(
                p,
                crate::parser::RecoveryContext::Clause,
                is_clause_boundary,
                "`;` or block terminator",
            );
        }
        if !at_symbol(p, Symbol::Semicolon) {
            break;
        }
        p.consume_lexical();
        production(p);
    }
}

fn at_clause_boundary(p: &Parser) -> bool {
    p.peek_lexical(0)
        .map(|(_, t)| is_clause_boundary(t))
        .unwrap_or(true)
}

fn is_clause_boundary(token: erl_tokenize::Token) -> bool {
    use erl_tokenize::{Keyword, Symbol, TokenKind};
    matches!(
        token.kind(),
        TokenKind::Symbol(Symbol::Semicolon)
            | TokenKind::Symbol(Symbol::Dot)
            | TokenKind::Keyword(Keyword::End)
            | TokenKind::Keyword(Keyword::After)
            | TokenKind::Keyword(Keyword::Catch)
            | TokenKind::Keyword(Keyword::Else)
    )
}

/// Parses `( [Expr, Expr, ...] )` as a [`SyntaxKind::ArgumentList`]
/// node, consuming both delimiters. Shared between the expression
/// call suffix and the fun-clause argument position; will also be
/// reused by function declarations in the form / module grammar.
pub(crate) fn parse_argument_list(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    expect_symbol(p, Symbol::OpenParen, "`(` to open argument list");
    if at_symbol(p, Symbol::CloseParen) {
        p.consume_lexical();
        return m.complete(p, SyntaxKind::ArgumentList);
    }
    parse_comma_separated_exprs(p, Symbol::CloseParen);
    expect_symbol(p, Symbol::CloseParen, "`)` to close argument list");
    m.complete(p, SyntaxKind::ArgumentList)
}

fn parse_exprs_comma(p: &mut Parser) {
    parse_expr(p);
    while at_symbol(p, Symbol::Comma) {
        p.consume_lexical();
        parse_expr(p);
    }
}
