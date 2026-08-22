//! Bundled output of a completed parse.
//!
//! A [`SyntaxTree`] owns everything the caller needs to keep around after
//! the parser goes away: the tokens they fed, the flat preorder syntax
//! index, and the accumulated [`Diagnostic`]s. Tokens and nodes
//! reference each other through [`TokenIndex`](crate::TokenIndex)
//! and [`NodeId`](crate::NodeId), so [`NodeView`](crate::NodeView) works
//! on a `SyntaxTree` in the same way it does against a live
//! [`Parser`](crate::Parser). Forest-level walks
//! ([`SyntaxTree::roots`], [`SyntaxTree::innermost_containing`]) and
//! [`SyntaxTree::view`] keep the tokens and index paired. See
//! [`docs::navigation`](crate::docs::navigation).
//!
//! `SyntaxTree` is `Clone` (all sub-components are `Clone`), so callers
//! can also snapshot the parser mid-parse via
//! [`Parser::syntax_tree`](crate::Parser::syntax_tree) and clone the result
//! if they want to decouple the snapshot from the running parser.

use crate::diagnostic::Diagnostic;
use crate::node::NodeView;
use crate::syntax::{NodeId, SyntaxIndex};
use crate::token_buffer::TokenBuffer;
use crate::token_range::TokenIndex;
use erl_tokenize::Token;

/// The full result of a parse: input tokens, syntax nodes, and
/// accumulated diagnostics.
///
/// A strict caller checks `diagnostics().is_empty()` as its success
/// condition. A best-effort caller walks the forest while displaying
/// the diagnostics alongside it; ranges consumed by error recovery
/// survive as [`SyntaxKind::Error`](crate::SyntaxKind::Error) nodes
/// and stay reachable through the same navigation surface as any
/// other node. See [`docs::diagnostics`](crate::docs::diagnostics)
/// for the recovery contract and
/// [`docs::navigation`](crate::docs::navigation) for walking the
/// tree.
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

    /// Tokens the caller fed, in feed order.
    ///
    /// The sequence is append-only: a [`TokenIndex`] obtained earlier
    /// still names the same token after later feeds. Index a single
    /// token with [`TokenIndex::get`]; slice a span with
    /// [`TokenRange::as_range`](crate::TokenRange::as_range).
    pub fn tokens(&self) -> &[Token] {
        self.tokens.as_slice()
    }

    /// Borrows the crate-internal token buffer.
    pub(crate) fn token_buffer(&self) -> &TokenBuffer {
        &self.tokens
    }

    /// Borrows the crate-internal syntax index.
    #[cfg(test)]
    pub(crate) fn syntax(&self) -> &SyntaxIndex {
        &self.syntax
    }

    /// Returns an iterator over root-level nodes (each `.`-terminated
    /// unit in the preorder array).
    pub fn roots(&self) -> impl Iterator<Item = NodeView<'_>> {
        crate::node::root_views(&self.tokens, &self.syntax)
    }

    /// Returns the innermost node whose non-empty range contains
    /// `target`.
    ///
    /// A non-empty range `[start, end)` contains `target` when
    /// `start <= target < end`. Empty ranges never contain any position.
    pub fn innermost_containing(&self, target: TokenIndex) -> Option<NodeView<'_>> {
        crate::node::innermost_containing(&self.tokens, &self.syntax, target)
    }

    /// Returns a [`NodeView`] for `node_id` in this tree, or `None`
    /// when the id does not refer to an existing entry.
    ///
    /// This is the public constructor for [`NodeView`] from a
    /// [`NodeId`]. Views obtained from [`SyntaxTree::roots`] or from
    /// another view's child / descendant / ancestor iterators are
    /// already bound to a tree.
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
