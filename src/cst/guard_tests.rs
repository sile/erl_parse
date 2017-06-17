use cst::GuardTest;
use cst::building_blocks;
use cst::collections;

pub type Tuple = collections::Tuple<GuardTest>;
pub type Map = collections::Map<GuardTest>;
pub type Record = collections::Record<GuardTest>;
pub type RecordFieldIndex = collections::RecordFieldIndex;
pub type List = collections::List<GuardTest>;
pub type Bits = collections::Bits<GuardTest>;
pub type Parenthesized = building_blocks::Parenthesized<GuardTest>;

// TODO: s/GuardTest/AtomToken/
pub type LocalCall = building_blocks::LocalCall<GuardTest>;
pub type RemoteCall = building_blocks::RemoteCall<GuardTest>;

pub type UnaryOpCall = building_blocks::UnaryOpCall<GuardTest>;
pub type BinaryOpCall = building_blocks::BinaryOpCall<GuardTest>;
