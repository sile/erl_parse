use erl_tokenize::{Position, PositionRange};
use erl_tokenize::tokens::SymbolToken;
use erl_tokenize::values::Symbol;

use {Result, Parser};
use traits::{Parse, TokenRead};
use super::Type;

/// `Type` `Option<NonEmpty>`
#[derive(Debug, Clone)]
pub struct ListElement {
    pub element_type: Type,
    pub non_empty: Option<NonEmpty>,
}
impl Parse for ListElement {
    fn parse<T: TokenRead>(parser: &mut Parser<T>) -> Result<Self> {
        Ok(ListElement {
            element_type: track!(parser.parse())?,
            non_empty: track!(parser.parse())?,
        })
    }
}
impl PositionRange for ListElement {
    fn start_position(&self) -> Position {
        self.element_type.start_position()
    }
    fn end_position(&self) -> Position {
        self.non_empty
            .as_ref()
            .map(|t| t.end_position())
            .unwrap_or_else(|| self.element_type.end_position())
    }
}

/// `,` `...`
#[derive(Debug, Clone)]
pub struct NonEmpty {
    pub _comma: SymbolToken,
    pub _triple_dot: SymbolToken,
}
impl Parse for NonEmpty {
    fn parse<T: TokenRead>(parser: &mut Parser<T>) -> Result<Self> {
        Ok(NonEmpty {
            _comma: track!(parser.expect(&Symbol::Comma))?,
            _triple_dot: track!(parser.expect(&Symbol::TripleDot))?,
        })
    }
}
impl PositionRange for NonEmpty {
    fn start_position(&self) -> Position {
        self._comma.start_position()
    }
    fn end_position(&self) -> Position {
        self._triple_dot.end_position()
    }
}
