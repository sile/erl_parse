use erl_tokenize::{LexicalToken, PositionRange, Position};

use {Result, ParseLeftRecur, Preprocessor, Parser};
use cst::GuardTest;
use cst::building_blocks;
use cst::collections;

// TODO: 共通化
#[derive(Debug, Clone)]
pub struct RecordFieldAccess {
    pub record: GuardTest,
    pub index: RecordFieldIndex,
}
impl ParseLeftRecur for RecordFieldAccess {
    type Left = GuardTest;
    fn parse_left_recur<T>(parser: &mut Parser<T>, left: Self::Left) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(RecordFieldAccess {
            record: left,
            index: track!(parser.parse())?,
        })
    }
}
impl PositionRange for RecordFieldAccess {
    fn start_position(&self) -> Position {
        self.record.start_position()
    }
    fn end_position(&self) -> Position {
        self.index.end_position()
    }
}

pub type Tuple = collections::Tuple<GuardTest>;
pub type Map = collections::Map<GuardTest>;
pub type Record = collections::Record<GuardTest>;
pub type RecordFieldIndex = collections::RecordFieldIndex;
//pub type RecordFieldAccess = collections::RecordFieldAccess<GuardTest>;
pub type List = collections::List<GuardTest>;
pub type Bits = collections::Bits<GuardTest>;
pub type Parenthesized = building_blocks::Parenthesized<GuardTest>;

// TODO: s/GuardTest/AtomToken/
pub type LocalCall = building_blocks::LocalCall<GuardTest>;
pub type RemoteCall = building_blocks::RemoteCall<GuardTest>;

pub type UnaryOpCall = building_blocks::UnaryOpCall<GuardTest>;
pub type BinaryOpCall = building_blocks::BinaryOpCall<GuardTest>;
