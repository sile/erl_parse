use cst::Term;
use cst::commons;

pub type Parenthesized = commons::Parenthesized<Term>;
pub type BitStr = commons::BitStr<Term, Term>;
pub type Tuple = commons::Tuple<Term>;
pub type Map = commons::Map<Term>;
pub type Record = commons::Record<Term>;
pub type List = commons::List<Term>;
pub type TailConsList = commons::TailConsList<Term>;
