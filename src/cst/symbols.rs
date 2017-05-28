use erl_tokenize::values::Symbol;

use {Result, TokenReader2, Parse, TokenRange};

#[derive(Debug, Clone, Copy)]
pub struct Slash {
    position: usize,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Slash {
    fn parse(reader: &mut TokenReader2<'token, 'text>) -> Result<Self> {
        reader.skip_hidden_tokens();
        let position = reader.position();
        track_try!(reader.expect_symbol(Symbol::Slash));
        Ok(Slash { position })
    }
}
impl TokenRange for Slash {
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.position + 1
    }
}
