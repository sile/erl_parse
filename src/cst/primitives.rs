use std::ops::Deref;
use erl_tokenize::tokens::{AtomToken, IntegerToken};

use {Result, TokenReader2, Parse, TokenRange};
use super::symbols;

#[derive(Debug)]
pub struct List<T> {
    pub open: symbols::OpenSquare,
    pub elements: Vec<ListElement<T>>,
    pub close: symbols::CloseSquare,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for List<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader2<'token, 'text>) -> Result<Self> {
        let open = track_try!(symbols::OpenSquare::parse(reader));
        let mut elements = Vec::new();
        loop {
            let e = track_try!(ListElement::parse(reader));
            let is_last = e.delimiter.is_none();
            elements.push(e);
            if is_last {
                break;
            }
        }
        let close = track_try!(symbols::CloseSquare::parse(reader));
        Ok(List {
               open,
               elements,
               close,
           })
    }
}
impl<T> TokenRange for List<T> {
    fn token_start(&self) -> usize {
        self.open.token_start()
    }
    fn token_end(&self) -> usize {
        self.close.token_end()
    }
}

#[derive(Debug)]
pub struct ListElement<T> {
    pub value: T,
    pub delimiter: Option<symbols::Comma>,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for ListElement<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader2<'token, 'text>) -> Result<Self> {
        let value = track_try!(T::parse(reader));
        let delimiter = symbols::Comma::try_parse(reader);
        Ok(ListElement { value, delimiter })
    }
}
impl<T> TokenRange for ListElement<T>
    where T: TokenRange
{
    fn token_start(&self) -> usize {
        self.value.token_start()
    }
    fn token_end(&self) -> usize {
        self.delimiter
            .map_or(self.value.token_end(), |d| d.token_end())
    }
}

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
