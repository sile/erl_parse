use erl_tokenize::LexicalToken;
use erl_tokenize::tokens::{AtomToken, CharToken, FloatToken, IntegerToken, KeywordToken,
                           StringToken, SymbolToken, VariableToken};

use {Result, ErrorKind, Parser, Error};
use traits::Preprocessor;

pub trait Parse: Sized {
    fn parse_non_left_recor<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        parser.parse()
    }
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

pub trait ParseTail: Sized {
    type Head;
    fn parse_tail<T>(parser: &mut Parser<T>, head: Self::Head) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor;
}
impl<T: ParseTail> ParseTail for Box<T> {
    type Head = T::Head;
    fn parse_tail<U>(parser: &mut Parser<U>, head: Self::Head) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        T::parse_tail(parser, head).map(Box::new)
    }
}