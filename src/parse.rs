use {Result, TokenReader2};

pub trait Parse<'token, 'text: 'token>: Sized {
    fn parse(reader: &mut TokenReader2<'token, 'text>) -> Result<Self>;
}
