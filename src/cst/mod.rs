use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{SymbolToken, VariableToken};
use erl_tokenize::values::{Symbol, Keyword};

use {Result, Parser, ErrorKind};
use traits::{Parse, TokenRead};

pub use self::form::Form;
pub use self::literal::Literal;
pub use self::pattern::Pattern;
pub use self::ty::Type;

pub mod building_blocks;
pub mod clauses;
pub mod collections;
pub mod exprs;
pub mod forms;
pub mod guard_tests;
pub mod patterns;
pub mod types;

mod form;
mod literal;
mod pattern;
mod ty;

/// `Vec<Form>`
#[derive(Debug, Clone)]
pub struct ModuleDecl {
    pub forms: Vec<Form>,
}
impl Parse for ModuleDecl {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        let mut forms = Vec::new();
        while !track!(parser.eos())? {
            let form = track!(parser.parse())?;
            forms.push(form);
        }
        Ok(ModuleDecl { forms })
    }
}

#[derive(Debug)]
enum RightKind {
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
        T: TokenRead,
    {
        match parser.parse() {
            Ok(LexicalToken::Symbol(t)) => {
                match t.value() {
                    Symbol::OpenParen => RightKind::LocalCall,
                    Symbol::Colon => RightKind::RemoteCall,
                    Symbol::Sharp => {
                        if parser
                            .parse::<LexicalToken>()
                            .ok()
                            .and_then(|t| t.as_atom_token().map(|_| ()))
                            .is_some()
                        {
                            if parser
                                .parse::<LexicalToken>()
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
enum RightKind2 {
    BinaryOpCall,
    Match,
    Range,
    Union,
    None,
}
impl RightKind2 {
    fn guess<T>(parser: &mut Parser<T>) -> Self
    where
        T: TokenRead,
    {
        match parser.parse() {
            Ok(LexicalToken::Symbol(t)) => {
                match t.value() {
                    Symbol::VerticalBar => RightKind2::Union,
                    Symbol::DoubleDot => RightKind2::Range,
                    Symbol::Match => RightKind2::Match,
                    Symbol::Plus | Symbol::Hyphen | Symbol::Multiply | Symbol::Slash |
                    Symbol::PlusPlus | Symbol::MinusMinus | Symbol::Eq | Symbol::ExactEq |
                    Symbol::NotEq | Symbol::ExactNotEq | Symbol::Less | Symbol::LessEq |
                    Symbol::Greater | Symbol::GreaterEq | Symbol::Not => RightKind2::BinaryOpCall,
                    _ => RightKind2::None,
                }
            }
            Ok(LexicalToken::Keyword(t)) => {
                match t.value() {
                    Keyword::Div | Keyword::Rem | Keyword::Bor | Keyword::Bxor |
                    Keyword::Band | Keyword::Bsl | Keyword::Bsr | Keyword::Or | Keyword::Xor |
                    Keyword::Andalso | Keyword::Orelse => RightKind2::BinaryOpCall,
                    _ => RightKind2::None,
                }
            }
            _ => RightKind2::None,
        }
    }
}

#[derive(Debug)]
enum LeftKind {
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
    Annotated,
}
impl LeftKind {
    fn guess<T, U>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
        U: Parse,
    {
        Ok(match track!(parser.parse())? {
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
                        let token = track!(parser.parse::<LexicalToken>())?;
                        if token.as_atom_token().is_some() {
                            if parser
                                .parse::<LexicalToken>()
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
                        let token1 = track!(parser.parse::<LexicalToken>())?;
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
            LexicalToken::Variable(_) => {
                if parser.expect::<SymbolToken>(&Symbol::DoubleColon).is_ok() {
                    LeftKind::Annotated
                } else {
                    LeftKind::Variable
                }
            }
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
        T: TokenRead,
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
            _ => track_panic!(ErrorKind::InvalidInput, "unreachable"),            
        };
        Ok(expr)
    }
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        if let Ok(expr) = parser.transaction(|parser| parser.parse()) {
            return Ok(Expr::Match(expr));
        }

        let expr = track!(Expr::parse_non_left_recor(parser))?;
        let kind = parser.peek(|parser| Ok(RightKind::guess(parser))).expect(
            "Never fails",
        );
        let left = match kind {
            RightKind::LocalCall => Expr::LocalCall(track!(parser.parse_tail(expr))?),
            RightKind::RemoteCall => Expr::RemoteCall(track!(parser.parse_tail(expr))?),
            RightKind::MapUpdate => Expr::MapUpdate(track!(parser.parse_tail(expr))?),
            RightKind::RecordUpdate => Expr::RecordUpdate(track!(parser.parse_tail(expr))?), 
            RightKind::RecordFieldAccess => Expr::RecordFieldAccess(
                track!(parser.parse_tail(expr))?,
            ), 
            RightKind::None => expr,
        };

        let kind = parser.peek(|parser| Ok(RightKind2::guess(parser))).expect(
            "Never fails",
        );
        match kind {
            RightKind2::BinaryOpCall => Ok(Expr::BinaryOpCall(track!(parser.parse_tail(left))?)),
            RightKind2::None |
            RightKind2::Union => Ok(left),
            _ => track_panic!(ErrorKind::InvalidInput, "unreachable"),            
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
pub struct GuardSeq {
    pub guards: clauses::Clauses<Guard>,
}
impl Parse for GuardSeq {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
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
        T: TokenRead,
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
    Tuple(Box<guard_tests::Tuple>),
    Map(Box<guard_tests::Map>),
    Record(Box<guard_tests::Record>),
    RecordFieldIndex(Box<guard_tests::RecordFieldIndex>),
    RecordFieldAccess(Box<guard_tests::RecordFieldAccess>),
    List(Box<guard_tests::List>),
    Bits(Box<guard_tests::Bits>),
    Parenthesized(Box<guard_tests::Parenthesized>),
    LocalCall(Box<guard_tests::LocalCall>),
    RemoteCall(Box<guard_tests::RemoteCall>),
    UnaryOpCall(Box<guard_tests::UnaryOpCall>),
    BinaryOpCall(Box<guard_tests::BinaryOpCall>),
}
impl Parse for GuardTest {
    fn parse_non_left_recor<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        let kind = track!(parser.peek(
            |parser| LeftKind::guess::<T, GuardTest>(parser),
        ))?;
        let test = match kind {
            LeftKind::Literal => GuardTest::Literal(track!(parser.parse())?),
            LeftKind::Variable => GuardTest::Variable(track!(parser.parse())?),
            LeftKind::Tuple => GuardTest::Tuple(track!(parser.parse())?),
            LeftKind::Map => GuardTest::Map(track!(parser.parse())?),
            LeftKind::Record => GuardTest::Record(track!(parser.parse())?),
            LeftKind::RecordFieldIndex => GuardTest::RecordFieldIndex(track!(parser.parse())?),
            LeftKind::List => GuardTest::List(track!(parser.parse())?),            
            LeftKind::Bits => GuardTest::Bits(track!(parser.parse())?),
            LeftKind::UnaryOpCall => GuardTest::UnaryOpCall(track!(parser.parse())?),
            LeftKind::Parenthesized => GuardTest::Parenthesized(track!(parser.parse())?),
            _ => track_panic!(ErrorKind::InvalidInput, "kind={:?}", kind),
        };
        Ok(test)
    }
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        let test = track!(Self::parse_non_left_recor(parser))?;
        let kind = parser.peek(|parser| Ok(RightKind::guess(parser))).expect(
            "Never fails",
        );
        let left = match kind {
            RightKind::LocalCall => GuardTest::LocalCall(track!(parser.parse_tail(test))?),
            RightKind::RemoteCall => GuardTest::RemoteCall(track!(parser.parse_tail(test))?),
            RightKind::RecordFieldAccess => GuardTest::RecordFieldAccess(
                track!(parser.parse_tail(test))?,
            ), 
            RightKind::None => test,
            _ => track_panic!(ErrorKind::InvalidInput, "kind={:?}", kind),
        };

        let kind = parser.peek(|parser| Ok(RightKind2::guess(parser))).expect(
            "Never fails",
        );
        match kind {
            RightKind2::BinaryOpCall => Ok(
                GuardTest::BinaryOpCall(track!(parser.parse_tail(left))?),
            ),
            RightKind2::None |
            RightKind2::Union => Ok(left),
            _ => track_panic!(ErrorKind::InvalidInput, "kind={:?}", kind),
        }
    }
}
impl PositionRange for GuardTest {
    fn start_position(&self) -> Position {
        match *self {
            GuardTest::Literal(ref x) => x.start_position(),
            GuardTest::Variable(ref x) => x.start_position(),
            GuardTest::Tuple(ref x) => x.start_position(),
            GuardTest::Map(ref x) => x.start_position(),
            GuardTest::Record(ref x) => x.start_position(),
            GuardTest::RecordFieldIndex(ref x) => x.start_position(),
            GuardTest::RecordFieldAccess(ref x) => x.start_position(),
            GuardTest::List(ref x) => x.start_position(),
            GuardTest::Bits(ref x) => x.start_position(),
            GuardTest::Parenthesized(ref x) => x.start_position(),
            GuardTest::LocalCall(ref x) => x.start_position(),
            GuardTest::RemoteCall(ref x) => x.start_position(),
            GuardTest::UnaryOpCall(ref x) => x.start_position(),
            GuardTest::BinaryOpCall(ref x) => x.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            GuardTest::Literal(ref x) => x.end_position(),
            GuardTest::Variable(ref x) => x.end_position(),
            GuardTest::Tuple(ref x) => x.end_position(),
            GuardTest::Map(ref x) => x.end_position(),
            GuardTest::Record(ref x) => x.end_position(),
            GuardTest::RecordFieldIndex(ref x) => x.end_position(),
            GuardTest::RecordFieldAccess(ref x) => x.end_position(),            
            GuardTest::List(ref x) => x.end_position(),
            GuardTest::Bits(ref x) => x.end_position(),
            GuardTest::Parenthesized(ref x) => x.end_position(),
            GuardTest::LocalCall(ref x) => x.end_position(),
            GuardTest::RemoteCall(ref x) => x.end_position(),
            GuardTest::UnaryOpCall(ref x) => x.end_position(),
            GuardTest::BinaryOpCall(ref x) => x.end_position(),
        }
    }
}
