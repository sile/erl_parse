//! Data model for an Erlang source parser.
//!
//! This crate provides the data-model types that will be owned by an
//! incremental Sans I/O parser core: the growing token buffer, a flat
//! preorder syntax index, and a lightweight navigation view over both. The
//! parser core that produces this model is added on top of this crate; only
//! the shared types live here.
//!
//! - [`TokenIndex`] and [`TokenRange`] describe positions and half-open
//!   spans over the logical token buffer.
//! - [`TokenBuffer`] is the append-only container that stores the pushed
//!   tokens.
//! - [`SyntaxIndex`] is a flat preorder array of [`SyntaxEntry`], addressed
//!   by [`NodeId`] (existing entries) and [`EntryIndex`] (entry-array
//!   boundaries, including the trailing sentinel). [`SyntaxKind`] tags each
//!   entry with a grammar-level nonterminal kind.
//! - [`NodeView`] and [`Cursor`] navigate the syntax index and token buffer
//!   together. Their iterator-returning methods hand back opaque
//!   `impl Iterator` values.

mod node;
mod syntax;
mod token_buffer;
mod token_range;

pub use crate::node::{Cursor, NodeView};
pub use crate::syntax::{EntryIndex, NodeId, SyntaxEntry, SyntaxIndex, SyntaxKind};
pub use crate::token_buffer::TokenBuffer;
pub use crate::token_range::{TokenIndex, TokenRange};
