use erl_tokenize::{Token, Result as TokenizeResult};

use Result;
use parse_tree::ModuleDecl;
use token_reader::{TokenReader, TokenReader2};

mod form_parser;

pub struct Parser<'text, I> {
    reader: TokenReader<'text, I>,
}
impl<'text, I> Parser<'text, I>
    where I: Iterator<Item = TokenizeResult<Token<'text>>>
{
    pub fn new(tokens: I) -> Self {
        Parser { reader: TokenReader::new(tokens) }
    }
    pub fn parse_module(mut self) -> Result<ModuleDecl<'text>> {
        let mut module = ModuleDecl::new();
        {
            let mut form_parser = form_parser::FormParser::new(&mut self.reader);
            while let Some(form) = track_try!(form_parser.parse_next()) {
                module.forms.push(form);
            }
        }
        module.trailing_tokens = self.reader.take_read_tokens();
        Ok(module)
    }
}
