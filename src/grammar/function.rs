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
//! belong to a later semantic phase.

use crate::diagnostic::{Diagnostic, DiagnosticKind, Expected};
use crate::grammar::clause::{parse_argument_list, parse_arrow_body, parse_clause_guard_opt};
use crate::grammar::util::at_symbol;
use crate::parser::{CompletedMarker, Parser};
use crate::syntax::SyntaxKind;
use crate::token_range::TokenRange;

/// Parses one function declaration starting at the name atom.
pub(crate) fn parse_function_decl(p: &mut Parser) -> CompletedMarker {
    let m = p.start();

    parse_function_clause(p);
    while at_symbol(p, erl_tokenize::Symbol::Semicolon) {
        p.consume_lexical();
        parse_function_clause(p);
    }

    m.complete(p, SyntaxKind::FunctionDecl)
}

fn parse_function_clause(p: &mut Parser) -> CompletedMarker {
    let m = p.start();

    let name_start = p.cursor_position();
    if matches!(
        p.peek_lexical(0).map(|(_, t)| t.kind()),
        Some(erl_tokenize::TokenKind::Atom)
    ) {
        p.consume_lexical();
    } else {
        let found = p.peek_lexical(0).map(|(_, t)| t);
        p.push_diagnostic(Diagnostic::new(
            if found.is_some() {
                DiagnosticKind::UnexpectedToken
            } else {
                DiagnosticKind::UnexpectedEof
            },
            TokenRange::empty_at(name_start),
            Expected::Category("function name (atom) at the start of a function clause"),
            found,
        ));
    }

    parse_argument_list(p);
    parse_clause_guard_opt(p);
    parse_arrow_body(p);

    m.complete(p, SyntaxKind::FunctionClause)
}
