//! Bundled output of a completed parse.
//!
//! A [`SyntaxTree`] owns everything the caller needs to keep around after
//! the parser goes away: the [`TokenBuffer`], the flat preorder
//! [`SyntaxIndex`], and the accumulated [`Diagnostic`]s. All three parts
//! reference each other through [`TokenIndex`](crate::TokenIndex) and
//! [`NodeId`](crate::NodeId), so [`Cursor`](crate::Cursor) and
//! [`NodeView`](crate::NodeView) work on a `SyntaxTree` in the same
//! way they do against a live [`Parser`](crate::Parser). Construct
//! them with [`SyntaxTree::cursor`] and [`SyntaxTree::view`]. See
//! [`docs::navigation`](crate::docs::navigation).
//!
//! `SyntaxTree` is `Clone` (all sub-components are `Clone`), so callers
//! can also snapshot the parser mid-parse via
//! [`Parser::syntax_tree`](crate::Parser::syntax_tree) and clone the result
//! if they want to decouple the snapshot from the running parser.

use crate::diagnostic::Diagnostic;
use crate::node::{Cursor, NodeView};
use crate::syntax::{NodeId, SyntaxIndex};
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
/// contract and [`docs::navigation`](crate::docs::navigation) for
/// walking the index.
#[derive(Debug, Clone)]
pub struct SyntaxTree {
    tokens: TokenBuffer,
    syntax: SyntaxIndex,
    diagnostics: Vec<Diagnostic>,
}

impl SyntaxTree {
    /// Creates an empty tree.
    // `pub(crate)`: callers receive a tree from `Parser::finish` /
    // `Parser::syntax_tree`. An empty tree has no tokens to pair with.
    pub(crate) const fn new() -> Self {
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

    /// Returns a [`Cursor`] over this tree's token buffer and syntax
    /// index.
    ///
    /// This is the public constructor for [`Cursor`]: the buffer and
    /// index always belong to the same tree.
    pub fn cursor(&self) -> Cursor<'_> {
        Cursor::new(&self.tokens, &self.syntax)
    }

    /// Returns a [`NodeView`] for `node_id` in this tree, or `None`
    /// when the id does not refer to an existing entry.
    ///
    /// This is the public constructor for [`NodeView`] from a
    /// [`NodeId`]. Views obtained from [`Cursor`] or from another
    /// view's child / descendant / ancestor iterators are already
    /// bound to a tree.
    pub fn view(&self, node_id: NodeId) -> Option<NodeView<'_>> {
        NodeView::new(&self.tokens, &self.syntax, node_id)
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
