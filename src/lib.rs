extern crate erl_tokenize;
extern crate num;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};
pub use parse::Parse;
pub use parser::Parser;
pub use token_reader::TokenReader2;

pub mod cst;
pub mod parse_tree;

mod error;
mod parse;
mod parser;
mod token_reader;

pub type Result<T> = ::std::result::Result<T, Error>;
