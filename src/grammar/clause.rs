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
//! Pattern positions inside clause heads currently reuse
//! [`crate::grammar::expr::parse_expr`]; a follow-up commit lays the
//! pattern-restriction pass on top so illegal expression forms (calls,
//! blocks, and other general expressions) are rejected in pattern
//! position while sharing the same node shape.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Exercised by the block-expression parser and, in later commits, by function-declaration grammar; only in-crate tests currently drive some of these directly"
    )
)]

use erl_tokenize::{Keyword, Symbol};

use crate::grammar::expr::parse_expr;
use crate::grammar::util::{at_keyword, at_symbol, expect_symbol};
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
    // Pattern position — see the module doc: currently reuses
    // `parse_expr` and defers the pattern-restriction pass.
    parse_expr(p);
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
    parse_expr(p);
    if at_symbol(p, Symbol::Colon) {
        // `Class : Reason [: Stack]`.
        p.consume_lexical();
        parse_expr(p);
        if at_symbol(p, Symbol::Colon) {
            p.consume_lexical();
            parse_expr(p);
        }
        parse_clause_guard_opt(p);
        parse_arrow_body(p);
        m.complete(p, SyntaxKind::CatchClause)
    } else {
        parse_clause_guard_opt(p);
        parse_arrow_body(p);
        m.complete(p, SyntaxKind::Clause)
    }
}

/// Parses one or more of `production` separated by `;`.
pub(crate) fn parse_semicolon_separated<F>(p: &mut Parser, mut production: F)
where
    F: FnMut(&mut Parser) -> CompletedMarker,
{
    production(p);
    while at_symbol(p, Symbol::Semicolon) {
        p.consume_lexical();
        production(p);
    }
}

fn parse_exprs_comma(p: &mut Parser) {
    parse_expr(p);
    while at_symbol(p, Symbol::Comma) {
        p.consume_lexical();
        parse_expr(p);
    }
}
