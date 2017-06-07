use cst::{Pattern, GuardSeq};
use cst::commons;
use cst::exprs;
use cst::literals;

#[derive(Debug, Clone)]
pub struct CaseClause {
    pub pattern: Pattern,
    pub guard: Option<Guard>,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub body: exprs::Body,
}
derive_parse!(CaseClause, pattern, guard, _arrow, body);
derive_token_range!(CaseClause, pattern, body);

#[derive(Debug, Clone)]
pub struct Guard {
    pub _when: literals::K_WHEN,
    pub seq: GuardSeq,
}
derive_parse!(Guard, _when, seq);
derive_token_range!(Guard, _when, seq);

#[derive(Debug, Clone)]
pub struct IfClause {
    pub cond: GuardSeq,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub body: exprs::Body,
}
derive_parse!(IfClause, cond, _arrow, body);
derive_token_range!(IfClause, cond, body);

#[derive(Debug, Clone)]
pub struct CatchClause {
    _position: commons::Void,
    pub class: Option<ExceptionClass>,
    pub pattern: Pattern,
    pub guard: Option<Guard>,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub body: exprs::Body,
}
derive_parse!(CatchClause, _position, class, pattern, guard, _arrow, body);
derive_token_range!(CatchClause, _position, body);

#[derive(Debug, Clone)]
pub struct ExceptionClass {
    pub class: commons::VarOrAtom,
    pub _colon: literals::S_COLON,
}
derive_parse!(ExceptionClass, class, _colon);
derive_token_range!(ExceptionClass, class, _colon);

#[derive(Debug, Clone)]
pub struct FunClause<N> {
    pub name: N,
    pub patterns: commons::Args<Pattern>,
    pub guard: Option<Guard>,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub body: exprs::Body,
}
derive_parse!(FunClause<N>, name, patterns, guard, _arrow, body);
derive_token_range!(FunClause<N>, name, body);
