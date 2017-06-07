use {Result, TokenReader};

pub trait Parse<'token>: Sized {
    fn parse(reader: &mut TokenReader<'token>) -> Result<Self>;
    fn try_parse(reader: &mut TokenReader<'token>) -> Option<Self> {
        let position = reader.position();
        if let Ok(value) = Self::parse(reader) {
            Some(value)
        } else {
            reader.set_position(position);
            None
        }
    }
}
impl<'token, P> Parse<'token> for Option<P>
    where P: Parse<'token>
{
    fn parse(reader: &mut TokenReader<'token>) -> Result<Self> {
        Ok(P::try_parse(reader))
    }
}
impl<'token, P> Parse<'token> for Vec<P>
    where P: Parse<'token>
{
    fn parse(reader: &mut TokenReader<'token>) -> Result<Self> {
        let mut vec = Vec::new();
        while let Some(v) = P::try_parse(reader) {
            vec.push(v);
        }
        Ok(vec)
    }
}
impl<'token, P> Parse<'token> for Box<P>
    where P: Parse<'token>
{
    fn parse(reader: &mut TokenReader<'token>) -> Result<Self> {
        let v = track_try!(P::parse(reader));
        Ok(Box::new(v))
    }
}

pub trait TokenRange {
    fn token_start(&self) -> usize;
    fn token_end(&self) -> usize;
}
