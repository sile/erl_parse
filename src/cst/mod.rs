use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{AtomToken, CharToken, FloatToken, IntegerToken, StringToken,
                           VariableToken, SymbolToken};
use erl_tokenize::values::{Symbol, Keyword};

use {Result, Parse, Preprocessor, Parser, ErrorKind, TryInto};

pub mod building_blocks;
pub mod clauses;
pub mod collections;
pub mod exprs;

#[derive(Debug)]
pub enum RightKind {
    LocalCall,
    RemoteCall,
    MapUpdate,
    RecordUpdate,
    RecordFieldAccess,
    None,
}
impl RightKind {
    fn guess<T>(parser: &mut Parser<T>) -> Self
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        match parser.read_token() {
            Ok(LexicalToken::Symbol(t)) => {
                match t.value() {
                    Symbol::OpenParen => RightKind::LocalCall,
                    Symbol::Colon => RightKind::RemoteCall,
                    Symbol::Sharp => {
                        if parser
                            .read_token()
                            .ok()
                            .and_then(|t| t.as_atom_token().map(|_| ()))
                            .is_some()
                        {
                            if parser
                                .read_token()
                                .ok()
                                .and_then(|t| {
                                    t.as_symbol_token().map(|t| t.value() == Symbol::OpenBrace)
                                })
                                .unwrap_or(false)
                            {
                                RightKind::RecordUpdate
                            } else {
                                RightKind::RecordFieldAccess
                            }
                        } else {
                            RightKind::MapUpdate
                        }
                    }
                    _ => RightKind::None,
                }
            }
            _ => RightKind::None,
        }
    }
}

#[derive(Debug)]
pub enum RightKind2 {
    BinaryOpCall,
    None,
}
impl RightKind2 {
    fn guess<T>(parser: &mut Parser<T>) -> Self
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        match parser.read_token() {
            Ok(LexicalToken::Symbol(t)) => {
                match t.value() {
                    Symbol::Plus | Symbol::Hyphen | Symbol::Multiply | Symbol::Slash |
                    Symbol::PlusPlus | Symbol::MinusMinus | Symbol::Eq | Symbol::ExactEq |
                    Symbol::NotEq | Symbol::ExactNotEq | Symbol::Less | Symbol::LessEq |
                    Symbol::Greater | Symbol::GreaterEq | Symbol::Not => RightKind2::BinaryOpCall,
                    _ => RightKind2::None,
                }
            }
            Ok(LexicalToken::Keyword(t)) => {
                match t.value() {
                    Keyword::Div | Keyword::Rem | Keyword::Bor | Keyword::Bxor | Keyword::Bsl |
                    Keyword::Bsr | Keyword::Or | Keyword::Xor | Keyword::Andalso |
                    Keyword::Orelse => RightKind2::BinaryOpCall,
                    _ => RightKind2::None,
                }
            }
            _ => RightKind2::None,
        }
    }
}

#[derive(Debug)]
pub enum LeftKind {
    Literal,
    Variable,
    Tuple,
    Map,
    Record,
    RecordFieldIndex,
    List,
    ListComprehension,
    Bits,
    BitsComprehension,
    LocalFun,
    RemoteFun,
    AnonymousFun,
    NamedFun,
    UnaryOpCall,
    Parenthesized,
    Block,
    Catch,
    If,
    Case,
    Receive,
    Try,
}
impl LeftKind {
    fn guess<T, U>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
        U: Parse,
    {
        Ok(match track!(parser.read_token())? {
            LexicalToken::Symbol(t) => {
                match t.value() {
                    Symbol::OpenBrace => LeftKind::Tuple,
                    Symbol::DoubleLeftAngle => {
                        let maybe_comprehension = parser.parse::<U>().is_ok() &&
                            parser
                                .expect::<SymbolToken>(&Symbol::DoubleVerticalBar)
                                .is_ok();
                        if maybe_comprehension {
                            LeftKind::BitsComprehension
                        } else {
                            LeftKind::Bits
                        }
                    }
                    Symbol::OpenParen => LeftKind::Parenthesized,
                    Symbol::OpenSquare => {
                        let maybe_comprehension = parser.parse::<U>().is_ok() &&
                            parser
                                .expect::<SymbolToken>(&Symbol::DoubleVerticalBar)
                                .is_ok();
                        if maybe_comprehension {
                            LeftKind::ListComprehension
                        } else {
                            LeftKind::List
                        }
                    }
                    Symbol::Sharp => {
                        if track!(parser.read_token())?.as_atom_token().is_some() {
                            if parser
                                .read_token()
                                .ok()
                                .and_then(|t| {
                                    t.as_symbol_token().map(|t| t.value() == Symbol::OpenBrace)
                                })
                                .unwrap_or(false)
                            {
                                LeftKind::Record
                            } else {
                                LeftKind::RecordFieldIndex
                            }
                        } else {
                            LeftKind::Map
                        }
                    }
                    Symbol::Plus | Symbol::Hyphen => LeftKind::UnaryOpCall,
                    _ => track_panic!(ErrorKind::UnexpectedToken(t.into())),
                }
            }
            LexicalToken::Keyword(t) => {
                match t.value() {
                    Keyword::Begin => LeftKind::Block,
                    Keyword::Catch => LeftKind::Catch,
                    Keyword::If => LeftKind::If,
                    Keyword::Case => LeftKind::Case,
                    Keyword::Receive => LeftKind::Receive,
                    Keyword::Try => LeftKind::Try,
                    Keyword::Fun => {
                        let token1 = track!(parser.read_token())?;
                        if token1.as_symbol_token().map_or(false, |t| {
                            t.value() == Symbol::OpenParen
                        })
                        {
                            LeftKind::AnonymousFun
                        } else {
                            let token2: SymbolToken = track!(parser.parse())?;
                            match token2.value() {
                                Symbol::Slash => LeftKind::LocalFun,
                                Symbol::Colon => LeftKind::RemoteFun,
                                Symbol::OpenParen => LeftKind::NamedFun,
                                _ => track_panic!(ErrorKind::UnexpectedToken(token2.into())),
                            }
                        }
                    }
                    Keyword::Bnot | Keyword::Not => LeftKind::UnaryOpCall,
                    _ => track_panic!(ErrorKind::UnexpectedToken(t.into())),
                }
            }
            LexicalToken::Variable(_) => LeftKind::Variable,
            _ => LeftKind::Literal,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Variable(VariableToken),
    Tuple(Box<exprs::Tuple>),
    Map(Box<exprs::Map>),
    MapUpdate(Box<exprs::MapUpdate>),
    Record(Box<exprs::Record>),
    RecordUpdate(Box<exprs::RecordUpdate>),
    RecordFieldIndex(Box<exprs::RecordFieldIndex>),
    RecordFieldAccess(Box<exprs::RecordFieldAccess>),
    List(Box<exprs::List>),
    ListComprehension(Box<exprs::ListComprehension>),
    Bits(Box<exprs::Bits>),
    BitsComprehension(Box<exprs::BitsComprehension>),
    LocalFun(Box<exprs::LocalFun>),
    RemoteFun(Box<exprs::RemoteFun>),
    AnonymousFun(Box<exprs::AnonymousFun>),
    NamedFun(Box<exprs::NamedFun>),
    Parenthesized(Box<exprs::Parenthesized>),
    LocalCall(Box<exprs::LocalCall>),
    RemoteCall(Box<exprs::RemoteCall>),
    UnaryOpCall(Box<exprs::UnaryOpCall>),
    BinaryOpCall(Box<exprs::BinaryOpCall>),
    Match(Box<exprs::Match>),
    Block(Box<exprs::Block>),
    Catch(Box<exprs::Catch>),
    If(Box<exprs::If>),
    Case(Box<exprs::Case>),
    Receive(Box<exprs::Receive>),
    Try(Box<exprs::Try>),
}
impl Parse for Expr {
    fn parse_non_left_recor<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let kind = track!(parser.peek(|parser| LeftKind::guess::<T, Expr>(parser)))?;
        let expr = match kind {
            LeftKind::Literal => Expr::Literal(track!(parser.parse())?),
            LeftKind::Variable => Expr::Variable(track!(parser.parse())?),
            LeftKind::Tuple => Expr::Tuple(track!(parser.parse())?),
            LeftKind::Map => Expr::Map(track!(parser.parse())?),
            LeftKind::Record => Expr::Record(track!(parser.parse())?),
            LeftKind::RecordFieldIndex => Expr::RecordFieldIndex(track!(parser.parse())?),
            LeftKind::List => Expr::List(track!(parser.parse())?),            
            LeftKind::ListComprehension => Expr::ListComprehension(track!(parser.parse())?),
            LeftKind::Bits => Expr::Bits(track!(parser.parse())?),
            LeftKind::BitsComprehension => Expr::BitsComprehension(track!(parser.parse())?),
            LeftKind::LocalFun => Expr::LocalFun(track!(parser.parse())?),
            LeftKind::RemoteFun => Expr::RemoteFun(track!(parser.parse())?),
            LeftKind::AnonymousFun => Expr::AnonymousFun(track!(parser.parse())?),
            LeftKind::NamedFun => Expr::NamedFun(track!(parser.parse())?),
            LeftKind::UnaryOpCall => Expr::UnaryOpCall(track!(parser.parse())?),
            LeftKind::Parenthesized => Expr::Parenthesized(track!(parser.parse())?),
            LeftKind::Block => Expr::Block(track!(parser.parse())?),
            LeftKind::Catch => Expr::Catch(track!(parser.parse())?),
            LeftKind::If => Expr::If(track!(parser.parse())?),
            LeftKind::Case => Expr::Case(track!(parser.parse())?),
            LeftKind::Receive => Expr::Receive(track!(parser.parse())?),
            LeftKind::Try => Expr::Try(track!(parser.parse())?),
        };
        Ok(expr)
    }
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let expr = {
            if let Ok(expr) = parser.transaction(|parser| parser.parse()) {
                Expr::Match(expr)
            } else {
                track!(Expr::parse_non_left_recor(parser))?
            }
        };
        let kind = parser.peek(|parser| Ok(RightKind::guess(parser))).expect(
            "Never fails",
        );
        let left = match kind {
            RightKind::LocalCall => Expr::LocalCall(track!(parser.parse_left_recur(expr))?),
            RightKind::RemoteCall => Expr::RemoteCall(track!(parser.parse_left_recur(expr))?),
            RightKind::MapUpdate => Expr::MapUpdate(track!(parser.parse_left_recur(expr))?),
            RightKind::RecordUpdate => Expr::RecordUpdate(track!(parser.parse_left_recur(expr))?), 
            RightKind::RecordFieldAccess => Expr::RecordFieldAccess(
                track!(parser.parse_left_recur(expr))?,
            ), 
            RightKind::None => expr,
        };

        let kind = parser.peek(|parser| Ok(RightKind2::guess(parser))).expect(
            "Never fails",
        );
        match kind {
            RightKind2::BinaryOpCall => Ok(
                Expr::BinaryOpCall(track!(parser.parse_left_recur(left))?),
            ),
            RightKind2::None => Ok(left),
        }
    }
}
impl TryInto<exprs::LocalCall> for Expr {
    fn try_into(self) -> Result<exprs::LocalCall> {
        if let Expr::LocalCall(x) = self {
            Ok(*x)
        } else {
            track_panic!(ErrorKind::InvalidInput, "Not a LocalCall: {:?}", self)
        }
    }
}
impl PositionRange for Expr {
    fn start_position(&self) -> Position {
        match *self {
            Expr::Literal(ref x) => x.start_position(),
            Expr::Variable(ref x) => x.start_position(),
            Expr::Tuple(ref x) => x.start_position(),
            Expr::Map(ref x) => x.start_position(),
            Expr::MapUpdate(ref x) => x.start_position(),
            Expr::Record(ref x) => x.start_position(),
            Expr::RecordUpdate(ref x) => x.start_position(),
            Expr::RecordFieldIndex(ref x) => x.start_position(),
            Expr::RecordFieldAccess(ref x) => x.start_position(),
            Expr::List(ref x) => x.start_position(),
            Expr::ListComprehension(ref x) => x.start_position(),
            Expr::Bits(ref x) => x.start_position(),
            Expr::BitsComprehension(ref x) => x.start_position(),
            Expr::Parenthesized(ref x) => x.start_position(),
            Expr::LocalFun(ref x) => x.start_position(),
            Expr::RemoteFun(ref x) => x.start_position(),
            Expr::AnonymousFun(ref x) => x.start_position(),
            Expr::NamedFun(ref x) => x.start_position(),
            Expr::LocalCall(ref x) => x.start_position(),
            Expr::RemoteCall(ref x) => x.start_position(),
            Expr::UnaryOpCall(ref x) => x.start_position(),
            Expr::BinaryOpCall(ref x) => x.start_position(),
            Expr::Match(ref x) => x.start_position(),
            Expr::Block(ref x) => x.start_position(),
            Expr::Catch(ref x) => x.start_position(),
            Expr::If(ref x) => x.start_position(),
            Expr::Case(ref x) => x.start_position(),
            Expr::Receive(ref x) => x.start_position(),
            Expr::Try(ref x) => x.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            Expr::Literal(ref x) => x.end_position(),
            Expr::Variable(ref x) => x.end_position(),
            Expr::Tuple(ref x) => x.end_position(),
            Expr::Map(ref x) => x.end_position(),
            Expr::MapUpdate(ref x) => x.end_position(),
            Expr::Record(ref x) => x.end_position(),
            Expr::RecordUpdate(ref x) => x.end_position(),
            Expr::RecordFieldIndex(ref x) => x.end_position(),
            Expr::RecordFieldAccess(ref x) => x.end_position(),
            Expr::List(ref x) => x.end_position(),
            Expr::ListComprehension(ref x) => x.end_position(),
            Expr::Bits(ref x) => x.end_position(),
            Expr::BitsComprehension(ref x) => x.end_position(),
            Expr::Parenthesized(ref x) => x.end_position(),
            Expr::LocalFun(ref x) => x.end_position(),
            Expr::RemoteFun(ref x) => x.end_position(),
            Expr::AnonymousFun(ref x) => x.end_position(),
            Expr::NamedFun(ref x) => x.end_position(),
            Expr::LocalCall(ref x) => x.end_position(),
            Expr::RemoteCall(ref x) => x.end_position(),
            Expr::UnaryOpCall(ref x) => x.end_position(),
            Expr::BinaryOpCall(ref x) => x.end_position(),
            Expr::Match(ref x) => x.end_position(),
            Expr::Block(ref x) => x.end_position(),
            Expr::Catch(ref x) => x.end_position(),
            Expr::If(ref x) => x.end_position(),
            Expr::Case(ref x) => x.end_position(),
            Expr::Receive(ref x) => x.end_position(),
            Expr::Try(ref x) => x.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Literal),
    Variable(VariableToken),
}
impl Parse for Pattern {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let kind = track!(parser.peek(|parser| LeftKind::guess::<T, Pattern>(parser)))?;
        let pattern = match kind {
            LeftKind::Literal => Pattern::Literal(track!(parser.parse())?),
            LeftKind::Variable => Pattern::Variable(track!(parser.parse())?),
            _ => track_panic!(ErrorKind::UnexpectedToken(track!(parser.read_token())?)),
        };
        Ok(pattern)
    }
}
impl PositionRange for Pattern {
    fn start_position(&self) -> Position {
        match *self {
            Pattern::Literal(ref x) => x.start_position(),
            Pattern::Variable(ref x) => x.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            Pattern::Literal(ref x) => x.end_position(),
            Pattern::Variable(ref x) => x.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Atom(AtomToken),
    Char(CharToken),
    Float(FloatToken),
    Integer(IntegerToken),
    String(StringToken),
}
impl Parse for Literal {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        match track!(parser.read_token())? {
            LexicalToken::Atom(t) => Ok(Literal::Atom(t)),
            LexicalToken::Char(t) => Ok(Literal::Char(t)),
            LexicalToken::Float(t) => Ok(Literal::Float(t)),
            LexicalToken::Integer(t) => Ok(Literal::Integer(t)),
            LexicalToken::String(t) => Ok(Literal::String(t)),
            token => track_panic!(ErrorKind::UnexpectedToken(token)),
        }
    }
}
impl PositionRange for Literal {
    fn start_position(&self) -> Position {
        match *self {
            Literal::Atom(ref x) => x.start_position(),
            Literal::Char(ref x) => x.start_position(),
            Literal::Float(ref x) => x.start_position(),
            Literal::Integer(ref x) => x.start_position(),
            Literal::String(ref x) => x.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            Literal::Atom(ref x) => x.end_position(),
            Literal::Char(ref x) => x.end_position(),
            Literal::Float(ref x) => x.end_position(),
            Literal::Integer(ref x) => x.end_position(),
            Literal::String(ref x) => x.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GuardSeq {
    pub guards: clauses::Clauses<Guard>,
}
impl Parse for GuardSeq {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(GuardSeq { guards: track!(parser.parse())? })
    }
}
impl PositionRange for GuardSeq {
    fn start_position(&self) -> Position {
        self.guards.start_position()
    }
    fn end_position(&self) -> Position {
        self.guards.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Guard {
    pub tests: building_blocks::Sequence<GuardTest>,
}
impl Parse for Guard {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Guard { tests: track!(parser.parse())? })
    }
}
impl PositionRange for Guard {
    fn start_position(&self) -> Position {
        self.tests.start_position()
    }
    fn end_position(&self) -> Position {
        self.tests.end_position()
    }
}

#[derive(Debug, Clone)]
pub enum GuardTest {
    Literal(Literal),
    Variable(VariableToken),
}
impl Parse for GuardTest {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let kind = track!(parser.peek(
            |parser| LeftKind::guess::<T, GuardTest>(parser),
        ))?;
        let pattern = match kind {
            LeftKind::Literal => GuardTest::Literal(track!(parser.parse())?),
            LeftKind::Variable => GuardTest::Variable(track!(parser.parse())?),
            _ => track_panic!(ErrorKind::UnexpectedToken(track!(parser.read_token())?)),
        };
        Ok(pattern)
    }
}
impl PositionRange for GuardTest {
    fn start_position(&self) -> Position {
        match *self {
            GuardTest::Literal(ref x) => x.start_position(),
            GuardTest::Variable(ref x) => x.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            GuardTest::Literal(ref x) => x.end_position(),
            GuardTest::Variable(ref x) => x.end_position(),
        }
    }
}
