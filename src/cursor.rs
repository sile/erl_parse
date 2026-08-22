//! Token cursor used by grammar functions.
//!
//! The cursor advances over the token buffer while treating hidden tokens
//! as transparent to lookahead but folded into consumed node ranges when a
//! lexical token is consumed.

use crate::token_buffer::TokenBuffer;
use crate::token_range::TokenIndex;

/// A cursor over a `TokenBuffer` that reads full tokens but exposes
/// hidden-token-aware lookahead.
pub(crate) struct TokenCursor<'a> {
    tokens: &'a TokenBuffer,
    at: usize,
}

impl<'a> TokenCursor<'a> {
    /// Creates a cursor positioned at `at` (a full-token index).
    pub(crate) fn new(tokens: &'a TokenBuffer, at: usize) -> Self {
        Self { tokens, at }
    }

    /// Returns the nth lexical token from the current position, skipping
    /// hidden tokens. Returns `None` when the nth lexical token has not
    /// been reached before buffer end.
    pub(crate) fn peek_lexical(&self, offset: usize) -> Option<(TokenIndex, erl_tokenize::Token)> {
        let mut cursor = self.at;
        let mut seen = 0usize;
        loop {
            let token = self.tokens.get(TokenIndex::new(cursor))?;
            if token.kind().is_lexical() {
                if seen == offset {
                    return Some((TokenIndex::new(cursor), token));
                }
                seen += 1;
            }
            cursor += 1;
        }
    }

    /// Advances past exactly one lexical token and then over any
    /// immediately-following hidden tokens.
    ///
    /// Returns the position of the boundary after the advance (the value
    /// suitable as `end` for a node's [`TokenRange`][crate::TokenRange]).
    /// Returns `None` when no lexical token is reachable before buffer end;
    /// in that case the cursor position is unchanged.
    pub(crate) fn advance_lexical(&mut self) -> Option<TokenIndex> {
        let mut cursor = self.at;
        loop {
            let token = self.tokens.get(TokenIndex::new(cursor))?;
            if token.kind().is_lexical() {
                cursor += 1;
                break;
            }
            cursor += 1;
        }
        while let Some(token) = self.tokens.get(TokenIndex::new(cursor)) {
            if token.kind().is_hidden() {
                cursor += 1;
            } else {
                break;
            }
        }
        self.at = cursor;
        Some(TokenIndex::new(cursor))
    }

    /// Captures the cursor position for later restoration.
    pub(crate) fn save(&self) -> CursorCheckpoint {
        CursorCheckpoint(self.at)
    }
}

/// Opaque handle capturing a `TokenCursor` position.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CursorCheckpoint(usize);

impl CursorCheckpoint {
    /// Returns the underlying full-token index for parser internals that
    /// bundle cursor + event + error state together into their own
    /// checkpoint type.
    pub(crate) fn at(self) -> usize {
        self.0
    }
}
