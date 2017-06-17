// TODO: rename
use erl_tokenize::{Position, PositionRange, LexicalToken};
use erl_tokenize::tokens::{AtomToken, SymbolToken, IntegerToken};
use erl_tokenize::values::Symbol;

use {Result, Parse, Preprocessor, Parser};
use cst::building_blocks::{Sequence, MapField, RecordField, ConsCell, HyphenSeq};

#[derive(Debug, Clone)]
pub struct Tuple<T> {
    pub _open: SymbolToken,
    pub elements: Option<Sequence<T>>,
    pub _close: SymbolToken,
}
impl<T: Parse> Parse for Tuple<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Tuple {
            _open: track!(parser.expect(&Symbol::OpenBrace))?,
            elements: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseBrace))?,
        })
    }
}
impl<T> PositionRange for Tuple<T> {
    fn start_position(&self) -> Position {
        self._open.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Bits<T> {
    pub _open: SymbolToken,
    pub elements: Option<Sequence<BitsElem<T>>>,
    pub _close: SymbolToken,
}
impl<T: Parse> Parse for Bits<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Bits {
            _open: track!(parser.expect(&Symbol::DoubleLeftAngle))?,
            elements: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::DoubleRightAngle))?,
        })
    }
}
impl<T> PositionRange for Bits<T> {
    fn start_position(&self) -> Position {
        self._open.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct BitsElem<T> {
    pub element: T,
    pub size: Option<BitsElemSize<T>>,
    pub type_specs: Option<BitsElemSpecs>,
}
impl<T: Parse> Parse for BitsElem<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(BitsElem {
            element: track!(T::parse_non_left_recor(parser))?,
            size: track!(parser.parse())?,
            type_specs: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for BitsElem<T> {
    fn start_position(&self) -> Position {
        self.element.start_position()
    }
    fn end_position(&self) -> Position {
        self.type_specs
            .as_ref()
            .map(|t| t.end_position())
            .or_else(|| self.size.as_ref().map(|t| t.end_position()))
            .unwrap_or_else(|| self.element.end_position())
    }
}

#[derive(Debug, Clone)]
pub struct BitsElemSize<T> {
    pub _colon: SymbolToken,
    pub size: T,
}
impl<T: Parse> Parse for BitsElemSize<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(BitsElemSize {
            _colon: track!(parser.expect(&Symbol::Colon))?,
            size: track!(T::parse_non_left_recor(parser))?,
        })
    }
}
impl<T: PositionRange> PositionRange for BitsElemSize<T> {
    fn start_position(&self) -> Position {
        self._colon.start_position()
    }
    fn end_position(&self) -> Position {
        self.size.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct BitsElemSpecs {
    pub _slash: SymbolToken,
    pub specs: HyphenSeq<BitsElemSpec>,
}
impl Parse for BitsElemSpecs {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(BitsElemSpecs {
            _slash: track!(parser.expect(&Symbol::Slash))?,
            specs: track!(parser.parse())?,
        })
    }
}
impl PositionRange for BitsElemSpecs {
    fn start_position(&self) -> Position {
        self._slash.start_position()
    }
    fn end_position(&self) -> Position {
        self.specs.end_position()
    }
}

#[derive(Debug, Clone)]
pub enum BitsElemSpec {
    Type(AtomToken),
    Unit {
        _unit: AtomToken,
        _colon: SymbolToken,
        unit: IntegerToken,
    },
}
impl Parse for BitsElemSpec {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let atom: AtomToken = track!(parser.parse())?;
        if atom.value() == "unit" {
            Ok(BitsElemSpec::Unit {
                _unit: atom,
                _colon: track!(parser.expect(&Symbol::Colon))?,
                unit: track!(parser.parse())?,
            })
        } else {
            Ok(BitsElemSpec::Type(atom))
        }
    }
}
impl PositionRange for BitsElemSpec {
    fn start_position(&self) -> Position {
        match *self {
            BitsElemSpec::Type(ref t) => t.start_position(),
            BitsElemSpec::Unit { ref _unit, .. } => _unit.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            BitsElemSpec::Type(ref t) => t.end_position(),
            BitsElemSpec::Unit { ref unit, .. } => unit.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct List<T> {
    pub _open: SymbolToken,
    pub elements: Option<ConsCell<T>>,
    pub _close: SymbolToken,
}
impl<T: Parse> Parse for List<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(List {
            _open: track!(parser.expect(&Symbol::OpenSquare))?,
            elements: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseSquare))?,
        })
    }
}
impl<T> PositionRange for List<T> {
    fn start_position(&self) -> Position {
        self._open.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Record<T> {
    pub _sharp: SymbolToken,
    pub name: AtomToken,
    pub _open: SymbolToken,
    pub fields: Option<Sequence<RecordField<T>>>,
    pub _close: SymbolToken,
}
impl<T: Parse> Parse for Record<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Record {
            _sharp: track!(parser.expect(&Symbol::Sharp))?,
            name: track!(parser.parse())?,
            _open: track!(parser.expect(&Symbol::OpenBrace))?,
            fields: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseBrace))?,
        })
    }
}
impl<T> PositionRange for Record<T> {
    fn start_position(&self) -> Position {
        self._sharp.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct RecordFieldIndex {
    pub _sharp: SymbolToken,
    pub name: AtomToken,
    pub _dot: SymbolToken,
    pub field: AtomToken,
}
impl Parse for RecordFieldIndex {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(RecordFieldIndex {
            _sharp: track!(parser.expect(&Symbol::Sharp))?,
            name: track!(parser.parse())?,
            _dot: track!(parser.expect(&Symbol::Dot))?,
            field: track!(parser.parse())?,
        })
    }
}
impl PositionRange for RecordFieldIndex {
    fn start_position(&self) -> Position {
        self._sharp.start_position()
    }
    fn end_position(&self) -> Position {
        self.field.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Map<T> {
    pub _sharp: SymbolToken,
    pub _open: SymbolToken,
    pub fields: Option<Sequence<MapField<T>>>,
    pub _close: SymbolToken,
}
impl<T: Parse> Parse for Map<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Map {
            _sharp: track!(parser.expect(&Symbol::Sharp))?,
            _open: track!(parser.expect(&Symbol::OpenBrace))?,
            fields: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseBrace))?,
        })
    }
}
impl<T> PositionRange for Map<T> {
    fn start_position(&self) -> Position {
        self._sharp.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
    }
}
