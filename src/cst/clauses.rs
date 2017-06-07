use cst::{Pattern, GuardSeq};
use cst::commons;
use cst::exprs;
use cst::literals;

#[derive(Debug, Clone)]
pub struct CaseClause<'token> {
    pub pattern: Pattern<'token>,
    pub guard: Option<Guard<'token>>,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub body: exprs::Body<'token>,
}
derive_parse!(CaseClause, pattern, guard, _arrow, body);
derive_token_range!(CaseClause, pattern, body);

#[derive(Debug, Clone)]
pub struct Guard<'token> {
    pub _when: literals::K_WHEN,
    pub seq: GuardSeq<'token>,
}
derive_parse!(Guard, _when, seq);
derive_token_range!(Guard, _when, seq);

#[derive(Debug, Clone)]
pub struct IfClause<'token> {
    pub cond: GuardSeq<'token>,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub body: exprs::Body<'token>,
}
derive_parse!(IfClause, cond, _arrow, body);
derive_token_range!(IfClause, cond, body);

#[derive(Debug, Clone)]
pub struct CatchClause<'token> {
    _position: commons::Void,
    pub class: Option<ExceptionClass<'token>>,
    pub pattern: Pattern<'token>,
    pub guard: Option<Guard<'token>>,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub body: exprs::Body<'token>,
}
derive_parse!(CatchClause, _position, class, pattern, guard, _arrow, body);
derive_token_range!(CatchClause, _position, body);

#[derive(Debug, Clone)]
pub struct ExceptionClass<'token> {
    pub class: commons::VarOrAtom<'token>,
    pub _colon: literals::S_COLON,
}
derive_parse!(ExceptionClass, class, _colon);
derive_token_range!(ExceptionClass, class, _colon);

#[derive(Debug, Clone)]
pub struct FunClause<'token, N> {
    pub name: N,
    pub patterns: commons::Args<Pattern<'token>>,
    pub guard: Option<Guard<'token>>,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub body: exprs::Body<'token>,
}
derive_parse!(FunClause<'token, N>, name, patterns, guard, _arrow, body);
derive_token_range!(FunClause<'token, N>, name, body);
