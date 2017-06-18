use std;
use erl_tokenize::LexicalToken;

use {Result, Error, ErrorKind};
use traits::Preprocessor;

#[derive(Debug)]
pub struct Tokens<T, E>(pub T)
where
    T: Iterator<Item = std::result::Result<LexicalToken, E>>,
    Error: From<E>;
impl<T, E> Tokens<T, E>
where
    T: Iterator<Item = std::result::Result<LexicalToken, E>>,
    Error: From<E>,
{
    pub fn new(inner: T) -> Self {
        Tokens(inner)
    }
}
impl<T, E> Iterator for Tokens<T, E>
where
    T: Iterator<Item = std::result::Result<LexicalToken, E>>,
    Error: From<E>,
{
    type Item = Result<LexicalToken>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|item| item.map_err(Error::from))
    }
}

#[derive(Debug)]
pub struct TokenReader<T> {
    pub tokens: T,
    pub unread: Vec<LexicalToken>, // TODO: private
}
impl<T, E> TokenReader<Tokens<T, E>>
where
    T: Iterator<Item = std::result::Result<LexicalToken, E>>
        + Preprocessor,
    Error: From<E>,
{
    pub fn new(tokens: T) -> Self {
        TokenReader {
            tokens: Tokens::new(tokens),
            unread: Vec::new(),
        }
    }
}
impl<T> TokenReader<T>
where
    T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
{
    pub fn read_token(&mut self) -> Result<LexicalToken> {
        if let Some(token) = track!(self.try_read_token())? {
            Ok(token)
        } else {
            track_panic!(ErrorKind::UnexpectedEos);
        }
    }
    pub fn try_read_token(&mut self) -> Result<Option<LexicalToken>> {
        match self.unread.pop().map(Ok).or_else(|| self.tokens.next()) {
            None => Ok(None),
            Some(Err(e)) => Err(e.into()),
            Some(Ok(t)) => Ok(Some(t)),
        }
    }
    pub fn unread_token(&mut self, token: LexicalToken) {
        self.unread.push(token);
    }
    // pub fn unread_tokens<I: IntoTokens>(&mut self, tokens: I) {
    //     let mut tokens = tokens.into_tokens().collect::<Vec<_>>();
    //     tokens.reverse();
    //     for t in tokens {
    //         self.unread_token(t);
    //     }
    // }
}
