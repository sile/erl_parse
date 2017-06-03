use erl_tokenize::values::Keyword;

use {Result, TokenReader, Parse, TokenRange};

macro_rules! define_keyword {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name {
            position: usize,
        }
        impl<'token, 'text: 'token> Parse<'token, 'text> for $name {
            fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
                // reader.skip_hidden_tokens();
                let position = reader.position();
                track_try!(reader.expect_keyword(Keyword::$name));
                Ok($name { position })
            }
        }
        impl TokenRange for $name {
            fn token_start(&self) -> usize {
                self.position
            }
            fn token_end(&self) -> usize {
                self.position + 1
            }
        }
    }
}

define_keyword!(Try);
define_keyword!(Of);
define_keyword!(End);
define_keyword!(Catch);
define_keyword!(After);
