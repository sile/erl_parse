use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{AtomToken, SymbolToken, VariableToken, IntegerToken, KeywordToken};
use erl_tokenize::values::{Symbol, Keyword};

use {Result, Parse, Preprocessor, Parser, ErrorKind, ParseLeftRecur, TryInto};
use cst::Pattern;

#[derive(Debug, Clone)]
pub struct Match<T> {
    pub pattern: Pattern,
    pub _match: SymbolToken,
    pub value: T,
}
impl<T: Parse> Parse for Match<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Match {
            pattern: track!(parser.parse())?,
            _match: track!(parser.expect(&Symbol::Match))?,
            value: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for Match<T> {
    fn start_position(&self) -> Position {
        self.pattern.start_position()
    }
    fn end_position(&self) -> Position {
        self.value.end_position()
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Plus(SymbolToken),
    Minus(SymbolToken),
    Mul(SymbolToken),
    FloatDiv(SymbolToken),
    IntDiv(KeywordToken),
    Rem(KeywordToken),
    Bor(KeywordToken),
    Bxor(KeywordToken),
    Bsl(KeywordToken),
    Bsr(KeywordToken),
    Or(KeywordToken),
    Xor(KeywordToken),
    PlusPlus(SymbolToken),
    MinusMinus(SymbolToken),
    Eq(SymbolToken),
    ExactEq(SymbolToken),
    NotEq(SymbolToken),
    ExactNotEq(SymbolToken),
    Less(SymbolToken),
    LessEq(SymbolToken),
    Greater(SymbolToken),
    GreaterEq(SymbolToken),
    Andalso(KeywordToken),
    Orelse(KeywordToken),
    Send(SymbolToken),
}
impl Parse for BinaryOp {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let token = track!(parser.read_token())?;
        match token {
            LexicalToken::Symbol(s) => {
                match s.value() {
                    Symbol::Plus => Ok(BinaryOp::Plus(s)),
                    Symbol::Hyphen => Ok(BinaryOp::Minus(s)),
                    Symbol::Multiply => Ok(BinaryOp::Mul(s)),
                    Symbol::Slash => Ok(BinaryOp::FloatDiv(s)),
                    Symbol::PlusPlus => Ok(BinaryOp::PlusPlus(s)),
                    Symbol::MinusMinus => Ok(BinaryOp::MinusMinus(s)),
                    Symbol::Eq => Ok(BinaryOp::Eq(s)),
                    Symbol::ExactEq => Ok(BinaryOp::ExactEq(s)),
                    Symbol::NotEq => Ok(BinaryOp::NotEq(s)),
                    Symbol::ExactNotEq => Ok(BinaryOp::ExactNotEq(s)),
                    Symbol::Less => Ok(BinaryOp::Less(s)),
                    Symbol::LessEq => Ok(BinaryOp::LessEq(s)),
                    Symbol::Greater => Ok(BinaryOp::Greater(s)),
                    Symbol::GreaterEq => Ok(BinaryOp::GreaterEq(s)),
                    Symbol::Not => Ok(BinaryOp::Send(s)),
                    _ => track_panic!(ErrorKind::UnexpectedToken(s.into())),
                }
            }
            LexicalToken::Keyword(k) => {
                match k.value() {
                    Keyword::Div => Ok(BinaryOp::IntDiv(k)),
                    Keyword::Rem => Ok(BinaryOp::Rem(k)),
                    Keyword::Bor => Ok(BinaryOp::Bor(k)),
                    Keyword::Bxor => Ok(BinaryOp::Bxor(k)),
                    Keyword::Bsl => Ok(BinaryOp::Bsl(k)),
                    Keyword::Bsr => Ok(BinaryOp::Bsl(k)),
                    Keyword::Or => Ok(BinaryOp::Or(k)),
                    Keyword::Xor => Ok(BinaryOp::Xor(k)),
                    Keyword::Andalso => Ok(BinaryOp::Andalso(k)),
                    Keyword::Orelse => Ok(BinaryOp::Orelse(k)),
                    _ => track_panic!(ErrorKind::UnexpectedToken(k.into())),
                }
            }
            _ => track_panic!(ErrorKind::UnexpectedToken(token)),
        }
    }
}
impl PositionRange for BinaryOp {
    fn start_position(&self) -> Position {
        match *self {
            BinaryOp::Plus(ref t) => t.start_position(),
            BinaryOp::Minus(ref t) => t.start_position(),
            BinaryOp::Mul(ref t) => t.start_position(),
            BinaryOp::FloatDiv(ref t) => t.start_position(),
            BinaryOp::IntDiv(ref t) => t.start_position(),
            BinaryOp::Rem(ref t) => t.start_position(),
            BinaryOp::Bor(ref t) => t.start_position(),
            BinaryOp::Bxor(ref t) => t.start_position(),
            BinaryOp::Bsl(ref t) => t.start_position(),
            BinaryOp::Bsr(ref t) => t.start_position(),
            BinaryOp::Or(ref t) => t.start_position(),
            BinaryOp::Xor(ref t) => t.start_position(),
            BinaryOp::PlusPlus(ref t) => t.start_position(),
            BinaryOp::MinusMinus(ref t) => t.start_position(),
            BinaryOp::Eq(ref t) => t.start_position(),
            BinaryOp::ExactEq(ref t) => t.start_position(),
            BinaryOp::NotEq(ref t) => t.start_position(),
            BinaryOp::ExactNotEq(ref t) => t.start_position(),
            BinaryOp::Less(ref t) => t.start_position(),
            BinaryOp::LessEq(ref t) => t.start_position(),
            BinaryOp::Greater(ref t) => t.start_position(),
            BinaryOp::GreaterEq(ref t) => t.start_position(),
            BinaryOp::Andalso(ref t) => t.start_position(),
            BinaryOp::Orelse(ref t) => t.start_position(),
            BinaryOp::Send(ref t) => t.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            BinaryOp::Plus(ref t) => t.end_position(),
            BinaryOp::Minus(ref t) => t.end_position(),
            BinaryOp::Mul(ref t) => t.end_position(),
            BinaryOp::FloatDiv(ref t) => t.end_position(),
            BinaryOp::IntDiv(ref t) => t.end_position(),
            BinaryOp::Rem(ref t) => t.end_position(),
            BinaryOp::Bor(ref t) => t.end_position(),
            BinaryOp::Bxor(ref t) => t.end_position(),
            BinaryOp::Bsl(ref t) => t.end_position(),
            BinaryOp::Bsr(ref t) => t.end_position(),
            BinaryOp::Or(ref t) => t.end_position(),
            BinaryOp::Xor(ref t) => t.end_position(),
            BinaryOp::PlusPlus(ref t) => t.end_position(),
            BinaryOp::MinusMinus(ref t) => t.end_position(),
            BinaryOp::Eq(ref t) => t.end_position(),
            BinaryOp::ExactEq(ref t) => t.end_position(),
            BinaryOp::NotEq(ref t) => t.end_position(),
            BinaryOp::ExactNotEq(ref t) => t.end_position(),
            BinaryOp::Less(ref t) => t.end_position(),
            BinaryOp::LessEq(ref t) => t.end_position(),
            BinaryOp::Greater(ref t) => t.end_position(),
            BinaryOp::GreaterEq(ref t) => t.end_position(),
            BinaryOp::Andalso(ref t) => t.end_position(),
            BinaryOp::Orelse(ref t) => t.end_position(),
            BinaryOp::Send(ref t) => t.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BinaryOpCall<T> {
    pub left: T,
    pub op: BinaryOp,
    pub right: T,
}
impl<T: Parse> ParseLeftRecur for BinaryOpCall<T> {
    type Left = T;
    fn parse_left_recur<U>(parser: &mut Parser<U>, left: Self::Left) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(BinaryOpCall {
            left,
            op: track!(parser.parse())?,
            right: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for BinaryOpCall<T> {
    fn start_position(&self) -> Position {
        self.left.start_position()
    }
    fn end_position(&self) -> Position {
        self.right.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct UnaryOpCall<T> {
    pub op: UnaryOp,
    pub operand: T,
}
impl<T: Parse> Parse for UnaryOpCall<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(UnaryOpCall {
            op: track!(parser.parse())?,
            operand: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for UnaryOpCall<T> {
    fn start_position(&self) -> Position {
        self.op.start_position()
    }
    fn end_position(&self) -> Position {
        self.operand.end_position()
    }
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Plus(SymbolToken),
    Minus(SymbolToken),
    Not(KeywordToken),
    Bnot(KeywordToken),
}
impl Parse for UnaryOp {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let token = track!(parser.read_token())?;
        match token {
            LexicalToken::Symbol(s) => {
                match s.value() {
                    Symbol::Plus => Ok(UnaryOp::Plus(s)),
                    Symbol::Hyphen => Ok(UnaryOp::Minus(s)),
                    _ => track_panic!(ErrorKind::UnexpectedToken(s.into())),
                }
            }
            LexicalToken::Keyword(k) => {
                match k.value() {
                    Keyword::Not => Ok(UnaryOp::Not(k)),
                    Keyword::Bnot => Ok(UnaryOp::Bnot(k)),
                    _ => track_panic!(ErrorKind::UnexpectedToken(k.into())),
                }
            }
            _ => track_panic!(ErrorKind::UnexpectedToken(token)),
        }
    }
}
impl PositionRange for UnaryOp {
    fn start_position(&self) -> Position {
        match *self {
            UnaryOp::Plus(ref t) => t.start_position(),
            UnaryOp::Minus(ref t) => t.start_position(),
            UnaryOp::Not(ref t) => t.start_position(),
            UnaryOp::Bnot(ref t) => t.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            UnaryOp::Plus(ref t) => t.end_position(),
            UnaryOp::Minus(ref t) => t.end_position(),
            UnaryOp::Not(ref t) => t.end_position(),
            UnaryOp::Bnot(ref t) => t.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalCall<T> {
    pub name: T,
    pub args: Args<T>,
}
impl<T: Parse> ParseLeftRecur for LocalCall<T> {
    type Left = T;
    fn parse_left_recur<U>(parser: &mut Parser<U>, left: Self::Left) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(LocalCall {
            name: left,
            args: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for LocalCall<T> {
    fn start_position(&self) -> Position {
        self.name.start_position()
    }
    fn end_position(&self) -> Position {
        self.args.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct RemoteCall<T> {
    pub module_name: T,
    pub _colon: SymbolToken,
    pub fun: LocalCall<T>,
}
impl<T: Parse + TryInto<LocalCall<T>>> ParseLeftRecur for RemoteCall<T> {
    type Left = T;
    fn parse_left_recur<U>(parser: &mut Parser<U>, left: Self::Left) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(RemoteCall {
            module_name: left,
            _colon: track!(parser.expect(&Symbol::Colon))?,
            fun: track!(parser.parse::<T>().and_then(|t| t.try_into()))?,
        })
    }
}
impl<T: PositionRange> PositionRange for RemoteCall<T> {
    fn start_position(&self) -> Position {
        self.module_name.start_position()
    }
    fn end_position(&self) -> Position {
        self.fun.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Args<T> {
    pub _open: SymbolToken,
    pub args: Option<Sequence<T>>,
    pub _close: SymbolToken,
}
impl<T: Parse> Parse for Args<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let _open = track!(parser.expect(&Symbol::OpenParen))?;
        let args = track!(parser.parse())?;
        let _close = track!(parser.expect(&Symbol::CloseParen))?;
        Ok(Args {
            _open, //: track!(parser.expect(&Symbol::OpenParen))?,
            args, //: track!(parser.parse())?,
            _close, //: track!(parser.expect(&Symbol::CloseParen))?,
        })
    }
}
impl<T> PositionRange for Args<T> {
    fn start_position(&self) -> Position {
        self._open.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Parenthesized<T> {
    pub _open: SymbolToken,
    pub item: T,
    pub _close: SymbolToken,
}
impl<T: Parse> Parse for Parenthesized<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Parenthesized {
            _open: track!(parser.expect(&Symbol::OpenParen))?,
            item: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseParen))?,
        })
    }
}
impl<T> PositionRange for Parenthesized<T> {
    fn start_position(&self) -> Position {
        self._open.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Sequence<T> {
    pub item: T,
    pub tail: Option<SequenceTail<T>>,
}
impl<T> Sequence<T> {
    pub fn iter(&self) -> SequenceIter<T> {
        let inner = SequenceIterInner::Head(self);
        SequenceIter(inner)
    }
}
impl<T: Parse> Parse for Sequence<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Sequence {
            item: track!(parser.parse())?,
            tail: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for Sequence<T> {
    fn start_position(&self) -> Position {
        self.item.start_position()
    }
    fn end_position(&self) -> Position {
        self.tail
            .as_ref()
            .map(|t| t.end_position())
            .unwrap_or_else(|| self.item.end_position())
    }
}

#[derive(Debug, Clone)]
pub struct SequenceTail<T> {
    pub _comma: SymbolToken,
    pub item: T,
    pub tail: Option<Box<SequenceTail<T>>>,
}
impl<T: Parse> Parse for SequenceTail<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(SequenceTail {
            _comma: track!(parser.expect(&Symbol::Comma))?,
            item: track!(parser.parse())?,
            tail: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for SequenceTail<T> {
    fn start_position(&self) -> Position {
        self._comma.start_position()
    }
    fn end_position(&self) -> Position {
        self.tail
            .as_ref()
            .map(|t| t.end_position())
            .unwrap_or_else(|| self.item.end_position())
    }
}

#[derive(Debug)]
pub struct SequenceIter<'a, T: 'a>(SequenceIterInner<'a, T>);
impl<'a, T: 'a> Iterator for SequenceIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[derive(Debug)]
enum SequenceIterInner<'a, T: 'a> {
    Head(&'a Sequence<T>),
    Tail(&'a SequenceTail<T>),
    Eos,
}
impl<'a, T: 'a> Iterator for SequenceIterInner<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        match *self {
            SequenceIterInner::Head(&Sequence { ref item, ref tail }) => {
                if let Some(ref tail) = *tail {
                    *self = SequenceIterInner::Tail(tail);
                } else {
                    *self = SequenceIterInner::Eos
                }
                Some(item)
            }
            SequenceIterInner::Tail(&SequenceTail { ref item, ref tail, .. }) => {
                if let Some(ref tail) = *tail {
                    *self = SequenceIterInner::Tail(tail);
                } else {
                    *self = SequenceIterInner::Eos
                }
                Some(item)
            }
            SequenceIterInner::Eos => None,
        }
    }
}

// TODO
#[derive(Debug, Clone)]
pub struct HyphenSeq<T> {
    pub item: T,
    pub tail: Option<HyphenSeqTail<T>>,
}
impl<T: Parse> Parse for HyphenSeq<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(HyphenSeq {
            item: track!(parser.parse())?,
            tail: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for HyphenSeq<T> {
    fn start_position(&self) -> Position {
        self.item.start_position()
    }
    fn end_position(&self) -> Position {
        self.tail
            .as_ref()
            .map(|t| t.end_position())
            .unwrap_or_else(|| self.item.end_position())
    }
}

#[derive(Debug, Clone)]
pub struct HyphenSeqTail<T> {
    pub _hyphen: SymbolToken,
    pub item: T,
    pub tail: Option<Box<HyphenSeqTail<T>>>,
}
impl<T: Parse> Parse for HyphenSeqTail<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(HyphenSeqTail {
            _hyphen: track!(parser.expect(&Symbol::Hyphen))?,
            item: track!(parser.parse())?,
            tail: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for HyphenSeqTail<T> {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self.tail
            .as_ref()
            .map(|t| t.end_position())
            .unwrap_or_else(|| self.item.end_position())
    }
}

#[derive(Debug, Clone)]
pub struct ConsCell<T> {
    pub item: T,
    pub tail: Option<ConsCellTail<T>>,
}
impl<T: Parse> Parse for ConsCell<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(ConsCell {
            item: track!(parser.parse())?,
            tail: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for ConsCell<T> {
    fn start_position(&self) -> Position {
        self.item.start_position()
    }
    fn end_position(&self) -> Position {
        self.tail
            .as_ref()
            .map(|t| t.end_position())
            .unwrap_or_else(|| self.item.end_position())
    }
}

#[derive(Debug, Clone)]
pub enum ConsCellTail<T> {
    Proper {
        _comma: SymbolToken,
        item: T,
        tail: Option<Box<ConsCellTail<T>>>,
    },
    Improper { _bar: SymbolToken, item: T },
}
impl<T: Parse> Parse for ConsCellTail<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let symbol: SymbolToken = track!(parser.parse())?;
        match symbol.value() {
            Symbol::Comma => {
                Ok(ConsCellTail::Proper {
                    _comma: symbol,
                    item: track!(parser.parse())?,
                    tail: track!(parser.parse())?,
                })
            }
            Symbol::VerticalBar => {
                Ok(ConsCellTail::Improper {
                    _bar: symbol,
                    item: track!(parser.parse())?,
                })
            }
            _ => {
                track_panic!(ErrorKind::InvalidInput, "Unexpected symbol: {:?}", symbol);
            }
        }
    }
}
impl<T: PositionRange> PositionRange for ConsCellTail<T> {
    fn start_position(&self) -> Position {
        match *self {
            ConsCellTail::Proper { ref _comma, .. } => _comma.start_position(),
            ConsCellTail::Improper { ref _bar, .. } => _bar.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            ConsCellTail::Proper { ref item, ref tail, .. } => {
                tail.as_ref().map(|t| t.end_position()).unwrap_or_else(|| {
                    item.end_position()
                })
            }
            ConsCellTail::Improper { ref item, .. } => item.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MapField<T> {
    pub key: T,
    pub _relation: SymbolToken,
    pub value: T,
}
impl<T: Parse> Parse for MapField<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(MapField {
            key: track!(parser.parse())?,
            _relation: track!(parser.expect_any(
                &[&Symbol::DoubleRightArrow, &Symbol::MapMatch],
            ))?,
            value: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for MapField<T> {
    fn start_position(&self) -> Position {
        self.key.start_position()
    }
    fn end_position(&self) -> Position {
        self.value.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct RecordField<T> {
    pub key: AtomOrVariable,
    pub _bind: SymbolToken,
    pub value: T,
}
impl<T: Parse> Parse for RecordField<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(RecordField {
            key: track!(parser.parse())?,
            _bind: track!(parser.parse())?,
            value: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for RecordField<T> {
    fn start_position(&self) -> Position {
        self.key.start_position()
    }
    fn end_position(&self) -> Position {
        self.value.end_position()
    }
}

#[derive(Debug, Clone)]
pub enum AtomOrVariable {
    Atom(AtomToken),
    Variable(VariableToken),
}
impl AtomOrVariable {
    pub fn value(&self) -> &str {
        match *self {
            AtomOrVariable::Atom(ref t) => t.value(),
            AtomOrVariable::Variable(ref t) => t.value(),
        }
    }
    pub fn text(&self) -> &str {
        match *self {
            AtomOrVariable::Atom(ref t) => t.text(),
            AtomOrVariable::Variable(ref t) => t.text(),
        }
    }
}
impl Parse for AtomOrVariable {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let token = track!(parser.read_token())?;
        match token {
            LexicalToken::Atom(token) => Ok(AtomOrVariable::Atom(token)),
            LexicalToken::Variable(token) => Ok(AtomOrVariable::Variable(token)),
            _ => track_panic!(ErrorKind::UnexpectedToken(token)),
        }
    }
}
impl PositionRange for AtomOrVariable {
    fn start_position(&self) -> Position {
        match *self {
            AtomOrVariable::Atom(ref t) => t.start_position(),
            AtomOrVariable::Variable(ref t) => t.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            AtomOrVariable::Atom(ref t) => t.end_position(),
            AtomOrVariable::Variable(ref t) => t.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IntegerOrVariable {
    Integer(IntegerToken),
    Variable(VariableToken),
}
impl IntegerOrVariable {
    pub fn text(&self) -> &str {
        match *self {
            IntegerOrVariable::Integer(ref t) => t.text(),
            IntegerOrVariable::Variable(ref t) => t.text(),
        }
    }
}
impl Parse for IntegerOrVariable {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let token = track!(parser.read_token())?;
        match token {
            LexicalToken::Integer(token) => Ok(IntegerOrVariable::Integer(token)),
            LexicalToken::Variable(token) => Ok(IntegerOrVariable::Variable(token)),
            _ => track_panic!(ErrorKind::UnexpectedToken(token)),
        }
    }
}
impl PositionRange for IntegerOrVariable {
    fn start_position(&self) -> Position {
        match *self {
            IntegerOrVariable::Integer(ref t) => t.start_position(),
            IntegerOrVariable::Variable(ref t) => t.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            IntegerOrVariable::Integer(ref t) => t.end_position(),
            IntegerOrVariable::Variable(ref t) => t.end_position(),
        }
    }
}
