use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::KeywordToken;
use erl_tokenize::values::Keyword;

use {Result, TokenReader, Preprocessor, Parse};
use cst::Expr;
use cst::building_blocks::Sequence;
use cst::collections;

#[derive(Debug, Clone)]
pub struct Block {
    pub _begin: KeywordToken,
    pub body: Body,
    pub _end: KeywordToken,
}
impl Parse for Block {
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let _begin = track_try_some!(Parse::try_parse_expect(reader, &Keyword::Begin));
        Ok(Some(Block {
            _begin,
            body: track!(Parse::parse(reader))?,
            _end: track!(Parse::parse(reader))?,
        }))
    }
}
impl PositionRange for Block {
    fn start_position(&self) -> Position {
        self._begin.start_position()
    }
    fn end_position(&self) -> Position {
        self._end.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Body {
    pub exprs: Sequence<Expr>,
}
impl Parse for Body {
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let exprs = track_try_some!(Parse::try_parse(reader));
        Ok(Some(Body { exprs }))
    }
}
impl PositionRange for Body {
    fn start_position(&self) -> Position {
        self.exprs.start_position()
    }
    fn end_position(&self) -> Position {
        self.exprs.end_position()
    }
}

pub type Tuple = collections::Tuple<Expr>;
pub type Map = collections::Map<Expr>;
pub type Record = collections::Record<Expr>;
pub type List = collections::List<Expr>;
