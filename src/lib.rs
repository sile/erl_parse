//! Erlang source code parser.
//!
//! # Examples
//!
//! ```
//! extern crate erl_parse;
//! extern crate erl_pp;
//! extern crate erl_tokenize;
//!
//! use erl_parse::{Parser, TokenReader};
//! use erl_parse::cst::Expr;
//! use erl_pp::Preprocessor;
//! use erl_tokenize::Lexer;
//!
//! # fn main() {
//! let text = r#"io:format("Hello World")"#;
//! let mut parser = Parser::new(TokenReader::new(Preprocessor::new(Lexer::new(text))));
//! parser.parse::<Expr>().unwrap();
//! # }
//! ```
extern crate erl_pp;
extern crate erl_tokenize;
extern crate num;
#[macro_use]
extern crate trackable;

pub use crate::error::{Error, ErrorKind};
pub use crate::parser::Parser;
pub use crate::token_reader::TokenReader;

pub mod builtin;
pub mod cst;
pub mod traits;

mod error;
mod parser;
mod token_reader;

/// This crate specific `Result` type.
pub type Result<T> = ::std::result::Result<T, Error>;
