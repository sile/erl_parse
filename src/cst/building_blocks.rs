use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{AtomToken, SymbolToken, VariableToken};
use erl_tokenize::values::Symbol;

use {Result, Parse, Preprocessor, IntoTokens, Parser, ErrorKind};

#[derive(Debug, Clone)]
pub struct FunCall<T> {
    pub module: Option<ModulePrefix<T>>,
    pub fun_name: T,
    pub args: Args<T>,
}
impl<T: Parse + IntoTokens> Parse for FunCall<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(FunCall {
            module: track!(parser.parse())?,
            fun_name: track!(parser.parse())?,
            args: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for FunCall<T> {
    fn start_position(&self) -> Position {
        self.module
            .as_ref()
            .map(|x| x.start_position())
            .unwrap_or_else(|| self.fun_name.start_position())
    }
    fn end_position(&self) -> Position {
        self.args.end_position()
    }
}
impl<T: IntoTokens> IntoTokens for FunCall<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(
            self.module
                .into_tokens()
                .chain(self.fun_name.into_tokens())
                .chain(self.args.into_tokens()),
        )
    }
}

#[derive(Debug, Clone)]
pub struct ModulePrefix<T> {
    pub name: T,
    pub _colon: SymbolToken,
}
impl<T: Parse + IntoTokens> Parse for ModulePrefix<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(ModulePrefix {
            name: track!(parser.parse())?,
            _colon: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for ModulePrefix<T> {
    fn start_position(&self) -> Position {
        self.name.start_position()
    }
    fn end_position(&self) -> Position {
        self._colon.end_position()
    }
}
impl<T: IntoTokens> IntoTokens for ModulePrefix<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(self.name.into_tokens().chain(self._colon.into_tokens()))
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
        Ok(Args {
            _open_paren: track!(parser.expect(&Symbol::OpenParen))?,
            args: track!(parser.parse())?,
            _close_paren: track!(parser.expect(&Symbol::CloseParen))?,
        })
    }
}
impl<T> PositionRange for Args<T> {
    fn start_position(&self) -> Position {
        self._open_paren.start_position()
    }
    fn end_position(&self) -> Position {
        self._close_paren.start_position()
    }
}
impl<T: IntoTokens> IntoTokens for Args<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(
            self._open_paren
                .into_tokens()
                .chain(self.args.into_tokens())
                .chain(self._close_paren.into_tokens()),
        )
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
impl<T: IntoTokens> IntoTokens for Parenthesized<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(
            self._open_paren
                .into_tokens()
                .chain(self.item.into_tokens())
                .chain(self._close_paren.into_tokens()),
        )
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
impl<T: IntoTokens> IntoTokens for Sequence<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(self.item.into_tokens().chain(self.tail.into_tokens()))
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
impl<T: IntoTokens> IntoTokens for SequenceTail<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(
            self._comma
                .into_tokens()
                .chain(self.item.into_tokens())
                .chain(self.tail.into_tokens()),
        )
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
impl<T: IntoTokens> IntoTokens for ConsCell<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(self.item.into_tokens().chain(self.tail.into_tokens()))
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
impl<T: IntoTokens> IntoTokens for ConsCellTail<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        match self {
            ConsCellTail::Proper { _comma, item, tail } => {
                Box::new(_comma.into_tokens().chain(item.into_tokens()).chain(
                    tail.into_tokens(),
                ))
            }
            ConsCellTail::Improper { _bar, item } => {
                Box::new(_bar.into_tokens().chain(item.into_tokens()))
            }
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
impl<T: IntoTokens> IntoTokens for MapField<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(
            self.key
                .into_tokens()
                .chain(self._relation.into_tokens())
                .chain(self.value.into_tokens()),
        )
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
impl<T: IntoTokens> IntoTokens for RecordField<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(
            self.key
                .into_tokens()
                .chain(self._bind.into_tokens())
                .chain(self.value.into_tokens()),
        )
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
impl IntoTokens for AtomOrVariable {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        match self {
            AtomOrVariable::Atom(x) => Box::new(x.into_tokens()),
            AtomOrVariable::Variable(x) => Box::new(x.into_tokens()),
        }
    }
}
