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
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Tuple {
            _open_brace: track!(parser.expect(&Symbol::OpenBrace))?,
            elements: track!(parser.parse())?,
            _close_brace: track!(parser.expect(&Symbol::CloseBrace))?,
        })
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
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(List {
            _open_square: track!(parser.expect(&Symbol::OpenSquare))?,
            elements: track!(parser.parse())?,
            _close_square: track!(parser.expect(&Symbol::CloseSquare))?,
        })
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
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Record {
            _sharp: track!(parser.expect(&Symbol::Sharp))?,
            name: track!(parser.parse())?,
            _open_brace: track!(parser.expect(&Symbol::OpenBrace))?,
            fields: track!(parser.parse())?,
            _close_brace: track!(parser.expect(&Symbol::CloseBrace))?,
        })
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
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Map {
            _sharp: track!(parser.expect(&Symbol::Sharp))?,
            _open_brace: track!(parser.expect(&Symbol::OpenBrace))?,
            fields: track!(parser.parse())?,
            _close_brace: track!(parser.expect(&Symbol::CloseBrace))?,
        })
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
