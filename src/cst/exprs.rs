use std::iter;
use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{KeywordToken, SymbolToken};
use erl_tokenize::values::{Keyword, Symbol};

use {Result, TokenReader, Preprocessor, Parse, IntoTokens};
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
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let _open_square =
            track_try_some!(SymbolToken::try_parse_expect(reader, &Symbol::OpenSquare));
        let element = if let Some(x) = track!(Parse::try_parse(reader))? {
            x
        } else {
            reader.unread_token(_open_square.into());
            return Ok(None);
        };
        let _bar =
            if let Some(x) = track!(Parse::try_parse_expect(reader, &Symbol::DoubleVerticalBar))? {
                x
            } else {
                reader.unread_tokens(element);
                reader.unread_token(_open_square.into());
                return Ok(None);
            };
        Ok(Some(ListComprehension {
            _open_square,
            element,
            _bar,
            qualifiers: track!(Parse::parse(reader))?,
            _close_square: track!(Parse::parse_expect(reader, &Symbol::CloseSquare))?,
        }))
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
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        if let Some(x) = track!(Parse::try_parse(reader))? {
            Ok(Some(Qualifier::Generator(x)))
        } else if let Some(x) = track!(Parse::try_parse(reader))? {
            Ok(Some(Qualifier::Filter(x)))
        } else {
            Ok(None)
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
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let pattern = track_try_some!(Parse::try_parse(reader));
        let _arrow = if let Some(x) = track!(
            Parse::try_parse_expect(reader, &Symbol::LeftArrow)
        )?
        {
            x
        } else if let Some(x) = track!(Parse::try_parse_expect(reader, &Symbol::DoubleLeftArrow))? {
            x
        } else {
            reader.unread_tokens(pattern);
            return Ok(None);
        };
        Ok(Some(Generator {
            pattern,
            _arrow,
            source: track!(Parse::parse(reader))?,
        }))
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
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let _catch = track_try_some!(Parse::try_parse_expect(reader, &Keyword::Catch));
        Ok(Some(Catch {
            _catch,
            expr: track!(Parse::parse(reader))?,
        }))
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
