use std::iter;
use num::BigUint;
use erl_tokenize::LexicalToken;
use erl_tokenize::tokens::{AtomToken, CharToken, FloatToken, IntegerToken, KeywordToken,
                           StringToken, SymbolToken, VariableToken};
use erl_tokenize::values::{Symbol, Keyword};

use {Result, ErrorKind, Preprocessor, IntoTokens, Parser};

pub trait Parse: Sized {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        if let Some(value) = track!(parser.try_parse())? {
            Ok(value)
        } else {
            track_panic!(ErrorKind::InvalidInput);
        }
    }

    fn try_parse<T>(parser: &mut Parser<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor;

    fn try_parse_expect<T>(parser: &mut Parser<T>, expected: &Self::Value) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
        Self: Expect,
    {
        let actual = track_try_some!(Self::try_parse(parser));
        if actual.expect(expected).is_ok() {
            Ok(Some(actual))
        } else {
            parser.unread_token(actual.into());
            Ok(None)
        }
    }

    fn parse_expect<T>(parser: &mut Parser<T>, expected: &Self::Value) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
        Self: Expect,
    {
        let actual = track!(Self::parse(parser))?;
        track!(actual.expect(expected))?;
        Ok(actual)
    }
}
impl<U: Parse> Parse for Box<U> {
    fn try_parse<T>(parser: &mut Parser<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Parse::try_parse(parser)?.map(Box::new))
    }
}
impl Parse for AtomToken {
    fn try_parse<T>(parser: &mut Parser<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(parser.try_read_token())?.and_then(|token| {
            token
                .into_atom_token()
                .map_err(|token| parser.unread_token(token))
                .ok()
        }))
    }
}
impl Parse for CharToken {
    fn try_parse<T>(parser: &mut Parser<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(parser.try_read_token())?.and_then(|token| {
            token
                .into_char_token()
                .map_err(|token| parser.unread_token(token))
                .ok()
        }))
    }
}
impl Parse for FloatToken {
    fn try_parse<T>(parser: &mut Parser<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(parser.try_read_token())?.and_then(|token| {
            token
                .into_float_token()
                .map_err(|token| parser.unread_token(token))
                .ok()
        }))
    }
}
impl Parse for IntegerToken {
    fn try_parse<T>(parser: &mut Parser<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(parser.try_read_token())?.and_then(|token| {
            token
                .into_integer_token()
                .map_err(|token| parser.unread_token(token))
                .ok()
        }))
    }
}
impl Parse for KeywordToken {
    fn try_parse<T>(parser: &mut Parser<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(parser.try_read_token())?.and_then(|token| {
            token
                .into_keyword_token()
                .map_err(|token| parser.unread_token(token))
                .ok()
        }))
    }
}
impl Parse for StringToken {
    fn try_parse<T>(parser: &mut Parser<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(parser.try_read_token())?.and_then(|token| {
            token
                .into_string_token()
                .map_err(|token| parser.unread_token(token))
                .ok()
        }))
    }
}
impl Parse for SymbolToken {
    fn try_parse<T>(parser: &mut Parser<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(parser.try_read_token())?.and_then(|token| {
            token
                .into_symbol_token()
                .map_err(|token| parser.unread_token(token))
                .ok()
        }))
    }
}
impl Parse for VariableToken {
    fn try_parse<T>(parser: &mut Parser<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(parser.try_read_token())?.and_then(|token| {
            token
                .into_variable_token()
                .map_err(|token| parser.unread_token(token))
                .ok()
        }))
    }
}

pub trait Expect: Sized + Into<LexicalToken> {
    type Value: ?Sized;
    fn expect(&self, expected: &Self::Value) -> Result<()>;
}
impl Expect for AtomToken {
    type Value = str;
    fn expect(&self, expected: &Self::Value) -> Result<()> {
        track_assert_eq!(self.value(), expected, ErrorKind::InvalidInput);
        Ok(())
    }
}
impl Expect for CharToken {
    type Value = char;
    fn expect(&self, expected: &Self::Value) -> Result<()> {
        track_assert_eq!(self.value(), *expected, ErrorKind::InvalidInput);
        Ok(())
    }
}
impl Expect for FloatToken {
    type Value = f64;
    fn expect(&self, expected: &Self::Value) -> Result<()> {
        track_assert_eq!(self.value(), *expected, ErrorKind::InvalidInput);
        Ok(())
    }
}
impl Expect for IntegerToken {
    type Value = BigUint;
    fn expect(&self, expected: &Self::Value) -> Result<()> {
        track_assert_eq!(self.value(), expected, ErrorKind::InvalidInput);
        Ok(())
    }
}
impl Expect for KeywordToken {
    type Value = Keyword;
    fn expect(&self, expected: &Self::Value) -> Result<()> {
        track_assert_eq!(self.value(), *expected, ErrorKind::InvalidInput);
        Ok(())
    }
}
impl Expect for StringToken {
    type Value = str;
    fn expect(&self, expected: &Self::Value) -> Result<()> {
        track_assert_eq!(self.value(), expected, ErrorKind::InvalidInput);
        Ok(())
    }
}
impl Expect for SymbolToken {
    type Value = Symbol;
    fn expect(&self, expected: &Self::Value) -> Result<()> {
        track_assert_eq!(self.value(), *expected, ErrorKind::InvalidInput);
        Ok(())
    }
}
impl Expect for VariableToken {
    type Value = str;
    fn expect(&self, expected: &Self::Value) -> Result<()> {
        track_assert_eq!(self.value(), expected, ErrorKind::InvalidInput);
        Ok(())
    }
}


impl IntoTokens for AtomToken {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(iter::once(self.into()))
    }
}
impl IntoTokens for CharToken {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(iter::once(self.into()))
    }
}
impl IntoTokens for FloatToken {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(iter::once(self.into()))
    }
}
impl IntoTokens for IntegerToken {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(iter::once(self.into()))
    }
}
impl IntoTokens for KeywordToken {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(iter::once(self.into()))
    }
}
impl IntoTokens for StringToken {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(iter::once(self.into()))
    }
}
impl IntoTokens for SymbolToken {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(iter::once(self.into()))
    }
}
impl IntoTokens for VariableToken {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(iter::once(self.into()))
    }
}
impl<T: IntoTokens> IntoTokens for Option<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        if let Some(x) = self {
            x.into_tokens()
        } else {
            Box::new(iter::empty())
        }
    }
}
impl<T: IntoTokens> IntoTokens for Box<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new((*self).into_tokens())
    }
}
