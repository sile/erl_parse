//! Index and range types over the token buffer.

use core::iter::FusedIterator;
use core::ops::Range;

/// A position in the token buffer.
///
/// A single [`TokenIndex`] can refer either to an existing token or to a
/// boundary (the trailing EOF position or the endpoint of an empty
/// [`TokenRange`]). Values lie in `0..=tokens.len()`.
///
/// Unlike [`NodeId`](crate::NodeId), this is the caller's address space:
/// [`SyntaxTree::tokens`](crate::SyntaxTree::tokens) is the sequence they
/// passed to [`parse`](crate::parse). Constructing from a slice
/// position or from arithmetic on [`TokenIndex::get`] is expected.
/// "Existing element" and "boundary" are not separate types here:
/// missing-token, EOF, and empty-range cases dominate on the token
/// side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TokenIndex(usize);

impl TokenIndex {
    /// Constructs a [`TokenIndex`] from a buffer offset in `0..=len`.
    ///
    /// [`parse`](crate::parse) assigns index `i` to `tokens[i]`.
    /// Callers also mint one from a slice position or from
    /// arithmetic on [`TokenIndex::get`].
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    /// Returns the offset as a `usize`.
    ///
    /// Use this to index [`SyntaxTree::tokens`](crate::SyntaxTree::tokens)
    /// (`tokens[index.get()]`), a parallel table kept beside the
    /// input slice, or to compute a neighbouring index and wrap it
    /// with [`TokenIndex::new`].
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
/// A [`TokenRange`] is itself an iterator over the [`TokenIndex`]es in
/// the span (`start` through `end - 1`), mirroring
/// `std::ops::Range`. Iteration consumes a copy, so `for i in range`
/// leaves the caller's `range` untouched.
///
/// # Panics
///
/// [`TokenRange::new`] panics if `start > end`. Callers are responsible for
/// preserving the ordering; token indices in the slice never shrink, so a
/// reversed range only occurs on an implementation bug.
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
    /// [`TokenRange::as_slice_index`].
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
    ///
    /// While this value is being iterated in place (via
    /// `Iterator::next`), this is the remaining cursor; `Copy` means
    /// `for` loops iterate a copy and never change the caller's value.
    pub const fn start(self) -> TokenIndex {
        self.start
    }

    /// Returns the end boundary of the range.
    ///
    /// While this value is being iterated in place (via
    /// `DoubleEndedIterator::next_back`), this is the remaining cursor;
    /// `Copy` means `for` loops iterate a copy and never change the
    /// caller's value.
    pub const fn end(self) -> TokenIndex {
        self.end
    }

    /// Returns `true` when the range is empty (`start == end`).
    pub const fn is_empty(self) -> bool {
        self.start.0 == self.end.0
    }

    /// Returns the number of tokens covered by the range.
    ///
    /// While this value is being iterated in place, this is the number
    /// of items remaining; `Copy` means `for` loops iterate a copy and
    /// never change the caller's value.
    pub const fn len(self) -> usize {
        self.end.0 - self.start.0
    }

    /// Returns this span as a `Range<usize>`, suitable for indexing a
    /// slice of [`SyntaxTree::tokens`](crate::SyntaxTree::tokens).
    ///
    /// The [`TokenIndex`]es are dereferenced to `usize` positions.
    /// Iterate the [`TokenIndex`]es directly with `for i in range`
    /// instead; this conversion is only for indexing a slice.
    pub const fn as_slice_index(self) -> Range<usize> {
        self.start.0..self.end.0
    }
}

/// Iterating a [`TokenRange`] walks the [`TokenIndex`]es in the span,
/// `start` through `end - 1`, mirroring `std::ops::Range`. Because
/// `TokenRange` is `Copy`, a `for` loop iterates a copy and leaves the
/// caller's value unchanged; calling `next` on a `mut` value advances
/// its boundaries in place, exactly like `std::ops::Range`.
impl Iterator for TokenRange {
    type Item = TokenIndex;

    fn next(&mut self) -> Option<TokenIndex> {
        if self.start < self.end {
            let index = self.start;
            self.start = TokenIndex::new(self.start.get() + 1);
            Some(index)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        // Call the inherent `len` explicitly: `self.len()` would resolve
        // to `ExactSizeIterator::len`, whose default consults `size_hint`
        // and would recurse. `std::ops::Range` has no inherent `len`, so
        // it never hits this collision.
        let len = TokenRange::len(*self);
        (len, Some(len))
    }
}

impl DoubleEndedIterator for TokenRange {
    fn next_back(&mut self) -> Option<TokenIndex> {
        if self.start < self.end {
            self.end = TokenIndex::new(self.end.get() - 1);
            Some(self.end)
        } else {
            None
        }
    }
}

impl ExactSizeIterator for TokenRange {
    fn len(&self) -> usize {
        // See the comment in `Iterator::size_hint`; same collision.
        TokenRange::len(*self)
    }
}

impl FusedIterator for TokenRange {}

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
    fn iteration_yields_span_indices() {
        let range = TokenRange::new(TokenIndex::new(2), TokenIndex::new(5));
        let indices: Vec<TokenIndex> = range.collect();
        assert_eq!(
            indices,
            vec![TokenIndex::new(2), TokenIndex::new(3), TokenIndex::new(4),]
        );
        assert_eq!(indices.len(), range.len());
    }

    #[test]
    fn iteration_consumes_a_copy() {
        let range = TokenRange::new(TokenIndex::new(1), TokenIndex::new(4));
        for i in range {
            assert!(range.start() <= i && i < range.end());
        }
        assert_eq!(range.start(), TokenIndex::new(1));
        assert_eq!(range.end(), TokenIndex::new(4));
    }

    #[test]
    fn empty_range_iteration_yields_nothing() {
        let range = TokenRange::empty_at(TokenIndex::new(4));
        assert_eq!(range.len(), 0);
        let mut fwd = range;
        let mut back = range;
        assert_eq!(fwd.next(), None);
        assert_eq!(back.next_back(), None);
    }

    #[test]
    fn iteration_reports_exact_size() {
        let range = TokenRange::new(TokenIndex::new(1), TokenIndex::new(6));
        assert_eq!(range.len(), 5);
        assert_eq!(range.size_hint(), (5, Some(5)));
        assert_eq!(ExactSizeIterator::len(&range), 5);
    }

    #[test]
    fn double_ended_iteration_walks_from_both_ends() {
        let range = TokenRange::new(TokenIndex::new(1), TokenIndex::new(4));
        let mut fwd = range;
        let mut back = range;
        assert_eq!(fwd.next(), Some(TokenIndex::new(1)));
        assert_eq!(back.next_back(), Some(TokenIndex::new(3)));
        assert_eq!(fwd.next(), Some(TokenIndex::new(2)));
        assert_eq!(back.next_back(), Some(TokenIndex::new(2)));
        assert_eq!(fwd.next(), Some(TokenIndex::new(3)));
        assert_eq!(back.next_back(), Some(TokenIndex::new(1)));
        assert_eq!(fwd.next(), None);
        assert_eq!(back.next_back(), None);
    }

    #[test]
    fn iteration_stays_fused_after_exhaustion() {
        let mut range = TokenRange::new(TokenIndex::new(0), TokenIndex::new(1));
        assert_eq!(range.next(), Some(TokenIndex::new(0)));
        assert_eq!(range.next(), None);
        assert_eq!(range.next(), None);
    }

    #[test]
    #[should_panic(expected = "start")]
    fn reversed_range_panics() {
        let _ = TokenRange::new(TokenIndex::new(5), TokenIndex::new(2));
    }
}
