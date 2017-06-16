use num::BigUint;
use erl_tokenize::LexicalToken;
use erl_tokenize::tokens::{AtomToken, CharToken, FloatToken, IntegerToken, KeywordToken,
                           StringToken, SymbolToken, VariableToken};
use erl_tokenize::values::{Symbol, Keyword};

use {Result, ErrorKind, Preprocessor, TokenReader};

pub trait Parse: Sized {
    fn parse<T>(reader: &mut TokenReader<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        if let Some(value) = track!(Self::try_parse(reader))? {
            Ok(value)
        } else {
            track_panic!(ErrorKind::InvalidInput);
        }
    }

    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor;

    fn try_parse_expect<T>(
        reader: &mut TokenReader<T>,
        expected: &Self::Value,
    ) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
        Self: Expect,
    {
        let actual = track_try_some!(Self::try_parse(reader));
        if actual.expect(expected).is_ok() {
            Ok(Some(actual))
        } else {
            reader.unread_token(actual.into());
            Ok(None)
        }
    }

    fn parse_expect<T>(reader: &mut TokenReader<T>, expected: &Self::Value) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
        Self: Expect,
    {
        let actual = track!(Self::parse(reader))?;
        track!(actual.expect(expected))?;
        Ok(actual)
    }
}
impl<U: Parse> Parse for Box<U> {
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Parse::try_parse(reader)?.map(Box::new))
    }
}
impl Parse for AtomToken {
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(reader.try_read_token())?.and_then(|token| {
            token
                .into_atom_token()
                .map_err(|token| reader.unread_token(token))
                .ok()
        }))
    }
}
impl Parse for CharToken {
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(reader.try_read_token())?.and_then(|token| {
            token
                .into_char_token()
                .map_err(|token| reader.unread_token(token))
                .ok()
        }))
    }
}
impl Parse for FloatToken {
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(reader.try_read_token())?.and_then(|token| {
            token
                .into_float_token()
                .map_err(|token| reader.unread_token(token))
                .ok()
        }))
    }
}
impl Parse for IntegerToken {
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(reader.try_read_token())?.and_then(|token| {
            token
                .into_integer_token()
                .map_err(|token| reader.unread_token(token))
                .ok()
        }))
    }
}
impl Parse for KeywordToken {
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(reader.try_read_token())?.and_then(|token| {
            token
                .into_keyword_token()
                .map_err(|token| reader.unread_token(token))
                .ok()
        }))
    }
}
impl Parse for StringToken {
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(reader.try_read_token())?.and_then(|token| {
            token
                .into_string_token()
                .map_err(|token| reader.unread_token(token))
                .ok()
        }))
    }
}
impl Parse for SymbolToken {
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(reader.try_read_token())?.and_then(|token| {
            token
                .into_symbol_token()
                .map_err(|token| reader.unread_token(token))
                .ok()
        }))
    }
}
impl Parse for VariableToken {
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(reader.try_read_token())?.and_then(|token| {
            token
                .into_variable_token()
                .map_err(|token| reader.unread_token(token))
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
