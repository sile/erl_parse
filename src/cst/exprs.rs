use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{KeywordToken, SymbolToken, AtomToken, IntegerToken};
use erl_tokenize::values::{Keyword, Symbol};

use {Result, Parser};
use cst::{Expr, Pattern};
use cst::building_blocks::{self, Sequence, AtomOrVariable, IntegerOrVariable};
use cst::clauses::{Clauses, FunClause, NamedFunClause, IfClause, CaseClause, CatchClause};
use cst::collections;
use traits::{Parse, ParseTail, Preprocessor};

#[derive(Debug, Clone)]
pub struct MapUpdate {
    pub map: Expr,
    pub update: Map,
}
impl ParseTail for MapUpdate {
    type Head = Expr;
    fn parse_tail<T>(parser: &mut Parser<T>, head: Self::Head) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(MapUpdate {
            map: head,
            update: track!(parser.parse())?,
        })
    }
}
impl PositionRange for MapUpdate {
    fn start_position(&self) -> Position {
        self.map.start_position()
    }
    fn end_position(&self) -> Position {
        self.update.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct RecordUpdate {
    pub record: Expr,
    pub update: Record,
}
impl ParseTail for RecordUpdate {
    type Head = Expr;
    fn parse_tail<T>(parser: &mut Parser<T>, head: Self::Head) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(RecordUpdate {
            record: head,
            update: track!(parser.parse())?,
        })
    }
}
impl PositionRange for RecordUpdate {
    fn start_position(&self) -> Position {
        self.record.start_position()
    }
    fn end_position(&self) -> Position {
        self.update.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct RecordFieldAccess {
    pub record: Expr,
    pub index: RecordFieldIndex,
}
impl ParseTail for RecordFieldAccess {
    type Head = Expr;
    fn parse_tail<T>(parser: &mut Parser<T>, head: Self::Head) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(RecordFieldAccess {
            record: head,
            index: track!(parser.parse())?,
        })
    }
}
impl PositionRange for RecordFieldAccess {
    fn start_position(&self) -> Position {
        self.record.start_position()
    }
    fn end_position(&self) -> Position {
        self.index.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Try {
    pub _try: KeywordToken,
    pub body: Body,
    pub branch: Option<TryOf>,
    pub catch: Option<TryCatch>,
    pub after: Option<TryAfter>,
    pub _end: KeywordToken,
}
impl Parse for Try {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Try {
            _try: track!(parser.expect(&Keyword::Try))?,
            body: track!(parser.parse())?,
            branch: track!(parser.parse())?,
            catch: track!(parser.parse())?,
            after: track!(parser.parse())?,
            _end: track!(parser.expect(&Keyword::End))?,
        })
    }
}
impl PositionRange for Try {
    fn start_position(&self) -> Position {
        self._try.start_position()
    }
    fn end_position(&self) -> Position {
        self._end.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct TryOf {
    pub _of: KeywordToken,
    pub clauses: Clauses<CaseClause>,
}
impl Parse for TryOf {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(TryOf {
            _of: track!(parser.expect(&Keyword::Of))?,
            clauses: track!(parser.parse())?,
        })
    }
}
impl PositionRange for TryOf {
    fn start_position(&self) -> Position {
        self._of.start_position()
    }
    fn end_position(&self) -> Position {
        self.clauses.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct TryCatch {
    pub _catch: KeywordToken,
    pub clauses: Clauses<CatchClause>,
}
impl Parse for TryCatch {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(TryCatch {
            _catch: track!(parser.expect(&Keyword::Catch))?,
            clauses: track!(parser.parse())?,
        })
    }
}
impl PositionRange for TryCatch {
    fn start_position(&self) -> Position {
        self._catch.start_position()
    }
    fn end_position(&self) -> Position {
        self.clauses.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct TryAfter {
    pub _after: KeywordToken,
    pub body: Body,
}
impl Parse for TryAfter {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(TryAfter {
            _after: track!(parser.expect(&Keyword::After))?,
            body: track!(parser.parse())?,
        })
    }
}
impl PositionRange for TryAfter {
    fn start_position(&self) -> Position {
        self._after.start_position()
    }
    fn end_position(&self) -> Position {
        self.body.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Receive {
    pub _receive: KeywordToken,
    pub clauses: Clauses<CaseClause>,
    pub timeout: Option<Timeout>,
    pub _end: KeywordToken,
}
impl Parse for Receive {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Receive {
            _receive: track!(parser.expect(&Keyword::Receive))?,
            clauses: track!(parser.parse())?,
            timeout: track!(parser.parse())?,
            _end: track!(parser.expect(&Keyword::End))?,
        })
    }
}
impl PositionRange for Receive {
    fn start_position(&self) -> Position {
        self._receive.start_position()
    }
    fn end_position(&self) -> Position {
        self._end.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Timeout {
    pub _after: KeywordToken,
    pub duration: Expr,
    pub _arrow: SymbolToken,
    pub body: Body,
}
impl Parse for Timeout {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Timeout {
            _after: track!(parser.expect(&Keyword::After))?,
            duration: track!(parser.parse())?,
            _arrow: track!(parser.expect(&Symbol::RightArrow))?,
            body: track!(parser.parse())?,
        })
    }
}
impl PositionRange for Timeout {
    fn start_position(&self) -> Position {
        self._after.start_position()
    }
    fn end_position(&self) -> Position {
        self.body.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct If {
    pub _if: KeywordToken,
    pub clauses: Clauses<IfClause>,
    pub _end: KeywordToken,
}
impl Parse for If {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(If {
            _if: track!(parser.expect(&Keyword::If))?,
            clauses: track!(parser.parse())?,
            _end: track!(parser.expect(&Keyword::End))?,
        })
    }
}
impl PositionRange for If {
    fn start_position(&self) -> Position {
        self._if.start_position()
    }
    fn end_position(&self) -> Position {
        self._end.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Case {
    pub _case: KeywordToken,
    pub expr: Expr,
    pub _of: KeywordToken,
    pub clauses: Clauses<CaseClause>,
    pub _end: KeywordToken,
}
impl Parse for Case {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Case {
            _case: track!(parser.expect(&Keyword::Case))?,
            expr: track!(parser.parse())?,
            _of: track!(parser.expect(&Keyword::Of))?,
            clauses: track!(parser.parse())?,
            _end: track!(parser.expect(&Keyword::End))?,
        })
    }
}
impl PositionRange for Case {
    fn start_position(&self) -> Position {
        self._case.start_position()
    }
    fn end_position(&self) -> Position {
        self._end.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct LocalFun {
    pub _fun: KeywordToken,
    pub fun_name: AtomToken,
    pub _slash: SymbolToken,
    pub arity: IntegerToken,
}
impl Parse for LocalFun {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(LocalFun {
            _fun: track!(parser.expect(&Keyword::Fun))?,
            fun_name: track!(parser.parse())?,
            _slash: track!(parser.expect(&Symbol::Slash))?,
            arity: track!(parser.parse())?,
        })
    }
}
impl PositionRange for LocalFun {
    fn start_position(&self) -> Position {
        self._fun.start_position()
    }
    fn end_position(&self) -> Position {
        self.arity.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct RemoteFun {
    pub _fun: KeywordToken,
    pub module_name: AtomOrVariable,
    pub _colon: SymbolToken,
    pub fun_name: AtomOrVariable,
    pub _slash: SymbolToken,
    pub arity: IntegerOrVariable,
}
impl Parse for RemoteFun {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(RemoteFun {
            _fun: track!(parser.expect(&Keyword::Fun))?,
            module_name: track!(parser.parse())?,
            _colon: track!(parser.expect(&Symbol::Colon))?,
            fun_name: track!(parser.parse())?,
            _slash: track!(parser.expect(&Symbol::Slash))?,
            arity: track!(parser.parse())?,
        })
    }
}
impl PositionRange for RemoteFun {
    fn start_position(&self) -> Position {
        self._fun.start_position()
    }
    fn end_position(&self) -> Position {
        self.arity.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct AnonymousFun {
    pub _fun: KeywordToken,
    pub clauses: Clauses<FunClause>,
    pub _end: KeywordToken,
}
impl Parse for AnonymousFun {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(AnonymousFun {
            _fun: track!(parser.expect(&Keyword::Fun))?,
            clauses: track!(parser.parse())?,
            _end: track!(parser.expect(&Keyword::End))?,
        })
    }
}
impl PositionRange for AnonymousFun {
    fn start_position(&self) -> Position {
        self._fun.start_position()
    }
    fn end_position(&self) -> Position {
        self._end.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct NamedFun {
    pub _fun: KeywordToken,
    pub clauses: Clauses<NamedFunClause>,
    pub _end: KeywordToken,
}
impl Parse for NamedFun {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(NamedFun {
            _fun: track!(parser.expect(&Keyword::Fun))?,
            clauses: track!(parser.parse())?,
            _end: track!(parser.expect(&Keyword::End))?,
        })
    }
}
impl PositionRange for NamedFun {
    fn start_position(&self) -> Position {
        self._fun.start_position()
    }
    fn end_position(&self) -> Position {
        self._end.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct ListComprehension {
    pub _open: SymbolToken,
    pub element: Expr,
    pub _bar: SymbolToken,
    pub qualifiers: Sequence<Qualifier>,
    pub _close: SymbolToken,
}
impl Parse for ListComprehension {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(ListComprehension {
            _open: track!(parser.expect(&Symbol::OpenSquare))?,
            element: track!(parser.parse())?,
            _bar: track!(parser.expect(&Symbol::DoubleVerticalBar))?,
            qualifiers: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseSquare))?,
        })
    }
}
impl PositionRange for ListComprehension {
    fn start_position(&self) -> Position {
        self._open.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct BitsComprehension {
    pub _open: SymbolToken,
    pub element: Expr,
    pub _bar: SymbolToken,
    pub qualifiers: Sequence<Qualifier>,
    pub _close: SymbolToken,
}
impl Parse for BitsComprehension {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(BitsComprehension {
            _open: track!(parser.expect(&Symbol::DoubleLeftAngle))?,
            element: track!(parser.parse())?,
            _bar: track!(parser.expect(&Symbol::DoubleVerticalBar))?,
            qualifiers: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::DoubleRightAngle))?,
        })
    }
}
impl PositionRange for BitsComprehension {
    fn start_position(&self) -> Position {
        self._open.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
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
pub type RecordFieldIndex = collections::RecordFieldIndex;
pub type List = collections::List<Expr>;
pub type Bits = collections::Bits<Expr>;
pub type Parenthesized = building_blocks::Parenthesized<Expr>;
pub type LocalCall = building_blocks::LocalCall<Expr>;
pub type RemoteCall = building_blocks::RemoteCall<Expr>;
pub type UnaryOpCall = building_blocks::UnaryOpCall<Expr>;
pub type BinaryOpCall = building_blocks::BinaryOpCall<Expr>;
pub type Match = building_blocks::Match<Expr>;
