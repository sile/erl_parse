use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{AtomToken, SymbolToken, VariableToken};
use erl_tokenize::values::Symbol;

use {Result, Parse, Preprocessor, TokenReader, IntoTokens};

#[derive(Debug, Clone)]
pub struct FunCall<T> {
    pub module: Option<ModulePrefix<T>>,
    pub fun_name: T,
    pub args: Args<T>,
}
impl<T: Parse + IntoTokens> Parse for FunCall<T> {
    fn try_parse<U>(reader: &mut TokenReader<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let module = track!(Parse::try_parse(reader))?;
        let fun_name = if module.is_some() {
            track!(Parse::parse(reader))?
        } else {
            track_try_some!(Parse::try_parse(reader))
        };
        Ok(Some(FunCall {
            module,
            fun_name,
            args: track!(Parse::parse(reader))?,
        }))
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
    fn try_parse<U>(reader: &mut TokenReader<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let name = track_try_some!(Parse::try_parse(reader));
        if let Some(_colon) = track!(Parse::try_parse_expect(reader, &Symbol::Colon))? {
            Ok(Some(ModulePrefix { name, _colon }))
        } else {
            reader.unread_tokens(name);
            Ok(None)
        }
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
    fn try_parse<U>(reader: &mut TokenReader<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Some(Args {
            _open_paren: track!(Parse::parse_expect(reader, &Symbol::OpenParen))?,
            args: track!(Parse::try_parse(reader))?,
            _close_paren: track!(Parse::parse_expect(reader, &Symbol::CloseParen))?,
        }))
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
    fn try_parse<U>(reader: &mut TokenReader<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let _open_paren = track_try_some!(Parse::try_parse_expect(reader, &Symbol::OpenParen));
        Ok(Some(Parenthesized {
            _open_paren,
            item: track!(Parse::parse(reader))?,
            _close_paren: track!(Parse::parse_expect(reader, &Symbol::CloseParen))?,
        }))
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
impl IntoTokens for AtomOrVariable {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        match self {
            AtomOrVariable::Atom(x) => Box::new(x.into_tokens()),
            AtomOrVariable::Variable(x) => Box::new(x.into_tokens()),
        }
    }
}
