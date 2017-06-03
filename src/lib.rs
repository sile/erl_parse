extern crate erl_tokenize;
extern crate num;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};

mod error;

// pub use parse::Parse;
// pub use parser::Parser;
// pub use token_reader::TokenReader;

// // TODO: エラー情報追加用の処理を入れる
// macro_rules! try_parse {
//     ($reader:expr) => { track_try!($reader.parse_next()) }
// }
// macro_rules! parse {
//     ($reader:expr) => { track!($reader.parse_next()) }
// }
// macro_rules! derive_parse {
//     ($type:ident, $($field:ident),*) => {
//         impl<'token, 'text: 'token> ::Parse<'token, 'text> for $type<'token, 'text> {
//             fn parse(reader: &mut ::TokenReader<'token, 'text>) -> ::Result<Self> {
//                 Ok($type {
//                     $($field: try_parse!(reader)),*
//                 })
//             }
//         }
//     }
// }
// macro_rules! derive_parse_trace {
//     ($type:ident, $($field:ident),*) => {
//         impl<'token, 'text: 'token> ::Parse<'token, 'text> for $type<'token, 'text> {
//             fn parse(reader: &mut ::TokenReader<'token, 'text>) -> ::Result<Self> {
//                 Ok($type {
//                     $($field: {let t = try_parse!(reader);
//                        println!("# {} {}: {:?}", stringify!($type), stringify!($field), t);
//                                t}),*
//                 })
//             }
//         }
//     }
// }
// macro_rules! derive_token_range {
//     ($type:ident, $first:ident, $last:ident) => {
//         impl<'token, 'text: 'token> ::TokenRange for $type<'token, 'text> {
//             fn token_start(&self) -> usize {
//                 self.$first.token_start()
//             }
//             fn token_end(&self) -> usize {
//                 self.$last.token_end()
//             }
//         }
//     }
// }
// macro_rules! derive_parse0 {
//     ($type:ident, $($field:ident),*) => {
//         impl<'token, 'text: 'token> ::Parse<'token, 'text> for $type {
//             fn parse(reader: &mut ::TokenReader<'token, 'text>) -> ::Result<Self> {
//                 Ok($type {
//                     $($field: try_parse!(reader)),*
//                 })
//             }
//         }
//     }
// }
// macro_rules! derive_token_range0 {
//     ($type:ident, $first:ident, $last:ident) => {
//         impl<'token, 'text: 'token> ::TokenRange for $type {
//             fn token_start(&self) -> usize {
//                 self.$first.token_start()
//             }
//             fn token_end(&self) -> usize {
//                 self.$last.token_end()
//             }
//         }
//     }
// }
// // TODO: 共通化
// macro_rules! derive_parse2 {
//     ($type:ident, $($field:ident),*) => {
//         impl<'token, 'text: 'token, T> ::Parse<'token, 'text> for $type<T>
//             where T: ::Parse<'token, 'text> {
//             fn parse(reader: &mut ::TokenReader<'token, 'text>) -> ::Result<Self> {
//                 Ok($type {
//                     $($field: try_parse!(reader)),*
//                 })
//             }
//         }
//     }
// }
// macro_rules! derive_token_range2 {
//     ($type:ident, $first:ident, $last:ident) => {
//         impl<T> ::TokenRange for $type<T> where T: ::TokenRange {
//             fn token_start(&self) -> usize {
//                 self.$first.token_start()
//             }
//             fn token_end(&self) -> usize {
//                 self.$last.token_end()
//             }
//         }
//     }
// }
// macro_rules! derive_parse3 {
//     ($type:ident, $($field:ident),*) => {
//         impl<'token, 'text: 'token, T, U> ::Parse<'token, 'text> for $type<T, U>
//             where T: ::Parse<'token, 'text>, U: ::Parse<'token, 'text> {
//             fn parse(reader: &mut ::TokenReader<'token, 'text>) -> ::Result<Self> {
//                 Ok($type {
//                     $($field: try_parse!(reader)),*
//                 })
//             }
//         }
//     }
// }
// macro_rules! derive_token_range3 {
//     ($type:ident, $first:ident, $last:ident) => {
//         impl<T, U> ::TokenRange for $type<T, U> where T: ::TokenRange, U: ::TokenRange {
//             fn token_start(&self) -> usize {
//                 self.$first.token_start()
//             }
//             fn token_end(&self) -> usize {
//                 self.$last.token_end()
//             }
//         }
//     }
// }

// pub mod cst;

// mod parse;
// mod parser;
// mod token_reader;

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
