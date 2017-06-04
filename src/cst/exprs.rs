use cst::Expr;
use cst::commons;
use cst::literals;

#[derive(Debug)]
pub struct Block<'token, 'text: 'token> {
    pub _begin: literals::K_BEGIN,
    pub body: Body<'token, 'text>,
    pub _end: literals::K_END,
}
derive_parse!(Block, _begin, body, _end);
derive_token_range!(Block, _begin, _end);

#[derive(Debug)]
pub struct Body<'token, 'text: 'token> {
    pub exprs: commons::NonEmptySeq<Expr<'token, 'text>, literals::S_COMMA>,
}
derive_parse!(Body, exprs);
derive_token_range!(Body, exprs, exprs);

#[derive(Debug)]
pub struct Catch<'token, 'text: 'token> {
    pub _catch: literals::K_CATCH,
    pub expr: Expr<'token, 'text>,
}
derive_parse!(Catch, _catch, expr);
derive_token_range!(Catch, _catch, expr);

#[derive(Debug)]
pub struct LocalFun<'token, 'text: 'token> {
    pub _fun: literals::K_FUN,
    pub fun_name: literals::Atom<'token, 'text>,
    pub _slash: literals::S_SLASH,
    pub arity: literals::Int<'token, 'text>,
}
derive_parse!(LocalFun, _fun, fun_name, _slash, arity);
derive_token_range!(LocalFun, _fun, arity);

#[derive(Debug)]
pub struct RemoteFun<'token, 'text: 'token> {
    pub _fun: literals::K_FUN,
    pub module_name: commons::VarOrAtom<'token, 'text>,
    pub _colon: literals::S_COLON,
    pub fun_name: commons::VarOrAtom<'token, 'text>,
    pub _slash: literals::S_SLASH,
    pub arity: commons::VarOrInt<'token, 'text>,
}
derive_parse!(RemoteFun,
              _fun,
              module_name,
              _colon,
              fun_name,
              _slash,
              arity);
derive_token_range!(RemoteFun, _fun, arity);

pub type Parenthesized<'token, 'text> = commons::Parenthesized<Expr<'token, 'text>>;
pub type Tuple<'token, 'text> = commons::Tuple<Expr<'token, 'text>>;
pub type Map<'token, 'text> = commons::Map<Expr<'token, 'text>>;
pub type Record<'token, 'text> = commons::Record<'token, 'text, Expr<'token, 'text>>;
pub type RecordFieldIndex<'token, 'text> = commons::RecordFieldIndex<'token, 'text>;
pub type List<'token, 'text> = commons::List<Expr<'token, 'text>>;
pub type TailConsList<'token, 'text> = commons::TailConsList<Expr<'token, 'text>>;
pub type UnaryOpCall<'token, 'text> = commons::UnaryOpCall<Expr<'token, 'text>>;
