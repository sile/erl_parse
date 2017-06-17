use cst::Pattern;
use cst::building_blocks;
use cst::collections;

pub type Tuple = collections::Tuple<Pattern>;
pub type Map = collections::Map<Pattern>;
pub type Record = collections::Record<Pattern>;
pub type RecordFieldIndex = collections::RecordFieldIndex;
pub type List = collections::List<Pattern>;
pub type Bits = collections::Bits<Pattern>;
pub type Parenthesized = building_blocks::Parenthesized<Pattern>;
pub type UnaryOpCall = building_blocks::UnaryOpCall<Pattern>;
pub type BinaryOpCall = building_blocks::BinaryOpCall<Pattern>;
pub type Match = building_blocks::Match<Pattern>;
