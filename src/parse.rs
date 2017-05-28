use {Result, TokenReader};

pub trait Parse<'token, 'text: 'token>: Sized {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self>;
    fn try_parse(reader: &mut TokenReader<'token, 'text>) -> Option<Self> {
        let position = reader.position();
        if let Ok(value) = Self::parse(reader) {
            Some(value)
        } else {
            reader.set_position(position);
            None
        }
    }
}
