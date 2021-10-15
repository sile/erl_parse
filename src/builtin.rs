use erl_pp::{self, Preprocessor};
use erl_tokenize::{Lexer, LexicalToken};

use crate::cst::ModuleDecl;
use crate::traits::TokenRead;
use crate::{Parser, Result, TokenReader};

#[derive(Debug)]
pub struct ModuleParser<'a>(Parser<TokenReader<Preprocessor<Lexer<&'a str>>, erl_pp::Error>>);
impl<'a> ModuleParser<'a> {
    pub fn new(tokens: Preprocessor<Lexer<&'a str>>) -> Self {
        ModuleParser(Parser::new(TokenReader::new(tokens)))
    }
    pub fn parse_module(&mut self) -> Result<ModuleDecl> {
        track!(self.0.parse(), "next={:?}", self.0.parse::<LexicalToken>())
    }
    pub fn preprocessor(&self) -> &Preprocessor<Lexer<&'a str>> {
        self.0.reader().inner()
    }
    pub fn preprocessor_mut(&mut self) -> &mut Preprocessor<Lexer<&'a str>> {
        self.0.reader_mut().inner_mut()
    }
}

pub fn parse_module(tokens: &mut dyn TokenRead) -> Result<ModuleDecl> {
    Parser::new(tokens).parse()
}
