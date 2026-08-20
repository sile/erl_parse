//! Parse error and the append-only sink that collects them.

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
/// Additional variants for recovery kinds are added by later changes; this
/// enum is not marked `#[non_exhaustive]` so adding a variant is a normal
/// breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParseErrorKind {
    /// A token was found where a different token was expected.
    UnexpectedToken,
    /// End of input was reached where more tokens were expected.
    UnexpectedEof,
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

/// Append-only sink for `ParseError` values.
///
/// The parser owns one of these; callers observe accumulated errors through
/// the parser's `errors()` method.
#[derive(Debug, Default)]
pub(crate) struct ErrorSink {
    errors: Vec<ParseError>,
}

impl ErrorSink {
    /// Creates an empty sink.
    pub(crate) const fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Returns the number of errors currently held.
    pub(crate) fn len(&self) -> usize {
        self.errors.len()
    }

    /// Borrows the sink contents as a slice.
    pub(crate) fn as_slice(&self) -> &[ParseError] {
        &self.errors
    }

    /// Appends an error to the sink.
    pub(crate) fn push(&mut self, error: ParseError) {
        self.errors.push(error);
    }

    /// Truncates the sink to the given length; used to unwind after a
    /// failed alternative during checkpoint restore.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Invoked from Parser::restore, which grammar code drives later; currently only tests exercise it"
        )
    )]
    pub(crate) fn truncate(&mut self, len: usize) {
        self.errors.truncate(len);
    }
}

/// Misuse of the parser's push/pull protocol by the caller.
///
/// This is the error type of the parser's public API surface where a
/// protocol violation is possible; it is deliberately kept separate from
/// [`ParseError`] so that programming errors and grammar errors take
/// different code paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProtocolError {
    /// A token was pushed after `finish` had already been asserted.
    PushAfterFinish,
    /// `finish` was called more than once.
    FinishTwice,
}
