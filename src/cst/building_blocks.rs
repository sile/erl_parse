use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{AtomToken, SymbolToken, VariableToken};
use erl_tokenize::values::Symbol;

use {Result, Parse, Preprocessor, TokenReader};

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
    fn try_parse<U>(reader: &mut TokenReader<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let item = track_try_some!(Parse::try_parse(reader));
        Ok(Some(Sequence {
            item,
            tail: track!(Parse::try_parse(reader))?,
        }))
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
    fn try_parse<U>(reader: &mut TokenReader<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let _comma = track_try_some!(Parse::try_parse_expect(reader, &Symbol::Comma));
        Ok(Some(SequenceTail {
            _comma,
            item: track!(Parse::parse(reader))?,
            tail: track!(Parse::try_parse(reader))?,
        }))
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
    fn try_parse<U>(reader: &mut TokenReader<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let item = track_try_some!(Parse::try_parse(reader));
        Ok(Some(ConsCell {
            item,
            tail: track!(Parse::try_parse(reader))?,
        }))
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
    fn try_parse<U>(reader: &mut TokenReader<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let cell = if let Some(_comma) = track!(Parse::try_parse_expect(reader, &Symbol::Comma))? {
            ConsCellTail::Proper {
                _comma,
                item: track!(Parse::parse(reader))?,
                tail: track!(Parse::try_parse(reader))?,
            }
        } else {
            ConsCellTail::Improper {
                _bar: track_try_some!(Parse::try_parse_expect(reader, &Symbol::VerticalBar)),
                item: track!(Parse::parse(reader))?,
            }
        };
        Ok(Some(cell))
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
    fn try_parse<U>(reader: &mut TokenReader<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let key = track_try_some!(Parse::try_parse(reader));

        let maybe_assoc_field = track!(Parse::try_parse_expect(reader, &Symbol::DoubleRightArrow))?;
        let _relation = if let Some(token) = maybe_assoc_field {
            token
        } else {
            track!(Parse::parse_expect(reader, &Symbol::MapMatch))?
        };

        Ok(Some(MapField {
            key,
            _relation,
            value: track!(Parse::parse(reader))?,
        }))
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
    fn try_parse<U>(reader: &mut TokenReader<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let key = track_try_some!(Parse::try_parse(reader));
        Ok(Some(RecordField {
            key,
            _bind: track!(Parse::parse(reader))?,
            value: track!(Parse::parse(reader))?,
        }))
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
    fn try_parse<U>(reader: &mut TokenReader<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        if let Some(token) = track!(Parse::try_parse(reader))? {
            Ok(Some(AtomOrVariable::Atom(token)))
        } else if let Some(token) = track!(Parse::try_parse(reader))? {
            Ok(Some(AtomOrVariable::Variable(token)))
        } else {
            Ok(None)
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
