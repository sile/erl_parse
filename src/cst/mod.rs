use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{AtomToken, CharToken, FloatToken, IntegerToken, StringToken,
                           VariableToken, SymbolToken};
use erl_tokenize::values::{Symbol, Keyword};

use {Result, Parse, Preprocessor, Parser, ErrorKind, Error};

pub mod building_blocks;
pub mod clauses;
pub mod collections;
pub mod exprs;
pub mod forms;
pub mod guard_tests;
pub mod patterns;
pub mod types;

#[derive(Debug, Clone)]
pub struct ModuleDecl {
    pub forms: Vec<Form>,
}
impl Parse for ModuleDecl {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let mut forms = Vec::new();
        while !track!(parser.is_eos())? {
            let form = track!(parser.parse())?;
            forms.push(form);
        }
        Ok(ModuleDecl { forms })
    }
}

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
    Match,
    Range,
    Union,
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
    Annotated,
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
            _ => track_panic!(ErrorKind::InvalidInput, "unreachable"),            
        };
        Ok(expr)
    }
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        if let Ok(expr) = parser.transaction(|parser| parser.parse()) {
            return Ok(Expr::Match(expr));
        }

        let expr = track!(Expr::parse_non_left_recor(parser))?;
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
pub enum Pattern {
    Literal(Literal),
    Variable(VariableToken),
    Tuple(Box<patterns::Tuple>),
    Map(Box<patterns::Map>),
    Record(Box<patterns::Record>),
    RecordFieldIndex(Box<patterns::RecordFieldIndex>),
    List(Box<patterns::List>),
    Bits(Box<patterns::Bits>),
    Parenthesized(Box<patterns::Parenthesized>),
    UnaryOpCall(Box<patterns::UnaryOpCall>),
    BinaryOpCall(Box<patterns::BinaryOpCall>),
    Match(Box<patterns::Match>),
}
impl Parse for Pattern {
    fn parse_non_left_recor<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let kind = track!(parser.peek(|parser| LeftKind::guess::<T, Pattern>(parser)))?;
        let pattern = match kind {
            LeftKind::Literal => Pattern::Literal(track!(parser.parse())?),
            LeftKind::Variable => Pattern::Variable(track!(parser.parse())?),
            LeftKind::Tuple => Pattern::Tuple(track!(parser.parse())?),
            LeftKind::Map => Pattern::Map(track!(parser.parse())?),
            LeftKind::Record => Pattern::Record(track!(parser.parse())?),
            LeftKind::RecordFieldIndex => Pattern::RecordFieldIndex(track!(parser.parse())?),
            LeftKind::List => Pattern::List(track!(parser.parse())?),            
            LeftKind::Bits => Pattern::Bits(track!(parser.parse())?),
            LeftKind::UnaryOpCall => Pattern::UnaryOpCall(track!(parser.parse())?),
            LeftKind::Parenthesized => Pattern::Parenthesized(track!(parser.parse())?),
            _ => track_panic!(ErrorKind::InvalidInput, "kind={:?}", kind),
        };
        Ok(pattern)
    }
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let left = track!(Pattern::parse_non_left_recor(parser))?;

        let kind = parser.peek(|parser| Ok(RightKind2::guess(parser))).expect(
            "Never fails",
        );
        match kind {
            RightKind2::BinaryOpCall => Ok(Pattern::BinaryOpCall(
                track!(parser.parse_left_recur(left))?,
            )),
            RightKind2::Match => Ok(Pattern::Match(track!(parser.parse_left_recur(left))?)),
            RightKind2::None |
            RightKind2::Union => Ok(left),
            _ => track_panic!(ErrorKind::InvalidInput, "kind={:?}", kind),            
        }
    }
}
impl PositionRange for Pattern {
    fn start_position(&self) -> Position {
        match *self {
            Pattern::Literal(ref x) => x.start_position(),
            Pattern::Variable(ref x) => x.start_position(),
            Pattern::Tuple(ref x) => x.start_position(),
            Pattern::Map(ref x) => x.start_position(),
            Pattern::Record(ref x) => x.start_position(),
            Pattern::RecordFieldIndex(ref x) => x.start_position(),
            Pattern::List(ref x) => x.start_position(),
            Pattern::Bits(ref x) => x.start_position(),
            Pattern::Parenthesized(ref x) => x.start_position(),
            Pattern::UnaryOpCall(ref x) => x.start_position(),
            Pattern::BinaryOpCall(ref x) => x.start_position(),
            Pattern::Match(ref x) => x.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            Pattern::Literal(ref x) => x.end_position(),
            Pattern::Variable(ref x) => x.end_position(),
            Pattern::Tuple(ref x) => x.end_position(),
            Pattern::Map(ref x) => x.end_position(),
            Pattern::Record(ref x) => x.end_position(),
            Pattern::RecordFieldIndex(ref x) => x.end_position(),
            Pattern::List(ref x) => x.end_position(),
            Pattern::Bits(ref x) => x.end_position(),
            Pattern::Parenthesized(ref x) => x.end_position(),
            Pattern::UnaryOpCall(ref x) => x.end_position(),
            Pattern::BinaryOpCall(ref x) => x.end_position(),
            Pattern::Match(ref x) => x.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Atom(AtomToken),
    Char(CharToken),
    Float(FloatToken),
    Integer(IntegerToken),

    // TODO
    // String(StringToken),
    String(Vec<StringToken>),
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
            LexicalToken::String(t) => {
                //Ok(Literal::String(t)),
                let mut s = vec![t];
                while let Ok(t) = parser.transaction(|p| p.parse()) {
                    s.push(t);
                }
                Ok(Literal::String(s))
            }
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
            Literal::String(ref x) => x[0].start_position(),//x.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            Literal::Atom(ref x) => x.end_position(),
            Literal::Char(ref x) => x.end_position(),
            Literal::Float(ref x) => x.end_position(),
            Literal::Integer(ref x) => x.end_position(),
            Literal::String(ref x) => x.last().unwrap().end_position(),//x.end_position(),
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
    Tuple(Box<guard_tests::Tuple>),
    Map(Box<guard_tests::Map>),
    Record(Box<guard_tests::Record>),
    RecordFieldIndex(Box<guard_tests::RecordFieldIndex>),
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
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
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
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let test = track!(Self::parse_non_left_recor(parser))?;
        let kind = parser.peek(|parser| Ok(RightKind::guess(parser))).expect(
            "Never fails",
        );
        let left = match kind {
            RightKind::LocalCall => GuardTest::LocalCall(track!(parser.parse_left_recur(test))?),
            RightKind::RemoteCall => GuardTest::RemoteCall(track!(parser.parse_left_recur(test))?),
            RightKind::None => test,
            _ => track_panic!(ErrorKind::InvalidInput, "kind={:?}", kind),
        };

        let kind = parser.peek(|parser| Ok(RightKind2::guess(parser))).expect(
            "Never fails",
        );
        match kind {
            RightKind2::BinaryOpCall => Ok(GuardTest::BinaryOpCall(
                track!(parser.parse_left_recur(left))?,
            )),
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

#[derive(Debug, Clone)]
pub enum Type {
    Literal(Literal),
    Variable(VariableToken),
    Annotated(Box<types::Annotated>),
    Tuple(Box<types::Tuple>),
    Map(Box<types::Map>),
    Record(Box<types::Record>),
    List(Box<types::List>),
    Bits(Box<types::Bits>),
    Parenthesized(Box<types::Parenthesized>),
    LocalCall(Box<types::LocalCall>),
    RemoteCall(Box<types::RemoteCall>),
    UnaryOpCall(Box<types::UnaryOpCall>),
    BinaryOpCall(Box<types::BinaryOpCall>),
    AnyArgFun(Box<types::AnyArgFun>),
    Fun(Box<types::Fun>),
    Range(Box<types::Range>),
    Union(Box<types::Union>),
}
impl Parse for Type {
    fn parse_non_left_recor<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let kind = track!(parser.peek(|parser| LeftKind::guess::<T, Type>(parser)))?;
        let ty = match kind {
            LeftKind::Literal => Type::Literal(track!(parser.parse())?),
            LeftKind::Variable => Type::Variable(track!(parser.parse())?),
            LeftKind::Annotated => Type::Annotated(track!(parser.parse())?),
            LeftKind::List => Type::List(track!(parser.parse())?),
            LeftKind::Bits => Type::Bits(track!(parser.parse())?),            
            LeftKind::Tuple => Type::Tuple(track!(parser.parse())?),
            LeftKind::Map => Type::Map(track!(parser.parse())?),
            LeftKind::Record => Type::Record(track!(parser.parse())?),
            LeftKind::UnaryOpCall => Type::UnaryOpCall(track!(parser.parse())?),
            LeftKind::Parenthesized => Type::Parenthesized(track!(parser.parse())?),
            LeftKind::AnonymousFun => {
                if let Ok(t) = parser.transaction(|parser| parser.parse()) {
                    Type::AnyArgFun(t)
                } else {
                    Type::Fun(track!(parser.parse())?)
                }
            }
            _ => track_panic!(ErrorKind::InvalidInput, "kind={:?}", kind),
        };
        Ok(ty)
    }
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let ty = track!(Type::parse_non_left_recor(parser))?;
        let kind = parser.peek(|parser| Ok(RightKind::guess(parser))).expect(
            "Never fails",
        );
        let left = match kind {
            RightKind::LocalCall => Type::LocalCall(track!(parser.parse_left_recur(ty))?),
            RightKind::RemoteCall => Type::RemoteCall(track!(parser.parse_left_recur(ty))?),
            RightKind::None => ty,
            _ => track_panic!(ErrorKind::InvalidInput, "kind={:?}", kind),
        };

        let kind = parser.peek(|parser| Ok(RightKind2::guess(parser))).expect(
            "Never fails",
        );
        match kind {
            RightKind2::BinaryOpCall => Ok(
                Type::BinaryOpCall(track!(parser.parse_left_recur(left))?),
            ),
            RightKind2::Union => Ok(Type::Union(track!(parser.parse_left_recur(left))?)),
            RightKind2::Range => Ok(Type::Range(track!(parser.parse_left_recur(left))?)),
            RightKind2::None => Ok(left),
            _ => track_panic!(ErrorKind::InvalidInput, "kind={:?}", kind),            
        }
    }
}
impl PositionRange for Type {
    fn start_position(&self) -> Position {
        match *self {
            Type::Literal(ref x) => x.start_position(),
            Type::Variable(ref x) => x.start_position(),
            Type::Annotated(ref x) => x.start_position(),
            Type::List(ref x) => x.start_position(),
            Type::Bits(ref x) => x.start_position(),            
            Type::Tuple(ref x) => x.start_position(),
            Type::Map(ref x) => x.start_position(),
            Type::Record(ref x) => x.start_position(),
            Type::AnyArgFun(ref x) => x.start_position(),
            Type::Fun(ref x) => x.start_position(),
            Type::Parenthesized(ref x) => x.start_position(),
            Type::LocalCall(ref x) => x.start_position(),
            Type::RemoteCall(ref x) => x.start_position(),
            Type::UnaryOpCall(ref x) => x.start_position(),
            Type::BinaryOpCall(ref x) => x.start_position(),
            Type::Range(ref x) => x.start_position(),
            Type::Union(ref x) => x.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            Type::Literal(ref x) => x.end_position(),
            Type::Variable(ref x) => x.end_position(),
            Type::Annotated(ref x) => x.end_position(),
            Type::List(ref x) => x.end_position(),
            Type::Bits(ref x) => x.end_position(),
            Type::Tuple(ref x) => x.end_position(),
            Type::Map(ref x) => x.end_position(),
            Type::Record(ref x) => x.end_position(),
            Type::AnyArgFun(ref x) => x.end_position(),
            Type::Fun(ref x) => x.end_position(),
            Type::Parenthesized(ref x) => x.end_position(),
            Type::LocalCall(ref x) => x.end_position(),
            Type::RemoteCall(ref x) => x.end_position(),
            Type::UnaryOpCall(ref x) => x.end_position(),
            Type::BinaryOpCall(ref x) => x.end_position(),
            Type::Range(ref x) => x.end_position(),
            Type::Union(ref x) => x.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FormKind {
    ModuleAttr,
    ExportAttr,
    ExportTypeAttr,
    ImportAttr,
    FileAttr,
    WildAttr,
    FunSpec,
    CallbackSpec,
    FunDecl,
    RecordDecl,
    TypeDecl,
}

#[derive(Debug, Clone)]
pub enum Form {
    ModuleAttr(forms::ModuleAttr),
    ExportAttr(forms::ExportAttr),
    ExportTypeAttr(forms::ExportTypeAttr),
    ImportAttr(forms::ImportAttr),
    FileAttr(forms::FileAttr),
    WildAttr(forms::WildAttr),
    FunSpec(forms::FunSpec),
    CallbackSpec(forms::CallbackSpec),
    FunDecl(forms::FunDecl),
    RecordDecl(forms::RecordDecl),
    TypeDecl(forms::TypeDecl),
}
impl Parse for Form {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let kind = parser.peek(|parser| {
            let token = track!(parser.read_token())?;
            Ok(match token {
                LexicalToken::Symbol(ref t) if t.value() == Symbol::Hyphen => {
                    let token = track!(parser.read_token())?;
                    let token = track!(
                        token
                            .into_atom_token()
                            .map_err(ErrorKind::UnexpectedToken)
                            .map_err(Error::from)
                    )?;
                    match token.value() {
                        "module" => FormKind::ModuleAttr,
                        "export" => FormKind::ExportAttr,
                        "export_type" => FormKind::ExportTypeAttr,
                        "import" => FormKind::ImportAttr,
                        "file" => FormKind::FileAttr,
                        "spec" => FormKind::FunSpec,
                        "callback" => FormKind::CallbackSpec,
                        "record" => FormKind::RecordDecl,
                        "type" | "opaque" => FormKind::TypeDecl,
                        _ => FormKind::WildAttr,
                    }
                }
                LexicalToken::Atom(_) => FormKind::FunDecl,
                _ => track_panic!(ErrorKind::UnexpectedToken(token)),
            })
        });
        Ok(match track!(kind)? {
            FormKind::ModuleAttr => Form::ModuleAttr(track!(parser.parse())?),
            FormKind::ExportAttr => Form::ExportAttr(track!(parser.parse())?),
            FormKind::ExportTypeAttr => Form::ExportTypeAttr(track!(parser.parse())?),
            FormKind::ImportAttr => Form::ImportAttr(track!(parser.parse())?),
            FormKind::FileAttr => Form::FileAttr(track!(parser.parse())?),
            FormKind::WildAttr => Form::WildAttr(track!(parser.parse())?),
            FormKind::FunSpec => Form::FunSpec(track!(parser.parse())?),
            FormKind::CallbackSpec => Form::CallbackSpec(track!(parser.parse())?),
            FormKind::FunDecl => Form::FunDecl(track!(parser.parse())?),
            FormKind::RecordDecl => Form::RecordDecl(track!(parser.parse())?),
            FormKind::TypeDecl => Form::TypeDecl(track!(parser.parse())?),
        })
    }
}
impl PositionRange for Form {
    fn start_position(&self) -> Position {
        match *self {
            Form::ModuleAttr(ref t) => t.start_position(),
            Form::ExportAttr(ref t) => t.start_position(),
            Form::ExportTypeAttr(ref t) => t.start_position(),
            Form::ImportAttr(ref t) => t.start_position(),
            Form::FileAttr(ref t) => t.start_position(),
            Form::WildAttr(ref t) => t.start_position(),
            Form::FunSpec(ref t) => t.start_position(),
            Form::CallbackSpec(ref t) => t.start_position(),
            Form::FunDecl(ref t) => t.start_position(),
            Form::RecordDecl(ref t) => t.start_position(),
            Form::TypeDecl(ref t) => t.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            Form::ModuleAttr(ref t) => t.end_position(),
            Form::ExportAttr(ref t) => t.end_position(),
            Form::ExportTypeAttr(ref t) => t.end_position(),
            Form::ImportAttr(ref t) => t.end_position(),
            Form::FileAttr(ref t) => t.end_position(),
            Form::WildAttr(ref t) => t.end_position(),
            Form::FunSpec(ref t) => t.end_position(),
            Form::CallbackSpec(ref t) => t.end_position(),
            Form::FunDecl(ref t) => t.end_position(),
            Form::RecordDecl(ref t) => t.end_position(),
            Form::TypeDecl(ref t) => t.end_position(),
        }
    }
}
