use std::ops::Deref;
use erl_tokenize::tokens::{AtomToken, IntegerToken};

use {Result, TokenReader2, Parse, TokenRange};
use super::symbols;

#[derive(Debug)]
pub struct Atom<'token, 'text: 'token> {
    position: usize,
    value: &'token AtomToken<'text>,
}
impl<'token, 'text: 'token> Deref for Atom<'token, 'text> {
    type Target = AtomToken<'text>;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Atom<'token, 'text> {
    fn parse(reader: &mut TokenReader2<'token, 'text>) -> Result<Self> {
        reader.skip_hidden_tokens();
        let position = reader.position();
        let value = track_try!(reader.read_atom());
        Ok(Atom { position, value })
    }
}
impl<'token, 'text: 'token> TokenRange for Atom<'token, 'text> {
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.position + 1
    }
}

#[derive(Debug)]
pub struct Integer<'token, 'text: 'token> {
    position: usize,
    value: &'token IntegerToken<'text>,
}
impl<'token, 'text: 'token> Deref for Integer<'token, 'text> {
    type Target = IntegerToken<'text>;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Integer<'token, 'text> {
    fn parse(reader: &mut TokenReader2<'token, 'text>) -> Result<Self> {
        reader.skip_hidden_tokens();
        let position = reader.position();
        let value = track_try!(reader.read_integer());
        Ok(Integer { position, value })
    }
}
impl<'token, 'text: 'token> TokenRange for Integer<'token, 'text> {
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.position + 1
    }
}

#[derive(Debug)]
pub struct Export<'token, 'text: 'token> {
    pub name: Atom<'token, 'text>,
    pub delimiter: symbols::Slash,
    pub arity: Integer<'token, 'text>,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Export<'token, 'text> {
    fn parse(reader: &mut TokenReader2<'token, 'text>) -> Result<Self> {
        let name = track_try!(Atom::parse(reader));
        let delimiter = track_try!(symbols::Slash::parse(reader));
        let arity = track_try!(Integer::parse(reader));
        Ok(Export {
               name,
               delimiter,
               arity,
           })
    }
}
impl<'token, 'text: 'token> TokenRange for Export<'token, 'text> {
    fn token_start(&self) -> usize {
        self.name.token_start()
    }
    fn token_end(&self) -> usize {
        self.arity.token_end()
    }
}
