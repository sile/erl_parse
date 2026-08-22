//! Small cursor-inspection and expectation helpers shared by the
//! grammar modules.
//!
//! These wrap the parser core's `peek_lexical` / `consume_lexical` /
//! `push_diagnostic` primitives with the "peek for a specific token /
//! consume-or-error" pattern that grammar productions repeat.

use crate::parser::Parser;

/// Returns `true` when the next lexical token is the given [`Symbol`].
pub(crate) fn at_symbol(p: &Parser, sym: erl_tokenize::Symbol) -> bool {
    matches!(
        p.peek_lexical(0).map(|(_, t)| t.kind()),
        Some(erl_tokenize::TokenKind::Symbol(s)) if s == sym
    )
}

/// Returns `true` when the next lexical token is the given [`Keyword`].
pub(crate) fn at_keyword(p: &Parser, kw: erl_tokenize::Keyword) -> bool {
    matches!(
        p.peek_lexical(0).map(|(_, t)| t.kind()),
        Some(erl_tokenize::TokenKind::Keyword(k)) if k == kw
    )
}

/// Returns `true` when `token` matches the given [`Symbol`].
pub(crate) fn is_symbol(token: erl_tokenize::Token, sym: erl_tokenize::Symbol) -> bool {
    matches!(token.kind(), erl_tokenize::TokenKind::Symbol(s) if s == sym)
}

/// Returns `true` when `token` matches the given [`Keyword`].
pub(crate) fn is_keyword(token: erl_tokenize::Token, kw: erl_tokenize::Keyword) -> bool {
    matches!(token.kind(), erl_tokenize::TokenKind::Keyword(k) if k == kw)
}

/// Consumes the next lexical token if it is [`Symbol`] `sym`.
/// Otherwise emits a [`DiagnosticKind::MissingToken`] diagnostic
/// (zero-width `TokenRange` at the cursor) via
/// [`crate::grammar::recovery::push_missing_token`] and does not
/// advance — the parser refuses to synthesize a fake `Token`, so
/// the caller either recovers or fails locally.
pub(crate) fn expect_symbol(p: &mut Parser, sym: erl_tokenize::Symbol, msg: &'static str) {
    if at_symbol(p, sym) {
        p.consume_lexical();
        return;
    }
    crate::grammar::recovery::push_missing_token(p, msg);
}

/// Consumes the next lexical token if it is [`Keyword`] `kw`.
/// Otherwise behaves as for [`expect_symbol`]: emits a
/// [`DiagnosticKind::MissingToken`] diagnostic and does not
/// advance.
pub(crate) fn expect_keyword(p: &mut Parser, kw: erl_tokenize::Keyword, msg: &'static str) {
    if at_keyword(p, kw) {
        p.consume_lexical();
        return;
    }
    crate::grammar::recovery::push_missing_token(p, msg);
}

/// Consumes the next lexical token if it is an atom or a variable;
/// on mismatch emits a [`DiagnosticKind::MissingToken`] diagnostic
/// (zero-width [`TokenRange`] at the cursor) and does not advance.
pub(crate) fn consume_atom_or_var(p: &mut Parser, msg: &'static str) {
    match p.peek_lexical(0).map(|(_, t)| t.kind()) {
        Some(erl_tokenize::TokenKind::Atom | erl_tokenize::TokenKind::Variable) => {
            p.consume_lexical();
        }
        _ => crate::grammar::recovery::push_missing_token(p, msg),
    }
}

/// Consumes the next lexical token if it is an integer or a
/// variable; on mismatch emits a [`DiagnosticKind::MissingToken`]
/// diagnostic and does not advance.
pub(crate) fn consume_integer_or_var(p: &mut Parser, msg: &'static str) {
    match p.peek_lexical(0).map(|(_, t)| t.kind()) {
        Some(erl_tokenize::TokenKind::Integer | erl_tokenize::TokenKind::Variable) => {
            p.consume_lexical();
        }
        _ => crate::grammar::recovery::push_missing_token(p, msg),
    }
}
