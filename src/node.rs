//! Lightweight navigation over the syntax index borrowed together with the
//! tokens the caller fed.
//!
//! Forest-level questions (`roots`, `nodes`, `innermost_containing`) live on
//! [`SyntaxTree`](crate::SyntaxTree). [`NodeView`] is one node. Neither
//! is a zipper. See [`docs::navigation`](crate::docs::navigation) for a
//! caller-facing walkthrough.
//!
//! [`NodeView`] is provided as a plain struct rather than a trait, so
//! navigation is a concrete value type rather than an abstraction. All
//! borrows share a single lifetime.

use crate::syntax::{NodeId, SyntaxIndex, SyntaxKind};
use crate::token_range::{TokenIndex, TokenRange};

/// Lightweight navigation view anchored on a specific [`NodeId`].
///
/// Kind, range, children, descendants, ancestors, and the tokens in
/// this span ([`NodeView::tokens`]). Build one with
/// [`SyntaxTree::view`](crate::SyntaxTree::view),
/// or take one from [`SyntaxTree::roots`](crate::SyntaxTree::roots) /
/// an existing view. See [`docs::navigation`](crate::docs::navigation).
#[derive(Debug, Clone, Copy)]
pub struct NodeView<'a> {
    tokens: &'a [erl_tokenize::Token],
    index: &'a SyntaxIndex,
    node_id: NodeId,
}

impl<'a> NodeView<'a> {
    /// Creates a view for a specific [`NodeId`]. Returns `None` when the id
    /// does not refer to an existing entry.
    // `pub(crate)`: pairing a token slice with an index is easy to get
    // wrong across trees. External callers use `SyntaxTree::view`.
    pub(crate) fn new(
        tokens: &'a [erl_tokenize::Token],
        index: &'a SyntaxIndex,
        node_id: NodeId,
    ) -> Option<Self> {
        if node_id.get() < index.len() {
            Some(Self {
                tokens,
                index,
                node_id,
            })
        } else {
            None
        }
    }

    /// Returns the [`NodeId`] this view is anchored on.
    pub fn node_id(self) -> NodeId {
        self.node_id
    }

    /// Returns the entry's [`SyntaxKind`].
    pub fn kind(self) -> SyntaxKind {
        self.entry_ref().kind()
    }

    /// Returns the entry's [`TokenRange`].
    pub fn token_range(self) -> TokenRange {
        self.entry_ref().range()
    }

    fn subtree_fence(self) -> usize {
        self.entry_ref().subtree_end().get()
    }

    /// Returns an iterator that walks direct children in preorder.
    pub fn children(self) -> impl Iterator<Item = NodeView<'a>> {
        Children {
            tokens: self.tokens,
            index: self.index,
            cursor: self.node_id.get() + 1,
            parent_end: self.subtree_fence(),
        }
    }

    /// Returns an iterator that walks descendants in preorder (excluding
    /// this node itself).
    pub fn descendants(self) -> impl Iterator<Item = NodeView<'a>> {
        Descendants {
            tokens: self.tokens,
            index: self.index,
            cursor: self.node_id.get() + 1,
            end: self.subtree_fence(),
        }
    }

    /// Returns the tokens inside this entry's [`TokenRange`] as a
    /// slice, in buffer order. Hidden tokens (whitespace and comments)
    /// are included, so the slice is 1:1 with the range.
    ///
    /// This is the same slice as
    /// [`SyntaxTree::tokens`](crate::SyntaxTree::tokens): the node's
    /// slice is `&tree.tokens()[self.token_range().as_slice_index()]`. The
    /// first token sits at [`TokenRange::start`](TokenRange::start), so
    /// a position in the slice is
    /// `TokenIndex::new(self.token_range().start().get() + i)` for the `i`-th
    /// element. Read a token's spelling or decoded
    /// value with [`erl_tokenize::Token::text`] /
    /// [`erl_tokenize::Token::value`] and the original source string.
    ///
    /// An empty node (a missing token or a zero-width entry) yields an
    /// empty slice; `token_range().start()` still names the anchor position.
    pub fn tokens(self) -> &'a [erl_tokenize::Token] {
        &self.tokens[self.token_range().as_slice_index()]
    }

    /// Returns an iterator over `(TokenIndex, Token)` pairs inside this
    /// entry's [`TokenRange`], in buffer order. Hidden tokens (whitespace
    /// and comments) are included, so the iterator is 1:1 with the range.
    ///
    /// The [`TokenIndex`] lets a caller look up a parallel side table —
    /// source metadata kept beside [`SyntaxTree::tokens`](crate::SyntaxTree::tokens),
    /// for example — while walking the node's tokens in one pass. This is
    /// `self.token_range().zip(self.tokens().iter().copied())`; use
    /// [`NodeView::tokens`] when a contiguous slice is needed instead.
    ///
    /// An empty node (a missing token or a zero-width entry) yields an
    /// empty iterator.
    pub fn indexed_tokens(self) -> impl Iterator<Item = (TokenIndex, erl_tokenize::Token)> {
        self.token_range().zip(self.tokens().iter().copied())
    }

    /// Returns an iterator over ancestors starting from the direct
    /// parent, moving toward the root. The node itself is not
    /// included.
    pub fn ancestors(self) -> impl Iterator<Item = NodeView<'a>> {
        Ancestors {
            tokens: self.tokens,
            index: self.index,
            child: self.node_id,
            cursor: self.node_id.get(),
        }
    }

    fn entry_ref(self) -> crate::syntax::SyntaxEntry {
        // The bounds check happens when NodeView is created, so this lookup
        // always succeeds.
        self.index
            .entry(self.node_id)
            .expect("NodeView must refer to an existing entry")
    }
}

struct Children<'a> {
    tokens: &'a [erl_tokenize::Token],
    index: &'a SyntaxIndex,
    cursor: usize,
    parent_end: usize,
}

impl<'a> Iterator for Children<'a> {
    type Item = NodeView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.parent_end {
            return None;
        }
        let node = NodeId::new(self.cursor);
        let entry = self
            .index
            .entry(node)
            .expect("children cursor stays inside the parent's subtree");
        // Skip past this child's subtree to reach the next sibling.
        self.cursor = entry.subtree_end().get();
        Some(NodeView {
            tokens: self.tokens,
            index: self.index,
            node_id: node,
        })
    }
}

struct Descendants<'a> {
    tokens: &'a [erl_tokenize::Token],
    index: &'a SyntaxIndex,
    cursor: usize,
    end: usize,
}

impl<'a> Iterator for Descendants<'a> {
    type Item = NodeView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.end {
            return None;
        }
        let node = NodeId::new(self.cursor);
        self.cursor += 1;
        Some(NodeView {
            tokens: self.tokens,
            index: self.index,
            node_id: node,
        })
    }
}

struct Ancestors<'a> {
    tokens: &'a [erl_tokenize::Token],
    index: &'a SyntaxIndex,
    child: NodeId,
    cursor: usize,
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = NodeView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // An ancestor is any entry that precedes the child and whose subtree
        // strictly contains it. Scanning cursor backward from just before
        // the child yields the direct parent first, then successively
        // closer-root ancestors: entries between an ancestor and the child
        // sit inside earlier siblings and end before the child, so the
        // first containing entry found going backward is always the
        // nearest ancestor not yet yielded.
        while self.cursor > 0 {
            self.cursor -= 1;
            let candidate = NodeId::new(self.cursor);
            let entry = self
                .index
                .entry(candidate)
                .expect("cursor stays inside the index");
            if entry.subtree_end().get() > self.child.get() {
                return Some(NodeView {
                    tokens: self.tokens,
                    index: self.index,
                    node_id: candidate,
                });
            }
        }
        None
    }
}

/// Iterator over root-level nodes of `tokens` / `index`.
pub(crate) fn root_views<'a>(
    tokens: &'a [erl_tokenize::Token],
    index: &'a SyntaxIndex,
) -> impl Iterator<Item = NodeView<'a>> {
    Roots {
        tokens,
        index,
        at: 0,
    }
}

/// Innermost node whose non-empty range contains `target`.
pub(crate) fn innermost_containing<'a>(
    tokens: &'a [erl_tokenize::Token],
    index: &'a SyntaxIndex,
    target: TokenIndex,
) -> Option<NodeView<'a>> {
    let entries = index.entries();
    let mut deepest: Option<NodeId> = None;
    let mut i = 0;
    while i < entries.len() {
        let entry = entries[i];
        let range = entry.range();
        let contains = !range.is_empty()
            && range.start().get() <= target.get()
            && target.get() < range.end().get();
        if contains {
            deepest = Some(NodeId::new(i));
            i += 1;
        } else {
            // This subtree does not contain the target; skip past it.
            i = entry.subtree_end().get();
        }
    }
    deepest.map(|node_id| NodeView {
        tokens,
        index,
        node_id,
    })
}

struct Roots<'a> {
    tokens: &'a [erl_tokenize::Token],
    index: &'a SyntaxIndex,
    at: usize,
}

impl<'a> Iterator for Roots<'a> {
    type Item = NodeView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let entries = self.index.entries();
        if self.at >= entries.len() {
            return None;
        }
        let node = NodeId::new(self.at);
        let entry = entries[self.at];
        self.at = entry.subtree_end().get();
        Some(NodeView {
            tokens: self.tokens,
            index: self.index,
            node_id: node,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{EntryIndex, SyntaxEntry, SyntaxIndex, SyntaxKind};
    use crate::token_range::{TokenIndex, TokenRange};

    fn range(start: usize, end: usize) -> TokenRange {
        TokenRange::new(TokenIndex::new(start), TokenIndex::new(end))
    }

    fn build_sample() -> (Vec<erl_tokenize::Token>, SyntaxIndex) {
        // Scan "foo bar" into a buffer of three tokens: atom, whitespace,
        // atom (i.e. two lexical + one hidden token).
        let source = "foo bar";
        let mut tokens = Vec::new();
        let mut pos = erl_tokenize::Position::new();
        while let Some(token) = erl_tokenize::scan_token(source, pos).expect("valid Erlang source")
        {
            tokens.push(token);
            pos = token.end();
        }
        assert_eq!(tokens.len(), 3, "foo, whitespace, bar");

        // Syntax index layout:
        //   0: parent      kind=Error  range=0..3  subtree_end=3
        //     1: child_a   range=0..1  subtree_end=2
        //     2: child_b   range=2..3  subtree_end=3
        let mut index = SyntaxIndex::new();
        let _parent = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            range(0, 3),
            EntryIndex::new(3),
        ));
        let _child_a = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            range(0, 1),
            EntryIndex::new(2),
        ));
        let _child_b = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            range(2, 3),
            EntryIndex::new(3),
        ));
        (tokens, index)
    }

    fn build_deep_sample() -> (Vec<erl_tokenize::Token>, SyntaxIndex) {
        // Same token buffer as `build_sample` (three tokens); the syntax
        // index is a 3-level tree:
        //   0: root      range=0..4  subtree_end=4
        //     1: mid      range=0..2  subtree_end=3
        //       2: leaf   range=0..1  subtree_end=3
        //     3: sibling  range=2..4  subtree_end=4
        let source = "foo bar";
        let mut tokens = Vec::new();
        let mut pos = erl_tokenize::Position::new();
        while let Some(token) = erl_tokenize::scan_token(source, pos).expect("valid Erlang source")
        {
            tokens.push(token);
            pos = token.end();
        }
        assert_eq!(tokens.len(), 3, "foo, whitespace, bar");

        let mut index = SyntaxIndex::new();
        let _root = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            range(0, 4),
            EntryIndex::new(4),
        ));
        let _mid = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            range(0, 2),
            EntryIndex::new(3),
        ));
        let _leaf = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            range(0, 1),
            EntryIndex::new(3),
        ));
        let _sibling = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            range(2, 4),
            EntryIndex::new(4),
        ));
        (tokens, index)
    }

    #[test]
    fn out_of_bounds_view_is_none() {
        let (tokens, index) = build_sample();
        assert!(NodeView::new(&tokens, &index, NodeId::new(3)).is_none());
    }

    #[test]
    fn direct_children_walk() {
        let (tokens, index) = build_sample();
        let parent = NodeView::new(&tokens, &index, NodeId::new(0))
            .expect("node id refers to an existing entry");
        let ids: Vec<NodeId> = parent.children().map(|v| v.node_id()).collect();
        assert_eq!(ids, vec![NodeId::new(1), NodeId::new(2)]);
    }

    #[test]
    fn descendants_walk_preorder() {
        let (tokens, index) = build_sample();
        let parent = NodeView::new(&tokens, &index, NodeId::new(0))
            .expect("node id refers to an existing entry");
        let ids: Vec<usize> = parent.descendants().map(|v| v.node_id().get()).collect();
        assert_eq!(ids, vec![1, 2]);
    }

    #[test]
    fn ancestors_walk_returns_containing_nodes_in_parent_first_order() {
        let (tokens, index) = build_deep_sample();
        let leaf = NodeView::new(&tokens, &index, NodeId::new(2))
            .expect("node id refers to an existing entry");
        let ids: Vec<usize> = leaf.ancestors().map(|v| v.node_id().get()).collect();
        assert_eq!(ids, vec![1, 0]);
    }

    #[test]
    fn tokens_returns_hidden_and_lexical_in_order() {
        let (tokens, index) = build_sample();
        let parent = NodeView::new(&tokens, &index, NodeId::new(0))
            .expect("node id refers to an existing entry");
        let kinds: Vec<erl_tokenize::TokenKind> =
            parent.tokens().iter().map(|t| t.kind()).collect();
        // 0: atom (foo), 1: whitespace, 2: atom (bar). The slice is 1:1
        // with the range and starts at its `start()`.
        assert_eq!(parent.tokens().len(), 3);
        assert_eq!(parent.token_range().start(), TokenIndex::new(0));
        assert_eq!(kinds.len(), 3);
        assert!(kinds[1].is_hidden(), "whitespace must be hidden");
        assert!(parent.tokens().len() == parent.token_range().len());
    }

    #[test]
    fn indexed_tokens_yields_index_and_token_pairs_in_order() {
        let (tokens, index) = build_sample();
        let parent = NodeView::new(&tokens, &index, NodeId::new(0))
            .expect("node id refers to an existing entry");
        let pairs: Vec<(TokenIndex, erl_tokenize::Token)> = parent.indexed_tokens().collect();
        assert_eq!(pairs.len(), 3);
        for (i, (idx, token)) in pairs.iter().enumerate() {
            assert_eq!(*idx, TokenIndex::new(i));
            assert_eq!(*token, tokens[i], "index {i} must fetch the same token");
        }
        assert!(pairs[1].1.kind().is_hidden(), "whitespace must be hidden");
    }

    #[test]
    fn indexed_tokens_is_empty_for_zero_width_node() {
        let source = "foo";
        let mut tokens = Vec::new();
        let mut pos = erl_tokenize::Position::new();
        while let Some(token) = erl_tokenize::scan_token(source, pos).expect("valid Erlang source")
        {
            tokens.push(token);
            pos = token.end();
        }

        let mut index = SyntaxIndex::new();
        let zero = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            TokenRange::empty_at(TokenIndex::new(1)),
            EntryIndex::new(2),
        ));

        let zero_view =
            NodeView::new(&tokens, &index, zero).expect("node id refers to an existing entry");
        assert_eq!(zero_view.indexed_tokens().count(), 0);
    }

    #[test]
    fn innermost_containing_prefers_deepest() {
        let (tokens, index) = build_sample();
        let found = innermost_containing(&tokens, &index, TokenIndex::new(0))
            .expect("target lies inside an entry");
        assert_eq!(found.node_id(), NodeId::new(1));
        let found2 = innermost_containing(&tokens, &index, TokenIndex::new(2))
            .expect("target lies inside an entry");
        assert_eq!(found2.node_id(), NodeId::new(2));
    }

    #[test]
    fn innermost_containing_returns_none_for_out_of_range() {
        let (tokens, index) = build_sample();
        assert!(innermost_containing(&tokens, &index, TokenIndex::new(3)).is_none());
    }

    #[test]
    fn zero_width_node_is_navigable_but_not_containing() {
        // A zero-width node exists as a navigable entry, but
        // `innermost_containing` never selects it: an empty range does not
        // contain any position.
        let source = "foo";
        let mut tokens = Vec::new();
        let mut pos = erl_tokenize::Position::new();
        while let Some(token) = erl_tokenize::scan_token(source, pos).expect("valid Erlang source")
        {
            tokens.push(token);
            pos = token.end();
        }

        let mut index = SyntaxIndex::new();
        let parent = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            range(0, 1),
            EntryIndex::new(2),
        ));
        let zero = index.push(SyntaxEntry::new(
            SyntaxKind::Error,
            TokenRange::empty_at(TokenIndex::new(1)),
            EntryIndex::new(2),
        ));

        let parent_view =
            NodeView::new(&tokens, &index, parent).expect("node id refers to an existing entry");
        let child_ids: Vec<NodeId> = parent_view.children().map(|v| v.node_id()).collect();
        assert_eq!(child_ids, vec![zero]);

        let zero_view =
            NodeView::new(&tokens, &index, zero).expect("node id refers to an existing entry");
        assert!(zero_view.token_range().is_empty());
        // The zero-width child yields no tokens; `token_range().start()` still
        // names the anchor position.
        assert!(zero_view.tokens().is_empty());
        assert_eq!(zero_view.token_range().start(), TokenIndex::new(1));

        // `innermost_containing(1)` selects neither the zero-width child
        // (empty range) nor the parent (range 0..1 does not contain 1).
        assert!(innermost_containing(&tokens, &index, TokenIndex::new(1)).is_none());
    }
}
