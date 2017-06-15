use erl_pp::Preprocessor;
use erl_tokenize::{LexicalToken, Lexer};

use {Result, Error, TokenReader, Parse};
use cst::{Expr, Pattern, Type, Form, ModuleDecl};

pub struct Parser {
    tokens: Vec<LexicalToken>,
}
impl Parser {
    pub fn new<T, E>(tokens: T) -> Result<Self>
    where
        T: Iterator<Item = ::std::result::Result<LexicalToken, E>>,
        Error: From<E>,
    {
        let result = tokens.collect::<::std::result::Result<Vec<_>, _>>();
        let tokens = track!(result.map_err(Error::from))?;
        Ok(Parser { tokens })
    }
    pub fn from_text(text: &str) -> Result<Self> {
        Parser::new(Preprocessor::new(Lexer::new(text)))
    }
    pub fn tokens(&self) -> &[LexicalToken] {
        &self.tokens
    }
    pub fn parse_expr(&self) -> Result<Expr> {
        let mut reader = TokenReader::new(&self.tokens);
        let expr = track!(Expr::parse(&mut reader))?;
        Ok(expr)
    }
    pub fn parse_pattern(&self) -> Result<Pattern> {
        let mut reader = TokenReader::new(&self.tokens);
        let pattern = track!(Pattern::parse(&mut reader))?;
        Ok(pattern)
    }
    pub fn parse_type(&self) -> Result<Type> {
        let mut reader = TokenReader::new(&self.tokens);
        let ty = track!(Type::parse(&mut reader))?;
        Ok(ty)
    }
    pub fn parse_form(&self) -> Result<Form> {
        let mut reader = TokenReader::new(&self.tokens);
        let form = track!(Form::parse(&mut reader))?;
        Ok(form)
    }
    pub fn parse_module(&self) -> Result<ModuleDecl> {
        let mut reader = TokenReader::new(&self.tokens);
        let form = track!(ModuleDecl::parse(&mut reader))?;
        Ok(form)
    }
}
