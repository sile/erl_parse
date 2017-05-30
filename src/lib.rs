extern crate erl_tokenize;
extern crate num;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};
pub use parse::Parse;
pub use parser::Parser;
pub use token_reader::TokenReader;

// TODO: エラー情報追加用の処理を入れる
macro_rules! try_parse {
    ($reader:expr) => { track_try!($reader.parse_next()) }
}
macro_rules! parse {
    ($reader:expr) => { track!($reader.parse_next()) }
}

pub mod cst;

mod error;
mod parse;
mod parser;
mod token_reader;

pub type Result<T> = ::std::result::Result<T, Error>;

use std::ops::Range;
pub trait TokenRange {
    fn token_range(&self) -> Range<usize> {
        Range {
            start: self.token_start(),
            end: self.token_end(),
        }
    }
    fn token_start(&self) -> usize {
        self.token_range().start
    }
    fn token_end(&self) -> usize {
        self.token_range().end
    }
}
