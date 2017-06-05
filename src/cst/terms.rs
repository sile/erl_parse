use cst::Term;
use cst::commons;

pub type Parenthesized<'token, 'text> = commons::Parenthesized<Term<'token, 'text>>;
pub type BitStr<'token, 'text> = commons::BitStr<'token,
                                                 'text,
                                                 Term<'token, 'text>,
                                                 Term<'token, 'text>>;
pub type Tuple<'token, 'text> = commons::Tuple<Term<'token, 'text>>;
pub type Map<'token, 'text> = commons::Map<Term<'token, 'text>>;
pub type Record<'token, 'text> = commons::Record<'token, 'text, Term<'token, 'text>>;
pub type List<'token, 'text> = commons::List<Term<'token, 'text>>;
pub type TailConsList<'token, 'text> = commons::TailConsList<Term<'token, 'text>>;
