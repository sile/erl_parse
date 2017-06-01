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

impl<'token, 'text: 'token, P0, P1> Parse<'token, 'text> for (P0, P1)
    where P0: Parse<'token, 'text>,
          P1: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let v0 = try_parse!(reader);
        let v1 = try_parse!(reader);
        Ok((v0, v1))
    }
}
impl<'token, 'text: 'token, P> Parse<'token, 'text> for Option<P>
    where P: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        Ok(reader.try_parse_next())
    }
}
impl<'token, 'text: 'token, P> Parse<'token, 'text> for Vec<P>
    where P: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let mut vec = Vec::new();
        while let Some(v) = reader.try_parse_next() {
            vec.push(v);
        }
        Ok(vec)
    }
}
