use erl_tokenize::Token;

use {Result, TokenReader2, Parse};

#[derive(Debug)]
pub struct Slash<'token, 'text: 'token> {
    pub leadings: &'token [Token<'text>],
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Slash<'token, 'text> {
    fn parse(reader: &mut TokenReader2<'token, 'text>) -> Result<Self> {
        let leadings = reader.read_hidden_tokens();
        Ok(Slash { leadings })
    }
}
