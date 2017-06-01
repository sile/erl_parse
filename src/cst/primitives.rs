use std::ops::Deref;
use erl_tokenize::tokens::{AtomToken, IntegerToken, VariableToken, StringToken};

use {Result, TokenReader, Parse, TokenRange};
use super::symbols;

#[derive(Debug)]
pub struct Seq2<T, D> {
    pub position: usize,
    pub elems: Option<NonEmptySeq<T, D>>,
}
impl<'token, 'text: 'token, T, D> Parse<'token, 'text> for Seq2<T, D>
    where T: Parse<'token, 'text>,
          D: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let position = reader.position();
        Ok(Seq2 {
               position,
               elems: try_parse!(reader),
           })
    }
}
impl<T, D> TokenRange for Seq2<T, D>
    where T: TokenRange,
          D: TokenRange
{
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.elems.as_ref().map_or(self.position, |e| e.token_end())
    }
}

#[derive(Debug)]
pub struct NonEmptySeq<T, D> {
    pub head: T,
    pub tail: Vec<SeqElem<T, D>>,
}
derive_parse3!(NonEmptySeq, head, tail);
impl<T, D> TokenRange for NonEmptySeq<T, D>
    where T: TokenRange,
          D: TokenRange
{
    fn token_start(&self) -> usize {
        self.head.token_start()
    }
    fn token_end(&self) -> usize {
        self.tail
            .last()
            .map_or(self.head.token_end(), |e| e.token_end())
    }
}

#[derive(Debug)]
pub struct SeqElem<T, D> {
    pub delim: D,
    pub elem: T,
}
derive_parse3!(SeqElem, delim, elem);
derive_token_range3!(SeqElem, delim, elem);

// non empty
#[derive(Debug)]
pub struct Seq<T> {
    pub position: usize,
    pub items: Vec<SeqItem<T>>,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for Seq<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let position = reader.position();
        let mut items = Vec::new();
        loop {
            let x = track_try!(SeqItem::parse(reader));
            let is_last = x.delimiter.is_none();
            items.push(x);
            if is_last {
                break;
            }
        }
        Ok(Seq { position, items })
    }
}
impl<T> TokenRange for Seq<T>
    where T: TokenRange
{
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.items.last().expect("Non empty").token_end()
    }
}

#[derive(Debug)]
pub struct SeqItem<T> {
    pub item: T,
    pub delimiter: Option<symbols::Comma>,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for SeqItem<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let item = track_try!(T::parse(reader));
        let delimiter = symbols::Comma::try_parse(reader);
        Ok(SeqItem { item, delimiter })
    }
}
impl<T> TokenRange for SeqItem<T>
    where T: TokenRange
{
    fn token_start(&self) -> usize {
        self.item.token_start()
    }
    fn token_end(&self) -> usize {
        self.delimiter
            .map_or(self.item.token_end(), |d| d.token_end())
    }
}

#[derive(Debug)]
pub struct Clauses<T> {
    pub position: usize,
    pub clauses: Vec<Clause<T>>,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for Clauses<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let position = reader.position();
        let mut clauses = Vec::new();
        loop {
            let c = track_try!(Clause::parse(reader));
            let is_last = c.delimiter.is_none();
            clauses.push(c);
            if is_last {
                break;
            }
        }
        Ok(Clauses { position, clauses })
    }
}
impl<T> TokenRange for Clauses<T>
    where T: TokenRange
{
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.clauses.last().map_or(self.position, |c| c.token_end())
    }
}

#[derive(Debug)]
pub struct Clause<T> {
    pub clause: T,
    pub delimiter: Option<symbols::Semicolon>,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for Clause<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let clause = track_try!(T::parse(reader));
        let delimiter = symbols::Semicolon::try_parse(reader);
        Ok(Clause { clause, delimiter })
    }
}
impl<T> TokenRange for Clause<T>
    where T: TokenRange
{
    fn token_start(&self) -> usize {
        self.clause.token_start()
    }
    fn token_end(&self) -> usize {
        self.delimiter
            .map_or(self.clause.token_end(), |d| d.token_end())
    }
}


#[derive(Debug)]
pub struct Tuple<T> {
    pub _open: symbols::OpenBrace,
    pub elements: Seq<T>,
    pub _close: symbols::CloseBrace,
}
derive_parse2!(Tuple, _open, elements, _close);
derive_token_range2!(Tuple, _open, _close);

#[derive(Debug)]
pub struct List<T> {
    pub open: symbols::OpenSquare,
    pub elements: Vec<ListElement<T>>,
    pub close: symbols::CloseSquare,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for List<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let open = track_try!(symbols::OpenSquare::parse(reader));
        let mut elements = Vec::new();
        if let Some(close) = symbols::CloseSquare::try_parse(reader) {
            return Ok(List {
                          open,
                          elements,
                          close,
                      });
        }
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
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
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
pub struct Args<T> {
    pub open: symbols::OpenParen,
    pub args: Vec<Arg<T>>,
    pub close: symbols::CloseParen,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for Args<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let open = track_try!(Parse::parse(reader));
        let mut args = Vec::new();
        if let Some(close) = symbols::CloseParen::try_parse(reader) {
            return Ok(Args { open, args, close });
        }

        loop {
            let a = track_try!(Arg::parse(reader));
            let is_last = a.delimiter.is_none();
            args.push(a);
            if is_last {
                break;
            }
        }
        let close = track_try!(Parse::parse(reader));
        Ok(Args { open, args, close })
    }
}
impl<T> TokenRange for Args<T> {
    fn token_start(&self) -> usize {
        self.open.token_start()
    }
    fn token_end(&self) -> usize {
        self.close.token_end()
    }
}

#[derive(Debug)]
pub struct Arg<T> {
    pub arg: T,
    pub delimiter: Option<symbols::Comma>,
}
impl<'token, 'text: 'token, T> Parse<'token, 'text> for Arg<T>
    where T: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let arg = track_try!(T::parse(reader));
        let delimiter = symbols::Comma::try_parse(reader);
        Ok(Arg { arg, delimiter })
    }
}
impl<T> TokenRange for Arg<T>
    where T: TokenRange
{
    fn token_start(&self) -> usize {
        self.arg.token_start()
    }
    fn token_end(&self) -> usize {
        self.delimiter
            .map_or(self.arg.token_end(), |d| d.token_end())
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
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        // reader.skip_hidden_tokens();
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
pub struct Str<'token, 'text: 'token> {
    position: usize,
    value: &'token StringToken<'text>,
}
impl<'token, 'text: 'token> Deref for Str<'token, 'text> {
    type Target = StringToken<'text>;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Str<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        // reader.skip_hidden_tokens();
        let position = reader.position();
        let value = track_try!(reader.read_string());
        Ok(Str { position, value })
    }
}
impl<'token, 'text: 'token> TokenRange for Str<'token, 'text> {
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.position + 1
    }
}

#[derive(Debug)]
pub struct Variable<'token, 'text: 'token> {
    position: usize,
    value: &'token VariableToken<'text>,
}
impl<'token, 'text: 'token> Deref for Variable<'token, 'text> {
    type Target = VariableToken<'text>;
    fn deref(&self) -> &Self::Target {
        self.value
    }
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Variable<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        // reader.skip_hidden_tokens();
        let position = reader.position();
        let value = track_try!(reader.read_variable());
        Ok(Variable { position, value })
    }
}
impl<'token, 'text: 'token> TokenRange for Variable<'token, 'text> {
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.position + 1
    }
}

#[derive(Debug)]
pub struct ModuleAtom<'token, 'text: 'token> {
    pub module_name: Atom<'token, 'text>,
    pub colon: symbols::Colon,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for ModuleAtom<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        Ok(ModuleAtom {
               module_name: track_try!(Parse::parse(reader)),
               colon: track_try!(Parse::parse(reader)),
           })
    }
}
impl<'token, 'text: 'token> TokenRange for ModuleAtom<'token, 'text> {
    fn token_start(&self) -> usize {
        self.module_name.token_start()
    }
    fn token_end(&self) -> usize {
        self.colon.token_end()
    }
}

#[derive(Debug)]
pub struct Int<'token, 'text: 'token> {
    pub sign: NumSign,
    pub value: Integer<'token, 'text>,
}
derive_parse!(Int, sign, value);
derive_token_range!(Int, sign, value);

#[derive(Debug)]
pub struct NumSign {
    position: usize,
    pub sign: Option<Sign>,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for NumSign {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let position = reader.position();
        let sign = if reader.try_parse_next::<symbols::Plus>().is_some() {
            Some(Sign::Plus)
        } else if reader.try_parse_next::<symbols::Hyphen>().is_some() {
            Some(Sign::Minus)
        } else {
            None
        };
        Ok(NumSign { position, sign })
    }
}
impl TokenRange for NumSign {
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.position + self.sign.map_or(0, |_| 1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Plus,
    Minus,
}

// TODO:
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
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
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
    pub fun_name: Atom<'token, 'text>,
    pub delim: symbols::Slash,
    pub arity: Integer<'token, 'text>,
}
derive_parse!(Export, fun_name, delim, arity);
derive_token_range!(Export, fun_name, arity);

#[derive(Debug)]
pub struct Import<'token, 'text: 'token> {
    pub fun_name: Atom<'token, 'text>,
    pub delim: symbols::Slash,
    pub arity: Integer<'token, 'text>,
}
derive_parse!(Import, fun_name, delim, arity);
derive_token_range!(Import, fun_name, arity);

#[derive(Debug)]
pub struct ExportType<'token, 'text: 'token> {
    pub type_name: Atom<'token, 'text>,
    pub delim: symbols::Slash,
    pub arity: Integer<'token, 'text>,
}
derive_parse!(ExportType, type_name, delim, arity);
derive_token_range!(ExportType, type_name, arity);
