#![allow(non_camel_case_types)]
use std::ops::Deref;
use erl_tokenize::Token;
use erl_tokenize::tokens::{AtomToken, CharToken, IntegerToken, FloatToken, StringToken};
use erl_tokenize::values::{Keyword, Symbol};

use {Result, Parse, TokenRange, TokenReader, ErrorKind};

macro_rules! derive_traits_for_value {
    ($name:ident, $variant:ident, $value:expr) => {
        impl<'token, 'text: 'token> Parse<'token, 'text> for $name {
            fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
                let position = reader.position();
                let token = track_try!(reader.read());
                if let Token::$variant(ref token) = *token {
                    track_assert_eq!(token.value(), $value, ErrorKind::Other);
                    Ok($name { position })
                 } else {
                    track_panic!(ErrorKind::Other,
                                 "An `{}` is expected: actual={:?}",
                                 stringify!($token),
                                 token);
                }
            }
        }
        impl<'token, 'text: 'token> TokenRange for $name {
            fn token_start(&self) -> usize {
                self.position
            }
            fn token_end(&self) -> usize {
                self.position + 1
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Atom<'token, 'text: 'token> {
    position: usize,
    value: &'token AtomToken<'text>,
}
derive_traits_for_token!(Atom, Atom, AtomToken);

#[derive(Debug, Clone)]
pub struct Char<'token, 'text: 'token> {
    position: usize,
    value: &'token CharToken<'text>,
}
derive_traits_for_token!(Char, Char, CharToken);

#[derive(Debug, Clone)]
pub struct Float<'token, 'text: 'token> {
    position: usize,
    value: &'token FloatToken<'text>,
}
derive_traits_for_token!(Float, Float, FloatToken);

#[derive(Debug, Clone)]
pub struct Int<'token, 'text: 'token> {
    position: usize,
    value: &'token IntegerToken<'text>,
}
derive_traits_for_token!(Int, Integer, IntegerToken);

#[derive(Debug, Clone)]
pub struct Str<'token, 'text: 'token> {
    position: usize,
    value: &'token StringToken<'text>,
}
derive_traits_for_token!(Str, String, StringToken);

#[derive(Debug, Clone)]
pub struct K_BEGIN {
    position: usize,
}
derive_traits_for_value!(K_BEGIN, Keyword, Keyword::Begin);

#[derive(Debug, Clone)]
pub struct K_END {
    position: usize,
}
derive_traits_for_value!(K_END, Keyword, Keyword::End);

#[derive(Debug, Clone)]
pub struct K_CATCH {
    position: usize,
}
derive_traits_for_value!(K_CATCH, Keyword, Keyword::Catch);

#[derive(Debug, Clone)]
pub struct K_NOT {
    position: usize,
}
derive_traits_for_value!(K_NOT, Keyword, Keyword::Not);

#[derive(Debug, Clone)]
pub struct K_BNOT {
    position: usize,
}
derive_traits_for_value!(K_BNOT, Keyword, Keyword::Bnot);

#[derive(Debug, Clone)]
pub struct K_FUN {
    position: usize,
}
derive_traits_for_value!(K_FUN, Keyword, Keyword::Fun);

#[derive(Debug, Clone)]
pub struct S_COMMA {
    position: usize,
}
derive_traits_for_value!(S_COMMA, Symbol, Symbol::Comma);

#[derive(Debug, Clone)]
pub struct S_COLON {
    position: usize,
}
derive_traits_for_value!(S_COLON, Symbol, Symbol::Colon);

#[derive(Debug, Clone)]
pub struct S_PLUS {
    position: usize,
}
derive_traits_for_value!(S_PLUS, Symbol, Symbol::Plus);

#[derive(Debug, Clone)]
pub struct S_HYPHEN {
    position: usize,
}
derive_traits_for_value!(S_HYPHEN, Symbol, Symbol::Hyphen);

#[derive(Debug, Clone)]
pub struct S_OPEN_PAREN {
    position: usize,
}
derive_traits_for_value!(S_OPEN_PAREN, Symbol, Symbol::OpenParen);

#[derive(Debug, Clone)]
pub struct S_CLOSE_PAREN {
    position: usize,
}
derive_traits_for_value!(S_CLOSE_PAREN, Symbol, Symbol::CloseParen);

#[derive(Debug, Clone)]
pub struct S_OPEN_BRACE {
    position: usize,
}
derive_traits_for_value!(S_OPEN_BRACE, Symbol, Symbol::OpenBrace);

#[derive(Debug, Clone)]
pub struct S_CLOSE_BRACE {
    position: usize,
}
derive_traits_for_value!(S_CLOSE_BRACE, Symbol, Symbol::CloseBrace);

#[derive(Debug, Clone)]
pub struct S_OPEN_SQUARE {
    position: usize,
}
derive_traits_for_value!(S_OPEN_SQUARE, Symbol, Symbol::OpenSquare);

#[derive(Debug, Clone)]
pub struct S_CLOSE_SQUARE {
    position: usize,
}
derive_traits_for_value!(S_CLOSE_SQUARE, Symbol, Symbol::CloseSquare);

#[derive(Debug, Clone)]
pub struct S_VERTICAL_BAR {
    position: usize,
}
derive_traits_for_value!(S_VERTICAL_BAR, Symbol, Symbol::VerticalBar);

#[derive(Debug, Clone)]
pub struct S_SLASH {
    position: usize,
}
derive_traits_for_value!(S_SLASH, Symbol, Symbol::Slash);

#[derive(Debug, Clone)]
pub struct S_SHARP {
    position: usize,
}
derive_traits_for_value!(S_SHARP, Symbol, Symbol::Sharp);

#[derive(Debug, Clone)]
pub struct S_DOUBLE_RIGHT_ALLOW {
    position: usize,
}
derive_traits_for_value!(S_DOUBLE_RIGHT_ALLOW, Symbol, Symbol::DoubleRightAllow);

#[derive(Debug, Clone)]
pub struct S_DOUBLE_RIGHT_ANGLE {
    position: usize,
}
derive_traits_for_value!(S_DOUBLE_RIGHT_ANGLE, Symbol, Symbol::DoubleRightAngle);

#[derive(Debug, Clone)]
pub struct S_DOUBLE_LEFT_ANGLE {
    position: usize,
}
derive_traits_for_value!(S_DOUBLE_LEFT_ANGLE, Symbol, Symbol::DoubleLeftAngle);

#[derive(Debug, Clone)]
pub struct S_MAP_MATCH {
    position: usize,
}
derive_traits_for_value!(S_MAP_MATCH, Symbol, Symbol::MapMatch);

#[derive(Debug, Clone)]
pub struct S_MATCH {
    position: usize,
}
derive_traits_for_value!(S_MATCH, Symbol, Symbol::Match);

#[derive(Debug, Clone)]
pub struct S_QUESTION {
    position: usize,
}
derive_traits_for_value!(S_QUESTION, Symbol, Symbol::Question);

#[derive(Debug, Clone)]
pub struct S_DOT {
    position: usize,
}
derive_traits_for_value!(S_DOT, Symbol, Symbol::Dot);

#[derive(Debug, Clone)]
pub struct A_INTEGER {
    position: usize,
}
derive_traits_for_value!(A_INTEGER, Atom, "integer");

#[derive(Debug, Clone)]
pub struct A_FLOAT {
    position: usize,
}
derive_traits_for_value!(A_FLOAT, Atom, "float");

#[derive(Debug, Clone)]
pub struct A_BINARY {
    position: usize,
}
derive_traits_for_value!(A_BINARY, Atom, "binary");

#[derive(Debug, Clone)]
pub struct A_BYTES {
    position: usize,
}
derive_traits_for_value!(A_BYTES, Atom, "bytes");

#[derive(Debug, Clone)]
pub struct A_BITSTRING {
    position: usize,
}
derive_traits_for_value!(A_BITSTRING, Atom, "bitstring");

#[derive(Debug, Clone)]
pub struct A_BITS {
    position: usize,
}
derive_traits_for_value!(A_BITS, Atom, "bits");

#[derive(Debug, Clone)]
pub struct A_UTF8 {
    position: usize,
}
derive_traits_for_value!(A_UTF8, Atom, "utf8");

#[derive(Debug, Clone)]
pub struct A_UTF16 {
    position: usize,
}
derive_traits_for_value!(A_UTF16, Atom, "utf16");

#[derive(Debug, Clone)]
pub struct A_UTF32 {
    position: usize,
}
derive_traits_for_value!(A_UTF32, Atom, "utf32");

#[derive(Debug, Clone)]
pub struct A_SIGNED {
    position: usize,
}
derive_traits_for_value!(A_SIGNED, Atom, "signed");

#[derive(Debug, Clone)]
pub struct A_UNSIGNED {
    position: usize,
}
derive_traits_for_value!(A_UNSIGNED, Atom, "unsigned");

#[derive(Debug, Clone)]
pub struct A_BIG {
    position: usize,
}
derive_traits_for_value!(A_BIG, Atom, "big");

#[derive(Debug, Clone)]
pub struct A_LITTLE {
    position: usize,
}
derive_traits_for_value!(A_LITTLE, Atom, "little");

#[derive(Debug, Clone)]
pub struct A_NATIVE {
    position: usize,
}
derive_traits_for_value!(A_NATIVE, Atom, "native");

#[derive(Debug, Clone)]
pub struct A_UNIT {
    position: usize,
}
derive_traits_for_value!(A_UNIT, Atom, "unit");
