//! Form-level dispatch.
//!
//! One module-mode form is either an [`SyntaxKind::Attribute`] (opens
//! with `-`) or a [`SyntaxKind::FunctionDecl`] (opens with an atom).
//! Both branches parse the form up to but not including the
//! terminating `.`; the module-mode top-level driver
//! ([`crate::grammar::module::parse_top_form`]) consumes the `.`
//! afterwards.
//!
//! An input whose first lexical token fits neither branch emits a
//! [`SyntaxKind::Error`] node covering the tokens the driver will
//! eventually skip over, plus a [`crate::Diagnostic`] anchored at the
//! bad token; the driver runs the shared unexpected-token loop
//! afterwards, so the form still terminates at the next `.`.

use crate::diagnostic::{Diagnostic, DiagnosticKind, Expected};
use crate::grammar::attribute::parse_attribute;
use crate::grammar::function::parse_function_decl;
use crate::parser::{CompletedMarker, Parser};
use crate::syntax::SyntaxKind;
use crate::token_range::TokenRange;

/// Parses one module-mode form starting at the current cursor.
pub(crate) fn parse_form(p: &mut Parser) -> CompletedMarker {
    match p.peek_lexical(0).map(|(_, t)| t.kind()) {
        Some(erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::Hyphen)) => parse_attribute(p),
        Some(erl_tokenize::TokenKind::Atom) => parse_function_decl(p),
        Some(_) => {
            let m = p.start();
            let found = p.peek_lexical(0).map(|(_, t)| t);
            p.push_diagnostic(Diagnostic::new(
                DiagnosticKind::UnexpectedToken,
                TokenRange::empty_at(p.cursor_position()),
                Expected::Category("`-` to open an attribute or an atom to open a function"),
                found,
            ));
            m.complete(p, SyntaxKind::Error)
        }
        None => {
            let m = p.start();
            p.push_diagnostic(Diagnostic::new(
                DiagnosticKind::UnexpectedEof,
                TokenRange::empty_at(p.cursor_position()),
                Expected::Category("start of a module-level form"),
                None,
            ));
            m.complete(p, SyntaxKind::Error)
        }
    }
}
