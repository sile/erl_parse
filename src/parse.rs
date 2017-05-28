use {Result, TokenReader2};

pub trait Parse<'token, 'text: 'token>: Sized {
    fn parse(reader: &mut TokenReader2<'token, 'text>) -> Result<Self>;
    fn try_parse(reader: &mut TokenReader2<'token, 'text>) -> Option<Self> {
        let position = reader.position();
        if let Ok(value) = Self::parse(reader) {
            Some(value)
        } else {
            reader.set_position(position);
            None
        }
    }
}
