//! Sans I/O parser core and data model for an Erlang source parser.
//!
//! This crate exposes an incremental parser that a caller drives by pushing
//! [`erl_tokenize::Token`] values one at a time and pulling completed
//! top-level units back out. The parser holds the input token buffer, a
//! flat preorder syntax index, and the accumulated parse errors bundled
//! together as a [`SyntaxTree`] that survives past the parser instance.
//! Grammar coverage is added by subsequent crates and modules; the
//! machinery kept here is grammar-agnostic.
//!
//! Top-level types:
//!
//! - [`Parser`] and [`ParseMode`] are the entry point. Construct a parser
//!   for the desired [`ParseMode`] and drive it with [`Parser::push_token`]
//!   / [`Parser::next_top_node`] / [`Parser::state`] /
//!   [`Parser::syntax_tree`] / [`Parser::finish`].
//! - [`SyntaxTree`] bundles the token buffer, the syntax index, and the
//!   parse errors so callers can keep the parse result around after the
//!   parser goes away.
//! - [`ParseError`], [`ParseErrorKind`], and [`Expected`] describe a
//!   grammar error.
//! - [`TokenIndex`] and [`TokenRange`] describe positions and half-open
//!   spans over the token buffer.
//! - [`TokenBuffer`] is the append-only container that stores the pushed
//!   tokens.
//! - [`SyntaxIndex`] is a flat preorder array of [`SyntaxEntry`], addressed
//!   by [`NodeId`] (existing entries) and [`EntryIndex`] (entry-array
//!   boundaries, including the trailing sentinel). [`SyntaxKind`] tags each
//!   entry with a grammar-level nonterminal kind.
//! - [`NodeView`] and [`Cursor`] navigate the syntax index and token buffer
//!   together. Their iterator-returning methods hand back opaque
//!   `impl Iterator` values.
#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod cursor;
mod error;
mod event;
mod grammar;
mod node;
mod parser;
mod syntax;
mod syntax_tree;
mod token_buffer;
mod token_range;

pub use crate::error::{Expected, ParseError, ParseErrorKind, ProtocolError};
pub use crate::node::{Cursor, NodeView};
pub use crate::parser::{FormKind, InProgressState, ParseMode, Parser};
pub use crate::syntax::{EntryIndex, NodeId, SyntaxEntry, SyntaxIndex, SyntaxKind};
pub use crate::syntax_tree::SyntaxTree;
pub use crate::token_buffer::TokenBuffer;
pub use crate::token_range::{TokenIndex, TokenRange};
