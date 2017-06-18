use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{AtomToken, SymbolToken, VariableToken, IntegerToken, KeywordToken};
use erl_tokenize::values::{Symbol, Keyword};

use {Result, Parser, ErrorKind};
use traits::{Parse, ParseTail, TokenRead};
use cst::{Pattern, Expr, Type, GuardTest};
use cst::collections::RecordFieldIndex;

#[derive(Debug, Clone)]
pub struct Match<T> {
    pub pattern: Pattern,
    pub _match: SymbolToken,
    pub value: T,
}
impl<T: Parse> Parse for Match<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: TokenRead,
    {
        Ok(Match {
            pattern: track!(Pattern::parse_non_left_recor(parser))?,
            _match: track!(parser.expect(&Symbol::Match))?,
            value: track!(parser.parse())?,
        })
    }
}
impl<T: Parse> ParseTail for Match<T> {
    type Head = Pattern;
    fn parse_tail<U>(parser: &mut Parser<U>, head: Pattern) -> Result<Self>
    where
        U: TokenRead,
    {
        Ok(Match {
            pattern: head,
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
    Band(KeywordToken),
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
impl BinaryOp {
    pub fn from_token(token: LexicalToken) -> ::std::result::Result<Self, LexicalToken> {
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
                    _ => Err(s.into()),
                }
            }
            LexicalToken::Keyword(k) => {
                match k.value() {
                    Keyword::Div => Ok(BinaryOp::IntDiv(k)),
                    Keyword::Rem => Ok(BinaryOp::Rem(k)),
                    Keyword::Bor => Ok(BinaryOp::Bor(k)),
                    Keyword::Bxor => Ok(BinaryOp::Bxor(k)),
                    Keyword::Band => Ok(BinaryOp::Band(k)),
                    Keyword::Bsl => Ok(BinaryOp::Bsl(k)),
                    Keyword::Bsr => Ok(BinaryOp::Bsl(k)),
                    Keyword::Or => Ok(BinaryOp::Or(k)),
                    Keyword::Xor => Ok(BinaryOp::Xor(k)),
                    Keyword::Andalso => Ok(BinaryOp::Andalso(k)),
                    Keyword::Orelse => Ok(BinaryOp::Orelse(k)),
                    _ => Err(k.into()),
                }
            }
            _ => Err(token),
        }
    }
}
impl Parse for BinaryOp {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        let token = track!(parser.parse::<LexicalToken>())?;
        match Self::from_token(token) {
            Err(token) => track_panic!(ErrorKind::UnexpectedToken(token)),
            Ok(op) => Ok(op),
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
            BinaryOp::Band(ref t) => t.start_position(),
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
            BinaryOp::Band(ref t) => t.end_position(),
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
impl<T: Parse> ParseTail for BinaryOpCall<T> {
    type Head = T;
    fn parse_tail<U>(parser: &mut Parser<U>, head: Self::Head) -> Result<Self>
    where
        U: TokenRead,
    {
        Ok(BinaryOpCall {
            left: head,
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
        U: TokenRead,
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
impl UnaryOp {
    pub fn from_token(token: LexicalToken) -> ::std::result::Result<Self, LexicalToken> {
        match token {
            LexicalToken::Symbol(s) => {
                match s.value() {
                    Symbol::Plus => Ok(UnaryOp::Plus(s)),
                    Symbol::Hyphen => Ok(UnaryOp::Minus(s)),
                    _ => Err(s.into()),
                }
            }
            LexicalToken::Keyword(k) => {
                match k.value() {
                    Keyword::Not => Ok(UnaryOp::Not(k)),
                    Keyword::Bnot => Ok(UnaryOp::Bnot(k)),
                    _ => Err(k.into()),
                }
            }
            token => Err(token),
        }
    }
}
impl Parse for UnaryOp {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        let token = track!(parser.parse())?;
        match UnaryOp::from_token(token) {
            Err(token) => track_panic!(ErrorKind::UnexpectedToken(token)),
            Ok(op) => Ok(op),
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
pub struct Call<T, A = T> {
    pub module: Option<ModulePrefix<T>>,
    pub name: T,
    pub args: Args<A>,
}
impl<T: Parse, A: Parse> Parse for Call<T, A> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: TokenRead,
    {
        Ok(Call {
            module: track!(parser.parse())?,
            name: track!(T::parse_non_left_recor(parser))?,
            args: track!(parser.parse())?,
        })
    }
}
impl<T: Parse, A: Parse> ParseTail for Call<T, A> {
    type Head = T;
    fn parse_tail<U>(parser: &mut Parser<U>, head: Self::Head) -> Result<Self>
    where
        U: TokenRead,
    {
        if let Ok(_colon) = parser.transaction(|parser| parser.expect(&Symbol::Colon)) {
            Ok(Call {
                module: Some(ModulePrefix { name: head, _colon }),
                name: track!(T::parse_non_left_recor(parser))?,
                args: track!(parser.parse())?,
            })
        } else {
            Ok(Call {
                module: None,
                name: head,
                args: track!(parser.parse())?,
            })
        }
    }
}
impl<T: PositionRange, A> PositionRange for Call<T, A> {
    fn start_position(&self) -> Position {
        self.module
            .as_ref()
            .map(|x| x.start_position())
            .unwrap_or_else(|| self.name.start_position())
    }
    fn end_position(&self) -> Position {
        self.args.end_position()
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
        U: TokenRead,
    {
        Ok(Args {
            _open: track!(parser.expect(&Symbol::OpenParen))?,
            args: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseParen))?,
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
        U: TokenRead,
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
        U: TokenRead,
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
        U: TokenRead,
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
        U: TokenRead,
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
        U: TokenRead,
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
        U: TokenRead,
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
        U: TokenRead,
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
        U: TokenRead,
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
        U: TokenRead,
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
        U: TokenRead,
    {
        match track!(parser.parse())? {
            LexicalToken::Atom(token) => Ok(AtomOrVariable::Atom(token)),
            LexicalToken::Variable(token) => Ok(AtomOrVariable::Variable(token)),
            token => track_panic!(ErrorKind::UnexpectedToken(token)),
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
        U: TokenRead,
    {
        match track!(parser.parse())? {
            LexicalToken::Integer(token) => Ok(IntegerOrVariable::Integer(token)),
            LexicalToken::Variable(token) => Ok(IntegerOrVariable::Variable(token)),
            token => track_panic!(ErrorKind::UnexpectedToken(token)),
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

#[derive(Debug, Clone)]
pub struct List<T> {
    pub _open: SymbolToken,
    pub elements: Option<Sequence<T>>,
    pub _close: SymbolToken,
}
impl<T: Parse> Parse for List<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: TokenRead,
    {
        Ok(List {
            _open: track!(parser.expect(&Symbol::OpenSquare))?,
            elements: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseSquare))?,
        })
    }
}
impl<T> PositionRange for List<T> {
    fn start_position(&self) -> Position {
        self._open.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct NameAndArity<N, A = IntegerToken> {
    pub name: N,
    pub _slash: SymbolToken,
    pub arity: A,
}
impl<N: Parse, A: Parse> Parse for NameAndArity<N, A> {
    fn parse<T: TokenRead>(parser: &mut Parser<T>) -> Result<Self> {
        Ok(NameAndArity {
            name: track!(parser.parse())?,
            _slash: track!(parser.expect(&Symbol::Slash))?,
            arity: track!(parser.parse())?,
        })
    }
}
impl<N: PositionRange, A: PositionRange> PositionRange for NameAndArity<N, A> {
    fn start_position(&self) -> Position {
        self.name.start_position()
    }
    fn end_position(&self) -> Position {
        self.arity.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct ModulePrefix<T> {
    pub name: T,
    pub _colon: SymbolToken,
}
impl<T: Parse> Parse for ModulePrefix<T> {
    fn parse<U: TokenRead>(parser: &mut Parser<U>) -> Result<Self> {
        Ok(ModulePrefix {
            name: track!(T::parse_non_left_recor(parser))?,
            _colon: track!(parser.expect(&Symbol::Colon))?,
        })
    }
}
impl<T: PositionRange> PositionRange for ModulePrefix<T> {
    fn start_position(&self) -> Position {
        self.name.start_position()
    }
    fn end_position(&self) -> Position {
        self._colon.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct RecordFieldDecl {
    pub field_name: AtomToken,
    pub field_default: Option<RecordFieldDefault>,
    pub field_type: Option<RecordFieldType>,
}
impl Parse for RecordFieldDecl {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(RecordFieldDecl {
            field_name: track!(parser.parse())?,
            field_default: track!(parser.parse())?,
            field_type: track!(parser.parse())?,
        })
    }
}
impl PositionRange for RecordFieldDecl {
    fn start_position(&self) -> Position {
        self.field_name.start_position()
    }
    fn end_position(&self) -> Position {
        self.field_type
            .as_ref()
            .map(|t| t.end_position())
            .or_else(|| self.field_default.as_ref().map(|t| t.end_position()))
            .unwrap_or_else(|| self.field_name.end_position())
    }
}

#[derive(Debug, Clone)]
pub struct RecordFieldDefault {
    pub _match: SymbolToken,
    pub value: Expr,
}
impl Parse for RecordFieldDefault {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(RecordFieldDefault {
            _match: track!(parser.expect(&Symbol::Match))?,
            value: track!(parser.parse())?,
        })
    }
}
impl PositionRange for RecordFieldDefault {
    fn start_position(&self) -> Position {
        self._match.start_position()
    }
    fn end_position(&self) -> Position {
        self.value.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct RecordFieldType {
    pub _double_colon: SymbolToken,
    pub field_type: Type,
}
impl Parse for RecordFieldType {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(RecordFieldType {
            _double_colon: track!(parser.expect(&Symbol::DoubleColon))?,
            field_type: track!(parser.parse())?,
        })
    }
}
impl PositionRange for RecordFieldType {
    fn start_position(&self) -> Position {
        self._double_colon.start_position()
    }
    fn end_position(&self) -> Position {
        self.field_type.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct ListElement {
    pub element_type: Type,
    pub non_empty: Option<NonEmpty>,
}
impl Parse for ListElement {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(ListElement {
            element_type: track!(parser.parse())?,
            non_empty: track!(parser.parse())?,
        })
    }
}
impl PositionRange for ListElement {
    fn start_position(&self) -> Position {
        self.element_type.start_position()
    }
    fn end_position(&self) -> Position {
        self.non_empty
            .as_ref()
            .map(|t| t.end_position())
            .unwrap_or_else(|| self.element_type.end_position())
    }
}

#[derive(Debug, Clone)]
pub struct NonEmpty {
    pub _comma: SymbolToken,
    pub _triple_dot: SymbolToken,
}
impl Parse for NonEmpty {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(NonEmpty {
            _comma: track!(parser.expect(&Symbol::Comma))?,
            _triple_dot: track!(parser.expect(&Symbol::TripleDot))?,
        })
    }
}
impl PositionRange for NonEmpty {
    fn start_position(&self) -> Position {
        self._comma.start_position()
    }
    fn end_position(&self) -> Position {
        self._triple_dot.end_position()
    }
}


#[derive(Debug, Clone)]
pub struct ByteAndBitSize {
    pub byte: ByteSize,
    pub _comma: SymbolToken,
    pub bit: BitSize,
}
impl Parse for ByteAndBitSize {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(ByteAndBitSize {
            byte: track!(parser.parse())?,
            _comma: track!(parser.expect(&Symbol::Comma))?,
            bit: track!(parser.parse())?,
        })
    }
}
impl PositionRange for ByteAndBitSize {
    fn start_position(&self) -> Position {
        self.byte.start_position()
    }
    fn end_position(&self) -> Position {
        self.bit.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct ByteSize {
    pub _underscore: VariableToken,
    pub _colon: SymbolToken,
    pub size: IntegerToken,
}
impl Parse for ByteSize {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(ByteSize {
            _underscore: track!(parser.expect("_"))?,
            _colon: track!(parser.expect(&Symbol::Colon))?,
            size: track!(parser.parse())?,
        })
    }
}
impl PositionRange for ByteSize {
    fn start_position(&self) -> Position {
        self._underscore.start_position()
    }
    fn end_position(&self) -> Position {
        self.size.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct BitSize {
    pub _underscore0: VariableToken,
    pub _colon: SymbolToken,
    pub _underscore1: VariableToken,
    pub _asterisk: SymbolToken,
    pub size: IntegerToken,
}
impl Parse for BitSize {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(BitSize {
            _underscore0: track!(parser.expect("_"))?,
            _colon: track!(parser.expect(&Symbol::Colon))?,
            _underscore1: track!(parser.expect("_"))?,
            _asterisk: track!(parser.expect(&Symbol::Multiply))?,
            size: track!(parser.parse())?,
        })
    }
}
impl PositionRange for BitSize {
    fn start_position(&self) -> Position {
        self._underscore0.start_position()
    }
    fn end_position(&self) -> Position {
        self.size.end_position()
    }
}

#[derive(Debug, Clone)]
pub enum BitsSpec {
    BytesAndBits(ByteAndBitSize),
    Bytes(ByteSize),
    Bits(BitSize),
}
impl Parse for BitsSpec {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        if let Ok(x) = parser.transaction(|parser| parser.parse()) {
            Ok(BitsSpec::BytesAndBits(x))
        } else if let Ok(x) = parser.transaction(|parser| parser.parse()) {
            Ok(BitsSpec::Bytes(x))
        } else {
            Ok(BitsSpec::Bits(track!(parser.parse())?))
        }
    }
}
impl PositionRange for BitsSpec {
    fn start_position(&self) -> Position {
        match *self {
            BitsSpec::BytesAndBits(ref t) => t.start_position(),
            BitsSpec::Bytes(ref t) => t.start_position(),
            BitsSpec::Bits(ref t) => t.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            BitsSpec::BytesAndBits(ref t) => t.end_position(),
            BitsSpec::Bytes(ref t) => t.end_position(),
            BitsSpec::Bits(ref t) => t.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RecordFieldAccess<T> {
    pub record: T,
    pub index: RecordFieldIndex,
}
impl<T> ParseTail for RecordFieldAccess<T> {
    type Head = T;
    fn parse_tail<U: TokenRead>(parser: &mut Parser<U>, head: Self::Head) -> Result<Self> {
        Ok(RecordFieldAccess {
            record: head,
            index: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for RecordFieldAccess<T> {
    fn start_position(&self) -> Position {
        self.record.start_position()
    }
    fn end_position(&self) -> Position {
        self.index.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct ExceptionClass {
    pub class: AtomOrVariable,
    pub _colon: SymbolToken,
}
impl Parse for ExceptionClass {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(ExceptionClass {
            class: track!(parser.parse())?,
            _colon: track!(parser.expect(&Symbol::Colon))?,
        })
    }
}
impl PositionRange for ExceptionClass {
    fn start_position(&self) -> Position {
        self.class.start_position()
    }
    fn end_position(&self) -> Position {
        self._colon.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct WhenGuard {
    pub _when: KeywordToken,
    pub seq: Clauses<Sequence<GuardTest>>,
}
impl Parse for WhenGuard {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(WhenGuard {
            _when: track!(parser.expect(&Keyword::When))?,
            seq: track!(parser.parse())?,
        })
    }
}
impl PositionRange for WhenGuard {
    fn start_position(&self) -> Position {
        self._when.start_position()
    }
    fn end_position(&self) -> Position {
        self.seq.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Clauses<T> {
    pub item: T,
    pub tail: Option<ClausesTail<T>>,
}
impl<T: Parse> Parse for Clauses<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: TokenRead,
    {
        Ok(Clauses {
            item: track!(parser.parse())?,
            tail: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for Clauses<T> {
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
pub struct ClausesTail<T> {
    pub _semicolon: SymbolToken,
    pub item: T,
    pub tail: Option<Box<ClausesTail<T>>>,
}
impl<T: Parse> Parse for ClausesTail<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: TokenRead,
    {
        Ok(ClausesTail {
            _semicolon: track!(parser.expect(&Symbol::Semicolon))?,
            item: track!(parser.parse())?,
            tail: track!(parser.parse())?,
        })
    }
}
impl<T: PositionRange> PositionRange for ClausesTail<T> {
    fn start_position(&self) -> Position {
        self._semicolon.start_position()
    }
    fn end_position(&self) -> Position {
        self.tail
            .as_ref()
            .map(|t| t.end_position())
            .unwrap_or_else(|| self.item.end_position())
    }
}
