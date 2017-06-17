use std::ops::{Deref, DerefMut};
use erl_tokenize::LexicalToken;

use {Result, TokenReader, Parse, Preprocessor};

#[derive(Debug)]
pub struct Parser<T> {
    reader: TokenReader<T>,
}
impl<T> Parser<T>
where
    T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
{
    pub fn new(reader: TokenReader<T>) -> Self {
        Parser { reader }
    }
    pub fn parse<P: Parse>(&mut self) -> Result<P> {
        track!(P::parse(self))
    }
    pub fn try_parse<P: Parse>(&mut self) -> Result<Option<P>> {
        track!(P::try_parse(self))
    }
}

// TODO: delete
impl<T> Deref for Parser<T> {
    type Target = TokenReader<T>;
    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}
impl<T> DerefMut for Parser<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}
