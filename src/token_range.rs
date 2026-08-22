//! Index and range types over the token buffer.

use core::ops::Range;

/// A position in the token buffer.
///
/// A single [`TokenIndex`] can refer either to an existing token or to a
/// boundary (the trailing EOF position or the endpoint of an empty
/// [`TokenRange`]). Values lie in `0..=tokens.len()`.
///
/// Unlike [`NodeId`](crate::NodeId), this is the caller's address space:
/// [`SyntaxTree::tokens`](crate::SyntaxTree::tokens) is the sequence they
/// fed via [`Parser::feed_token`](crate::Parser::feed_token). Constructing
/// from a slice position or from arithmetic on [`TokenIndex::get`] is
/// expected. "Existing element" and "boundary" are not separate types
/// here: missing-token, EOF, and empty-range cases dominate on the
/// token side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TokenIndex(usize);

impl TokenIndex {
    /// Constructs a [`TokenIndex`] from a buffer offset in `0..=len`.
    ///
    /// [`Parser::feed_token`](crate::Parser::feed_token) returns the
    /// index of the token just appended. Callers also mint one from a
    /// slice position or from arithmetic on [`TokenIndex::get`].
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the offset as a `usize`.
    ///
    /// Use this to index [`SyntaxTree::tokens`](crate::SyntaxTree::tokens)
    /// (`tokens[index.get()]`), a parallel table kept beside the feed,
    /// or to compute a neighbouring index and wrap it with
    /// [`TokenIndex::new`].
    pub const fn get(self) -> usize {
        self.0
    }
}

/// A half-open token span `start..end` over the token buffer.
///
/// Empty ranges (`start == end`) are permitted and are used for missing
/// tokens, empty syntactic elements, and errors that anchor at the EOF
/// boundary. The range is expressed in logical token indices; it is not a
/// source byte range.
///
/// # Panics
///
/// [`TokenRange::new`] panics if `start > end`. Callers are responsible for
/// preserving the ordering; token indices in the buffer never shrink after a
/// push, so a reversed range only occurs on an implementation bug.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenRange {
    start: TokenIndex,
    end: TokenIndex,
}

impl TokenRange {
    /// Constructs a [`TokenRange`] from a `start..end` pair.
    ///
    /// Empty ranges are `start == end`. Callers compose spans that are
    /// not already a node's or diagnostic's range — for example the
    /// tokens between two child ranges — and slice
    /// [`SyntaxTree::tokens`](crate::SyntaxTree::tokens) with
    /// [`TokenRange::as_range`].
    ///
    /// Panics if `start > end`.
    pub fn new(start: TokenIndex, end: TokenIndex) -> Self {
        assert!(
            start.get() <= end.get(),
            "TokenRange::new: start ({}) must be <= end ({})",
            start.get(),
            end.get()
        );
        Self { start, end }
    }

    /// Returns an empty range anchored at `position`.
    // `pub(crate)`: missing-token / EOF shape. External empty spans use
    // `TokenRange::new(position, position)`.
    pub(crate) const fn empty_at(position: TokenIndex) -> Self {
        Self {
            start: position,
            end: position,
        }
    }

    /// Returns the start boundary of the range.
    pub const fn start(self) -> TokenIndex {
        self.start
    }

    /// Returns the end boundary of the range.
    pub const fn end(self) -> TokenIndex {
        self.end
    }

    /// Returns `true` when the range is empty (`start == end`).
    pub const fn is_empty(self) -> bool {
        self.start.0 == self.end.0
    }

    /// Returns the number of tokens covered by the range.
    pub const fn len(self) -> usize {
        self.end.0 - self.start.0
    }

    /// Returns the range as `Range<usize>`, suitable for slicing
    /// [`SyntaxTree::tokens`](crate::SyntaxTree::tokens).
    pub const fn as_range(self) -> Range<usize> {
        self.start.0..self.end.0
    }

    /// Returns `true` when this range fully contains `other`.
    pub const fn contains_range(self, other: TokenRange) -> bool {
        self.start.0 <= other.start.0 && other.end.0 <= self.end.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_range_from_token_index_only() {
        // A TokenRange is constructed from TokenIndex alone; it has no fields
        // that carry source-derived information.
        let range = TokenRange::new(TokenIndex::new(2), TokenIndex::new(5));
        assert_eq!(range.start(), TokenIndex::new(2));
        assert_eq!(range.end(), TokenIndex::new(5));
        assert_eq!(range.len(), 3);
        assert!(!range.is_empty());
    }

    #[test]
    fn empty_range() {
        let range = TokenRange::empty_at(TokenIndex::new(4));
        assert!(range.is_empty());
        assert_eq!(range.len(), 0);
        assert_eq!(range.start(), range.end());
    }

    #[test]
    fn contains_range() {
        let outer = TokenRange::new(TokenIndex::new(0), TokenIndex::new(10));
        let inner = TokenRange::new(TokenIndex::new(2), TokenIndex::new(5));
        assert!(outer.contains_range(inner));
        assert!(outer.contains_range(outer));
        assert!(!inner.contains_range(outer));
    }

    #[test]
    #[should_panic(expected = "start")]
    fn reversed_range_panics() {
        let _ = TokenRange::new(TokenIndex::new(5), TokenIndex::new(2));
    }
}
