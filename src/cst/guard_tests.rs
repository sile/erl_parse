use cst::{LeftGuardTest, GuardTest};
use cst::commons;
use cst::literals;

pub type Parenthesized<'token> = commons::Parenthesized<GuardTest<'token>>;
pub type BitStr<'token> = commons::BitStr<'token, GuardTest<'token>, LeftGuardTest<'token>>;
pub type Tuple<'token> = commons::Tuple<GuardTest<'token>>;
pub type Map<'token> = commons::Map<GuardTest<'token>>;
pub type Record<'token> = commons::Record<'token, GuardTest<'token>>;
pub type RecordFieldIndex<'token> = commons::RecordFieldIndex<'token>;
pub type RecordFieldAccess<'token> = commons::RecordFieldAccess<'token, LeftGuardTest<'token>>;
pub type List<'token> = commons::List<GuardTest<'token>>;
pub type TailConsList<'token> = commons::TailConsList<GuardTest<'token>>;
pub type UnaryOpCall<'token> = commons::UnaryOpCall<GuardTest<'token>>;
pub type BinaryOpCall<'token> = commons::BinaryOpCall<LeftGuardTest<'token>, GuardTest<'token>>;
pub type LocalCall<'token> = commons::LocalCall<literals::Atom<'token>, GuardTest<'token>>;
pub type RemoteCall<'token> = commons::RemoteCall<literals::A_ERLANG,
                                                  literals::Atom<'token>,
                                                  GuardTest<'token>>;
