use std;
use erl_pp::{self, MacroDef};
use erl_tokenize::{LexicalToken, Lexer};

use {Result, Error, ErrorKind, IntoTokens};

#[derive(Debug)]
pub struct Tokens<T, E>(T)
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
    tokens: T,
    unread: Vec<LexicalToken>,
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
    pub fn peek_token(&mut self) -> Result<Option<LexicalToken>> {
        match track!(self.try_read_token())? {
            None => Ok(None),
            Some(t) => {
                self.unread_token(t.clone());
                Ok(Some(t))
            }
        }
    }
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
    pub fn unread_tokens<I: IntoTokens>(&mut self, tokens: I) {
        let mut tokens = tokens.into_tokens().collect::<Vec<_>>();
        tokens.reverse();
        for t in tokens {
            self.unread_token(t);
        }
    }
}

pub trait Preprocessor {
    fn define_macro(&mut self, name: &str, replacement: Vec<LexicalToken>);
    fn undef_macro(&mut self, name: &str);
}
impl<T, E> Preprocessor for erl_pp::Preprocessor<T, E> {
    fn define_macro(&mut self, name: &str, replacement: Vec<LexicalToken>) {
        self.macros_mut().insert(
            name.to_string(),
            MacroDef::Dynamic(replacement),
        );
    }
    fn undef_macro(&mut self, name: &str) {
        self.macros_mut().remove(name);
    }
}
impl<T> Preprocessor for Lexer<T> {
    fn define_macro(&mut self, _name: &str, _replacement: Vec<LexicalToken>) {}
    fn undef_macro(&mut self, _name: &str) {}
}
impl<T, E> Preprocessor for Tokens<T, E>
where
    T: Iterator<Item = std::result::Result<LexicalToken, E>>
        + Preprocessor,
    Error: From<E>,
{
    fn define_macro(&mut self, name: &str, replacement: Vec<LexicalToken>) {
        self.0.define_macro(name, replacement);
    }
    fn undef_macro(&mut self, name: &str) {
        self.0.undef_macro(name);
    }
}
