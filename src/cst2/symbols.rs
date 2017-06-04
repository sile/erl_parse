use erl_tokenize::values::Symbol;

use {Result, TokenReader, Parse, TokenRange};

macro_rules! define_symbol {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $name {
            position: usize,
        }
        impl<'token, 'text: 'token> Parse<'token, 'text> for $name {
            fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
                // reader.skip_hidden_tokens();
                let position = reader.position();
                track_try!(reader.expect_symbol(Symbol::$name));
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

define_symbol!(Slash);
define_symbol!(Hyphen);
define_symbol!(OpenParen);
define_symbol!(CloseParen);
define_symbol!(OpenSquare);
define_symbol!(CloseSquare);
define_symbol!(OpenBrace);
define_symbol!(CloseBrace);
define_symbol!(Dot);
define_symbol!(Comma);
define_symbol!(Colon);
define_symbol!(DoubleColon);
define_symbol!(DoubleDot);
define_symbol!(TripleDot);
define_symbol!(Semicolon);
define_symbol!(RightAllow);
define_symbol!(Match);
define_symbol!(VerticalBar);
define_symbol!(Plus);
define_symbol!(Question);
