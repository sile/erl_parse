//! Lightweight navigation over a [`SyntaxIndex`] borrowed together with its
//! [`TokenBuffer`].
//!
//! [`Cursor`] is the whole forest; [`NodeView`] is one node. Neither is
//! a zipper. Construct them from
//! [`SyntaxTree::cursor`](crate::SyntaxTree::cursor) /
//! [`SyntaxTree::view`](crate::SyntaxTree::view). See
//! [`docs::navigation`](crate::docs::navigation) for a caller-facing
//! walkthrough.
//!
//! [`NodeView`] is provided as a plain struct rather than a trait, so
//! navigation is a concrete value type rather than an abstraction. All
//! borrows share a single lifetime.

use erl_tokenize::Token;

use crate::syntax::{NodeId, SyntaxIndex, SyntaxKind};
use crate::token_buffer::TokenBuffer;
use crate::token_range::{TokenIndex, TokenRange};

/// Lightweight navigation view anchored on a specific [`NodeId`].
///
/// Kind, range, children, descendants, ancestors, and the tokens in
/// this span. Build one with [`SyntaxTree::view`](crate::SyntaxTree::view),
/// or take one from a [`Cursor`] / existing view. See
/// [`docs::navigation`](crate::docs::navigation).
#[derive(Debug, Clone, Copy)]
pub struct NodeView<'a> {
    tokens: &'a TokenBuffer,
    index: &'a SyntaxIndex,
    node_id: NodeId,
}

impl<'a> NodeView<'a> {
    /// Creates a view for a specific [`NodeId`]. Returns `None` when the id
    /// does not refer to an existing entry.
    // `pub(crate)`: pairing a buffer with an index is easy to get
    // wrong across trees. External callers use `SyntaxTree::view`.
    pub(crate) fn new(
        tokens: &'a TokenBuffer,
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
    pub fn range(self) -> TokenRange {
        self.entry_ref().range()
    }

    fn subtree_fence(self) -> usize {
        self.entry_ref().subtree_end().get()
    }

    /// Returns the first direct child, or `None` when this node is a leaf.
    pub fn first_child(self) -> Option<NodeView<'a>> {
        let candidate = self.node_id.get() + 1;
        if candidate >= self.subtree_fence() {
            None
        } else {
            Some(Self {
                tokens: self.tokens,
                index: self.index,
                node_id: NodeId::new(candidate),
            })
        }
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

    /// Returns an iterator over `(TokenIndex, Token)` pairs within this
    /// entry's [`TokenRange`]. Hidden tokens appear in their original buffer
    /// order.
    pub fn tokens_in_range(self) -> impl Iterator<Item = (TokenIndex, Token)> {
        self.tokens.iter_range(self.range())
    }

    /// Returns an iterator over ancestors starting from the root, moving
    /// toward the direct parent. The node itself is not included.
    pub fn ancestors(self) -> impl Iterator<Item = NodeView<'a>> {
        Ancestors {
            tokens: self.tokens,
            index: self.index,
            child: self.node_id,
            cursor: 0,
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
    tokens: &'a TokenBuffer,
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
    tokens: &'a TokenBuffer,
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
    tokens: &'a TokenBuffer,
    index: &'a SyntaxIndex,
    child: NodeId,
    cursor: usize,
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = NodeView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // An ancestor is any entry that precedes the child and whose subtree
        // strictly contains it. Scanning cursor forward from the start
        // yields ancestors in outermost-first order (root, then successively
        // closer ancestors).
        while self.cursor < self.child.get() {
            let candidate = NodeId::new(self.cursor);
            self.cursor += 1;
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

/// Whole-index traversal helper.
///
/// Provides operations that a [`NodeView`] cannot express by itself, such
/// as listing root units or finding the innermost node containing a
/// specific [`TokenIndex`]. There is no current node: this is not a
/// zipper. Construct with [`SyntaxTree::cursor`](crate::SyntaxTree::cursor).
/// See [`docs::navigation`](crate::docs::navigation).
#[derive(Debug, Clone, Copy)]
pub struct Cursor<'a> {
    tokens: &'a TokenBuffer,
    index: &'a SyntaxIndex,
}

impl<'a> Cursor<'a> {
    /// Creates a cursor from a token buffer and a syntax index.
    // `pub(crate)`: pairing a buffer with an index is easy to get
    // wrong across trees. External callers use `SyntaxTree::cursor`.
    pub(crate) fn new(tokens: &'a TokenBuffer, index: &'a SyntaxIndex) -> Self {
        Self { tokens, index }
    }

    /// Returns an iterator over root-level nodes (topmost entries in the
    /// preorder array).
    pub fn roots(self) -> impl Iterator<Item = NodeView<'a>> {
        Roots {
            tokens: self.tokens,
            index: self.index,
            cursor: 0,
        }
    }

    /// Returns the innermost node that contains `target`.
    ///
    /// A non-empty range `[start, end)` contains `target` when
    /// `start <= target < end`. Empty ranges never contain any position.
    pub fn innermost_containing(self, target: TokenIndex) -> Option<NodeView<'a>> {
        let entries = self.index.entries();
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
            tokens: self.tokens,
            index: self.index,
            node_id,
        })
    }
}

struct Roots<'a> {
    tokens: &'a TokenBuffer,
    index: &'a SyntaxIndex,
    cursor: usize,
}

impl<'a> Iterator for Roots<'a> {
    type Item = NodeView<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let entries = self.index.entries();
        if self.cursor >= entries.len() {
            return None;
        }
        let node = NodeId::new(self.cursor);
        let entry = entries[self.cursor];
        self.cursor = entry.subtree_end().get();
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
    use erl_tokenize::{Position, scan_token};

    fn range(start: usize, end: usize) -> TokenRange {
        TokenRange::new(TokenIndex::new(start), TokenIndex::new(end))
    }

    fn build_sample() -> (TokenBuffer, SyntaxIndex) {
        // Scan "foo bar" into a buffer of three tokens: atom, whitespace,
        // atom (i.e. two lexical + one hidden token).
        let source = "foo bar";
        let mut tokens = TokenBuffer::new();
        let mut pos = Position::new();
        while let Some(token) = scan_token(source, pos).expect("valid Erlang source") {
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
    fn ancestors_walk_returns_containing_nodes_in_root_first_order() {
        let (tokens, index) = build_sample();
        let child = NodeView::new(&tokens, &index, NodeId::new(2))
            .expect("node id refers to an existing entry");
        let ids: Vec<usize> = child.ancestors().map(|v| v.node_id().get()).collect();
        assert_eq!(ids, vec![0]);
    }

    #[test]
    fn tokens_in_range_returns_hidden_and_lexical_in_order() {
        let (tokens, index) = build_sample();
        let parent = NodeView::new(&tokens, &index, NodeId::new(0))
            .expect("node id refers to an existing entry");
        let collected: Vec<(usize, erl_tokenize::TokenKind)> = parent
            .tokens_in_range()
            .map(|(idx, tok)| (idx.get(), tok.kind()))
            .collect();
        // 0: atom (foo), 1: whitespace, 2: atom (bar).
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0].0, 0);
        assert_eq!(collected[1].0, 1);
        assert!(collected[1].1.is_hidden(), "whitespace must be hidden");
        assert_eq!(collected[2].0, 2);
    }

    #[test]
    fn cursor_innermost_containing_prefers_deepest() {
        let (tokens, index) = build_sample();
        let cursor = Cursor::new(&tokens, &index);
        let found = cursor
            .innermost_containing(TokenIndex::new(0))
            .expect("target lies inside an entry");
        assert_eq!(found.node_id(), NodeId::new(1));
        let found2 = cursor
            .innermost_containing(TokenIndex::new(2))
            .expect("target lies inside an entry");
        assert_eq!(found2.node_id(), NodeId::new(2));
    }

    #[test]
    fn cursor_innermost_containing_returns_none_for_out_of_range() {
        let (tokens, index) = build_sample();
        let cursor = Cursor::new(&tokens, &index);
        assert!(cursor.innermost_containing(TokenIndex::new(3)).is_none());
    }

    #[test]
    fn zero_width_node_is_navigable_but_not_containing() {
        // A zero-width node exists as a navigable entry, but
        // `innermost_containing` never selects it: an empty range does not
        // contain any position.
        let source = "foo";
        let mut tokens = TokenBuffer::new();
        let mut pos = Position::new();
        while let Some(token) = scan_token(source, pos).expect("valid Erlang source") {
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
        assert!(zero_view.range().is_empty());
        // The zero-width child yields no tokens through tokens_in_range.
        assert_eq!(zero_view.tokens_in_range().count(), 0);

        // `innermost_containing(1)` selects neither the zero-width child
        // (empty range) nor the parent (range 0..1 does not contain 1).
        let cursor = Cursor::new(&tokens, &index);
        assert!(cursor.innermost_containing(TokenIndex::new(1)).is_none());
    }
}
