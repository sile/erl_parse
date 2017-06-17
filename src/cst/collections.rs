// TODO: rename
use erl_tokenize::{Position, PositionRange, LexicalToken};
use erl_tokenize::tokens::{AtomToken, SymbolToken};
use erl_tokenize::values::Symbol;

use {Result, Parse, Preprocessor, Parser, IntoTokens};
use cst::building_blocks::{Sequence, MapField, RecordField, ConsCell};

#[derive(Debug, Clone)]
pub struct Tuple<T> {
    pub _open_brace: SymbolToken,
    pub elements: Option<Sequence<T>>,
    pub _close_brace: SymbolToken,
}
impl<T: Parse> Parse for Tuple<T> {
    fn try_parse<U>(reader: &mut Parser<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let _open_brace = track_try_some!(Parse::try_parse_expect(reader, &Symbol::OpenBrace));
        Ok(Some(Tuple {
            _open_brace,
            elements: track!(Parse::try_parse(reader))?,
            _close_brace: track!(Parse::parse_expect(reader, &Symbol::CloseBrace))?,
        }))
    }
}
impl<T> PositionRange for Tuple<T> {
    fn start_position(&self) -> Position {
        self._open_brace.start_position()
    }
    fn end_position(&self) -> Position {
        self._close_brace.end_position()
    }
}
impl<T: IntoTokens> IntoTokens for Tuple<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(
            self._open_brace
                .into_tokens()
                .chain(self.elements.into_tokens())
                .chain(self._close_brace.into_tokens()),
        )
    }
}

#[derive(Debug, Clone)]
pub struct List<T> {
    pub _open_square: SymbolToken,
    pub elements: Option<ConsCell<T>>,
    pub _close_square: SymbolToken,
}
impl<T: Parse + IntoTokens> Parse for List<T> {
    fn try_parse<U>(reader: &mut Parser<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let _open_square = track_try_some!(Parse::try_parse_expect(reader, &Symbol::OpenSquare));
        let elements = track!(Parse::try_parse(reader))?;
        let _close_square =
            if let Some(token) = track!(Parse::try_parse_expect(reader, &Symbol::CloseSquare))? {
                token
            } else {
                reader.unread_tokens(elements);
                reader.unread_tokens(_open_square);
                return Ok(None);
            };
        Ok(Some(List {
            _open_square,
            elements,
            _close_square,
        }))
    }
}
impl<T> PositionRange for List<T> {
    fn start_position(&self) -> Position {
        self._open_square.start_position()
    }
    fn end_position(&self) -> Position {
        self._close_square.end_position()
    }
}
impl<T: IntoTokens> IntoTokens for List<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(
            self._open_square
                .into_tokens()
                .chain(self.elements.into_tokens())
                .chain(self._close_square.into_tokens()),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Record<T> {
    pub _sharp: SymbolToken,
    pub name: AtomToken,
    pub _open_brace: SymbolToken,
    pub fields: Option<Sequence<RecordField<T>>>,
    pub _close_brace: SymbolToken,
}
impl<T: Parse> Parse for Record<T> {
    fn try_parse<U>(reader: &mut Parser<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let _sharp: SymbolToken = track_try_some!(Parse::try_parse_expect(reader, &Symbol::Sharp));
        let name: AtomToken = if let Some(token) = track!(Parse::try_parse(reader))? {
            token
        } else {
            reader.unread_token(_sharp.into());
            return Ok(None);
        };
        let _open_brace =
            if let Some(token) = track!(Parse::try_parse_expect(reader, &Symbol::OpenBrace))? {
                token
            } else {
                reader.unread_token(name.into());
                reader.unread_token(_sharp.into());
                return Ok(None);
            };
        Ok(Some(Record {
            _sharp,
            name,
            _open_brace,
            fields: track!(Parse::try_parse(reader))?,
            _close_brace: track!(Parse::parse_expect(reader, &Symbol::CloseBrace))?,
        }))
    }
}
impl<T> PositionRange for Record<T> {
    fn start_position(&self) -> Position {
        self._sharp.start_position()
    }
    fn end_position(&self) -> Position {
        self._close_brace.end_position()
    }
}
impl<T: IntoTokens> IntoTokens for Record<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(
            self._sharp
                .into_tokens()
                .chain(self.name.into_tokens())
                .chain(self._open_brace.into_tokens())
                .chain(self.fields.into_tokens())
                .chain(self._close_brace.into_tokens()),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Map<T> {
    pub _sharp: SymbolToken,
    pub _open_brace: SymbolToken,
    pub fields: Option<Sequence<MapField<T>>>,
    pub _close_brace: SymbolToken,
}
impl<T: Parse> Parse for Map<T> {
    fn try_parse<U>(reader: &mut Parser<U>) -> Result<Option<Self>>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let _sharp: SymbolToken = track_try_some!(Parse::try_parse_expect(reader, &Symbol::Sharp));
        let _open_brace =
            if let Some(token) = track!(Parse::try_parse_expect(reader, &Symbol::OpenBrace))? {
                token
            } else {
                reader.unread_token(_sharp.into());
                return Ok(None);
            };
        Ok(Some(Map {
            _sharp,
            _open_brace,
            fields: track!(Parse::try_parse(reader))?,
            _close_brace: track!(Parse::parse_expect(reader, &Symbol::CloseBrace))?,
        }))
    }
}
impl<T> PositionRange for Map<T> {
    fn start_position(&self) -> Position {
        self._sharp.start_position()
    }
    fn end_position(&self) -> Position {
        self._close_brace.end_position()
    }
}
impl<T: IntoTokens> IntoTokens for Map<T> {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(
            self._sharp
                .into_tokens()
                .chain(self._open_brace.into_tokens())
                .chain(self.fields.into_tokens())
                .chain(self._close_brace.into_tokens()),
        )
    }
}
