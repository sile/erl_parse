//! Bundled output of a completed parse.
//!
//! A [`SyntaxTree`] owns everything the caller needs to keep around after
//! the parser goes away: the pushed [`TokenBuffer`], the flat preorder
//! [`SyntaxIndex`], and the accumulated [`ParseError`]s. All three parts
//! reference each other through [`crate::TokenIndex`] and [`crate::NodeId`],
//! so navigation helpers work on a `SyntaxTree` in the same way they do
//! against a live [`crate::Parser`].
//!
//! `SyntaxTree` is `Clone` (all sub-components are `Clone`), so callers
//! can also snapshot the parser mid-parse via
//! [`crate::Parser::syntax_tree`] and clone the result if they want to
//! decouple the snapshot from the running parser.

use crate::error::ParseError;
use crate::syntax::SyntaxIndex;
use crate::token_buffer::TokenBuffer;

/// The full result of a parse: input tokens, flat syntax index, and
/// accumulated errors.
///
/// A strict caller checks `errors().is_empty()` as its success condition.
/// A best-effort caller walks the syntax index while displaying the
/// errors alongside it; ranges consumed by error recovery survive as
/// [`crate::SyntaxKind::Error`] nodes in the index and stay reachable
/// through the same navigation surface as any other node.
#[derive(Debug, Default, Clone)]
pub struct SyntaxTree {
    tokens: TokenBuffer,
    syntax: SyntaxIndex,
    errors: Vec<ParseError>,
}

impl SyntaxTree {
    /// Creates an empty tree.
    pub const fn new() -> Self {
        Self {
            tokens: TokenBuffer::new(),
            syntax: SyntaxIndex::new(),
            errors: Vec::new(),
        }
    }

    /// Borrows the token buffer.
    pub fn tokens(&self) -> &TokenBuffer {
        &self.tokens
    }

    /// Borrows the syntax index.
    pub fn syntax(&self) -> &SyntaxIndex {
        &self.syntax
    }

    /// Borrows the accumulated parse errors.
    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    /// Mutable access to the token buffer, for the in-crate parser core.
    pub(crate) fn tokens_mut(&mut self) -> &mut TokenBuffer {
        &mut self.tokens
    }

    /// Mutable access to the syntax index, for the in-crate parser core.
    pub(crate) fn syntax_mut(&mut self) -> &mut SyntaxIndex {
        &mut self.syntax
    }

    /// Mutable access to the error list, for the in-crate parser core.
    pub(crate) fn errors_mut(&mut self) -> &mut Vec<ParseError> {
        &mut self.errors
    }
}
