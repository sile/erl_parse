//! Bundled output of a completed parse.
//!
//! A [`SyntaxTree`] owns everything the caller needs to keep around after
//! the parser goes away: the [`TokenBuffer`], the flat preorder
//! [`SyntaxIndex`], and the accumulated [`Diagnostic`]s. All three parts
//! reference each other through [`TokenIndex`](crate::TokenIndex) and
//! [`NodeId`](crate::NodeId), so navigation helpers work on a `SyntaxTree`
//! in the same way they do against a live [`Parser`](crate::Parser).
//!
//! `SyntaxTree` is `Clone` (all sub-components are `Clone`), so callers
//! can also snapshot the parser mid-parse via
//! [`Parser::syntax_tree`](crate::Parser::syntax_tree) and clone the result
//! if they want to decouple the snapshot from the running parser.

use crate::diagnostic::Diagnostic;
use crate::syntax::SyntaxIndex;
use crate::token_buffer::TokenBuffer;

/// The full result of a parse: input tokens, flat syntax index, and
/// accumulated diagnostics.
///
/// A strict caller checks `diagnostics().is_empty()` as its success
/// condition. A best-effort caller walks the syntax index while
/// displaying the diagnostics alongside it; ranges consumed by error
/// recovery survive as [`SyntaxKind::Error`](crate::SyntaxKind::Error)
/// nodes in the index and stay reachable through the same navigation
/// surface as any other node. See
/// [`docs::diagnostics`](crate::docs::diagnostics) for the recovery
/// contract.
#[derive(Debug, Default, Clone)]
pub struct SyntaxTree {
    tokens: TokenBuffer,
    syntax: SyntaxIndex,
    diagnostics: Vec<Diagnostic>,
}

impl SyntaxTree {
    /// Creates an empty tree.
    pub const fn new() -> Self {
        Self {
            tokens: TokenBuffer::new(),
            syntax: SyntaxIndex::new(),
            diagnostics: Vec::new(),
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

    /// Borrows the accumulated diagnostics.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Mutable access to the token buffer, for the in-crate parser core.
    pub(crate) fn tokens_mut(&mut self) -> &mut TokenBuffer {
        &mut self.tokens
    }

    /// Mutable access to the syntax index, for the in-crate parser core.
    pub(crate) fn syntax_mut(&mut self) -> &mut SyntaxIndex {
        &mut self.syntax
    }

    /// Mutable access to the diagnostic list, for the in-crate parser core.
    pub(crate) fn diagnostics_mut(&mut self) -> &mut Vec<Diagnostic> {
        &mut self.diagnostics
    }
}
