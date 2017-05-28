use std::mem;
use erl_tokenize::{Token, Result as TokenizeResult};
use erl_tokenize::tokens::{AtomToken, SymbolToken, IntegerToken};
use erl_tokenize::values::Symbol;

use {Result, ErrorKind};

#[derive(Debug)]
pub struct TokenReader2<'token, 'text: 'token> {
    tokens: &'token [Token<'text>],
}
impl<'token, 'text: 'token> TokenReader2<'token, 'text> {
    pub fn new(tokens: &'token [Token<'text>]) -> Self {
        TokenReader2 { tokens }
    }
}

// TODO: Track line number for debugging
#[derive(Debug)]
pub struct TokenReader<'a, I> {
    tokens: I,
    unreads: Vec<Token<'a>>,
    reads: Vec<Token<'a>>,
}
impl<'a, I> TokenReader<'a, I>
    where I: Iterator<Item = TokenizeResult<Token<'a>>>
{
    pub fn new(tokens: I) -> Self {
        TokenReader {
            tokens,
            unreads: Vec::new(),
            reads: Vec::new(),
        }
    }
    pub fn take_read_tokens(&mut self) -> Vec<Token<'a>> {
        mem::replace(&mut self.reads, Vec::new())
    }
    pub fn read(&mut self) -> Result<Option<&Token<'a>>> {
        if let Some(token) = self.unreads.pop() {
            let i = self.reads.len();
            self.reads.push(token);
            while self.unreads.last().map_or(false, |t| is_hidden_token(t)) {
                self.reads.push(self.unreads.pop().expect("Never fails"));
            }
            Ok(self.reads.get(i))
        } else if let Some(token) = track_try!(self.next_non_hidden()) {
            self.reads.push(token);
            Ok(self.reads.last())
        } else {
            Ok(None)
        }
    }
    // pub fn unread(&mut self) -> Result<()> {
    //     let token = track_try!(self.reads.pop().ok_or(ErrorKind::Other));
    //     self.unreads.push(token);
    //     Ok(())
    // }
    pub fn peek(&mut self) -> Result<Option<&Token<'a>>> {
        if self.unreads.is_empty() {
            if let Some(token) = track_try!(self.next_non_hidden()) {
                self.unreads.push(token);
            }
        }
        assert!(self.unreads.last().map_or(true, |t| !is_hidden_token(t)));
        Ok(self.unreads.last())
    }
    fn next_non_hidden(&mut self) -> Result<Option<Token<'a>>> {
        while let Some(token) = self.tokens.next() {
            let token = track_try!(token);
            if is_hidden_token(&token) {
                self.reads.push(token);
            } else {
                return Ok(Some(token));
            }
        }
        Ok(None)
    }
    pub fn read_atom(&mut self) -> Result<&AtomToken<'a>> {
        let token = track_try!(self.read());
        let token = track_try!(token.ok_or(ErrorKind::UnexpectedEos));
        if let Token::Atom(ref token) = *token {
            Ok(token)
        } else {
            track_panic!(ErrorKind::InvalidInput,
                         "expected=AtomToken, actual={:?}",
                         token);
        }
    }
    pub fn read_integer(&mut self) -> Result<&IntegerToken<'a>> {
        let token = track_try!(self.read());
        let token = track_try!(token.ok_or(ErrorKind::UnexpectedEos));
        if let Token::Integer(ref token) = *token {
            Ok(token)
        } else {
            track_panic!(ErrorKind::InvalidInput,
                         "expected=IntegerToken, actual={:?}",
                         token);
        }
    }
    pub fn read_symbol(&mut self) -> Result<SymbolToken<'a>> {
        let token = track_try!(self.read());
        let token = track_try!(token.ok_or(ErrorKind::UnexpectedEos));
        if let Token::Symbol(ref token) = *token {
            Ok(token.clone())
        } else {
            track_panic!(ErrorKind::InvalidInput,
                         "expected=SymbolToken, actual={:?}",
                         token);
        }
    }
    pub fn expect_symbol(&mut self, expected: Symbol) -> Result<()> {
        let symbol = track_try!(self.read_symbol());
        track_assert_eq!(symbol.value(), expected, ErrorKind::InvalidInput);
        Ok(())
    }
    pub fn read_list<F, T>(&mut self, f: F) -> Result<Vec<T>>
        where F: Fn(&mut Self) -> Result<T>
    {
        let mut list = Vec::new();
        track_try!(self.expect_symbol(Symbol::OpenSquare));
        loop {
            let value = track_try!(f(self));
            list.push(value);
            let symbol = track_try!(self.read_symbol());
            match symbol.value() {
                Symbol::Comma => {}
                Symbol::CloseSquare => break,
                _ => track_panic!(ErrorKind::InvalidInput, "Unexpected symbol: {:?}", symbol),
            }
        }
        Ok(list)
    }
}

fn is_hidden_token(token: &Token) -> bool {
    match *token {
        Token::Whitespace(_) |
        Token::Comment(_) => true,
        _ => false,
    }
}
