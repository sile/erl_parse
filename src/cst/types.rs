use {Result, TokenReader, Parse, TokenRange};
use cst::Type;
use cst::primitives::Args;
use cst::symbols;

#[derive(Debug)]
pub struct Function<'token, 'text: 'token> {
    pub args: Args<Type<'token, 'text>>,
    pub allow: symbols::RightAllow,
    pub return_type: Type<'token, 'text>,
    // TODO: pub constraints: Constraints<'token, 'text>,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Function<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        Ok(Function {
               args: track_try!(Parse::parse(reader)),
               allow: track_try!(Parse::parse(reader)),
               return_type: track_try!(Parse::parse(reader)),
           })
    }
}
impl<'token, 'text: 'token> TokenRange for Function<'token, 'text> {
    fn token_start(&self) -> usize {
        self.args.token_start()
    }
    fn token_end(&self) -> usize {
        self.return_type.token_end()
    }
}

// #[derive(Debug)]
// pub struct Constraints<'token, 'text: 'token> {
//     _a: &'token (),
//     _b: &'text (),
// }
