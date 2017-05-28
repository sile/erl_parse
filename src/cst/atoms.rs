use {Result, TokenReader, Parse, TokenRange};

macro_rules! define_atom {
    ($name:ident, $value:expr) => {
        #[derive(Debug)]
        pub struct $name {
            position: usize,
        }
        impl<'token, 'text: 'token> Parse<'token, 'text> for $name {
            fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
                reader.skip_hidden_tokens();
                let position = reader.position();
                track_try!(reader.expect_atom($value));
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

define_atom!(Module, "module");
define_atom!(Export, "export");
