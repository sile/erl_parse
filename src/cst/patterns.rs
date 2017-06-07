use cst::{LeftPattern, Pattern};
use cst::commons;

pub type BitStr = commons::BitStr<Pattern, LeftPattern>;
pub type List = commons::List<Pattern>;
pub type TailConsList = commons::TailConsList<Pattern>;
pub type Map = commons::Map<Pattern>;
pub type Parenthesized = commons::Parenthesized<Pattern>;
pub type Tuple = commons::Tuple<Pattern>;
pub type Record = commons::Record<Pattern>;
pub type RecordFieldIndex = commons::RecordFieldIndex;
pub type UnaryOpCall = commons::UnaryNumOpCall<Pattern>;
pub type BinaryOpCall = commons::BinaryOpCall<LeftPattern, Pattern>;
pub type Match = commons::Match<Pattern>;
