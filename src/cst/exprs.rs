use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{KeywordToken, SymbolToken};
use erl_tokenize::values::{Keyword, Symbol};

use {Result, Parser, Preprocessor, Parse};
use cst::{Expr, Pattern};
use cst::building_blocks::{self, Sequence};
use cst::collections;

#[derive(Debug, Clone)]
pub struct ListComprehension {
    pub _open_square: SymbolToken,
    pub element: Expr,
    pub _bar: SymbolToken,
    pub qualifiers: Sequence<Qualifier>,
    pub _close_square: SymbolToken,
}
impl Parse for ListComprehension {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(ListComprehension {
            _open_square: track!(parser.expect(&Symbol::OpenSquare))?,
            element: track!(parser.parse())?,
            _bar: track!(parser.expect(&Symbol::DoubleVerticalBar))?,
            qualifiers: track!(parser.parse())?,
            _close_square: track!(parser.expect(&Symbol::CloseSquare))?,
        })
    }
}
impl PositionRange for ListComprehension {
    fn start_position(&self) -> Position {
        self._open_square.start_position()
    }
    fn end_position(&self) -> Position {
        self._close_square.end_position()
    }
}

#[derive(Debug, Clone)]
pub enum Qualifier {
    Generator(Generator),
    Filter(Expr),
}
impl Parse for Qualifier {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        if let Ok(generator) = parser.transaction(|parser| parser.parse()) {
            Ok(Qualifier::Generator(generator))
        } else {
            Ok(Qualifier::Filter(track!(parser.parse())?))
        }
    }
}
impl PositionRange for Qualifier {
    fn start_position(&self) -> Position {
        match *self {
            Qualifier::Generator(ref x) => x.start_position(),
            Qualifier::Filter(ref x) => x.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            Qualifier::Generator(ref x) => x.end_position(),
            Qualifier::Filter(ref x) => x.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Generator {
    pub pattern: Pattern,
    pub _arrow: SymbolToken,
    pub source: Expr,
}
impl Parse for Generator {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Generator {
            pattern: track!(parser.parse())?,
            _arrow: track!(parser.expect_any(
                &[&Symbol::LeftArrow, &Symbol::DoubleLeftArrow],
            ))?,
            source: track!(parser.parse())?,
        })
    }
}
impl PositionRange for Generator {
    fn start_position(&self) -> Position {
        self.pattern.start_position()
    }
    fn end_position(&self) -> Position {
        self.source.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Catch {
    pub _catch: KeywordToken,
    pub expr: Body,
}
impl Parse for Catch {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Catch {
            _catch: track!(parser.expect(&Keyword::Catch))?,
            expr: track!(parser.parse())?,
        })
    }
}
impl PositionRange for Catch {
    fn start_position(&self) -> Position {
        self._catch.start_position()
    }
    fn end_position(&self) -> Position {
        self.expr.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub _begin: KeywordToken,
    pub body: Body,
    pub _end: KeywordToken,
}
impl Parse for Block {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Block {
            _begin: track!(parser.expect(&Keyword::Begin))?,
            body: track!(parser.parse())?,
            _end: track!(parser.expect(&Keyword::End))?,
        })
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
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let exprs = track!(parser.parse())?;
        Ok(Body { exprs })
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
pub type Parenthesized = building_blocks::Parenthesized<Expr>;
pub type LocalCall = building_blocks::LocalCall<Expr>;
pub type RemoteCall = building_blocks::RemoteCall<Expr>;
