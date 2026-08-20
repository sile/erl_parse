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
