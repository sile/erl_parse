use std::mem;
use erl_tokenize::{Token, Result as TokenizeResult};
use erl_tokenize::tokens::{AtomToken, SymbolToken, IntegerToken};
use erl_tokenize::values::Symbol;

use {Result, ErrorKind};

#[derive(Debug)]
pub struct TokenReader<'token, 'text: 'token> {
    tokens: &'token [Token<'text>],
    position: usize,
}
impl<'token, 'text: 'token> TokenReader<'token, 'text> {
    pub fn new(tokens: &'token [Token<'text>]) -> Self {
        TokenReader {
            tokens,
            position: 0,
        }
    }
    pub fn position(&self) -> usize {
        self.position
    }
    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }
    pub fn read_hidden_tokens(&mut self) -> &'token [Token<'text>] {
        let start = self.position;
        let end = self.tokens
            .iter()
            .skip(start)
            .position(|t| !is_hidden_token(t))
            .unwrap_or(self.tokens.len());
        self.position = end;
        &self.tokens[start..self.position]
    }
    pub fn skip_hidden_tokens(&mut self) {
        let start = self.position;
        let end = self.tokens
            .iter()
            .skip(start)
            .position(|t| !is_hidden_token(t))
            .unwrap_or(self.tokens.len());
        self.position = end;
    }
    pub fn read(&mut self) -> Result<&'token Token<'text>> {
        if let Some(token) = self.tokens.get(self.position) {
            self.position += 1;
            Ok(token)
        } else {
            track_panic!(ErrorKind::UnexpectedEos);
        }
    }
    pub fn read_atom(&mut self) -> Result<&'token AtomToken<'text>> {
        let token = track_try!(self.read());
        if let Token::Atom(ref token) = *token {
            Ok(token)
        } else {
            track_panic!(ErrorKind::InvalidInput,
                         "expected=AtomToken, actual={:?}",
                         token);
        }
    }
    pub fn read_symbol(&mut self) -> Result<&'token SymbolToken<'text>> {
        let token = track_try!(self.read());
        if let Token::Symbol(ref token) = *token {
            Ok(token)
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
    pub fn expect_atom(&mut self, expected: &str) -> Result<()> {
        let atom = track_try!(self.read_atom());
        track_assert_eq!(atom.value(), expected, ErrorKind::InvalidInput);
        Ok(())
    }
    pub fn read_integer(&mut self) -> Result<&'token IntegerToken<'text>> {
        let token = track_try!(self.read());
        if let Token::Integer(ref token) = *token {
            Ok(token)
        } else {
            track_panic!(ErrorKind::InvalidInput,
                         "expected=IntegerToken, actual={:?}",
                         token);
        }
    }
}
