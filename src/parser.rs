use erl_tokenize::{Token, Result as TokenizeResult};

use Result;
use parse_tree::ModuleDecl;
use token_reader::TokenReader;

pub struct Parser<'a, I> {
    tokens: TokenReader<'a, I>,
}
impl<'a, I> Parser<'a, I>
    where I: Iterator<Item = TokenizeResult<Token<'a>>>
{
    pub fn new(tokens: I) -> Self {
        Parser { tokens: TokenReader::new(tokens) }
    }
    pub fn parse_module(self) -> Result<ModuleDecl> {
        panic!()
    }
}
