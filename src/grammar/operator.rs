//! Operator precedence table.
//!
//! Mirrors the declarations at the top of OTP 29's
//! `lib/stdlib/src/erl_parse.yrl`:
//!
//! ```text
//! Unary 0 'catch'.
//! Right 100 '=' '!'.
//! Right 150 'orelse'.
//! Right 160 'andalso'.
//! Nonassoc 200 comp_op.
//! Right 300 list_op.
//! Left 400 add_op.
//! Left 500 mult_op.
//! Unary 600 prefix_op.
//! Nonassoc 700 '#'.
//! Left 750 '('.
//! Nonassoc 800 ':'.
//! ```
//!
//! The Pratt-style parser converts each precedence into a
//! `(left_bp, right_bp)` pair with strict "greater than" comparison
//! against a running `min_bp`:
//!
//! - **Left-associative** operator at precedence P: `(P, P)`; the same
//!   operator appearing on the right side will fail the strict `>` check
//!   and unwind to the outer call, giving `((a op b) op c)`.
//! - **Right-associative** operator at precedence P: `(P, P - 1)`; the
//!   same operator on the right side passes `>` and recurses, giving
//!   `(a op (b op c))`.
//! - **Non-associative** operator at precedence P: `(P, P)`; the caller
//!   is expected to reject a second same-precedence operator on the
//!   right side.
//!
//! Precedence values may change with future OTP releases; keep this
//! table synced with the `erl_parse.yrl` source of truth.

#![cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Consumed by the expression / pattern / guard / term grammars added on top of this table; only the in-module tests currently drive every entry"
    )
)]

/// Binding-power pair for an infix operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct InfixBp {
    /// Left binding power (compared against the caller's `min_bp` with
    /// strict `>`).
    pub(crate) lbp: u16,
    /// Right binding power (passed as `min_bp` to the recursive call for
    /// the right operand).
    pub(crate) rbp: u16,
    pub(crate) assoc: Assoc,
}

/// Associativity flag; used by non-associative operators to reject a
/// same-precedence operator appearing on the right side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Assoc {
    Left,
    Right,
    Nonassoc,
}

const fn left(p: u16) -> InfixBp {
    InfixBp {
        lbp: p,
        rbp: p,
        assoc: Assoc::Left,
    }
}

const fn right(p: u16) -> InfixBp {
    InfixBp {
        lbp: p,
        rbp: p - 1,
        assoc: Assoc::Right,
    }
}

const fn nonassoc(p: u16) -> InfixBp {
    InfixBp {
        lbp: p,
        rbp: p,
        assoc: Assoc::Nonassoc,
    }
}

/// Returns the binding-power pair for an infix operator, or `None` when
/// the token is not one.
///
/// Recognizes: `= ! orelse andalso` and all `comp_op` / `list_op` /
/// `add_op` / `mult_op` categories from OTP 29's `erl_parse.yrl`.
pub(crate) fn infix_binding_power(token: erl_tokenize::Token) -> Option<InfixBp> {
    match token.kind() {
        erl_tokenize::TokenKind::Symbol(sym) => match sym {
            // Right 100 '=' '!'.
            erl_tokenize::Symbol::Match | erl_tokenize::Symbol::Bang => Some(right(100)),
            // `?=` in maybe blocks shares the match precedence.
            erl_tokenize::Symbol::MaybeMatch => Some(right(100)),
            // Nonassoc 200 comp_op.
            erl_tokenize::Symbol::Eq
            | erl_tokenize::Symbol::NotEq
            | erl_tokenize::Symbol::LessEq
            | erl_tokenize::Symbol::Less
            | erl_tokenize::Symbol::GreaterEq
            | erl_tokenize::Symbol::Greater
            | erl_tokenize::Symbol::ExactEq
            | erl_tokenize::Symbol::ExactNotEq => Some(nonassoc(200)),
            // Right 300 list_op.
            erl_tokenize::Symbol::PlusPlus | erl_tokenize::Symbol::MinusMinus => Some(right(300)),
            // Left 400 add_op (symbols).
            erl_tokenize::Symbol::Plus | erl_tokenize::Symbol::Hyphen => Some(left(400)),
            // Left 500 mult_op (symbols).
            erl_tokenize::Symbol::Multiply | erl_tokenize::Symbol::Slash => Some(left(500)),
            _ => None,
        },
        erl_tokenize::TokenKind::Keyword(kw) => match kw {
            // Right 150 'orelse'.
            erl_tokenize::Keyword::Orelse => Some(right(150)),
            // Right 160 'andalso'.
            erl_tokenize::Keyword::Andalso => Some(right(160)),
            // Left 400 add_op (keywords).
            erl_tokenize::Keyword::Bor
            | erl_tokenize::Keyword::Bxor
            | erl_tokenize::Keyword::Bsl
            | erl_tokenize::Keyword::Bsr
            | erl_tokenize::Keyword::Or
            | erl_tokenize::Keyword::Xor => Some(left(400)),
            // Left 500 mult_op (keywords).
            erl_tokenize::Keyword::Div
            | erl_tokenize::Keyword::Rem
            | erl_tokenize::Keyword::Band
            | erl_tokenize::Keyword::And => Some(left(500)),
            _ => None,
        },
        _ => None,
    }
}

/// Returns the right binding-power for a prefix operator, or `None` when
/// the token is not one.
///
/// Recognizes: `+ - bnot not` at precedence 600 (`prefix_op` in the yrl),
/// plus `catch` at precedence 0 (`Unary 0 'catch'`).
pub(crate) fn prefix_binding_power(token: erl_tokenize::Token) -> Option<u16> {
    match token.kind() {
        erl_tokenize::TokenKind::Symbol(
            erl_tokenize::Symbol::Plus | erl_tokenize::Symbol::Hyphen,
        ) => Some(600),
        erl_tokenize::TokenKind::Keyword(
            erl_tokenize::Keyword::Bnot | erl_tokenize::Keyword::Not,
        ) => Some(600),
        erl_tokenize::TokenKind::Keyword(erl_tokenize::Keyword::Catch) => Some(0),
        _ => None,
    }
}

/// Left binding-power for the call suffix `(` — `Left 750 '('` in the
/// yrl. Returned separately from [`infix_binding_power`] because the
/// grammar treats a call as a distinct construct rather than an infix
/// operator.
pub(crate) const CALL_LBP: u16 = 750;

/// Left binding-power for the remote qualifier `:` — `Nonassoc 800 ':'`
/// in the yrl. Used at the boundary between `Module:Function` in call
/// targets and fun references.
pub(crate) const REMOTE_LBP: u16 = 800;

/// Left binding-power for the record / map suffix `#` — `Nonassoc 700
/// '#'` in the yrl.
pub(crate) const RECORD_MAP_LBP: u16 = 700;

/// `?=` binding-power used inside `maybe` blocks; the yrl models it
/// separately as `maybe_match` production but conceptually shares the
/// match precedence.
pub(crate) const MAYBE_MATCH_LBP: u16 = 100;
pub(crate) const MAYBE_MATCH_RBP: u16 = 99;

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(source: &str) -> erl_tokenize::Token {
        erl_tokenize::scan_token(source, erl_tokenize::Position::new())
            .expect("valid source")
            .expect("non-empty source")
    }

    #[test]
    fn match_and_send_are_right_assoc_100() {
        let bp = infix_binding_power(tok("=")).expect("`=` is an infix op");
        assert_eq!(bp.lbp, 100);
        assert_eq!(bp.rbp, 99);
        assert_eq!(bp.assoc, Assoc::Right);

        let bp = infix_binding_power(tok("!")).expect("`!` is an infix op");
        assert_eq!(bp.lbp, 100);
        assert_eq!(bp.assoc, Assoc::Right);
    }

    #[test]
    fn orelse_and_andalso_are_right_assoc() {
        assert_eq!(
            infix_binding_power(tok("orelse"))
                .expect("`orelse` is an infix op")
                .lbp,
            150
        );
        assert_eq!(
            infix_binding_power(tok("andalso"))
                .expect("`andalso` is an infix op")
                .lbp,
            160
        );
    }

    #[test]
    fn comparison_ops_are_nonassoc_200() {
        for op in ["==", "/=", "=<", "<", ">=", ">", "=:=", "=/="] {
            let bp = infix_binding_power(tok(op)).expect(op);
            assert_eq!(bp.lbp, 200, "op {op}");
            assert_eq!(bp.assoc, Assoc::Nonassoc, "op {op}");
        }
    }

    #[test]
    fn list_ops_are_right_assoc_300() {
        for op in ["++", "--"] {
            let bp = infix_binding_power(tok(op)).expect(op);
            assert_eq!(bp.lbp, 300);
            assert_eq!(bp.assoc, Assoc::Right);
        }
    }

    #[test]
    fn add_ops_are_left_assoc_400() {
        for op in ["+", "-", "bor", "bxor", "bsl", "bsr", "or", "xor"] {
            let bp = infix_binding_power(tok(op)).expect(op);
            assert_eq!(bp.lbp, 400, "op {op}");
            assert_eq!(bp.assoc, Assoc::Left, "op {op}");
        }
    }

    #[test]
    fn mult_ops_are_left_assoc_500() {
        for op in ["*", "/", "div", "rem", "band", "and"] {
            let bp = infix_binding_power(tok(op)).expect(op);
            assert_eq!(bp.lbp, 500, "op {op}");
            assert_eq!(bp.assoc, Assoc::Left, "op {op}");
        }
    }

    #[test]
    fn prefix_ops_bind_at_600() {
        for op in ["+", "-", "bnot", "not"] {
            assert_eq!(prefix_binding_power(tok(op)), Some(600), "op {op}");
        }
    }

    #[test]
    fn catch_is_lowest_prefix_op() {
        assert_eq!(prefix_binding_power(tok("catch")), Some(0));
    }

    #[test]
    fn non_operator_tokens_have_no_binding_power() {
        assert!(infix_binding_power(tok("foo")).is_none());
        assert!(prefix_binding_power(tok("foo")).is_none());
    }

    #[test]
    fn suffix_and_qualifier_binding_powers_match_yrl() {
        assert_eq!(CALL_LBP, 750);
        assert_eq!(REMOTE_LBP, 800);
        assert_eq!(RECORD_MAP_LBP, 700);
        assert_eq!(MAYBE_MATCH_LBP, 100);
        assert_eq!(MAYBE_MATCH_RBP, 99);
    }
}
