use std::ops::Deref;

use erl_tokenize::Token;
use erl_tokenize::tokens::{AtomToken, CommentToken, WhitespaceToken, IntegerToken};

#[derive(Debug)]
pub struct Atom<'token, 'text: 'token> {
    pub leadings: &'token [Token<'text>],
    value: AtomToken<'text>,
}
impl<'token, 'text: 'token> Deref for Atom<'token, 'text> {
    type Target = AtomToken<'text>;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug)]
pub struct Integer<'token, 'text: 'token> {
    pub leadings: &'token [Token<'text>],
    value: IntegerToken<'text>,
}
impl<'token, 'text: 'token> Deref for Integer<'token, 'text> {
    type Target = IntegerToken<'text>;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[derive(Debug)]
pub struct Export<'token, 'text: 'token> {
    pub name: Atom<'token, 'text>,
    pub delimiters: &'token [Token<'text>], // Token* '/'
    pub arity: Integer<'token, 'text>,
}

#[derive(Debug)]
pub enum HiddenToken<'text> {
    Comment(CommentToken<'text>),
    Whitespace(WhitespaceToken<'text>),
}
