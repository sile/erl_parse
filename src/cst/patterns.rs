use cst::Pattern;
use cst::primitives::Seq2;
use cst::symbols;

#[derive(Debug)]
pub struct Tuple<'token, 'text: 'token> {
    pub _open: symbols::OpenBrace,
    pub elements: Seq2<Pattern<'token, 'text>, symbols::Comma>,
    pub _close: symbols::CloseBrace,
}
derive_parse!(Tuple, _open, elements, _close);
derive_token_range!(Tuple, _open, _close);

#[derive(Debug)]
pub struct List<'token, 'text: 'token> {
    pub _open: symbols::OpenSquare,
    pub elements: Seq2<Pattern<'token, 'text>, symbols::Comma>,
    // TODO: [|0]が許容されてしまう
    pub tail: Option<ListTail<'token, 'text>>,
    pub _close: symbols::CloseSquare,
}
derive_parse!(List, _open, elements, tail, _close);
derive_token_range!(List, _open, _close);

#[derive(Debug)]
pub struct ListTail<'token, 'text: 'token> {
    pub bar: symbols::VerticalBar,
    pub element: Pattern<'token, 'text>,
}
derive_parse!(ListTail, bar, element);
derive_token_range!(ListTail, bar, element);
