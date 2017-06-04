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

#[derive(Debug)]
pub struct Atom<'token, 'text: 'token> {
    position: usize,
    value: &'token AtomToken<'text>,
}
derive_traits_for_token!(Atom, Atom, AtomToken);

#[derive(Debug)]
pub struct Char<'token, 'text: 'token> {
    position: usize,
    value: &'token CharToken<'text>,
}
derive_traits_for_token!(Char, Char, CharToken);

#[derive(Debug)]
pub struct Float<'token, 'text: 'token> {
    position: usize,
    value: &'token FloatToken<'text>,
}
derive_traits_for_token!(Float, Float, FloatToken);

#[derive(Debug)]
pub struct Int<'token, 'text: 'token> {
    position: usize,
    value: &'token IntegerToken<'text>,
}
derive_traits_for_token!(Int, Integer, IntegerToken);

#[derive(Debug)]
pub struct Str<'token, 'text: 'token> {
    position: usize,
    value: &'token StringToken<'text>,
}
derive_traits_for_token!(Str, String, StringToken);

#[derive(Debug)]
pub struct K_BEGIN {
    position: usize,
}
derive_traits_for_value!(K_BEGIN, Keyword, Keyword::Begin);

#[derive(Debug)]
pub struct K_END {
    position: usize,
}
derive_traits_for_value!(K_END, Keyword, Keyword::End);

#[derive(Debug)]
pub struct K_CATCH {
    position: usize,
}
derive_traits_for_value!(K_CATCH, Keyword, Keyword::Catch);

#[derive(Debug)]
pub struct K_NOT {
    position: usize,
}
derive_traits_for_value!(K_NOT, Keyword, Keyword::Not);

#[derive(Debug)]
pub struct K_BNOT {
    position: usize,
}
derive_traits_for_value!(K_BNOT, Keyword, Keyword::Bnot);

#[derive(Debug)]
pub struct K_FUN {
    position: usize,
}
derive_traits_for_value!(K_FUN, Keyword, Keyword::Fun);

#[derive(Debug)]
pub struct S_COMMA {
    position: usize,
}
derive_traits_for_value!(S_COMMA, Symbol, Symbol::Comma);

#[derive(Debug)]
pub struct S_COLON {
    position: usize,
}
derive_traits_for_value!(S_COLON, Symbol, Symbol::Colon);

#[derive(Debug)]
pub struct S_PLUS {
    position: usize,
}
derive_traits_for_value!(S_PLUS, Symbol, Symbol::Plus);

#[derive(Debug)]
pub struct S_HYPHEN {
    position: usize,
}
derive_traits_for_value!(S_HYPHEN, Symbol, Symbol::Hyphen);

#[derive(Debug)]
pub struct S_OPEN_PAREN {
    position: usize,
}
derive_traits_for_value!(S_OPEN_PAREN, Symbol, Symbol::OpenParen);

#[derive(Debug)]
pub struct S_CLOSE_PAREN {
    position: usize,
}
derive_traits_for_value!(S_CLOSE_PAREN, Symbol, Symbol::CloseParen);

#[derive(Debug)]
pub struct S_OPEN_BRACE {
    position: usize,
}
derive_traits_for_value!(S_OPEN_BRACE, Symbol, Symbol::OpenBrace);

#[derive(Debug)]
pub struct S_CLOSE_BRACE {
    position: usize,
}
derive_traits_for_value!(S_CLOSE_BRACE, Symbol, Symbol::CloseBrace);

#[derive(Debug)]
pub struct S_OPEN_SQUARE {
    position: usize,
}
derive_traits_for_value!(S_OPEN_SQUARE, Symbol, Symbol::OpenSquare);

#[derive(Debug)]
pub struct S_CLOSE_SQUARE {
    position: usize,
}
derive_traits_for_value!(S_CLOSE_SQUARE, Symbol, Symbol::CloseSquare);

#[derive(Debug)]
pub struct S_VERTICAL_BAR {
    position: usize,
}
derive_traits_for_value!(S_VERTICAL_BAR, Symbol, Symbol::VerticalBar);

#[derive(Debug)]
pub struct S_SLASH {
    position: usize,
}
derive_traits_for_value!(S_SLASH, Symbol, Symbol::Slash);

#[derive(Debug)]
pub struct S_SHARP {
    position: usize,
}
derive_traits_for_value!(S_SHARP, Symbol, Symbol::Sharp);

#[derive(Debug)]
pub struct S_DOUBLE_RIGHT_ALLOW {
    position: usize,
}
derive_traits_for_value!(S_DOUBLE_RIGHT_ALLOW, Symbol, Symbol::DoubleRightAllow);

#[derive(Debug)]
pub struct S_MAP_MATCH {
    position: usize,
}
derive_traits_for_value!(S_MAP_MATCH, Symbol, Symbol::MapMatch);

#[derive(Debug)]
pub struct S_MATCH {
    position: usize,
}
derive_traits_for_value!(S_MATCH, Symbol, Symbol::Match);

#[derive(Debug)]
pub struct S_QUESTION {
    position: usize,
}
derive_traits_for_value!(S_QUESTION, Symbol, Symbol::Question);

#[derive(Debug)]
pub struct S_DOT {
    position: usize,
}
derive_traits_for_value!(S_DOT, Symbol, Symbol::Dot);
