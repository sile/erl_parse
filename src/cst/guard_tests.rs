use cst::{LeftGuardTest, GuardTest};
use cst::commons;
use cst::literals;

pub type Parenthesized<'token, 'text> = commons::Parenthesized<GuardTest<'token, 'text>>;
pub type BitStr<'token, 'text> = commons::BitStr<'token,
                                                 'text,
                                                 GuardTest<'token, 'text>,
                                                 LeftGuardTest<'token, 'text>>;
pub type Tuple<'token, 'text> = commons::Tuple<GuardTest<'token, 'text>>;
pub type Map<'token, 'text> = commons::Map<GuardTest<'token, 'text>>;
pub type Record<'token, 'text> = commons::Record<'token, 'text, GuardTest<'token, 'text>>;
pub type RecordFieldIndex<'token, 'text> = commons::RecordFieldIndex<'token, 'text>;
pub type RecordFieldAccess<'token, 'text> = commons::RecordFieldAccess<'token,
                                                                       'text,
                                                                       LeftGuardTest<'token,
                                                                                     'text>>;
pub type List<'token, 'text> = commons::List<GuardTest<'token, 'text>>;
pub type TailConsList<'token, 'text> = commons::TailConsList<GuardTest<'token, 'text>>;
pub type UnaryOpCall<'token, 'text> = commons::UnaryOpCall<GuardTest<'token, 'text>>;
pub type BinaryOpCall<'token, 'text> = commons::BinaryOpCall<LeftGuardTest<'token, 'text>,
                                                             GuardTest<'token, 'text>>;
pub type LocalCall<'token, 'text> = commons::LocalCall<literals::Atom<'token, 'text>,
                                                       GuardTest<'token, 'text>>;
pub type RemoteCall<'token, 'text> = commons::RemoteCall<literals::A_ERLANG,
                                                         literals::Atom<'token, 'text>,
                                                         GuardTest<'token, 'text>>;
