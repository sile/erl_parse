use std::ops::{Deref, DerefMut};
use erl_tokenize::LexicalToken;

use {Result, TokenReader, Parse, Preprocessor, Expect};

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
    fn start_transaction(&mut self) {
        panic!()
    }
    fn commit_transaction(&mut self) {
        panic!()
    }
    fn abort_transaction(&mut self) {
        panic!()
    }
    pub fn transaction<F, P>(&mut self, f: F) -> Result<P>
    where
        F: FnOnce(&mut Self) -> Result<P>,
    {
        self.start_transaction();
        let result = track!(f(self));
        if result.is_ok() {
            self.commit_transaction();
        } else {
            self.abort_transaction();
        }
        result
    }
    pub fn parse<P: Parse>(&mut self) -> Result<P> {
        track!(P::parse(self))
    }
    pub fn expect<P: Parse + Expect>(&mut self, expected: &P::Value) -> Result<P> {
        let actual = track!(self.parse::<P>())?;
        if let Err(e) = track!(actual.expect(expected)) {
            self.unread_tokens(actual);
            Err(e)
        } else {
            Ok(actual)
        }
    }
    pub fn expect_any<P: Parse + Expect>(&mut self, expected: &[&P::Value]) -> Result<P> {
        let actual = track!(self.parse::<P>())?;
        let mut last_error = None;
        for e in expected.iter() {
            if let Err(e) = track!(actual.expect(e)) {
                last_error = Some(e);
            } else {
                last_error = None;
                break;
            }
        }
        if let Some(e) = last_error {
            Err(e)
        } else {
            Ok(actual)
        }
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
