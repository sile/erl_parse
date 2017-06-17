use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{AtomToken, SymbolToken, VariableToken};
use erl_tokenize::values::Symbol;

use {Result, Parse, Preprocessor, Parser, ErrorKind, ParseLeftRecur, TryInto};

#[derive(Debug, Clone)]
pub struct LocalCall<T> {
    pub name: T,
    pub args: Args<T>,
}
impl<T: Parse> ParseLeftRecur for LocalCall<T> {
    type Left = T;
    fn parse_left_recur<U>(parser: &mut Parser<U>, left: Self::Left) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(LocalCall {
            name: left,
            args: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for LocalCall<T> {
    fn start_position(&self) -> Position {
        self.name.start_position()
    }
    fn end_position(&self) -> Position {
        self.args.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct RemoteCall<T> {
    pub module_name: T,
    pub _colon: SymbolToken,
    pub fun: LocalCall<T>,
}
impl<T: Parse + TryInto<LocalCall<T>>> ParseLeftRecur for RemoteCall<T> {
    type Left = T;
    fn parse_left_recur<U>(parser: &mut Parser<U>, left: Self::Left) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(RemoteCall {
            module_name: left,
            _colon: track!(parser.expect(&Symbol::Colon))?,
            fun: track!(parser.parse::<T>().and_then(|t| t.try_into()))?,
        })
    }
}
impl<T: PositionRange> PositionRange for RemoteCall<T> {
    fn start_position(&self) -> Position {
        self.module_name.start_position()
    }
    fn end_position(&self) -> Position {
        self.fun.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Args<T> {
    pub _open_paren: SymbolToken,
    pub args: Option<Sequence<T>>,
    pub _close_paren: SymbolToken,
}
impl<T: Parse> Parse for Args<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let _open_paren = track!(parser.expect(&Symbol::OpenParen))?;
        let args = track!(parser.parse())?;
        let _close_paren = track!(parser.expect(&Symbol::CloseParen))?;
        Ok(Args {
            _open_paren, //: track!(parser.expect(&Symbol::OpenParen))?,
            args, //: track!(parser.parse())?,
            _close_paren, //: track!(parser.expect(&Symbol::CloseParen))?,
        })
    }
}
impl<T> PositionRange for Args<T> {
    fn start_position(&self) -> Position {
        self._open_paren.start_position()
    }
    fn end_position(&self) -> Position {
        self._close_paren.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Parenthesized<T> {
    pub _open_paren: SymbolToken,
    pub item: T,
    pub _close_paren: SymbolToken,
}
impl<T: Parse> Parse for Parenthesized<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Parenthesized {
            _open_paren: track!(parser.expect(&Symbol::OpenParen))?,
            item: track!(parser.parse())?,
            _close_paren: track!(parser.expect(&Symbol::CloseParen))?,
        })
    }
}
impl<T> PositionRange for Parenthesized<T> {
    fn start_position(&self) -> Position {
        self._open_paren.start_position()
    }
    fn end_position(&self) -> Position {
        self._close_paren.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Sequence<T> {
    pub item: T,
    pub tail: Option<SequenceTail<T>>,
}
impl<T> Sequence<T> {
    pub fn iter(&self) -> SequenceIter<T> {
        let inner = SequenceIterInner::Head(self);
        SequenceIter(inner)
    }
}
impl<T: Parse> Parse for Sequence<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Sequence {
            item: track!(parser.parse())?,
            tail: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for Sequence<T> {
    fn start_position(&self) -> Position {
        self.item.start_position()
    }
    fn end_position(&self) -> Position {
        self.tail
            .as_ref()
            .map(|t| t.end_position())
            .unwrap_or_else(|| self.item.end_position())
    }
}

#[derive(Debug, Clone)]
pub struct SequenceTail<T> {
    pub _comma: SymbolToken,
    pub item: T,
    pub tail: Option<Box<SequenceTail<T>>>,
}
impl<T: Parse> Parse for SequenceTail<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(SequenceTail {
            _comma: track!(parser.expect(&Symbol::Comma))?,
            item: track!(parser.parse())?,
            tail: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for SequenceTail<T> {
    fn start_position(&self) -> Position {
        self._comma.start_position()
    }
    fn end_position(&self) -> Position {
        self.tail
            .as_ref()
            .map(|t| t.end_position())
            .unwrap_or_else(|| self.item.end_position())
    }
}

#[derive(Debug)]
pub struct SequenceIter<'a, T: 'a>(SequenceIterInner<'a, T>);
impl<'a, T: 'a> Iterator for SequenceIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[derive(Debug)]
enum SequenceIterInner<'a, T: 'a> {
    Head(&'a Sequence<T>),
    Tail(&'a SequenceTail<T>),
    Eos,
}
impl<'a, T: 'a> Iterator for SequenceIterInner<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            SequenceIterInner::Head(&Sequence { ref item, ref tail }) => {
                if let Some(ref tail) = *tail {
                    *self = SequenceIterInner::Tail(tail);
                } else {
                    *self = SequenceIterInner::Eos
                }
                Some(item)
            }
            SequenceIterInner::Tail(&SequenceTail { ref item, ref tail, .. }) => {
                if let Some(ref tail) = *tail {
                    *self = SequenceIterInner::Tail(tail);
                } else {
                    *self = SequenceIterInner::Eos
                }
                Some(item)
            }
            SequenceIterInner::Eos => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConsCell<T> {
    pub item: T,
    pub tail: Option<ConsCellTail<T>>,
}
impl<T: Parse> Parse for ConsCell<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(ConsCell {
            item: track!(parser.parse())?,
            tail: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for ConsCell<T> {
    fn start_position(&self) -> Position {
        self.item.start_position()
    }
    fn end_position(&self) -> Position {
        self.tail
            .as_ref()
            .map(|t| t.end_position())
            .unwrap_or_else(|| self.item.end_position())
    }
}

#[derive(Debug, Clone)]
pub enum ConsCellTail<T> {
    Proper {
        _comma: SymbolToken,
        item: T,
        tail: Option<Box<ConsCellTail<T>>>,
    },
    Improper { _bar: SymbolToken, item: T },
}
impl<T: Parse> Parse for ConsCellTail<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let symbol: SymbolToken = track!(parser.parse())?;
        match symbol.value() {
            Symbol::Comma => {
                Ok(ConsCellTail::Proper {
                    _comma: symbol,
                    item: track!(parser.parse())?,
                    tail: track!(parser.parse())?,
                })
            }
            Symbol::VerticalBar => {
                Ok(ConsCellTail::Improper {
                    _bar: symbol,
                    item: track!(parser.parse())?,
                })
            }
            _ => {
                track_panic!(ErrorKind::InvalidInput, "Unexpected symbol: {:?}", symbol);
            }
        }
    }
}
impl<T: PositionRange> PositionRange for ConsCellTail<T> {
    fn start_position(&self) -> Position {
        match *self {
            ConsCellTail::Proper { ref _comma, .. } => _comma.start_position(),
            ConsCellTail::Improper { ref _bar, .. } => _bar.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            ConsCellTail::Proper { ref item, ref tail, .. } => {
                tail.as_ref().map(|t| t.end_position()).unwrap_or_else(|| {
                    item.end_position()
                })
            }
            ConsCellTail::Improper { ref item, .. } => item.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MapField<T> {
    pub key: T,
    pub _relation: SymbolToken,
    pub value: T,
}
impl<T: Parse> Parse for MapField<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(MapField {
            key: track!(parser.parse())?,
            _relation: track!(parser.expect_any(
                &[&Symbol::DoubleRightArrow, &Symbol::MapMatch],
            ))?,
            value: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for MapField<T> {
    fn start_position(&self) -> Position {
        self.key.start_position()
    }
    fn end_position(&self) -> Position {
        self.value.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct RecordField<T> {
    pub key: AtomOrVariable,
    pub _bind: SymbolToken,
    pub value: T,
}
impl<T: Parse> Parse for RecordField<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(RecordField {
            key: track!(parser.parse())?,
            _bind: track!(parser.parse())?,
            value: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for RecordField<T> {
    fn start_position(&self) -> Position {
        self.key.start_position()
    }
    fn end_position(&self) -> Position {
        self.value.end_position()
    }
}

#[derive(Debug, Clone)]
pub enum AtomOrVariable {
    Atom(AtomToken),
    Variable(VariableToken),
}
impl AtomOrVariable {
    pub fn value(&self) -> &str {
        match *self {
            AtomOrVariable::Atom(ref t) => t.value(),
            AtomOrVariable::Variable(ref t) => t.value(),
        }
    }
    pub fn text(&self) -> &str {
        match *self {
            AtomOrVariable::Atom(ref t) => t.text(),
            AtomOrVariable::Variable(ref t) => t.text(),
        }
    }
}
impl Parse for AtomOrVariable {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let token = track!(parser.read_token())?;
        match token {
            LexicalToken::Atom(token) => Ok(AtomOrVariable::Atom(token)),
            LexicalToken::Variable(token) => Ok(AtomOrVariable::Variable(token)),
            _ => track_panic!(ErrorKind::UnexpectedToken(token)),
        }
    }
}
impl PositionRange for AtomOrVariable {
    fn start_position(&self) -> Position {
        match *self {
            AtomOrVariable::Atom(ref t) => t.start_position(),
            AtomOrVariable::Variable(ref t) => t.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            AtomOrVariable::Atom(ref t) => t.end_position(),
            AtomOrVariable::Variable(ref t) => t.end_position(),
        }
    }
}
