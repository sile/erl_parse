//! Attribute form grammar.
//!
//! Parses `-Name`, `-Name(Payload)`, and `-Name BarePayload` up to
//! (but not including) the terminating `.`. The module-mode top-level
//! driver consumes the `.` afterwards, so the attribute's
//! `TokenRange` covers the leading `-` and the payload but stops
//! before the boundary dot.
//!
//! The attribute name is preserved as an [`AttributeName`] child.
//! When the form has a payload, the payload tokens are preserved as
//! an [`AttributePayload`] child whose range covers everything the
//! parser consumed up to (but not including) the terminating `.`.
//! Both the parenthesized form (`-module(m).`) and the bare form used
//! by `-spec`, `-callback`, `-type`, etc. (`-spec f() -> t.`) route
//! through the same node shape — nested `()`, `{}`, `[]`, and `<<>>`
//! are balanced so an inner `.` inside a nested group does not
//! prematurely end the payload. A bare `-Name.` form still emits a
//! zero-width `AttributePayload` so callers do not have to inspect
//! the token stream to distinguish "no payload" from "empty payload".
//!
//! The parser does not interpret the attribute name: `-module`,
//! `-export`, `-spec`, `-type`, `-record`, `-callback`, and even
//! unpreprocessed `-define` / `-include` / conditional directives all
//! flow through the same code path. The name atom's spelling is
//! available to the caller by reading the [`AttributeName`] child's
//! range from the token buffer.

use crate::diagnostic::{Diagnostic, DiagnosticKind, Expected};
use crate::grammar::util::expect_symbol;
use crate::parser::{CompletedMarker, Parser};
use crate::syntax::SyntaxKind;
use crate::token_range::TokenRange;

/// Parses one attribute form starting at the leading `-`.
pub(crate) fn parse_attribute(p: &mut Parser) -> CompletedMarker {
    let m = p.start();

    expect_symbol(
        p,
        erl_tokenize::Symbol::Hyphen,
        "`-` to open attribute form",
    );
    parse_attribute_name(p);
    parse_attribute_payload(p);

    m.complete(p, SyntaxKind::Attribute)
}

fn parse_attribute_name(p: &mut Parser) {
    let m = p.start();
    let start_at = p.cursor_position();
    match p.peek_lexical(0).map(|(_, t)| t.kind()) {
        Some(erl_tokenize::TokenKind::Atom) => {
            p.consume_lexical();
        }
        _ => {
            let found = p.peek_lexical(0).map(|(_, t)| t);
            p.push_diagnostic(Diagnostic::new(
                if found.is_some() {
                    DiagnosticKind::UnexpectedToken
                } else {
                    DiagnosticKind::UnexpectedEof
                },
                TokenRange::empty_at(start_at),
                Expected::Category("attribute name (atom) after `-`"),
                found,
            ));
        }
    }
    m.complete(p, SyntaxKind::AttributeName);
}

/// Consumes the attribute's payload up to (but not including) the
/// terminating `.`, wrapping it as [`SyntaxKind::AttributePayload`].
///
/// A form whose payload is empty (`-Name.`) emits a zero-width
/// payload node so consumers do not have to inspect the token stream
/// to distinguish "no payload" from "empty payload".
///
/// Nested `()`, `{}`, `[]`, and `<<>>` are balanced so a `.` inside
/// a nested group does not prematurely close the form. If parens are
/// still open at end-of-input (a truncated `-name(payload`), the
/// unclosed-paren count surfaces as a `Diagnostic` and the outer
/// close-paren is never expected explicitly — the balanced consumer
/// handles both the paren-wrapped and bare-payload cases uniformly.
fn parse_attribute_payload(p: &mut Parser) {
    // Empty payload: the next lexical is the terminating `.`.
    if matches!(
        p.peek_lexical(0).map(|(_, t)| t.kind()),
        Some(erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::Dot))
    ) {
        let m = p.start();
        m.complete(p, SyntaxKind::AttributePayload);
        return;
    }
    let m = p.start();
    let unclosed_parens = consume_balanced_until_top_level_dot(p);
    m.complete(p, SyntaxKind::AttributePayload);
    if unclosed_parens > 0 {
        // Report as a missing `)` at the current cursor position; the
        // driver still terminates the form at the (possibly consumed)
        // boundary `.` afterwards.
        expect_symbol(
            p,
            erl_tokenize::Symbol::CloseParen,
            "`)` to close attribute payload",
        );
    }
}

/// Consumes tokens up to (but not including) the outermost lexical
/// `.` at bracket depth 0. Nested `()`, `{}`, `[]`, and `<<>>` are
/// balanced. Returns the number of `(` that were still open when the
/// scan stopped (nonzero when the input ends without a matching
/// closing paren).
fn consume_balanced_until_top_level_dot(p: &mut Parser) -> usize {
    let mut depth_paren: usize = 0;
    let mut depth_brace: usize = 0;
    let mut depth_square: usize = 0;
    let mut depth_binary: usize = 0;
    while let Some((_, token)) = p.peek_lexical(0) {
        match token.kind() {
            erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::Dot)
                if depth_paren == 0
                    && depth_brace == 0
                    && depth_square == 0
                    && depth_binary == 0 =>
            {
                return depth_paren;
            }
            erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::OpenParen) => depth_paren += 1,
            erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::CloseParen) => {
                depth_paren = depth_paren.saturating_sub(1);
            }
            erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::OpenBrace) => depth_brace += 1,
            erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::CloseBrace) => {
                depth_brace = depth_brace.saturating_sub(1);
            }
            erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::OpenSquare) => depth_square += 1,
            erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::CloseSquare) => {
                depth_square = depth_square.saturating_sub(1);
            }
            erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::DoubleLeftAngle) => {
                depth_binary += 1
            }
            erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::DoubleRightAngle) => {
                depth_binary = depth_binary.saturating_sub(1);
            }
            _ => {}
        }
        p.consume_lexical();
    }
    depth_paren
}
