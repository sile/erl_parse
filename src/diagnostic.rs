//! Parse diagnostic type.
//!
//! A [`Diagnostic`] is a report about a syntax problem at a token-buffer
//! range. The parser accumulates these on [`crate::SyntaxTree`] rather than
//! returning them as `Result::Err`; a parse always produces a tree, and
//! a strict caller treats [`crate::SyntaxTree::diagnostics`] being empty
//! as success.
//!
//! Every diagnostic currently produced is a syntax error. Warnings and
//! informational notes are not emitted yet. How the grammar continues
//! after a diagnostic is recorded is in [`crate::docs::diagnostics`].

use erl_tokenize::Token;

use crate::token_range::TokenRange;

/// A syntax diagnostic surfaced by the parser.
///
/// This is a diagnostic record, not an operation-failure type: it does
/// not implement [`std::error::Error`], and the parser never returns it
/// as `Result::Err`. See [`crate::docs::diagnostics`] for the recovery
/// contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Diagnostic {
    kind: DiagnosticKind,
    range: TokenRange,
    expected: Expected,
    found: Option<Token>,
}

impl Diagnostic {
    /// Builds a `Diagnostic` from its components.
    pub const fn new(
        kind: DiagnosticKind,
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

    /// Returns the diagnostic's kind.
    pub const fn kind(self) -> DiagnosticKind {
        self.kind
    }

    /// Returns the primary range the diagnostic anchors on. Empty ranges
    /// are used for boundary reports (for example unexpected EOF).
    pub const fn range(self) -> TokenRange {
        self.range
    }

    /// Returns what the grammar was expecting.
    pub const fn expected(self) -> Expected {
        self.expected
    }

    /// Returns the token that was actually found, if any. `None` when the
    /// diagnostic anchors at a boundary (unexpected EOF) or when no
    /// specific token can be blamed.
    pub const fn found(self) -> Option<Token> {
        self.found
    }
}

/// Category of a [`Diagnostic`].
///
/// The enum is not marked `#[non_exhaustive]`; adding a variant is treated
/// as a normal breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticKind {
    /// A token was found where a different token was expected.
    UnexpectedToken,
    /// End of input was reached where more tokens were expected.
    UnexpectedEof,
    /// One or more tokens were skipped by error recovery. The
    /// [`Diagnostic::range`] covers the skipped span and matches the
    /// [`crate::SyntaxKind::Error`] node emitted for the same span so
    /// consumers can navigate from a diagnostic to the structural
    /// hole (and vice versa).
    SkippedToken,
    /// A required token was missing at the current cursor position.
    /// The [`Diagnostic::range`] is zero-width (`start == end`) at
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

/// Appends `diagnostic` unless the immediately preceding element already
/// carries the same `kind` and starts at the same
/// [`TokenRange::start`]. This is a lightweight deduplication that
/// keeps a recovery loop from re-emitting the same diagnostic when
/// it tries several alternatives against the same cursor position;
/// it deliberately does not scan the whole vector so appending
/// stays O(1).
pub(crate) fn push_unique_at_cursor(diagnostics: &mut Vec<Diagnostic>, diagnostic: Diagnostic) {
    if let Some(last) = diagnostics.last()
        && last.kind() == diagnostic.kind()
        && last.range().start() == diagnostic.range().start()
    {
        return;
    }
    diagnostics.push(diagnostic);
}

/// What the grammar was expecting when a [`Diagnostic`] fired.
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
