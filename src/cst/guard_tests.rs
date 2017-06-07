use cst::{LeftGuardTest, GuardTest};
use cst::commons;
use cst::literals;

pub type Parenthesized = commons::Parenthesized<GuardTest>;
pub type BitStr = commons::BitStr<GuardTest, LeftGuardTest>;
pub type Tuple = commons::Tuple<GuardTest>;
pub type Map = commons::Map<GuardTest>;
pub type Record = commons::Record<GuardTest>;
pub type RecordFieldIndex = commons::RecordFieldIndex;
pub type RecordFieldAccess = commons::RecordFieldAccess<LeftGuardTest>;
pub type List = commons::List<GuardTest>;
pub type TailConsList = commons::TailConsList<GuardTest>;
pub type UnaryOpCall = commons::UnaryOpCall<GuardTest>;
pub type BinaryOpCall = commons::BinaryOpCall<LeftGuardTest, GuardTest>;
pub type LocalCall = commons::LocalCall<literals::Atom, GuardTest>;
pub type RemoteCall = commons::RemoteCall<literals::A_ERLANG, literals::Atom, GuardTest>;
