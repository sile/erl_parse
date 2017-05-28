use erl_tokenize::Token;
use erl_tokenize::tokens::{AtomToken, IntegerToken};
use num::ToPrimitive;

use {Result, ErrorKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Arity(pub usize);
impl Arity {
    pub fn from_integer(n: &IntegerToken) -> Result<Self> {
        let n = track_try!(n.value().to_usize().ok_or(ErrorKind::InvalidInput),
                           "n={:?}",
                           n);
        Ok(Arity(n))
    }
}

#[derive(Debug)]
pub struct Export<'a> {
    pub name: AtomToken<'a>,
    pub arity: Arity,
}
impl<'a> Export<'a> {
    pub fn new(name: AtomToken<'a>, arity: Arity) -> Self {
        Export { name, arity }
    }
}

#[derive(Debug)]
pub struct List<'a, T> {
    pub leading_tokens: Vec<Token<'a>>, // token* '['
    pub elements: Vec<ListElement<'a, T>>, // element* ']'
}

#[derive(Debug)]
pub struct ListElement<'a, T> {
    pub value: T, // token* T
    pub trailing_tokens: Vec<Token<'a>>, // token* (','|']')
}
