use cst::{LeftPattern, Pattern};
use cst::commons;

pub type BitStr<'token, 'text> = commons::BitStr<'token,
                                                 'text,
                                                 Pattern<'token, 'text>,
                                                 LeftPattern<'token, 'text>>;
pub type List<'token, 'text> = commons::List<Pattern<'token, 'text>>;
pub type TailConsList<'token, 'text> = commons::TailConsList<Pattern<'token, 'text>>;
pub type Map<'token, 'text> = commons::Map<Pattern<'token, 'text>>;
pub type Parenthesized<'token, 'text> = commons::Parenthesized<Pattern<'token, 'text>>;
pub type Tuple<'token, 'text> = commons::Tuple<Pattern<'token, 'text>>;
pub type Record<'token, 'text> = commons::Record<'token, 'text, Pattern<'token, 'text>>;
pub type RecordFieldIndex<'token, 'text> = commons::RecordFieldIndex<'token, 'text>;
pub type UnaryOpCall<'token, 'text> = commons::UnaryNumOpCall<Pattern<'token, 'text>>;
pub type BinaryOpCall<'token, 'text> = commons::BinaryOpCall<LeftPattern<'token, 'text>,
                                                             Pattern<'token, 'text>>;
pub type Match<'token, 'text> = commons::Match<'token, 'text, Pattern<'token, 'text>>;
