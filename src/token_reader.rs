use erl_tokenize::Token;
use erl_tokenize::tokens::{AtomToken, SymbolToken, IntegerToken, VariableToken};
use erl_tokenize::values::Symbol;

use {Result, ErrorKind, Parse};

#[derive(Debug)]
pub struct TokenReader<'token, 'text: 'token> {
    tokens: &'token [Token<'text>],
    position: usize,
}
impl<'token, 'text: 'token> TokenReader<'token, 'text> {
    pub fn new(tokens: &'token [Token<'text>]) -> Self {
        let mut this = TokenReader {
            tokens,
            position: 0,
        };
        this.skip_hidden_tokens();
        this
    }
    pub fn parse_next<T: Parse<'token, 'text>>(&mut self) -> Result<T> {
        track!(T::parse(self))
    }
    pub fn try_parse_next<T: Parse<'token, 'text>>(&mut self) -> Option<T> {
        T::try_parse(self)
    }

    pub fn remaining_tokens(&self) -> &'token [Token<'text>] {
        &self.tokens[self.position..]
    }
    pub fn position(&self) -> usize {
        self.position
    }
    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }
    fn skip_hidden_tokens(&mut self) {
        let count = self.tokens
            .iter()
            .skip(self.position)
            .take_while(|&t| match *t {
                            Token::Comment(_) |
                            Token::Whitespace(_) => true,
                            _ => false,
                        })
            .count();
        self.position += count;
    }
    pub fn read(&mut self) -> Result<&'token Token<'text>> {
        if let Some(token) = self.tokens.get(self.position) {
            self.position += 1;
            self.skip_hidden_tokens();
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
    pub fn read_variable(&mut self) -> Result<&'token VariableToken<'text>> {
        let token = track_try!(self.read());
        if let Token::Variable(ref token) = *token {
            Ok(token)
        } else {
            track_panic!(ErrorKind::InvalidInput,
                         "expected=VariableToken, actual={:?}",
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
