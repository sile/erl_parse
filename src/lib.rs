extern crate erl_pp;
extern crate erl_tokenize;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};
// pub use parser::Parser;
// pub use reader::TokenReader;
// pub use traits::{Parse, TokenRange};

// pub mod cst;

mod error;
// mod parser;
// mod reader;
// mod traits;

/// This crate specific `Result` type.
pub type Result<T> = ::std::result::Result<T, Error>;
