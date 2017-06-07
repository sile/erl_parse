use cst::Term;
use cst::commons;

pub type Parenthesized<'token> = commons::Parenthesized<Term<'token>>;
pub type BitStr<'token> = commons::BitStr<'token, Term<'token>, Term<'token>>;
pub type Tuple<'token> = commons::Tuple<Term<'token>>;
pub type Map<'token> = commons::Map<Term<'token>>;
pub type Record<'token> = commons::Record<'token, Term<'token>>;
pub type List<'token> = commons::List<Term<'token>>;
pub type TailConsList<'token> = commons::TailConsList<Term<'token>>;
