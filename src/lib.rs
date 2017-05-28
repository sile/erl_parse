extern crate erl_tokenize;
extern crate num;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};
pub use parser::Parser;

pub mod cst;
pub mod parse_tree;

mod error;
mod parser;
mod token_reader;

pub type Result<T> = ::std::result::Result<T, Error>;
