//! Function declaration grammar.
//!
//! Parses `Name(Args) [when Guard] -> Body;
//! Name(Args) [when Guard] -> Body; ...` up to (but not including)
//! the terminating `.`. The module-mode top-level driver consumes
//! the boundary `.` afterwards.
//!
//! Clause-internal grouping (argument list, guard, body) reuses the
//! shared helpers in [`crate::grammar::clause`], so the clause node
//! shape matches the block-expression clause shape. Only
//! the top-level grouping ([`SyntaxKind::FunctionClause`] wrapper,
//! `;`-separated sequence, [`SyntaxKind::FunctionDecl`] wrapper) is
//! new here.
//!
//! Same-name / same-arity checks and cross-form same-name grouping
//! belong to a later semantic phase — the parser only counts clauses
//! and records the current clause's name-atom range and argument-list
//! arity in [`crate::InProgressState`] so callers can key off them.

use erl_tokenize::{Symbol, TokenKind};

use crate::error::{Expected, ParseError, ParseErrorKind};
use crate::grammar::clause::{parse_argument_list, parse_arrow_body, parse_clause_guard_opt};
use crate::grammar::util::at_symbol;
use crate::parser::{CompletedMarker, FormKind, Parser};
use crate::syntax::SyntaxKind;
use crate::token_range::TokenRange;

/// Parses one function declaration starting at the name atom.
pub(crate) fn parse_function_decl(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.in_progress_mut().form_kind = Some(FormKind::FunctionDecl);
    p.in_progress_mut().current_clause = Some(1);

    parse_function_clause(p);
    while at_symbol(p, Symbol::Semicolon) {
        p.consume_lexical();
        let next_clause = p.in_progress_mut().current_clause.unwrap_or(1) + 1;
        p.in_progress_mut().current_clause = Some(next_clause);
        parse_function_clause(p);
    }

    let completed = m.complete(p, SyntaxKind::FunctionDecl);
    let state = p.in_progress_mut();
    state.function_name = None;
    state.function_arity = None;
    state.current_clause = None;
    completed
}

fn parse_function_clause(p: &mut Parser) -> CompletedMarker {
    let m = p.start();

    let name_start = p.cursor_position();
    if matches!(
        p.peek_lexical(0).map(|(_, t)| t.kind()),
        Some(TokenKind::Atom)
    ) {
        p.consume_lexical();
        let name_end = p.cursor_position();
        p.in_progress_mut().function_name = Some(TokenRange::new(name_start, name_end));
    } else {
        let found = p.peek_lexical(0).map(|(_, t)| t);
        p.push_error(ParseError::new(
            if found.is_some() {
                ParseErrorKind::UnexpectedToken
            } else {
                ParseErrorKind::UnexpectedEof
            },
            TokenRange::empty_at(name_start),
            Expected::Category("function name (atom) at the start of a function clause"),
            found,
        ));
    }

    let arity = peek_arity(p);
    parse_argument_list(p);
    p.in_progress_mut().function_arity = Some(arity);

    parse_clause_guard_opt(p);
    parse_arrow_body(p);

    m.complete(p, SyntaxKind::FunctionClause)
}

/// Counts the arguments in the argument list starting at the cursor
/// without consuming tokens. Assumes the cursor is at the `(` that
/// opens the list. Returns `0` for an empty list, otherwise
/// `top_level_commas + 1`. Nested `()`, `{}`, `[]`, and `<<>>` are
/// tracked so that a comma inside a nested group does not inflate
/// the count. Falls back to `0` on unterminated input; the malformed
/// list will surface a `ParseError` via [`parse_argument_list`].
fn peek_arity(p: &Parser) -> usize {
    let Some((_, open)) = p.peek_lexical(0) else {
        return 0;
    };
    if !matches!(open.kind(), TokenKind::Symbol(Symbol::OpenParen)) {
        return 0;
    }
    let mut i: usize = 1;
    let mut depth_paren: usize = 1;
    let mut depth_brace: usize = 0;
    let mut depth_square: usize = 0;
    let mut depth_binary: usize = 0;
    let mut commas: usize = 0;
    let mut saw_arg_token: bool = false;
    while let Some((_, t)) = p.peek_lexical(i) {
        i += 1;
        match t.kind() {
            TokenKind::Symbol(Symbol::OpenParen) => {
                depth_paren += 1;
                saw_arg_token = true;
            }
            TokenKind::Symbol(Symbol::CloseParen) => {
                depth_paren -= 1;
                if depth_paren == 0 {
                    return if saw_arg_token { commas + 1 } else { 0 };
                }
                saw_arg_token = true;
            }
            TokenKind::Symbol(Symbol::OpenBrace) => {
                depth_brace += 1;
                saw_arg_token = true;
            }
            TokenKind::Symbol(Symbol::CloseBrace) => {
                depth_brace = depth_brace.saturating_sub(1);
                saw_arg_token = true;
            }
            TokenKind::Symbol(Symbol::OpenSquare) => {
                depth_square += 1;
                saw_arg_token = true;
            }
            TokenKind::Symbol(Symbol::CloseSquare) => {
                depth_square = depth_square.saturating_sub(1);
                saw_arg_token = true;
            }
            TokenKind::Symbol(Symbol::DoubleLeftAngle) => {
                depth_binary += 1;
                saw_arg_token = true;
            }
            TokenKind::Symbol(Symbol::DoubleRightAngle) => {
                depth_binary = depth_binary.saturating_sub(1);
                saw_arg_token = true;
            }
            TokenKind::Symbol(Symbol::Comma)
                if depth_paren == 1
                    && depth_brace == 0
                    && depth_square == 0
                    && depth_binary == 0 =>
            {
                commas += 1;
                saw_arg_token = true;
            }
            TokenKind::Symbol(Symbol::Dot) => break,
            _ => {
                saw_arg_token = true;
            }
        }
    }
    if saw_arg_token { commas + 1 } else { 0 }
}
