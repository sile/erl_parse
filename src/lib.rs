//! Erlang source parser for language tools.
//!
//! The caller tokenizes ([`erl_tokenize::scan_token`]) and feeds every
//! token, including whitespace and comments. This crate does not read
//! files, tokenize, preprocess, or resolve names. It records syntax
//! problems as [`Diagnostic`]s and always returns a [`SyntaxTree`]
//! rather than `Result::Err`. The recovery contract is
//! [`docs::diagnostics`]; walking the tree is [`docs::navigation`].
//!
//! # Minimal loop
//!
//! ```
//! let source = "-module(foo).";
//! let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Module);
//! let mut pos = erl_tokenize::Position::new();
//! while let Some(token) = erl_tokenize::scan_token(source, pos).expect("valid source") {
//!     parser.feed_token(token);
//!     pos = token.end();
//! }
//! let mut roots = Vec::new();
//! while let Some(id) = parser.next_node() {
//!     roots.push(id);
//! }
//! let tree = parser.finish();
//! assert!(tree.diagnostics().is_empty());
//! assert_eq!(roots.len(), 1);
//! assert_eq!(
//!     tree.view(roots[0]).expect("root").kind(),
//!     erl_parse::SyntaxKind::Attribute,
//! );
//! ```
//!
//! Construct a [`Parser`] for a [`ParseMode`], feed tokens, pull completed
//! `.`-terminated units with [`Parser::next_node`], then
//! [`Parser::finish`]. Strict success is
//! [`SyntaxTree::diagnostics`] being empty.
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
pub use crate::syntax::{NodeId, SyntaxKind};
pub use crate::syntax_tree::SyntaxTree;
pub use crate::token_range::{TokenIndex, TokenRange};

pub mod docs;
