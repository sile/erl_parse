//! Parse error type.

use erl_tokenize::Token;

use crate::token_range::TokenRange;

/// A syntactic error surfaced by the parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseError {
    kind: ParseErrorKind,
    range: TokenRange,
    expected: Expected,
    found: Option<Token>,
}

impl ParseError {
    /// Builds a `ParseError` from its components.
    pub const fn new(
        kind: ParseErrorKind,
        range: TokenRange,
        expected: Expected,
        found: Option<Token>,
    ) -> Self {
        Self {
            kind,
            range,
            expected,
            found,
        }
    }

    /// Returns the error's kind.
    pub const fn kind(self) -> ParseErrorKind {
        self.kind
    }

    /// Returns the primary range the error anchors on. Empty ranges are used
    /// for boundary errors (for example unexpected EOF).
    pub const fn range(self) -> TokenRange {
        self.range
    }

    /// Returns what the grammar was expecting.
    pub const fn expected(self) -> Expected {
        self.expected
    }

    /// Returns the token that was actually found, if any. `None` when the
    /// error anchors at a boundary (unexpected EOF) or when no specific
    /// token can be blamed.
    pub const fn found(self) -> Option<Token> {
        self.found
    }
}

/// Category of a `ParseError`.
///
/// The enum is not marked `#[non_exhaustive]`; adding a variant is treated
/// as a normal breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseErrorKind {
    /// A token was found where a different token was expected.
    UnexpectedToken,
    /// End of input was reached where more tokens were expected.
    UnexpectedEof,
    /// One or more tokens were skipped by error recovery. The
    /// [`ParseError::range`] covers the skipped span and matches the
    /// [`crate::SyntaxKind::Error`] node emitted for the same span so
    /// consumers can navigate from a diagnostic to the structural
    /// hole (and vice versa).
    SkippedToken,
    /// A required token was missing at the current cursor position.
    /// The [`ParseError::range`] is zero-width (`start == end`) at
    /// the boundary where the token would have appeared. The parser
    /// does not synthesize a fake token — no
    /// [`crate::SyntaxKind::Error`] node is emitted for a missing
    /// token.
    MissingToken,
    /// The grammar's nesting depth exceeded
    /// [`crate::Parser::MAX_NESTING_DEPTH`] at this position. The
    /// parser stops recursing (instead of panicking or overflowing
    /// the stack), unwinds to a bounded depth, and continues
    /// recovery.
    NestingDepthExceeded,
}

/// Appends `error` to `errors` unless the immediately preceding
/// element already carries the same `kind` and starts at the same
/// [`TokenRange::start`]. This is a lightweight deduplication that
/// keeps a recovery loop from re-emitting the same diagnostic when
/// it tries several alternatives against the same cursor position;
/// it deliberately does not scan the whole vector so appending
/// stays O(1).
pub(crate) fn push_unique_at_cursor(errors: &mut Vec<ParseError>, error: ParseError) {
    if let Some(last) = errors.last()
        && last.kind() == error.kind()
        && last.range().start() == error.range().start()
    {
        return;
    }
    errors.push(error);
}

/// What the grammar was expecting when a `ParseError` fired.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Expected {
    /// The site did not commit to a specific expectation.
    Unspecified,
    /// A specific token kind was expected.
    TokenKind(erl_tokenize::TokenKind),
    /// A grammar-level category was expected. The identifier is chosen by
    /// the grammar site.
    Category(&'static str),
}
