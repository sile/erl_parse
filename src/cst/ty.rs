use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{AtomToken, VariableToken};
use erl_tokenize::values::{Symbol, Keyword};
use trackable::error::ErrorKindExt;

use {Result, Parser, ErrorKind};
use traits::{Parse, TokenRead};
use super::Literal;
use super::building_blocks::{UnaryOp, BinaryOp};
use super::types;

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
    Fun(Box<types::Fun>),
    Range(Box<types::Range>),
    Union(Box<types::Union>),
}
impl Parse for Type {
    fn parse_non_left_recor<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        let kind = track!(parser.peek(|parser| HeadKind::guess(parser)))?;
        let ty = match kind {
            HeadKind::Literal => Type::Literal(track!(parser.parse())?),
            HeadKind::Variable => Type::Variable(track!(parser.parse())?),
            HeadKind::Annotated => Type::Annotated(track!(parser.parse())?),
            HeadKind::List => Type::List(track!(parser.parse())?),
            HeadKind::Bits => Type::Bits(track!(parser.parse())?),            
            HeadKind::Tuple => Type::Tuple(track!(parser.parse())?),
            HeadKind::Map => Type::Map(track!(parser.parse())?),
            HeadKind::Record => Type::Record(track!(parser.parse())?),
            HeadKind::LocalCall => Type::LocalCall(track!(parser.parse())?),
            HeadKind::RemoteCall => Type::RemoteCall(track!(parser.parse())?),
            HeadKind::UnaryOpCall => Type::UnaryOpCall(track!(parser.parse())?),
            HeadKind::Parenthesized => Type::Parenthesized(track!(parser.parse())?),
            HeadKind::Fun => Type::Fun(track!(parser.parse())?),
        };
        Ok(ty)
    }
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        let head = track!(Type::parse_non_left_recor(parser))?;
        let tail_kind = track!(parser.peek(|parser| TailKind::guess(parser)))?;
        match tail_kind {
            TailKind::BinaryOpCall => Ok(Type::BinaryOpCall(track!(parser.parse_tail(head))?)),
            TailKind::Union => Ok(Type::Union(track!(parser.parse_tail(head))?)),
            TailKind::Range => Ok(Type::Range(track!(parser.parse_tail(head))?)),
            TailKind::None => Ok(head),
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

#[derive(Debug)]
enum HeadKind {
    Literal,
    Variable,
    Annotated,
    Tuple,
    Map,
    Record,
    List,
    Bits,
    Fun,
    LocalCall,
    RemoteCall,
    UnaryOpCall,
    Parenthesized,
}
impl HeadKind {
    fn guess<T: TokenRead>(parser: &mut Parser<T>) -> Result<Self> {
        Ok(match track!(parser.parse())? {
            LexicalToken::Symbol(t) => {
                match t.value() {
                    Symbol::OpenBrace => HeadKind::Tuple,
                    Symbol::DoubleLeftAngle => HeadKind::Bits,
                    Symbol::OpenParen => HeadKind::Parenthesized,
                    Symbol::OpenSquare => HeadKind::List,
                    Symbol::Sharp => {
                        if parser.parse::<AtomToken>().is_ok() {
                            HeadKind::Record
                        } else {
                            HeadKind::Map
                        }
                    }
                    _ => {
                        track!(
                            UnaryOp::from_token(t.into())
                                .map(|_| HeadKind::UnaryOpCall)
                                .map_err(|e| ErrorKind::UnexpectedToken(e).error())
                        )?
                    }
                }
            }
            LexicalToken::Keyword(t) => {
                if t.value() == Keyword::Fun {
                    HeadKind::Fun
                } else {
                    track!(
                        UnaryOp::from_token(t.into())
                            .map(|_| HeadKind::UnaryOpCall)
                            .map_err(|e| ErrorKind::UnexpectedToken(e).error())
                    )?
                }
            }
            LexicalToken::Variable(_) => {
                if track!(parser.eos())? {
                    HeadKind::Variable
                } else {
                    let token = track!(parser.parse::<LexicalToken>())?;
                    match token.as_symbol_token().map(|t| t.value()) {
                        Some(Symbol::DoubleColon) => HeadKind::Annotated,
                        _ => HeadKind::Variable,
                    }
                }
            }
            LexicalToken::Atom(_) => {
                if track!(parser.eos())? {
                    HeadKind::Literal
                } else {
                    let token = track!(parser.parse::<LexicalToken>())?;
                    match token.as_symbol_token().map(|t| t.value()) {
                        Some(Symbol::OpenParen) => HeadKind::LocalCall,
                        Some(Symbol::Colon) => HeadKind::RemoteCall,
                        _ => HeadKind::Literal,
                    }
                }
            }
            _ => HeadKind::Literal,
        })
    }
}

#[derive(Debug)]
enum TailKind {
    BinaryOpCall,
    Union,
    Range,
    None,
}
impl TailKind {
    fn guess<T: TokenRead>(parser: &mut Parser<T>) -> Result<Self> {
        if track!(parser.eos())? {
            return Ok(TailKind::None);
        }
        let token = track!(parser.parse::<LexicalToken>())?;
        Ok(match token.as_symbol_token().map(|t| t.value()) {
            Some(Symbol::VerticalBar) => TailKind::Union,
            Some(Symbol::DoubleDot) => TailKind::Range,
            _ => {
                if BinaryOp::from_token(token).is_ok() {
                    TailKind::BinaryOpCall
                } else {
                    TailKind::None
                }
            }
        })
    }
}