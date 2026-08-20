//! Common error-recovery helpers shared by grammar modules.
//!
//! Three helpers cover the recovery shapes the grammar needs:
//!
//! - [`skip_one_token`] retrofits the "unmatched token at atomic
//!   grammar position" sites (`parse_expr_max` and `parse_type_max`).
//!   Consumes exactly one lexical token, wraps it as a
//!   [`SyntaxKind::Error`] node, and emits a
//!   [`ParseErrorKind::SkippedToken`] diagnostic whose
//!   [`ParseError::range`] matches the node's `TokenRange`.
//! - [`skip_until_sync`] implements unbounded skipping up to a
//!   caller-supplied sync token predicate. Wraps the whole skipped
//!   span as a [`SyntaxKind::Error`] node with a matching
//!   [`ParseErrorKind::SkippedToken`] diagnostic. Returns early
//!   without emitting anything when the cursor is already at a sync
//!   token or when the same
//!   [`RecoveryContext`][crate::parser::RecoveryContext] has already
//!   fired at the current cursor position without the cursor moving
//!   — this is what stops recovery loops.
//! - [`push_missing_token`] emits a
//!   [`ParseErrorKind::MissingToken`] diagnostic with a zero-width
//!   `TokenRange` at the current cursor position. Does not consume
//!   tokens and does not emit any [`SyntaxKind::Error`] node — the
//!   parser never synthesizes a fake [`erl_tokenize::Token`].
//!
//! All three helpers deduplicate at the append site through
//! [`Parser::push_error`], so recovery loops that revisit the same
//! cursor position do not surface the same diagnostic twice.

use erl_tokenize::Token;

use crate::error::{Expected, ParseError, ParseErrorKind};
use crate::parser::{CompletedMarker, Parser, RecoveryContext};
use crate::syntax::SyntaxKind;
use crate::token_range::TokenRange;

/// Consumes a single lexical token at the current cursor as a
/// [`SyntaxKind::Error`] node and emits a
/// [`ParseErrorKind::SkippedToken`] diagnostic whose `range` covers
/// the same span. When the buffer is empty at the cursor, no node
/// is emitted and a [`ParseErrorKind::UnexpectedEof`] diagnostic is
/// appended at the boundary — the caller receives a zero-width
/// [`SyntaxKind::Error`] node anchored at the cursor to keep the
/// call site's signature uniform.
pub(crate) fn skip_one_token(p: &mut Parser, category: &'static str) -> CompletedMarker {
    let start = p.cursor_position();
    let found = p.peek_lexical(0).map(|(_, t)| t);
    let m = p.start();
    let consumed = p.consume_lexical().is_some();
    let completed = m.complete(p, SyntaxKind::Error);
    if consumed {
        let end = p.cursor_position();
        let node_range = TokenRange::new(start, end);
        p.push_error(ParseError::new(
            ParseErrorKind::SkippedToken,
            node_range,
            Expected::Category(category),
            found,
        ));
    } else {
        p.push_error(ParseError::new(
            ParseErrorKind::UnexpectedEof,
            TokenRange::empty_at(start),
            Expected::Category(category),
            None,
        ));
    }
    completed
}

/// Consumes tokens up to (but not including) the first token
/// `is_sync` accepts, wrapping the skipped span as a
/// [`SyntaxKind::Error`] node and emitting a
/// [`ParseErrorKind::SkippedToken`] diagnostic whose `range` matches
/// the node. Returns `None` — without emitting a node or diagnostic
/// — when the cursor is already at a sync token / EOF, or when the
/// same `(context, cursor position)` recovery pair has already fired
/// (see [`Parser::begin_recovery_attempt`]).
pub(crate) fn skip_until_sync<F>(
    p: &mut Parser,
    context: RecoveryContext,
    is_sync: F,
    category: &'static str,
) -> Option<CompletedMarker>
where
    F: Fn(Token) -> bool,
{
    match p.peek_lexical(0) {
        None => return None,
        Some((_, t)) if is_sync(t) => return None,
        Some(_) => {}
    }
    if !p.begin_recovery_attempt(context) {
        return None;
    }
    let start = p.cursor_position();
    let m = p.start();
    let mut consumed = false;
    while let Some((_, t)) = p.peek_lexical(0) {
        if is_sync(t) {
            break;
        }
        p.consume_lexical();
        consumed = true;
    }
    let completed = m.complete(p, SyntaxKind::Error);
    if consumed {
        let end = p.cursor_position();
        let node_range = TokenRange::new(start, end);
        p.push_error(ParseError::new(
            ParseErrorKind::SkippedToken,
            node_range,
            Expected::Category(category),
            None,
        ));
    }
    Some(completed)
}

/// Emits a [`ParseErrorKind::MissingToken`] diagnostic at the
/// current cursor position (zero-width [`TokenRange`]). Never
/// consumes tokens and never emits a [`SyntaxKind::Error`] node —
/// the parser refuses to synthesize a fake [`Token`].
pub(crate) fn push_missing_token(p: &mut Parser, category: &'static str) {
    let at = p.cursor_position();
    let found = p.peek_lexical(0).map(|(_, t)| t);
    p.push_error(ParseError::new(
        ParseErrorKind::MissingToken,
        TokenRange::empty_at(at),
        Expected::Category(category),
        found,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ParseMode, Parser};
    use erl_tokenize::{Position, scan_token};

    fn scan_all(source: &str) -> Vec<Token> {
        let mut out = Vec::new();
        let mut pos = Position::new();
        while let Some(t) = scan_token(source, pos).expect("valid source") {
            out.push(t);
            pos = t.end();
        }
        out
    }

    fn load(source: &str) -> Parser {
        let mut p = Parser::new(ParseMode::Module);
        for t in scan_all(source) {
            p.push_token_without_grammar_for_test(t);
        }
        p.reset_for_test();
        p
    }

    #[test]
    fn skip_one_token_emits_error_node_and_matching_diagnostic() {
        let mut p = load("foo bar");
        let outer = p.start();
        let completed = skip_one_token(&mut p, "test");
        outer.complete(&mut p, SyntaxKind::Error);
        p.finalize_pending_units_for_test();
        let _ = completed;

        let tree = p.syntax_tree();
        assert_eq!(tree.errors().len(), 1);
        let err = tree.errors()[0];
        assert_eq!(err.kind(), ParseErrorKind::SkippedToken);
        // The diagnostic range and the Error node range agree.
        let node = tree
            .syntax()
            .entry(crate::NodeId::new(1))
            .expect("skip_one_token node");
        assert_eq!(err.range(), node.range());
    }

    #[test]
    fn skip_until_sync_stops_at_sync_and_matches_range() {
        let mut p = load("junk1 junk2 . tail");
        let outer = p.start();
        let _ = skip_until_sync(
            &mut p,
            RecoveryContext::Form,
            |t| {
                matches!(
                    t.kind(),
                    erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::Dot)
                )
            },
            "test-sync",
        )
        .expect("skipped");
        outer.complete(&mut p, SyntaxKind::Error);
        p.finalize_pending_units_for_test();

        let tree = p.syntax_tree();
        assert_eq!(tree.errors().len(), 1);
        let err = tree.errors()[0];
        assert_eq!(err.kind(), ParseErrorKind::SkippedToken);
        let node = tree
            .syntax()
            .entry(crate::NodeId::new(1))
            .expect("skip_until_sync node");
        assert_eq!(err.range(), node.range());
        // Cursor stopped at `.`, not past it.
        assert!(matches!(
            p.peek_lexical(0).map(|(_, t)| t.kind()),
            Some(erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::Dot))
        ));
    }

    #[test]
    fn skip_until_sync_is_noop_when_cursor_is_already_at_sync() {
        let mut p = load(". tail");
        let outer = p.start();
        let result = skip_until_sync(
            &mut p,
            RecoveryContext::Form,
            |t| {
                matches!(
                    t.kind(),
                    erl_tokenize::TokenKind::Symbol(erl_tokenize::Symbol::Dot)
                )
            },
            "test-sync",
        );
        assert!(result.is_none());
        outer.complete(&mut p, SyntaxKind::Error);
        p.finalize_pending_units_for_test();
        assert!(p.syntax_tree().errors().is_empty());
    }

    #[test]
    fn begin_recovery_attempt_refuses_to_reenter_without_cursor_progress() {
        // `skip_until_sync` uses this hook to prevent recovery loops
        // that would otherwise fire the same helper at the same
        // cursor position over and over. The invariant is stated on
        // the low-level API: same `(context, cursor)` → refused.
        let mut p = load("foo bar");
        assert!(p.begin_recovery_attempt(RecoveryContext::Form));
        assert!(
            !p.begin_recovery_attempt(RecoveryContext::Form),
            "re-entry at the same cursor must be refused"
        );
        // A different context at the same cursor is a different pair,
        // so it is allowed.
        assert!(p.begin_recovery_attempt(RecoveryContext::Container));
        // Advancing the cursor invalidates the marker; the original
        // context is allowed again.
        let _ = p.consume_lexical();
        assert!(p.begin_recovery_attempt(RecoveryContext::Form));
    }

    #[test]
    fn push_missing_token_emits_zero_width_diagnostic_without_a_node() {
        let mut p = load("foo");
        let outer = p.start();
        let before_entries = p.syntax_tree().syntax().len();
        push_missing_token(&mut p, "missing thing");
        let after_entries = p.syntax_tree().syntax().len();
        outer.complete(&mut p, SyntaxKind::Error);
        p.finalize_pending_units_for_test();

        assert_eq!(
            after_entries, before_entries,
            "push_missing_token must not add syntax entries"
        );
        let tree = p.syntax_tree();
        assert_eq!(tree.errors().len(), 1);
        let err = tree.errors()[0];
        assert_eq!(err.kind(), ParseErrorKind::MissingToken);
        assert!(err.range().is_empty(), "missing-token range is zero-width");
    }

    #[test]
    fn push_error_deduplicates_same_kind_at_same_cursor() {
        let mut p = load("foo");
        push_missing_token(&mut p, "a");
        // Second push at the same cursor with the same kind is
        // collapsed.
        push_missing_token(&mut p, "b");
        assert_eq!(p.syntax_tree().errors().len(), 1);
    }
}
