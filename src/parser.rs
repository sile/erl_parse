use erl_tokenize::{Token, Tokenizer};

use {Result, TokenReader, Parse};
use cst::{Expr, Pattern, Type, Form, ModuleDecl};

pub struct Parser {
    tokens: Vec<Token>,
}
impl Parser {
    pub fn new(text: &str) -> Result<Self> {
        let result = Tokenizer::new(text)
            .map(|r| r.map(|(t, _)| t)) // TODO: Handle `Position`
            .collect::<::std::result::Result<Vec<_>, _>>();
        let tokens = track_try!(result);
        Ok(Parser { tokens })
    }
    pub fn tokens(&self) -> &[Token] {
        &self.tokens
    }
    pub fn parse_expr(&self) -> Result<Expr> {
        let mut reader = TokenReader::new(&self.tokens);
        let expr = track_try!(Expr::parse(&mut reader), "line_num={}", reader.line_num());
        Ok(expr)
    }
    pub fn parse_pattern(&self) -> Result<Pattern> {
        let mut reader = TokenReader::new(&self.tokens);
        let pattern = track_try!(Pattern::parse(&mut reader),
                                 "line_num={}",
                                 reader.line_num());
        Ok(pattern)
    }
    pub fn parse_type(&self) -> Result<Type> {
        let mut reader = TokenReader::new(&self.tokens);
        let ty = track_try!(Type::parse(&mut reader), "line_num={}", reader.line_num());
        Ok(ty)
    }
    pub fn parse_form(&self) -> Result<Form> {
        let mut reader = TokenReader::new(&self.tokens);
        let form = track_try!(Form::parse(&mut reader), "line_num={}", reader.line_num());
        Ok(form)
    }
    pub fn parse_module(&self) -> Result<ModuleDecl> {
        let mut reader = TokenReader::new(&self.tokens);
        let form = track_try!(ModuleDecl::parse(&mut reader),
                              "line_num={}",
                              reader.line_num());
        Ok(form)
    }
}
