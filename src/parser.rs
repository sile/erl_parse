use erl_tokenize::{Token, Tokenizer};

use {Result, TokenReader, Parse};
use cst::ModuleDecl;

pub struct Parser<'text> {
    tokens: Vec<Token<'text>>,
}
impl<'text> Parser<'text> {
    pub fn new(text: &'text str) -> Result<Self> {
        let result = Tokenizer::new(text).collect::<::std::result::Result<Vec<_>, _>>();
        let tokens = track_try!(result);
        Ok(Parser { tokens })
    }
    pub fn tokens(&self) -> &[Token<'text>] {
        &self.tokens
    }
    pub fn parse_module<'token>(&'token self) -> Result<ModuleDecl<'token, 'text>> {
        let mut reader = TokenReader::new(&self.tokens);
        track!(ModuleDecl::parse(&mut reader))
    }
}
