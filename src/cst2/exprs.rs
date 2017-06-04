use erl_tokenize::values::Symbol;

use {Result, TokenReader, Parse, TokenRange, ErrorKind};
use cst::{Expr, IdExpr, Pattern};
use cst::clauses::{CaseClause, CatchClause, AnonymousFunClause};
use cst::keywords;
use cst::primitives::{Args, Seq2, NonEmptySeq, Clauses, Optional, AtomOrVar, Integer, VarOr, Atom};
use cst::symbols;

#[derive(Debug, Clone)]
pub struct ModulePrefix<'token, 'text: 'token> {
    pub name: IdExpr<'token, 'text>,
    pub _colon: symbols::Colon,
}
derive_parse!(ModulePrefix, name, _colon);
derive_token_range!(ModulePrefix, name, _colon);

#[derive(Debug, Clone)]
pub struct Match<'token, 'text: 'token> {
    pub pattern: Pattern<'token, 'text>,
    pub _match: symbols::Match,
    pub value: Expr<'token, 'text>,
}
derive_parse!(Match, pattern, _match, value);
derive_token_range!(Match, pattern, value);

#[derive(Debug, Clone)]
pub struct AnonymousFun<'token, 'text: 'token> {
    pub _fun: keywords::Fun,
    pub clauses: Clauses<AnonymousFunClause<'token, 'text>>,
    pub _end: keywords::End,
}
derive_parse!(AnonymousFun, _fun, clauses, _end);
derive_token_range!(AnonymousFun, _fun, _end);

#[derive(Debug, Clone)]
pub struct LocalFun<'token, 'text: 'token> {
    pub _fun: keywords::Fun,
    pub fun_name: Atom<'token, 'text>,
    pub _slash: symbols::Slash,
    pub arity: Integer<'token, 'text>,
}
derive_parse!(LocalFun, _fun, fun_name, _slash, arity);
derive_token_range!(LocalFun, _fun, arity);

#[derive(Debug, Clone)]
pub struct RemoteFun<'token, 'text: 'token> {
    pub _fun: keywords::Fun,
    pub module_name: VarOr<'token, 'text, Atom<'token, 'text>>,
    pub _colon: symbols::Colon,
    pub fun_name: VarOr<'token, 'text, Atom<'token, 'text>>,
    pub _slash: symbols::Slash,
    pub arity: VarOr<'token, 'text, Integer<'token, 'text>>,
}
derive_parse!(RemoteFun,
              _fun,
              module_name,
              _colon,
              fun_name,
              _slash,
              arity);
derive_token_range!(RemoteFun, _fun, arity);

// #[derive(Debug, Clone)]
// pub struct LocalCall<'token, 'text: 'token> {
//     pub fun_name: Expr<'token, 'text>,
//     pub args: Args<Expr<'token, 'text>>,
// }
// derive_parse!(LocalCall, fun_name, args);
// de rive_token_range!(LocalCall, fun_name, args);

// #[derive(Debug, Clone)]
// pub struct RemoteCall<'token, 'text: 'token> {
//     pub module_name: Expr<'token, 'text>,
//     pub _colon: symbols::Colon,
//     pub fun_name: Expr<'token, 'text>,
//     pub args: Args<Expr<'token, 'text>>,
// }
// derive_parse_trace!(RemoteCall, module_name, _colon, fun_name, args);
// derive_token_range!(RemoteCall, module_name, args);

#[derive(Debug, Clone)]
pub struct Call<'token, 'text: 'token> {
    pub module: Optional<ModulePrefix<'token, 'text>>,
    pub fun_name: IdExpr<'token, 'text>,
    pub args: Args<Expr<'token, 'text>>,
}
derive_parse!(Call, module, fun_name, args);
derive_token_range!(Call, module, args);

// TODO: マクロはちゃんと展開する必要があるので、これでは不適切
#[derive(Debug, Clone)]
pub struct MacroCall<'token, 'text: 'token> {
    pub _question: symbols::Question,
    pub macro_name: AtomOrVar<'token, 'text>,
    pub args: Optional<Args<Expr<'token, 'text>>>,
}
derive_parse!(MacroCall, _question, macro_name, args);
derive_token_range!(MacroCall, _question, args);

#[derive(Debug, Clone)]
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


#[derive(Debug, Clone)]
pub struct Case<'token, 'text: 'token> {
    pub _case: keywords::Case,
    pub expr: Expr<'token, 'text>,
    pub _of: keywords::Of,
    pub clauses: Clauses<CaseClause<'token, 'text>>,
    pub _end: keywords::End,
}
derive_parse!(Case, _case, expr, _of, clauses, _end);
derive_token_range!(Case, _case, _end);

#[derive(Debug, Clone)]
pub struct TryOf<'token, 'text: 'token> {
    pub _of: keywords::Of,
    pub clauses: Clauses<CaseClause<'token, 'text>>,
}
derive_parse!(TryOf, _of, clauses);
derive_token_range!(TryOf, _of, clauses);

#[derive(Debug, Clone)]
pub struct TryCatch<'token, 'text: 'token> {
    pub _catch: keywords::Catch,
    pub clauses: Clauses<CatchClause<'token, 'text>>,
}
derive_parse!(TryCatch, _catch, clauses);
derive_token_range!(TryCatch, _catch, clauses);

#[derive(Debug, Clone)]
pub struct TryAfter<'token, 'text: 'token> {
    pub _after: keywords::After,
    pub body: NonEmptySeq<Expr<'token, 'text>, symbols::Comma>,
}
derive_parse!(TryAfter, _after, body);
derive_token_range!(TryAfter, _after, body);

#[derive(Debug, Clone)]
pub struct List<'token, 'text: 'token> {
    pub _open: symbols::OpenSquare,
    pub elements: Seq2<Expr<'token, 'text>, symbols::Comma>,
    // TODO: [|0]が許容されてしまう
    pub tail: Option<ListTail<'token, 'text>>,
    pub _close: symbols::CloseSquare,
}
derive_parse!(List, _open, elements, tail, _close);
derive_token_range!(List, _open, _close);

#[derive(Debug, Clone)]
pub struct Tuple<'token, 'text: 'token> {
    pub _open: symbols::OpenBrace,
    pub elements: Seq2<Expr<'token, 'text>, symbols::Comma>,
    pub _close: symbols::CloseBrace,
}
derive_parse!(Tuple, _open, elements, _close);
derive_token_range!(Tuple, _open, _close);

#[derive(Debug, Clone)]
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
    LessThan { position: usize },
}
impl<'token, 'text: 'token> Parse<'token, 'text> for BinaryOp {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let position = reader.position();
        let symbol = track_try!(reader.read_symbol());
        match symbol.value() {
            Symbol::Plus => Ok(BinaryOp::Add { position }),
            Symbol::Hyphen => Ok(BinaryOp::Sub { position }),
            Symbol::Less => Ok(BinaryOp::LessThan { position }),
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
            BinaryOp::LessThan { position } => position,
        }
    }
    fn token_end(&self) -> usize {
        match *self {
            BinaryOp::Add { position } => position + 1,
            BinaryOp::Sub { position } => position + 1,
            BinaryOp::LessThan { position } => position + 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinaryOpCall<'token, 'text: 'token> {
    pub left: Expr<'token, 'text>,
    pub op: BinaryOp,
    pub right: Expr<'token, 'text>,
}
derive_parse!(BinaryOpCall, left, op, right);
derive_token_range!(BinaryOpCall, left, right);
