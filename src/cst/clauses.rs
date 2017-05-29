use {Result, TokenReader, Parse, TokenRange};
use cst::{Pattern, Expression};
use cst::primitives::{Seq, Args};
use cst::symbols;

#[derive(Debug)]
pub struct FunctionClause<'token, 'text: 'token> {
    pub patterns: Args<Pattern<'token, 'text>>,
    // TODO: guard
    pub allow: symbols::RightAllow,
    pub body: Seq<Expression<'token, 'text>>,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for FunctionClause<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        Ok(FunctionClause {
               patterns: track_try!(reader.parse_next()),
               allow: track_try!(reader.parse_next()),
               body: track_try!(reader.parse_next()),
           })
    }
}
impl<'token, 'text: 'token> TokenRange for FunctionClause<'token, 'text> {
    fn token_start(&self) -> usize {
        self.patterns.token_start()
    }
    fn token_end(&self) -> usize {
        self.body.token_end()
    }
}
