use num::BigUint;
use erl_tokenize::LexicalToken;
use erl_tokenize::tokens::{AtomToken, CharToken, FloatToken, IntegerToken, KeywordToken,
                           StringToken, SymbolToken, VariableToken};
use erl_tokenize::values::{Symbol, Keyword};

use {Result, ErrorKind, Preprocessor, Parser, Error};

pub trait ParseLeftRecur: Sized {
    type Left;
    fn parse_left_recur<T>(parser: &mut Parser<T>, left: Self::Left) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor;
}
impl<T: ParseLeftRecur> ParseLeftRecur for Box<T> {
    type Left = T::Left;
    fn parse_left_recur<U>(parser: &mut Parser<U>, left: Self::Left) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        parser.parse_left_recur(left).map(Box::new)
    }
}

pub trait Parse: Sized {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor;
}
impl<U: Parse> Parse for Box<U> {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        parser.parse().map(Box::new)
    }
}
impl<T: Parse> Parse for Option<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(parser.transaction(|parser| parser.parse()).ok())
    }
}
impl Parse for AtomToken {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let token = track!(parser.read_token())?;
        let token = track!(
            token
                .into_atom_token()
                .map_err(ErrorKind::UnexpectedToken)
                .map_err(Error::from)
        )?;
        Ok(token)
    }
}
impl Parse for CharToken {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let token = track!(parser.read_token())?;
        let token = track!(
            token
                .into_char_token()
                .map_err(ErrorKind::UnexpectedToken)
                .map_err(Error::from)
        )?;
        Ok(token)
    }
}
impl Parse for FloatToken {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let token = track!(parser.read_token())?;
        let token = track!(
            token
                .into_float_token()
                .map_err(ErrorKind::UnexpectedToken)
                .map_err(Error::from)
        )?;
        Ok(token)
    }
}
impl Parse for IntegerToken {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let token = track!(parser.read_token())?;
        let token = track!(
            token
                .into_integer_token()
                .map_err(ErrorKind::UnexpectedToken)
                .map_err(Error::from)
        )?;
        Ok(token)
    }
}
impl Parse for KeywordToken {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let token = track!(parser.read_token())?;
        let token = track!(
            token
                .into_keyword_token()
                .map_err(ErrorKind::UnexpectedToken)
                .map_err(Error::from)
        )?;
        Ok(token)
    }
}
impl Parse for StringToken {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let token = track!(parser.read_token())?;
        let token = track!(
            token
                .into_string_token()
                .map_err(ErrorKind::UnexpectedToken)
                .map_err(Error::from)
        )?;
        Ok(token)
    }
}
impl Parse for SymbolToken {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let token = track!(parser.read_token())?;
        let token = track!(
            token
                .into_symbol_token()
                .map_err(ErrorKind::UnexpectedToken)
                .map_err(Error::from)
        )?;
        Ok(token)
    }
}
impl Parse for VariableToken {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let token = track!(parser.read_token())?;
        let token = track!(
            token
                .into_variable_token()
                .map_err(ErrorKind::UnexpectedToken)
                .map_err(Error::from)
        )?;
        Ok(token)
    }
}

pub trait Expect: Sized {
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
