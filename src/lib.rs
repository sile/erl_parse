//! Erlang source parser for language tools.
//!
//! You feed tokens; this crate does not read files, tokenize, preprocess,
//! or resolve names. Whitespace and comments stay in the stream. A parse
//! always produces a [`SyntaxTree`]; syntax problems are [`Diagnostic`]s
//! rather than `Result::Err`. The grammar tracks OTP 29's `erl_parse.yrl`
//! (CI checks against OTP-29.0.5).
//!
//! Tokenize with [erl_tokenize](https://docs.rs/erl_tokenize). For macros,
//! includes, and conditionals, preprocess first with
//! [erl_pp](https://docs.rs/erl_pp).
//! Recovery is [`docs::diagnostics`]; walking the tree is
//! [`docs::navigation`].
//!
//! # Minimal loop
//!
//! ```
//! # fn main() -> Result<(), erl_tokenize::Error> {
//! let source = "-module(foo).";
//! let mut parser = erl_parse::Parser::new(erl_parse::ParseMode::Module);
//! for token in erl_tokenize::scan_tokens(source)? {
//!     parser.feed_token(token);
//! }
//! let mut roots = Vec::new();
//! while let Some(id) = parser.next_node() {
//!     roots.push(id);
//! }
//! let tree = parser.finish();
//! assert!(tree.diagnostics().is_empty());
//! assert_eq!(roots.len(), 1);
//! assert_eq!(
//!     tree.view(roots[0]).map(|v| v.kind()),
//!     Some(erl_parse::SyntaxKind::Attribute),
//! );
//! # Ok(())
//! # }
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
