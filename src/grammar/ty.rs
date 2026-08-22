//! Type grammar (Erlang type expressions).
//!
//! Parses the OTP 29 `top_type` / `type` productions from
//! `lib/stdlib/src/erl_parse.yrl` under [`ParseContext::Type`]. Atomic
//! types (atom / variable / integer / float / char / string / paren)
//! reuse the expression-side `SyntaxKind` variants for shape parity;
//! compound types (tuple / list / map / record / bitstring / function /
//! call / range / union / annotated) get dedicated `*Type` variants
//! declared in [`crate::syntax`] because the child structure differs
//! from the like-named expression variants.
//!
//! Type-specific operator precedences (from the yrl):
//!
//! ```text
//! Right 150 '::'.        annotation
//! Left  170 '|'.         union
//! Nonassoc 200 '..'.     integer range
//! ```
//!
//! Integer arithmetic operators reuse the expression-side
//! `infix_binding_power` table.

use crate::diagnostic::{Diagnostic, DiagnosticKind, Expected};
use crate::grammar::operator;
use crate::grammar::util::{
    at_symbol, consume_atom_or_var, expect_keyword, expect_symbol, is_symbol,
};
use crate::parser::{CompletedMarker, Marker, ParseContext, Parser};
use crate::syntax::SyntaxKind;
use crate::token_range::TokenRange;

// -------------------------------------------------------------------
// Type-specific operator binding powers.
// -------------------------------------------------------------------

const ANNOTATION_LBP: u16 = 150;
const ANNOTATION_RBP: u16 = 149; // right-associative
const UNION_LBP: u16 = 170;
const UNION_RBP: u16 = 170; // left-associative
const RANGE_LBP: u16 = 200;
const RANGE_RBP: u16 = 200; // nonassoc

/// Parses a top-level type expression at the current cursor position.
/// The active [`ParseContext`] is switched to [`ParseContext::Type`]
/// for the span and restored on return.
pub(crate) fn parse_type(p: &mut Parser) -> CompletedMarker {
    let prev = p.set_context(ParseContext::Type);
    let completed = parse_top_type(p);
    p.set_context(prev);
    completed
}

/// Entry point for the yrl's `top_type` production:
///
/// ```text
/// top_type -> var '::' top_type     (annotated)
/// top_type -> type '|' top_type     (union)
/// top_type -> type
/// ```
///
/// Handled uniformly with a Pratt loop that recognizes `|`, `..`, and
/// integer arithmetic on top of [`parse_type_max`], plus `::` when the
/// operand on the left is a bare variable (annotated types).
fn parse_top_type(p: &mut Parser) -> CompletedMarker {
    parse_type_bp(p, 0)
}

/// Same depth-guard shape as `parse_expr_bp`: bounded recursion via
/// [`Parser::enter_depth`] / [`Parser::leave_depth`], with a
/// short-circuit at [`Parser::MAX_NESTING_DEPTH`] that emits a
/// zero-width [`SyntaxKind::Error`] node and a
/// [`DiagnosticKind::NestingDepthExceeded`] diagnostic instead of
/// recursing further.
fn parse_type_bp(p: &mut Parser, min_bp: u16) -> CompletedMarker {
    if !p.enter_depth() {
        let m = p.start();
        p.push_nesting_depth_exceeded();
        return m.complete(p, SyntaxKind::Error);
    }
    let result = parse_type_bp_inner(p, min_bp);
    p.leave_depth();
    result
}

fn parse_type_bp_inner(p: &mut Parser, min_bp: u16) -> CompletedMarker {
    let mut lhs = parse_type_max(p);
    let mut last_nonassoc_bp: Option<u16> = None;

    while let Some((_, token)) = p.peek_lexical(0) {
        // Union `|` (Left 170).
        if is_symbol(token, erl_tokenize::Symbol::VerticalBar) && UNION_LBP > min_bp {
            let m = lhs.precede(p);
            p.consume_lexical();
            parse_type_bp(p, UNION_RBP);
            lhs = m.complete(p, SyntaxKind::UnionType);
            last_nonassoc_bp = None;
            continue;
        }

        // Range `..` (Nonassoc 200).
        if is_symbol(token, erl_tokenize::Symbol::DoubleDot) && RANGE_LBP > min_bp {
            if last_nonassoc_bp == Some(RANGE_LBP) {
                p.push_diagnostic(Diagnostic::new(
                    DiagnosticKind::UnexpectedToken,
                    TokenRange::empty_at(p.cursor_position()),
                    Expected::Category("non-associative range operator used twice"),
                    Some(token),
                ));
            }
            let m = lhs.precede(p);
            p.consume_lexical();
            parse_type_bp(p, RANGE_RBP);
            lhs = m.complete(p, SyntaxKind::RangeType);
            last_nonassoc_bp = Some(RANGE_LBP);
            continue;
        }

        // Annotation `::` (Right 150) — bind when the caller allows it.
        if is_symbol(token, erl_tokenize::Symbol::DoubleColon) && ANNOTATION_LBP > min_bp {
            let m = lhs.precede(p);
            p.consume_lexical();
            parse_type_bp(p, ANNOTATION_RBP);
            lhs = m.complete(p, SyntaxKind::AnnotatedType);
            last_nonassoc_bp = None;
            continue;
        }

        // Integer arithmetic (add_op 400, mult_op 500) reused from the
        // expression operator table. We only accept the pure-integer
        // subset — comparison / logical / list / match / send operators
        // are silently unused here because they never appear in a type
        // position emitted by the yrl.
        if let Some(bp) = operator::infix_binding_power(token) {
            let is_integer_op = matches!(
                token.kind(),
                erl_tokenize::TokenKind::Symbol(
                    erl_tokenize::Symbol::Plus
                        | erl_tokenize::Symbol::Hyphen
                        | erl_tokenize::Symbol::Multiply
                        | erl_tokenize::Symbol::Slash
                ) | erl_tokenize::TokenKind::Keyword(
                    erl_tokenize::Keyword::Div
                        | erl_tokenize::Keyword::Rem
                        | erl_tokenize::Keyword::Band
                        | erl_tokenize::Keyword::Bor
                        | erl_tokenize::Keyword::Bxor
                        | erl_tokenize::Keyword::Bsl
                        | erl_tokenize::Keyword::Bsr,
                )
            );
            if is_integer_op && bp.lbp > min_bp {
                let m = lhs.precede(p);
                p.consume_lexical();
                parse_type_bp(p, bp.rbp);
                lhs = m.complete(p, SyntaxKind::BinaryOpType);
                last_nonassoc_bp = None;
                continue;
            }
        }

        break;
    }

    lhs
}

/// Primary type production: everything the yrl calls just `type` (no
/// infix consumption at this level).
fn parse_type_max(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    let Some((_, token)) = p.peek_lexical(0) else {
        p.push_diagnostic(Diagnostic::new(
            DiagnosticKind::UnexpectedEof,
            TokenRange::empty_at(p.cursor_position()),
            Expected::Category("type expression"),
            None,
        ));
        return m.complete(p, SyntaxKind::Error);
    };

    // Prefix integer operator (`+ - bnot not`).
    if operator::prefix_binding_power(token).is_some()
        && !matches!(
            token.kind(),
            erl_tokenize::TokenKind::Keyword(erl_tokenize::Keyword::Catch)
        )
    {
        p.consume_lexical();
        parse_type_max(p);
        return m.complete(p, SyntaxKind::UnaryOpType);
    }

    match token.kind() {
        // atom or type call (`name` / `name()` / `name(T)` / `mod:name(T)`).
        erl_tokenize::TokenKind::Atom => parse_atom_head(p, m),
        erl_tokenize::TokenKind::Variable => atomic(p, m, SyntaxKind::VarExpr),
        erl_tokenize::TokenKind::Integer => atomic(p, m, SyntaxKind::IntegerExpr),
        erl_tokenize::TokenKind::Float => atomic(p, m, SyntaxKind::FloatExpr),
        erl_tokenize::TokenKind::Char => atomic(p, m, SyntaxKind::CharExpr),
        erl_tokenize::TokenKind::String => atomic(p, m, SyntaxKind::StringExpr),
        erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::OpenParen) => parse_paren_type(p, m),
        erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::OpenBrace) => parse_tuple_type(p, m),
        erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::OpenSquare) => parse_list_type(p, m),
        erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::Sharp) => parse_hash_type(p, m),
        erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::DoubleLeftAngle) => {
            parse_bitstring_type(p, m)
        }
        erl_tokenize::TokenKind::Keyword(erl_tokenize::Keyword::Fun) => parse_fun_type(p, m),
        _ => {
            let _ = token;
            // Abandon the outer marker so the recovery site's Error
            // node covers only the one skipped token, giving
            // `Diagnostic::range() == Error node's TokenRange`.
            m.abandon(p);
            crate::grammar::recovery::skip_one_token(p, "type expression")
        }
    }
}

fn atomic(p: &mut Parser, m: Marker, kind: SyntaxKind) -> CompletedMarker {
    p.consume_lexical();
    m.complete(p, kind)
}

/// Handles the shape `atom` / `atom ( ... )` / `atom : atom ( ... )`.
fn parse_atom_head(p: &mut Parser, m: Marker) -> CompletedMarker {
    p.consume_lexical(); // atom
    // Remote qualifier `Mod:Name` (must be followed by another atom).
    if at_symbol(p, erl_tokenize::Symbol::Colon)
        && matches!(
            p.peek_lexical(1).map(|(_, t)| t.kind()),
            Some(erl_tokenize::TokenKind::Atom)
        )
    {
        p.consume_lexical(); // `:`
        p.consume_lexical(); // second atom
        if at_symbol(p, erl_tokenize::Symbol::OpenParen) {
            parse_type_argument_list(p);
            return m.complete(p, SyntaxKind::TypeCall);
        }
        // `mod:name` without `(` — treat as a bare remote reference.
        return m.complete(p, SyntaxKind::RemoteType);
    }
    // Local type call `name(...)`.
    if at_symbol(p, erl_tokenize::Symbol::OpenParen) {
        parse_type_argument_list(p);
        return m.complete(p, SyntaxKind::TypeCall);
    }
    // Bare atom type.
    m.complete(p, SyntaxKind::AtomExpr)
}

fn parse_paren_type(p: &mut Parser, m: Marker) -> CompletedMarker {
    p.consume_lexical(); // `(`
    parse_top_type(p);
    expect_symbol(
        p,
        erl_tokenize::Symbol::CloseParen,
        "`)` to close parenthesized type",
    );
    m.complete(p, SyntaxKind::ParenExpr)
}

fn parse_tuple_type(p: &mut Parser, m: Marker) -> CompletedMarker {
    p.consume_lexical(); // `{`
    if at_symbol(p, erl_tokenize::Symbol::CloseBrace) {
        p.consume_lexical();
        return m.complete(p, SyntaxKind::TupleType);
    }
    parse_top_types_comma(p, erl_tokenize::Symbol::CloseBrace);
    expect_symbol(
        p,
        erl_tokenize::Symbol::CloseBrace,
        "`}` to close tuple type",
    );
    m.complete(p, SyntaxKind::TupleType)
}

/// `[]` / `[T]` / `[T, ...]`.
fn parse_list_type(p: &mut Parser, m: Marker) -> CompletedMarker {
    p.consume_lexical(); // `[`
    if at_symbol(p, erl_tokenize::Symbol::CloseSquare) {
        p.consume_lexical();
        return m.complete(p, SyntaxKind::ListType);
    }
    parse_top_type(p);
    if at_symbol(p, erl_tokenize::Symbol::Comma) {
        p.consume_lexical();
        if at_symbol(p, erl_tokenize::Symbol::TripleDot) {
            p.consume_lexical();
            expect_symbol(
                p,
                erl_tokenize::Symbol::CloseSquare,
                "`]` to close non-empty list type",
            );
            return m.complete(p, SyntaxKind::NonemptyListType);
        }
        // A comma without a trailing `...` is not part of the yrl list-
        // type production; keep consuming the extra element as a plain
        // type so the tree stays navigable and record an error.
        p.push_diagnostic(Diagnostic::new(
            DiagnosticKind::UnexpectedToken,
            TokenRange::empty_at(p.cursor_position()),
            Expected::Category("`...]` after list type comma"),
            p.peek_lexical(0).map(|(_, t)| t),
        ));
        parse_top_type(p);
    }
    expect_symbol(
        p,
        erl_tokenize::Symbol::CloseSquare,
        "`]` to close list type",
    );
    m.complete(p, SyntaxKind::ListType)
}

/// Dispatch for `#`: `#{...}` map type / `#Name{...}` record type.
fn parse_hash_type(p: &mut Parser, m: Marker) -> CompletedMarker {
    p.consume_lexical(); // `#`
    if at_symbol(p, erl_tokenize::Symbol::OpenBrace) {
        parse_map_type_body(p);
        return m.complete(p, SyntaxKind::MapType);
    }
    // `#Name` — record type. Consume the name, then optionally the
    // `:remote` qualifier per the yrl's `#atom ':' record_name`.
    consume_atom_or_var(p, "record type name");
    if at_symbol(p, erl_tokenize::Symbol::Colon)
        && matches!(
            p.peek_lexical(1).map(|(_, t)| t.kind()),
            Some(erl_tokenize::TokenKind::Atom)
        )
    {
        p.consume_lexical(); // `:`
        p.consume_lexical(); // second atom (record_name)
    }
    expect_symbol(
        p,
        erl_tokenize::Symbol::OpenBrace,
        "`{` after record type name",
    );
    if at_symbol(p, erl_tokenize::Symbol::CloseBrace) {
        p.consume_lexical();
        return m.complete(p, SyntaxKind::RecordType);
    }
    parse_record_type_fields(p);
    expect_symbol(
        p,
        erl_tokenize::Symbol::CloseBrace,
        "`}` to close record type",
    );
    m.complete(p, SyntaxKind::RecordType)
}

/// `{ [MapTypeField, MapTypeField, ...] }` — assumes `{` is next.
fn parse_map_type_body(p: &mut Parser) {
    p.consume_lexical(); // `{`
    if at_symbol(p, erl_tokenize::Symbol::CloseBrace) {
        p.consume_lexical();
        return;
    }
    loop {
        parse_map_type_field(p);
        if !at_symbol(p, erl_tokenize::Symbol::Comma) {
            break;
        }
        p.consume_lexical();
    }
    expect_symbol(p, erl_tokenize::Symbol::CloseBrace, "`}` to close map type");
}

fn parse_map_type_field(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    parse_top_type(p);
    if at_symbol(p, erl_tokenize::Symbol::DoubleRightArrow)
        || at_symbol(p, erl_tokenize::Symbol::MapMatch)
    {
        p.consume_lexical();
        parse_top_type(p);
    } else {
        let found = p.peek_lexical(0).map(|(_, t)| t);
        p.push_diagnostic(Diagnostic::new(
            if found.is_some() {
                DiagnosticKind::UnexpectedToken
            } else {
                DiagnosticKind::UnexpectedEof
            },
            TokenRange::empty_at(p.cursor_position()),
            Expected::Category("`=>` or `:=` in map type field"),
            found,
        ));
    }
    m.complete(p, SyntaxKind::MapTypeField)
}

fn parse_record_type_fields(p: &mut Parser) {
    loop {
        parse_record_type_field(p);
        if !at_symbol(p, erl_tokenize::Symbol::Comma) {
            break;
        }
        p.consume_lexical();
    }
}

fn parse_record_type_field(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    consume_atom_or_var(p, "record type field name");
    expect_symbol(
        p,
        erl_tokenize::Symbol::DoubleColon,
        "`::` in record type field",
    );
    parse_top_type(p);
    m.complete(p, SyntaxKind::RecordTypeField)
}

/// `<< >>` / `<< bin_base_type >>` / `<< bin_unit_type >>` /
/// `<< bin_base_type , bin_unit_type >>`. Each `bin_*_type` is a
/// segment shape `Var : Size [ * Type]`; the parser accepts any
/// comma-separated segment list and lets a semantic pass tighten the
/// exact yrl combinations.
fn parse_bitstring_type(p: &mut Parser, m: Marker) -> CompletedMarker {
    p.consume_lexical(); // `<<`
    if at_symbol(p, erl_tokenize::Symbol::DoubleRightAngle) {
        p.consume_lexical();
        return m.complete(p, SyntaxKind::BitstringType);
    }
    loop {
        parse_bitstring_type_segment(p);
        if !at_symbol(p, erl_tokenize::Symbol::Comma) {
            break;
        }
        p.consume_lexical();
    }
    expect_symbol(
        p,
        erl_tokenize::Symbol::DoubleRightAngle,
        "`>>` to close bitstring type",
    );
    m.complete(p, SyntaxKind::BitstringType)
}

fn parse_bitstring_type_segment(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    // A segment starts with a variable (typically `_`), an integer
    // literal, or a type. Accept any type_max so the grammar mirrors
    // the yrl's build_bin_type helper.
    parse_type_max(p);
    if at_symbol(p, erl_tokenize::Symbol::Colon) {
        p.consume_lexical();
        parse_type_max(p);
    }
    if at_symbol(p, erl_tokenize::Symbol::Multiply) {
        p.consume_lexical();
        parse_type_max(p);
    }
    m.complete(p, SyntaxKind::BitstringTypeSegment)
}

/// `fun ()` — any-arity function type — or
/// `fun ( ( ... ) -> Return )` — universal-args form — or
/// `fun ( ( T, T ) -> Return )` — explicit-args form.
fn parse_fun_type(p: &mut Parser, m: Marker) -> CompletedMarker {
    p.consume_lexical(); // `fun`
    expect_symbol(
        p,
        erl_tokenize::Symbol::OpenParen,
        "`(` after `fun` in function type",
    );
    if at_symbol(p, erl_tokenize::Symbol::CloseParen) {
        p.consume_lexical();
        return m.complete(p, SyntaxKind::FunctionType);
    }
    parse_fun_type_signature(p);
    expect_symbol(
        p,
        erl_tokenize::Symbol::CloseParen,
        "`)` to close `fun` type",
    );
    m.complete(p, SyntaxKind::FunctionType)
}

fn parse_fun_type_signature(p: &mut Parser) {
    let params = p.start();
    expect_symbol(
        p,
        erl_tokenize::Symbol::OpenParen,
        "`(` to open function type parameters",
    );
    if at_symbol(p, erl_tokenize::Symbol::TripleDot) {
        p.consume_lexical();
    } else if !at_symbol(p, erl_tokenize::Symbol::CloseParen) {
        parse_top_types_comma(p, erl_tokenize::Symbol::CloseParen);
    }
    expect_symbol(
        p,
        erl_tokenize::Symbol::CloseParen,
        "`)` to close function type parameters",
    );
    params.complete(p, SyntaxKind::FunctionTypeParams);

    expect_symbol(p, erl_tokenize::Symbol::RightArrow, "`->` in function type");

    let ret = p.start();
    parse_top_type(p);
    ret.complete(p, SyntaxKind::FunctionTypeReturn);
}

/// `( T, T, ... )` — used by [`parse_atom_head`] for local / remote
/// type calls; consumes the parentheses.
fn parse_type_argument_list(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    expect_symbol(
        p,
        erl_tokenize::Symbol::OpenParen,
        "`(` to open type argument list",
    );
    if at_symbol(p, erl_tokenize::Symbol::CloseParen) {
        p.consume_lexical();
        return m.complete(p, SyntaxKind::TypeArgumentList);
    }
    parse_top_types_comma(p, erl_tokenize::Symbol::CloseParen);
    expect_symbol(
        p,
        erl_tokenize::Symbol::CloseParen,
        "`)` to close type argument list",
    );
    m.complete(p, SyntaxKind::TypeArgumentList)
}

/// `TopType, TopType, ...` — one or more comma-separated
/// [`parse_top_type`]s. Recovers under
/// [`crate::parser::RecoveryContext::Type`] when a type production
/// leaves the cursor at a token that is neither `,` nor the caller-
/// supplied `close` delimiter — the skipped span becomes a
/// [`SyntaxKind::Error`] node with a matching
/// [`crate::diagnostic::DiagnosticKind::SkippedToken`] diagnostic so the
/// list can continue.
fn parse_top_types_comma(p: &mut Parser, close: erl_tokenize::Symbol) {
    parse_top_type(p);
    loop {
        if !at_symbol(p, erl_tokenize::Symbol::Comma) && !at_symbol(p, close) {
            let _ = crate::grammar::recovery::skip_until_sync(
                p,
                crate::parser::RecoveryContext::Type,
                |t| {
                    crate::grammar::util::is_symbol(t, erl_tokenize::Symbol::Comma)
                        || crate::grammar::util::is_symbol(t, close)
                },
                "`,` or closing delimiter in type list",
            );
        }
        if !at_symbol(p, erl_tokenize::Symbol::Comma) {
            break;
        }
        p.consume_lexical();
        parse_top_type(p);
    }
}

// -------------------------------------------------------------------
// Constraint / when clause.
// -------------------------------------------------------------------

/// Parses a `when Constraint, Constraint, ...` clause. Wraps as
/// [`SyntaxKind::TypeGuard`]; each constraint is a
/// [`SyntaxKind::TypeConstraint`] holding either the yrl's
/// `type_guard -> var '::' top_type` shape (annotated) or the
/// compatibility form `type_guard -> atom '(' top_types ')'`.
///
/// Left as a `pub(crate)` helper for the form / module grammar's
/// spec handling; not exposed as a public entry point.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Called by the form / module grammar when it lands; only in-module tests currently exercise it"
    )
)]
pub(crate) fn parse_type_guard(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    expect_keyword(
        p,
        erl_tokenize::Keyword::When,
        "`when` at start of type guard",
    );
    parse_type_constraint(p);
    while at_symbol(p, erl_tokenize::Symbol::Comma) {
        p.consume_lexical();
        parse_type_constraint(p);
    }
    m.complete(p, SyntaxKind::TypeGuard)
}

fn parse_type_constraint(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    consume_atom_or_var(p, "constraint variable or class");
    if at_symbol(p, erl_tokenize::Symbol::DoubleColon) {
        p.consume_lexical();
        parse_top_type(p);
    } else if at_symbol(p, erl_tokenize::Symbol::OpenParen) {
        parse_type_argument_list(p);
    } else {
        let found = p.peek_lexical(0).map(|(_, t)| t);
        p.push_diagnostic(Diagnostic::new(
            if found.is_some() {
                DiagnosticKind::UnexpectedToken
            } else {
                DiagnosticKind::UnexpectedEof
            },
            TokenRange::empty_at(p.cursor_position()),
            Expected::Category("`::` or `(...)` in type constraint"),
            found,
        ));
    }
    m.complete(p, SyntaxKind::TypeConstraint)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{NodeId, SyntaxKind};
    use crate::{ParseMode, Parser};
    use core::assert_matches;

    fn scan_all(source: &str) -> Vec<erl_tokenize::Token> {
        let mut out = Vec::new();
        let mut pos = erl_tokenize::Position::new();
        while let Some(t) = erl_tokenize::scan_token(source, pos).expect("valid source") {
            out.push(t);
            pos = t.end();
        }
        out
    }

    fn drive_type(source: &str) -> Parser {
        // Push tokens without triggering the module-mode top-level
        // driver so `drive_type` can reset state and invoke
        // `parse_type` directly on the accumulated buffer.
        let mut p = Parser::new(ParseMode::Module);
        for t in scan_all(source) {
            p.feed_token_without_grammar_for_test(t);
        }
        p.reset_for_test();
        let outer = p.start();
        parse_type(&mut p);
        outer.complete(&mut p, SyntaxKind::Error);
        p.finalize_pending_units_for_test();
        p
    }

    fn first_child_kind(p: &Parser, root: NodeId) -> SyntaxKind {
        p.syntax_tree()
            .syntax()
            .entry(NodeId::new(root.get() + 1))
            .expect("first child")
            .kind()
    }

    fn tree_contains_kind(p: &Parser, kind: SyntaxKind) -> bool {
        let syntax = p.syntax_tree().syntax();
        (0..syntax.len())
            .any(|i| syntax.entry(NodeId::new(i)).expect("entry exists").kind() == kind)
    }

    #[test]
    fn parses_atomic_types() {
        for (source, kind) in [
            ("atom", SyntaxKind::AtomExpr),
            ("Var", SyntaxKind::VarExpr),
            ("42", SyntaxKind::IntegerExpr),
            ("1.5", SyntaxKind::FloatExpr),
            ("$a", SyntaxKind::CharExpr),
        ] {
            let mut p = drive_type(source);
            let root = p.next_node().expect("unit");
            assert_eq!(first_child_kind(&p, root), kind, "source {source}");
            assert!(
                p.syntax_tree().diagnostics().is_empty(),
                "source {source} produced unexpected errors: {:?}",
                p.syntax_tree().diagnostics()
            );
        }
    }

    #[test]
    fn parses_tuple_and_list_types() {
        for (source, kind) in [
            ("{}", SyntaxKind::TupleType),
            ("{integer(), atom()}", SyntaxKind::TupleType),
            ("[]", SyntaxKind::ListType),
            ("[integer()]", SyntaxKind::ListType),
            ("[integer(), ...]", SyntaxKind::NonemptyListType),
        ] {
            let mut p = drive_type(source);
            let root = p.next_node().expect("unit");
            assert_eq!(first_child_kind(&p, root), kind, "source {source}");
            assert!(
                p.syntax_tree().diagnostics().is_empty(),
                "source {source} produced unexpected errors: {:?}",
                p.syntax_tree().diagnostics()
            );
        }
    }

    #[test]
    fn parses_map_and_record_types() {
        let mut p = drive_type("#{atom() => integer(), binary() := boolean()}");
        let root = p.next_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::MapType);
        assert!(tree_contains_kind(&p, SyntaxKind::MapTypeField));
        assert!(p.syntax_tree().diagnostics().is_empty());

        let mut p = drive_type("#user{name :: binary(), age :: integer()}");
        let root = p.next_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::RecordType);
        assert!(tree_contains_kind(&p, SyntaxKind::RecordTypeField));
        assert!(p.syntax_tree().diagnostics().is_empty());
    }

    #[test]
    fn parses_bitstring_types() {
        for source in ["<<>>", "<<_:8>>", "<<_:_*8>>", "<<_:8, _:_*4>>"] {
            let mut p = drive_type(source);
            let root = p.next_node().expect("unit");
            assert_eq!(
                first_child_kind(&p, root),
                SyntaxKind::BitstringType,
                "source {source}"
            );
            assert!(
                p.syntax_tree().diagnostics().is_empty(),
                "source {source} produced unexpected errors: {:?}",
                p.syntax_tree().diagnostics()
            );
        }
    }

    #[test]
    fn parses_type_call_and_remote_type() {
        let mut p = drive_type("list(integer())");
        let root = p.next_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::TypeCall);
        assert!(p.syntax_tree().diagnostics().is_empty());

        let mut p = drive_type("erlang:map(atom(), integer())");
        let root = p.next_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::TypeCall);
        assert!(p.syntax_tree().diagnostics().is_empty());
    }

    #[test]
    fn parses_function_types() {
        for source in [
            "fun()",
            "fun((...) -> integer())",
            "fun((atom(), integer()) -> boolean())",
        ] {
            let mut p = drive_type(source);
            let root = p.next_node().expect("unit");
            assert_eq!(
                first_child_kind(&p, root),
                SyntaxKind::FunctionType,
                "source {source}"
            );
            assert!(
                p.syntax_tree().diagnostics().is_empty(),
                "source {source} produced errors: {:?}",
                p.syntax_tree().diagnostics()
            );
        }
    }

    #[test]
    fn parses_union_range_annotated_types() {
        // Union.
        let mut p = drive_type("atom() | integer() | binary()");
        let root = p.next_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::UnionType);
        assert!(p.syntax_tree().diagnostics().is_empty());

        // Range.
        let mut p = drive_type("1 .. 100");
        let root = p.next_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::RangeType);
        assert!(p.syntax_tree().diagnostics().is_empty());

        // Annotated.
        let mut p = drive_type("Var :: integer()");
        let root = p.next_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::AnnotatedType);
        assert!(p.syntax_tree().diagnostics().is_empty());
    }

    #[test]
    fn parses_integer_type_arithmetic() {
        let mut p = drive_type("-1 .. 1 + 5");
        let root = p.next_node().expect("unit");
        // Range takes 200, add_op takes 400 → 400 > 200 so `+ 5` is
        // consumed as the range's right operand: `-1 .. (1 + 5)`.
        assert_eq!(first_child_kind(&p, root), SyntaxKind::RangeType);
        assert!(tree_contains_kind(&p, SyntaxKind::UnaryOpType));
        assert!(tree_contains_kind(&p, SyntaxKind::BinaryOpType));
        assert!(p.syntax_tree().diagnostics().is_empty());
    }

    #[test]
    fn parses_parenthesized_type() {
        let mut p = drive_type("(atom() | integer())");
        let root = p.next_node().expect("unit");
        assert_eq!(first_child_kind(&p, root), SyntaxKind::ParenExpr);
        assert!(tree_contains_kind(&p, SyntaxKind::UnionType));
        assert!(p.syntax_tree().diagnostics().is_empty());
    }

    #[test]
    fn parses_type_guard_when_clause() {
        // Drive parse_type_guard directly by feeding tokens for a bare
        // `when` clause.
        let source = "when X :: integer(), Y :: atom()";
        let mut p = Parser::new(ParseMode::Module);
        for t in scan_all(source) {
            p.feed_token_without_grammar_for_test(t);
        }
        p.reset_for_test();
        let outer = p.start();
        // Type guard runs under Type context like top-level type parse.
        let prev = p.set_context(ParseContext::Type);
        parse_type_guard(&mut p);
        p.set_context(prev);
        outer.complete(&mut p, SyntaxKind::Error);
        p.finalize_pending_units_for_test();
        let _ = p.next_node().expect("unit");
        assert!(tree_contains_kind(&p, SyntaxKind::TypeGuard));
        assert!(tree_contains_kind(&p, SyntaxKind::TypeConstraint));
        assert!(p.syntax_tree().diagnostics().is_empty());
    }

    #[test]
    fn unrecognized_leading_token_produces_error_but_still_completes_unit() {
        let mut p = drive_type(")");
        let root = p.next_node().expect("unit");
        // The outer Error wrapper is the root; the type parser
        // completes an inner Error for the unrecognized `)`.
        assert_matches!(
            first_child_kind(&p, root),
            SyntaxKind::Error | SyntaxKind::AtomExpr | SyntaxKind::VarExpr
        );
        assert!(!p.syntax_tree().diagnostics().is_empty());
    }
}
