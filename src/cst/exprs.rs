use erl_tokenize::values::Symbol;

use {Result, TokenReader, Parse, TokenRange, ErrorKind};
use cst::{Expr, IdExpr, Pattern};
use cst::clauses::{CaseClause, CatchClause};
use cst::keywords;
use cst::primitives::{Args, Seq2, NonEmptySeq, Clauses, Optional};
use cst::symbols;

#[derive(Debug)]
pub struct ModulePrefix<'token, 'text: 'token> {
    pub name: IdExpr<'token, 'text>,
    pub _colon: symbols::Colon,
}
derive_parse!(ModulePrefix, name, _colon);
derive_token_range!(ModulePrefix, name, _colon);

#[derive(Debug)]
pub struct Match<'token, 'text: 'token> {
    pub pattern: Pattern<'token, 'text>,
    pub _match: symbols::Match,
    pub value: Expr<'token, 'text>,
}
derive_parse!(Match, pattern, _match, value);
derive_token_range!(Match, pattern, value);

// #[derive(Debug)]
// pub struct LocalCall<'token, 'text: 'token> {
//     pub fun_name: Expr<'token, 'text>,
//     pub args: Args<Expr<'token, 'text>>,
// }
// derive_parse!(LocalCall, fun_name, args);
// de rive_token_range!(LocalCall, fun_name, args);

// #[derive(Debug)]
// pub struct RemoteCall<'token, 'text: 'token> {
//     pub module_name: Expr<'token, 'text>,
//     pub _colon: symbols::Colon,
//     pub fun_name: Expr<'token, 'text>,
//     pub args: Args<Expr<'token, 'text>>,
// }
// derive_parse_trace!(RemoteCall, module_name, _colon, fun_name, args);
// derive_token_range!(RemoteCall, module_name, args);

#[derive(Debug)]
pub struct Call<'token, 'text: 'token> {
    pub module: Optional<ModulePrefix<'token, 'text>>,
    pub fun_name: IdExpr<'token, 'text>,
    pub args: Args<Expr<'token, 'text>>,
}
derive_parse!(Call, module, fun_name, args);
derive_token_range!(Call, module, args);

#[derive(Debug)]
pub struct Try<'token, 'text: 'token> {
    pub _try: keywords::Try,
    pub body: NonEmptySeq<Expr<'token, 'text>, symbols::Comma>,
    pub try_of: Option<TryOf<'token, 'text>>,
    pub try_catch: Option<TryCatch<'token, 'text>>,
    pub try_after: Option<TryAfter<'token, 'text>>,
    pub _end: keywords::End,
}
derive_parse!(Try, _try, body, try_of, try_catch, try_after, _end);
derive_token_range!(Try, _try, _end);
// TODO: catchとafterの両方がNoneなのはillegal

#[derive(Debug)]
pub struct TryOf<'token, 'text: 'token> {
    pub _of: keywords::Of,
    pub clauses: Clauses<CaseClause<'token, 'text>>,
}
derive_parse!(TryOf, _of, clauses);
derive_token_range!(TryOf, _of, clauses);

#[derive(Debug)]
pub struct TryCatch<'token, 'text: 'token> {
    pub _catch: keywords::Catch,
    pub clauses: Clauses<CatchClause<'token, 'text>>,
}
derive_parse!(TryCatch, _catch, clauses);
derive_token_range!(TryCatch, _catch, clauses);

#[derive(Debug)]
pub struct TryAfter<'token, 'text: 'token> {
    pub _after: keywords::After,
    pub body: NonEmptySeq<Expr<'token, 'text>, symbols::Comma>,
}
derive_parse!(TryAfter, _after, body);
derive_token_range!(TryAfter, _after, body);

#[derive(Debug)]
pub struct List<'token, 'text: 'token> {
    pub _open: symbols::OpenSquare,
    pub elements: Seq2<Expr<'token, 'text>, symbols::Comma>,
    // TODO: [|0]が許容されてしまう
    pub tail: Option<ListTail<'token, 'text>>,
    pub _close: symbols::CloseSquare,
}
derive_parse!(List, _open, elements, tail, _close);
derive_token_range!(List, _open, _close);

#[derive(Debug)]
pub struct ListTail<'token, 'text: 'token> {
    pub bar: symbols::VerticalBar,
    pub element: Expr<'token, 'text>,
}
derive_parse!(ListTail, bar, element);
derive_token_range!(ListTail, bar, element);

#[derive(Debug, Copy,Clone)]
pub enum BinaryOp {
    Add { position: usize },
    Sub { position: usize },
}
impl<'token, 'text: 'token> Parse<'token, 'text> for BinaryOp {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let position = reader.position();
        let symbol = track_try!(reader.read_symbol());
        match symbol.value() {
            Symbol::Plus => Ok(BinaryOp::Add { position }),
            Symbol::Hyphen => Ok(BinaryOp::Sub { position }),
            _ => {
                track_panic!(ErrorKind::InvalidInput,
                             "Not a binary operator: {:?}",
                             symbol)
            }
        }
    }
}
impl<'token, 'text: 'token> TokenRange for BinaryOp {
    fn token_start(&self) -> usize {
        match *self {
            BinaryOp::Add { position } => position,
            BinaryOp::Sub { position } => position,
        }
    }
    fn token_end(&self) -> usize {
        match *self {
            BinaryOp::Add { position } => position + 1,
            BinaryOp::Sub { position } => position + 1,
        }
    }
}

#[derive(Debug)]
pub struct BinaryOpCall<'token, 'text: 'token> {
    pub left: Expr<'token, 'text>,
    pub op: BinaryOp,
    pub right: Expr<'token, 'text>,
}
derive_parse!(BinaryOpCall, left, op, right);
derive_token_range!(BinaryOpCall, left, right);
