use cst::Type;
use cst::commons;
use cst::literals;

#[derive(Debug, Clone)]
pub struct Annotated<'token, 'text: 'token> {
    pub var: commons::Var<'token, 'text>,
    pub _double_colon: literals::S_DOUBLE_COLON,
    pub ty: Type<'token, 'text>,
}
derive_parse!(Annotated, var, _double_colon, ty);
derive_token_range!(Annotated, var, ty);

#[derive(Debug, Clone)]
pub struct List<'token, 'text: 'token> {
    pub _open: literals::S_OPEN_SQUARE,
    pub elem: Option<Type<'token, 'text>>,
    pub _close: literals::S_CLOSE_SQUARE,
}
derive_parse!(List, _open, elem, _close);
derive_token_range!(List, _open, _close);

#[derive(Debug, Clone)]
pub struct Parenthesized<'token, 'text: 'token> {
    pub _open: literals::S_OPEN_PAREN,
    pub inner: Type<'token, 'text>,
    pub _close: literals::S_CLOSE_PAREN,
}
derive_parse!(Parenthesized, _open, inner, _close);
derive_token_range!(Parenthesized, _open, _close);
