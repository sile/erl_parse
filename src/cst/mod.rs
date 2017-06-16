use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{AtomToken, CharToken, FloatToken, IntegerToken, StringToken,
                           VariableToken};

use {Result, Parse, Preprocessor, TokenReader};

pub mod building_blocks;
pub mod collections;
pub mod exprs;

#[derive(Debug, Clone)]
pub enum Literal {
    Atom(AtomToken),
    Char(CharToken),
    Float(FloatToken),
    Integer(IntegerToken),
    String(StringToken),
}
impl Parse for Literal {
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(track!(reader.try_read_token())?.and_then(
            |token| match token {
                LexicalToken::Atom(t) => Some(Literal::Atom(t)),
                LexicalToken::Char(t) => Some(Literal::Char(t)),
                LexicalToken::Float(t) => Some(Literal::Float(t)),
                LexicalToken::Integer(t) => Some(Literal::Integer(t)),
                LexicalToken::String(t) => Some(Literal::String(t)),
                _ => {
                    reader.unread_token(token);
                    None
                }
            },
        ))
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
pub enum Expr {
    Literal(Literal),
    Variable(VariableToken),
    Tuple(Box<exprs::Tuple>),
    Map(Box<exprs::Map>),
    Record(Box<exprs::Record>),
    List(Box<exprs::List>),
    Block(Box<exprs::Block>),
}
impl Parse for Expr {
    fn try_parse<T>(reader: &mut TokenReader<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        // TODO: optimize
        if let Some(e) = track!(Parse::try_parse(reader))? {
            Ok(Some(Expr::Tuple(e)))
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Ok(Some(Expr::List(e)))
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Ok(Some(Expr::Map(e)))
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Ok(Some(Expr::Record(e)))
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Ok(Some(Expr::Block(e)))
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Ok(Some(Expr::Variable(e)))
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Ok(Some(Expr::Literal(e)))
        } else {
            Ok(None)
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
            Expr::Record(ref x) => x.start_position(),
            Expr::List(ref x) => x.start_position(),
            Expr::Block(ref x) => x.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            Expr::Literal(ref x) => x.end_position(),
            Expr::Variable(ref x) => x.end_position(),
            Expr::Tuple(ref x) => x.end_position(),
            Expr::Map(ref x) => x.end_position(),
            Expr::Record(ref x) => x.end_position(),
            Expr::List(ref x) => x.end_position(),
            Expr::Block(ref x) => x.end_position(),
        }
    }
}
