use cst::{LeftGuardTest, GuardTest};
use cst::commons;

pub type Parenthesized<'token, 'text> = commons::Parenthesized<GuardTest<'token, 'text>>;
pub type BitStr<'token, 'text> = commons::BitStr<'token,
                                                 'text,
                                                 GuardTest<'token, 'text>,
                                                 LeftGuardTest<'token, 'text>>;
pub type Tuple<'token, 'text> = commons::Tuple<GuardTest<'token, 'text>>;
pub type Map<'token, 'text> = commons::Map<GuardTest<'token, 'text>>;
pub type Record<'token, 'text> = commons::Record<'token, 'text, GuardTest<'token, 'text>>;
pub type RecordFieldIndex<'token, 'text> = commons::RecordFieldIndex<'token, 'text>;
pub type List<'token, 'text> = commons::List<GuardTest<'token, 'text>>;
pub type TailConsList<'token, 'text> = commons::TailConsList<GuardTest<'token, 'text>>;
pub type UnaryOpCall<'token, 'text> = commons::UnaryOpCall<GuardTest<'token, 'text>>;
