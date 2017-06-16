use std::marker::PhantomData;
use erl_tokenize::LexicalToken;

use Error;

#[derive(Debug)]
pub struct Parser<T, E> {
    tokens: T,
    unread: Vec<LexicalToken>,
    _phantom: PhantomData<E>,
}
impl<T, E> Parser<T, E>
where
    T: Iterator<Item = Result<LexicalToken, E>>,
    Error: From<E>,
{
    pub fn new(tokens: T) -> Self {
        Parser {
            tokens,
            unread: Vec::new(),
            _phantom: PhantomData,
        }
    }
    pub fn tokens(&self) -> &T {
        &self.tokens
    }
    pub fn tokens_mut(&mut self) -> &mut T {
        &mut self.tokens
    }
}
