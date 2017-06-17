extern crate erl_pp;
extern crate erl_tokenize;
extern crate num;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};
pub use parse::{Parse, ParseLeftRecur, Expect};
pub use parser::Parser;
pub use token_reader::{TokenReader, Tokens, Preprocessor};
// pub use parser::Parser;
// pub use reader::TokenReader;
// pub use traits::{Parse, TokenRange};

macro_rules! track_try_some {
    ($expr:expr) => {
        if let Some(value) = track!($expr)? {
            value
        } else {
            return Ok(None)
        }
    }
}

pub mod cst;

mod error;
mod parse;
mod parser;
mod token_reader;
// mod parser;
// mod reader;
// mod traits;

/// This crate specific `Result` type.
pub type Result<T> = ::std::result::Result<T, Error>;

pub trait TryInto<T>: Sized {
    fn try_into(self) -> Result<T>;
}
