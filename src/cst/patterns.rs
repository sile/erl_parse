use cst::{LeftPattern, Pattern};
use cst::commons;

pub type BitStr<'token> = commons::BitStr<'token, Pattern<'token>, LeftPattern<'token>>;
pub type List<'token> = commons::List<Pattern<'token>>;
pub type TailConsList<'token> = commons::TailConsList<Pattern<'token>>;
pub type Map<'token> = commons::Map<Pattern<'token>>;
pub type Parenthesized<'token> = commons::Parenthesized<Pattern<'token>>;
pub type Tuple<'token> = commons::Tuple<Pattern<'token>>;
pub type Record<'token> = commons::Record<'token, Pattern<'token>>;
pub type RecordFieldIndex<'token> = commons::RecordFieldIndex<'token>;
pub type UnaryOpCall<'token> = commons::UnaryNumOpCall<Pattern<'token>>;
pub type BinaryOpCall<'token> = commons::BinaryOpCall<LeftPattern<'token>, Pattern<'token>>;
pub type Match<'token> = commons::Match<'token, Pattern<'token>>;
