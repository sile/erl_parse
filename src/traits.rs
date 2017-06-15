use {Result, TokenReader};

pub trait Parse: Sized {
    fn parse(reader: &mut TokenReader) -> Result<Self>;
    fn try_parse(reader: &mut TokenReader) -> Option<Self> {
        let position = reader.position();
        if let Ok(value) = Self::parse(reader) {
            Some(value)
        } else {
            reader.set_position(position);
            None
        }
    }
}
impl<P> Parse for Option<P>
where
    P: Parse,
{
    fn parse(reader: &mut TokenReader) -> Result<Self> {
        Ok(P::try_parse(reader))
    }
}
impl<P> Parse for Vec<P>
where
    P: Parse,
{
    fn parse(reader: &mut TokenReader) -> Result<Self> {
        let mut vec = Vec::new();
        while let Some(v) = P::try_parse(reader) {
            vec.push(v);
        }
        Ok(vec)
    }
}
impl<P> Parse for Box<P>
where
    P: Parse,
{
    fn parse(reader: &mut TokenReader) -> Result<Self> {
        let v = track_try!(P::parse(reader));
        Ok(Box::new(v))
    }
}

pub trait TokenRange {
    fn token_start(&self) -> usize;
    fn token_end(&self) -> usize;
}
