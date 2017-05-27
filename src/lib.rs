extern crate erl_tokenize;
extern crate num;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};
pub use parser::Parser;
pub use parse_tree::ParseTree;

mod error;
mod parser;
mod parse_tree;

pub type Result<T> = ::std::result::Result<T, Error>;
