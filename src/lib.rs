//! A Rust library for parsing Erlang source code, designed for language tooling.
//!
//! Callers provide the tokens; this crate does not read files, tokenize source,
//! preprocess it, or resolve names. Every parse produces a [`SyntaxTree`], with
//! syntax problems reported as [`Diagnostic`]s rather than `Result::Err`. The
//! parser recovers from syntax errors so later forms, terms, or elements can
//! still be parsed. The grammar tracks OTP 29's `erl_parse.yrl` (CI checks
//! against OTP-29.0.5).
//!
//! Tokenize with [erl_tokenize](https://docs.rs/erl_tokenize). For macros,
//! includes, and conditionals, preprocess first with
//! [erl_pp](https://docs.rs/erl_pp).
//! [`ParseMode`] selects the top-level construct; recovery and tree
//! walking are in [`docs::diagnostics`] and [`docs::navigation`].
//!
//! # Minimal parse
//!
//! This example tokenizes a minimal Erlang module and parses the
//! tokens in one call.
//!
//! ```
//! # fn main() -> Result<(), erl_tokenize::Error> {
//! let source = "-module(foo).";
//! let tokens = erl_tokenize::scan_tokens(source)?;
//! let tree = erl_parse::parse(&tokens, erl_parse::ParseMode::Module);
//! let roots: Vec<_> = tree.roots().collect();
//! assert!(tree.diagnostics().is_empty());
//! assert_eq!(roots.len(), 1);
//! assert_eq!(roots[0].kind(), erl_parse::SyntaxKind::Attribute);
//! # Ok(())
//! # }
//! ```
//!
//! Pass every token, including whitespace and comments, to [`parse`].
//! Strict success is [`SyntaxTree::diagnostics`] being empty.
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
mod token_range;

pub use crate::diagnostic::{Diagnostic, DiagnosticKind, Expected};
pub use crate::node::NodeView;
#[cfg(test)]
pub(crate) use crate::parser::Parser;
pub use crate::parser::{MAX_NESTING_DEPTH, ParseMode, parse};
pub use crate::syntax::{NodeId, SyntaxKind};
pub use crate::syntax_tree::SyntaxTree;
pub use crate::token_range::{TokenIndex, TokenRange};

pub mod docs;
