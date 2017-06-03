use {Result, TokenReader, Parse, TokenRange};
use cst::{Pattern, Expr};
use cst::primitives::{Seq, Args, Atom, AtomOrVar};
use cst::symbols;

#[derive(Debug)]
pub struct FunctionClause<'token, 'text: 'token> {
    pub name: Atom<'token, 'text>,
    pub patterns: Args<Pattern<'token, 'text>>,
    // TODO: guard
    pub allow: symbols::RightAllow,
    pub body: Seq<Expr<'token, 'text>>,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for FunctionClause<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        Ok(FunctionClause {
               name: track_try!(reader.parse_next()),
               patterns: track_try!(reader.parse_next()),
               allow: track_try!(reader.parse_next()),
               body: track_try!(reader.parse_next()),
           })
    }
}
impl<'token, 'text: 'token> TokenRange for FunctionClause<'token, 'text> {
    fn token_start(&self) -> usize {
        self.name.token_start()
    }
    fn token_end(&self) -> usize {
        self.body.token_end()
    }
}

#[derive(Debug)]
pub struct CaseClause<'token, 'text: 'token> {
    pub pattern: Pattern<'token, 'text>,
    // TODO: guard
    pub _allow: symbols::RightAllow,
    pub body: Seq<Expr<'token, 'text>>,
}
derive_parse!(CaseClause, pattern, _allow, body);
derive_token_range!(CaseClause, pattern, body);

#[derive(Debug)]
pub struct CatchClause<'token, 'text: 'token> {
    pub exception_class: Option<ExceptionClass<'token, 'text>>,
    pub pattern: Pattern<'token, 'text>,
    // TODO: guard
    pub _allow: symbols::RightAllow,
    pub body: Seq<Expr<'token, 'text>>,
}
derive_parse!(CatchClause, exception_class, pattern, _allow, body);
impl<'token, 'text: 'token> TokenRange for CatchClause<'token, 'text> {
    fn token_start(&self) -> usize {
        self.exception_class
            .as_ref()
            .map_or(self.pattern.token_start(), |t| t.token_start())
    }
    fn token_end(&self) -> usize {
        self.body.token_end()
    }
}


#[derive(Debug)]
pub struct ExceptionClass<'token, 'text: 'token> {
    pub class: AtomOrVar<'token, 'text>,
    pub _colon: symbols::Colon,
}
derive_parse!(ExceptionClass, class, _colon);
derive_token_range!(ExceptionClass, class, _colon);
