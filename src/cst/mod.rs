use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{AtomToken, CharToken, FloatToken, IntegerToken, StringToken,
                           VariableToken};
use erl_tokenize::values::Symbol;

use {Result, Parse, Preprocessor, Parser, IntoTokens};

pub mod building_blocks;
pub mod collections;
pub mod exprs;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Variable(VariableToken),
    Tuple(Box<exprs::Tuple>),
    Map(Box<exprs::Map>),
    Record(Box<exprs::Record>),
    List(Box<exprs::List>),
    ListComprehension(Box<exprs::ListComprehension>),
    Block(Box<exprs::Block>),
    Parenthesized(Box<exprs::Parenthesized>),
    Catch(Box<exprs::Catch>),
    FunCall(Box<exprs::FunCall>),
}
impl Parse for Expr {
    fn try_parse<T>(reader: &mut Parser<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        // TODO: optimize
        let expr = if let Some(e) = track!(Parse::try_parse(reader))? {
            Expr::Tuple(e)
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Expr::List(e)
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Expr::ListComprehension(e)
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Expr::Map(e)
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Expr::Record(e)
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Expr::Block(e)
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Expr::Parenthesized(e)
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Expr::Catch(e)
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Expr::Variable(e)
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Expr::Literal(e)
        } else {
            return Ok(None);
        };

        // TODO: optimize
        match track!(reader.peek_token())? {
            Some(LexicalToken::Symbol(ref s))
                if s.value() == Symbol::OpenParen || s.value() == Symbol::Colon => {
                let e = track!(Parse::parse(reader))?;
                Ok(Some(Expr::FunCall(e)))
            }
            _ => Ok(Some(expr)),
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
            Expr::ListComprehension(ref x) => x.start_position(),
            Expr::Block(ref x) => x.start_position(),
            Expr::Parenthesized(ref x) => x.start_position(),
            Expr::Catch(ref x) => x.start_position(),
            Expr::FunCall(ref x) => x.start_position(),
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
            Expr::ListComprehension(ref x) => x.end_position(),
            Expr::Block(ref x) => x.end_position(),
            Expr::Parenthesized(ref x) => x.end_position(),
            Expr::Catch(ref x) => x.end_position(),
            Expr::FunCall(ref x) => x.end_position(),
        }
    }
}
impl IntoTokens for Expr {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        match self {
            Expr::Literal(x) => x.into_tokens(),
            Expr::Variable(x) => x.into_tokens(),
            Expr::Tuple(x) => x.into_tokens(),
            Expr::Map(x) => x.into_tokens(),
            Expr::Record(x) => x.into_tokens(),
            Expr::List(x) => x.into_tokens(),
            Expr::ListComprehension(x) => x.into_tokens(),
            Expr::Block(x) => x.into_tokens(),
            Expr::Parenthesized(x) => x.into_tokens(),
            Expr::Catch(x) => x.into_tokens(),
            Expr::FunCall(x) => x.into_tokens(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Literal),
    Variable(VariableToken),
}
impl Parse for Pattern {
    fn try_parse<T>(reader: &mut Parser<T>) -> Result<Option<Self>>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        // TODO: optimize
        if let Some(e) = track!(Parse::try_parse(reader))? {
            Ok(Some(Pattern::Variable(e)))
        } else if let Some(e) = track!(Parse::try_parse(reader))? {
            Ok(Some(Pattern::Literal(e)))
        } else {
            Ok(None)
        }
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
impl IntoTokens for Pattern {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        match self {
            Pattern::Literal(x) => x.into_tokens(),
            Pattern::Variable(x) => x.into_tokens(),
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
    fn try_parse<T>(reader: &mut Parser<T>) -> Result<Option<Self>>
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
impl IntoTokens for Literal {
    fn into_tokens(self) -> Box<Iterator<Item = LexicalToken>> {
        match self {
            Literal::Atom(x) => x.into_tokens(),
            Literal::Char(x) => x.into_tokens(),
            Literal::Float(x) => x.into_tokens(),
            Literal::Integer(x) => x.into_tokens(),
            Literal::String(x) => x.into_tokens(),
        }
    }
}
