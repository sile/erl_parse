//! Token buffer owned by the parser core.
//!
//! Accumulates [`erl_tokenize::Token`] values pushed by the caller into a
//! growing buffer. Removing, reordering, or mutating already-pushed tokens
//! is not exposed.

use erl_tokenize::Token;

use crate::token_range::{TokenIndex, TokenRange};

/// Append-only token buffer.
///
/// Backed by a `Vec<Token>` without pre-allocating capacity via
/// `Vec::with_capacity`; `Token` is `Copy`, so `push` is cheap.
///
/// The type does not implement `Clone`: it is designed to be owned by a
/// stateful parser core. The `push` mutator is `pub(crate)` and reachable
/// only from in-crate callers (the parser core and unit tests); external
/// callers reach it through the higher-level parser API.
#[derive(Debug, Default)]
pub struct TokenBuffer {
    tokens: Vec<Token>,
}

impl TokenBuffer {
    /// Creates an empty buffer.
    pub const fn new() -> Self {
        Self { tokens: Vec::new() }
    }

    /// Returns the current number of tokens in the buffer.
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    /// Returns `true` when the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    /// Returns the token at `index`, or `None` if the index is out of range.
    pub fn get(&self, index: TokenIndex) -> Option<Token> {
        self.tokens.get(index.get()).copied()
    }

    /// Borrows the buffer as a slice.
    pub fn as_slice(&self) -> &[Token] {
        &self.tokens
    }

    /// Returns the trailing boundary index (`len()`, the upper end of the
    /// `0..=len()` domain).
    pub fn end_index(&self) -> TokenIndex {
        TokenIndex::new(self.tokens.len())
    }

    /// Returns the range that covers the entire buffer.
    pub fn full_range(&self) -> TokenRange {
        TokenRange::new(TokenIndex::new(0), self.end_index())
    }

    /// Returns an iterator that yields `(TokenIndex, Token)` pairs inside
    /// `range`.
    pub fn iter_range(&self, range: TokenRange) -> impl Iterator<Item = (TokenIndex, Token)> {
        BufferRange {
            tokens: &self.tokens,
            cursor: range.start().get(),
            end: range.end().get(),
        }
    }

    /// Appends a token to the end of the buffer.
    ///
    /// Callable only from within this crate; the parser core drives this
    /// mutator, and unit tests exercise it directly.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Invoked by the parser core added later; currently only unit tests call it"
        )
    )]
    pub(crate) fn push(&mut self, token: Token) {
        self.tokens.push(token);
    }
}

struct BufferRange<'a> {
    tokens: &'a [Token],
    cursor: usize,
    end: usize,
}

impl Iterator for BufferRange<'_> {
    type Item = (TokenIndex, Token);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.end {
            return None;
        }
        let idx = self.cursor;
        let token = self.tokens[idx];
        self.cursor += 1;
        Some((TokenIndex::new(idx), token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use erl_tokenize::{Position, scan_token};

    fn scan_all(source: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut pos = Position::new();
        while let Some(token) = scan_token(source, pos).expect("valid Erlang source") {
            tokens.push(token);
            pos = token.end();
        }
        tokens
    }

    #[test]
    fn empty_buffer() {
        let buffer = TokenBuffer::new();
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert_eq!(buffer.end_index(), TokenIndex::new(0));
        assert!(buffer.full_range().is_empty());
    }

    #[test]
    fn push_preserves_order() {
        let source = "foo bar baz";
        let scanned = scan_all(source);
        let mut buffer = TokenBuffer::new();
        for token in &scanned {
            buffer.push(*token);
        }
        assert_eq!(buffer.len(), scanned.len());
        for (i, expected) in scanned.iter().enumerate() {
            let got = buffer.get(TokenIndex::new(i)).expect("in range");
            assert_eq!(got, *expected, "index {} mismatch", i);
        }
    }

    #[test]
    fn single_token_buffer() {
        let scanned = scan_all("foo");
        let mut buffer = TokenBuffer::new();
        for token in &scanned {
            buffer.push(*token);
        }
        assert!(!buffer.is_empty());
        assert_eq!(buffer.len(), 1);
        assert!(buffer.get(TokenIndex::new(0)).is_some());
        assert!(buffer.get(TokenIndex::new(1)).is_none());
    }

    #[test]
    fn hidden_tokens_stay_in_buffer_range() {
        // A range containing comments and whitespace preserves the original
        // token order.
        let source = "foo % comment\n bar";
        let scanned = scan_all(source);
        let mut buffer = TokenBuffer::new();
        for token in &scanned {
            buffer.push(*token);
        }

        let collected: Vec<Token> = buffer
            .iter_range(buffer.full_range())
            .map(|(_idx, tok)| tok)
            .collect();
        assert_eq!(collected, scanned);
    }
}
