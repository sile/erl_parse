use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{KeywordToken, SymbolToken, VariableToken};
use erl_tokenize::values::{Keyword, Symbol};

use {Result, Parser, Preprocessor, Parse};
use cst::{Pattern, GuardSeq, Type};
use cst::exprs::Body;
use cst::building_blocks::{Args, AtomOrVariable};
use cst::types;

#[derive(Debug, Clone)]
pub struct ExceptionClass {
    pub class: AtomOrVariable,
    pub _colon: SymbolToken,
}
impl Parse for ExceptionClass {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(ExceptionClass {
            class: track!(parser.parse())?,
            _colon: track!(parser.expect(&Symbol::Colon))?,
        })
    }
}
impl PositionRange for ExceptionClass {
    fn start_position(&self) -> Position {
        self.class.start_position()
    }
    fn end_position(&self) -> Position {
        self._colon.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct CatchClause {
    pub class: Option<ExceptionClass>,
    pub pattern: Pattern,
    pub guard: Option<Guard>,
    pub _arrow: SymbolToken,
    pub body: Body,
}
impl Parse for CatchClause {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(CatchClause {
            class: track!(parser.parse())?,
            pattern: track!(parser.parse())?,
            guard: track!(parser.parse())?,
            _arrow: track!(parser.expect(&Symbol::RightArrow))?,
            body: track!(parser.parse())?,
        })
    }
}
impl PositionRange for CatchClause {
    fn start_position(&self) -> Position {
        self.class
            .as_ref()
            .map(|x| x.start_position())
            .unwrap_or_else(|| self.pattern.start_position())
    }
    fn end_position(&self) -> Position {
        self.body.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct SpecClause {
    pub args: Args<Type>,
    pub _arrow: SymbolToken,
    pub return_type: Type,
    pub constraints: Option<types::FunConstraints>,
}
impl Parse for SpecClause {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(SpecClause {
            args: track!(parser.parse())?,
            _arrow: track!(parser.expect(&Symbol::RightArrow))?,
            return_type: track!(parser.parse())?,
            constraints: track!(parser.parse())?,
        })
    }
}
impl PositionRange for SpecClause {
    fn start_position(&self) -> Position {
        self.args.start_position()
    }
    fn end_position(&self) -> Position {
        self.constraints
            .as_ref()
            .map(|t| t.end_position())
            .unwrap_or_else(|| self.return_type.end_position())
    }
}

#[derive(Debug, Clone)]
pub struct CaseClause {
    pub pattern: Pattern,
    pub guard: Option<Guard>,
    pub _arrow: SymbolToken,
    pub body: Body,
}
impl Parse for CaseClause {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(CaseClause {
            pattern: track!(parser.parse())?,
            guard: track!(parser.parse())?,
            _arrow: track!(parser.expect(&Symbol::RightArrow))?,
            body: track!(parser.parse())?,
        })
    }
}
impl PositionRange for CaseClause {
    fn start_position(&self) -> Position {
        self.pattern.start_position()
    }
    fn end_position(&self) -> Position {
        self.body.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct IfClause {
    pub guard: GuardSeq,
    pub _arrow: SymbolToken,
    pub body: Body,
}
impl Parse for IfClause {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(IfClause {
            guard: track!(parser.parse())?,
            _arrow: track!(parser.expect(&Symbol::RightArrow))?,
            body: track!(parser.parse())?,
        })
    }
}
impl PositionRange for IfClause {
    fn start_position(&self) -> Position {
        self.guard.start_position()
    }
    fn end_position(&self) -> Position {
        self.body.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct FunClause {
    pub patterns: Args<Pattern>,
    pub guard: Option<Guard>,
    pub _arrow: SymbolToken,
    pub body: Body,
}
impl Parse for FunClause {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(FunClause {
            patterns: track!(parser.parse())?,
            guard: track!(parser.parse())?,
            _arrow: track!(parser.expect(&Symbol::RightArrow))?,
            body: track!(parser.parse())?,
        })
    }
}
impl PositionRange for FunClause {
    fn start_position(&self) -> Position {
        self.patterns.start_position()
    }
    fn end_position(&self) -> Position {
        self.body.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct NamedFunClause {
    pub name: VariableToken,
    pub patterns: Args<Pattern>,
    pub guard: Option<Guard>,
    pub _arrow: SymbolToken,
    pub body: Body,
}
impl Parse for NamedFunClause {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(NamedFunClause {
            name: track!(parser.parse())?,
            patterns: track!(parser.parse())?,
            guard: track!(parser.parse())?,
            _arrow: track!(parser.expect(&Symbol::RightArrow))?,
            body: track!(parser.parse())?,
        })
    }
}
impl PositionRange for NamedFunClause {
    fn start_position(&self) -> Position {
        self.name.start_position()
    }
    fn end_position(&self) -> Position {
        self.body.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Guard {
    pub _when: KeywordToken,
    pub seq: GuardSeq,
}
impl Parse for Guard {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Guard {
            _when: track!(parser.expect(&Keyword::When))?,
            seq: track!(parser.parse())?,
        })
    }
}
impl PositionRange for Guard {
    fn start_position(&self) -> Position {
        self._when.start_position()
    }
    fn end_position(&self) -> Position {
        self.seq.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Clauses<T> {
    pub item: T,
    pub tail: Option<ClausesTail<T>>,
}
impl<T: Parse> Parse for Clauses<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Clauses {
            item: track!(parser.parse())?,
            tail: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for Clauses<T> {
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
pub struct ClausesTail<T> {
    pub _semicolon: SymbolToken,
    pub item: T,
    pub tail: Option<Box<ClausesTail<T>>>,
}
impl<T: Parse> Parse for ClausesTail<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(ClausesTail {
            _semicolon: track!(parser.expect(&Symbol::Semicolon))?,
            item: track!(parser.parse())?,
            tail: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for ClausesTail<T> {
    fn start_position(&self) -> Position {
        self._semicolon.start_position()
    }
    fn end_position(&self) -> Position {
        self.tail
            .as_ref()
            .map(|t| t.end_position())
            .unwrap_or_else(|| self.item.end_position())
    }
}
