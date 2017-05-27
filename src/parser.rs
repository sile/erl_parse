use erl_tokenize::{Token, Tokenizer, Error as TokenizeError};

use {Result, ParseTree};

type TokenizeResult<'a> = ::std::result::Result<Token<'a>, TokenizeError>;

pub struct Parser;
impl Parser {
    pub fn parse_str<'a>(self, text: &'a str) -> Result<ParseTree> {
        track!(self.parse_tokens(Tokenizer::new(text)))
    }
    pub fn parse_tokens<'a, I>(self, _tokens: I) -> Result<ParseTree>
        where I: Iterator<Item = TokenizeResult<'a>>
    {
        unimplemented!()
    }
}
