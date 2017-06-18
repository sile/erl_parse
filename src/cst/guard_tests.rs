use erl_tokenize::tokens::AtomToken;

use cst::GuardTest;
use cst::building_blocks;
use cst::collections;

pub type Tuple = collections::Tuple<GuardTest>;
pub type Map = collections::Map<GuardTest>;
pub type Record = collections::Record<GuardTest>;
pub type RecordFieldIndex = collections::RecordFieldIndex;
pub type RecordFieldAccess = building_blocks::RecordFieldAccess<GuardTest>;
pub type List = collections::List<GuardTest>;
pub type Bits = collections::Bits<GuardTest>;
pub type Parenthesized = building_blocks::Parenthesized<GuardTest>;
pub type FunCall = building_blocks::Call<AtomToken, GuardTest>;
pub type UnaryOpCall = building_blocks::UnaryOpCall<GuardTest>;
pub type BinaryOpCall = building_blocks::BinaryOpCall<GuardTest>;
