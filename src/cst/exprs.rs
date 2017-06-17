use std::iter;
use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{KeywordToken, SymbolToken};
use erl_tokenize::values::{Keyword, Symbol};

use {Result, Parser, Preprocessor, Parse, IntoTokens, ErrorKind};
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
impl IntoTokens for ListComprehension {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(
            self._open_square
                .into_tokens()
                .chain(self.element.into_tokens())
                .chain(self._bar.into_tokens())
                .chain(self.qualifiers.into_tokens())
                .chain(self._close_square.into_tokens()),
        )
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
        if let Ok(expr) = parser.transaction(|parser| {
            let expr = track!(parser.parse())?;
            if track!(parser.peek_token())?
                .and_then(|t| t.as_symbol_token().map(|t| t.value() == Symbol::Comma))
                .unwrap_or(false)
            {
                Ok(expr)
            } else {
                track_panic!(ErrorKind::InvalidInput);
            }
        })
        {
            Ok(Qualifier::Filter(expr))
        } else {
            Ok(Qualifier::Generator(track!(parser.parse())?))
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
impl IntoTokens for Qualifier {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        match self {
            Qualifier::Generator(x) => x.into_tokens(),
            Qualifier::Filter(x) => x.into_tokens(),
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
impl IntoTokens for Generator {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(
            self.pattern
                .into_tokens()
                .chain(self._arrow.into_tokens())
                .chain(self.source.into_tokens()),
        )
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
impl IntoTokens for Catch {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(iter::once(self._catch.into()).chain(
            self.expr.into_tokens(),
        ))
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
impl IntoTokens for Block {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(
            self._begin
                .into_tokens()
                .chain(self.body.into_tokens())
                .chain(self._end.into_tokens()),
        )
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
impl IntoTokens for Body {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        Box::new(self.exprs.into_tokens())
    }
}

pub type Tuple = collections::Tuple<Expr>;
pub type Map = collections::Map<Expr>;
pub type Record = collections::Record<Expr>;
pub type List = collections::List<Expr>;
pub type Parenthesized = building_blocks::Parenthesized<Expr>;
pub type FunCall = building_blocks::FunCall<Expr>;
