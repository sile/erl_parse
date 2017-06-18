extern crate erl_pp;
extern crate erl_tokenize;
extern crate num;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};
pub use parser::Parser;
pub use token_reader::TokenReader;

pub mod cst;
pub mod traits;

mod error;
mod parser;
mod token_reader;

/// This crate specific `Result` type.
pub type Result<T> = ::std::result::Result<T, Error>;
