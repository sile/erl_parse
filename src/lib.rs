//! Sans I/O parser core and data model for an Erlang source parser.
//!
//! This crate exposes an incremental parser that a caller drives by feeding
//! [`erl_tokenize::Token`] values one at a time and pulling completed
//! top-level units back out. The parser holds the input token buffer, a
//! flat preorder syntax index, and the accumulated diagnostics bundled
//! together as a [`SyntaxTree`] that survives past the parser instance.
//! A parse always produces that tree: syntax problems are recorded as
//! [`Diagnostic`]s and the grammar recovers to the next sync point
//! rather than returning `Result::Err`. The caller-facing contract is
//! in [`docs::diagnostics`]. Walking the finished tree is in
//! [`docs::navigation`].
//!
//! Top-level types:
//!
//! - [`Parser`] and [`ParseMode`] are the entry point. Construct a parser
//!   for the desired [`ParseMode`] and drive it with [`Parser::feed_token`]
//!   / [`Parser::next_node`] / [`Parser::syntax_tree`] /
//!   [`Parser::finish`].
//! - [`SyntaxTree`] bundles the token buffer, the syntax index, and the
//!   diagnostics so callers can keep the parse result around after the
//!   parser goes away.
//! - [`Diagnostic`], [`DiagnosticKind`], and [`Expected`] describe a
//!   syntax diagnostic. Every diagnostic currently produced is an error;
//!   warnings and notes are not emitted yet. See [`docs::diagnostics`]
//!   for how recovery continues after a diagnostic is recorded.
//! - [`TokenIndex`] and [`TokenRange`] describe positions and half-open
//!   spans over the token buffer.
//! - [`TokenBuffer`] is the append-only container that stores the tokens
//!   the caller fed.
//! - [`SyntaxIndex`] is a flat preorder array of [`SyntaxEntry`], addressed
//!   by [`NodeId`]. [`SyntaxKind`] tags each entry with a grammar-level
//!   nonterminal kind.
//! - [`NodeView`] walks one node together with the token buffer.
//!   Forest-level questions (`roots`, `innermost_containing`) and
//!   [`SyntaxTree::view`] live on [`SyntaxTree`] so the buffer and
//!   index always belong to the same tree. See [`docs::navigation`].
//!   Iterator-returning methods hand back opaque `impl Iterator`
//!   values.
#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod cursor;
mod diagnostic;
mod event;
mod grammar;
mod node;
mod parser;
mod syntax;
mod syntax_tree;
mod token_buffer;
mod token_range;

pub use crate::diagnostic::{Diagnostic, DiagnosticKind, Expected};
pub use crate::node::NodeView;
pub use crate::parser::{ParseMode, Parser};
pub use crate::syntax::{NodeId, SyntaxEntry, SyntaxIndex, SyntaxKind};
pub use crate::syntax_tree::SyntaxTree;
pub use crate::token_buffer::TokenBuffer;
pub use crate::token_range::{TokenIndex, TokenRange};

pub mod docs;
