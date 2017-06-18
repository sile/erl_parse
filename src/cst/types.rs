use erl_tokenize::{Position, PositionRange};
use erl_tokenize::tokens::{SymbolToken, VariableToken, IntegerToken, KeywordToken, AtomToken};
use erl_tokenize::values::{Symbol, Keyword};

use {Result, Parser};
use cst::Type;
use cst::building_blocks::{self, Args, Sequence};
use cst::collections;
use traits::{Parse, ParseTail, TokenRead};

#[derive(Debug, Clone)]
pub enum Fun {
    Any(AnyFun),
    AnyArity(AnyArityFun),
    Normal(NormalFun),
}
impl Parse for Fun {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        // TODO: look ahead
        if let Ok(x) = parser.transaction(|parser| parser.parse()) {
            Ok(Fun::Any(x))
        } else if let Ok(x) = parser.transaction(|parser| parser.parse()) {
            Ok(Fun::AnyArity(x))
        } else {
            Ok(Fun::Normal(track!(parser.parse())?))
        }
    }
}
impl PositionRange for Fun {
    fn start_position(&self) -> Position {
        match *self {
            Fun::Any(ref x) => x.start_position(),
            Fun::AnyArity(ref x) => x.start_position(),
            Fun::Normal(ref x) => x.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            Fun::Any(ref x) => x.end_position(),
            Fun::AnyArity(ref x) => x.end_position(),
            Fun::Normal(ref x) => x.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AnyFun {
    pub _fun: KeywordToken,
    pub _open: SymbolToken,
    pub _close: SymbolToken,
}
impl Parse for AnyFun {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(AnyFun {
            _fun: track!(parser.expect(&Keyword::Fun))?,
            _open: track!(parser.expect(&Symbol::OpenParen))?,
            _close: track!(parser.expect(&Symbol::CloseParen))?,
        })
    }
}
impl PositionRange for AnyFun {
    fn start_position(&self) -> Position {
        self._fun.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct AnyArityFun {
    pub _fun: KeywordToken,
    pub _open: SymbolToken,
    pub _args_open: SymbolToken,
    pub _args: SymbolToken,
    pub _args_close: SymbolToken,
    pub _arrow: SymbolToken,
    pub return_type: Type,
    pub _close: SymbolToken,
}
impl Parse for AnyArityFun {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(AnyArityFun {
            _fun: track!(parser.expect(&Keyword::Fun))?,
            _open: track!(parser.expect(&Symbol::OpenParen))?,
            _args_open: track!(parser.expect(&Symbol::OpenParen))?,
            _args: track!(parser.expect(&Symbol::TripleDot))?,
            _args_close: track!(parser.expect(&Symbol::CloseParen))?,
            _arrow: track!(parser.expect(&Symbol::RightArrow))?,
            return_type: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseParen))?,
        })
    }
}
impl PositionRange for AnyArityFun {
    fn start_position(&self) -> Position {
        self._fun.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct NormalFun {
    pub _fun: KeywordToken,
    pub _open: SymbolToken,
    pub args: Args<Type>,
    pub _arrow: SymbolToken,
    pub return_type: Type,
    pub _close: SymbolToken,
}
impl Parse for NormalFun {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(NormalFun {
            _fun: track!(parser.expect(&Keyword::Fun))?,
            _open: track!(parser.expect(&Symbol::OpenParen))?,
            args: track!(parser.parse())?,
            _arrow: track!(parser.expect(&Symbol::RightArrow))?,
            return_type: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseParen))?,
        })
    }
}
impl PositionRange for NormalFun {
    fn start_position(&self) -> Position {
        self._fun.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct FunConstraints {
    pub _when: KeywordToken,
    pub constraints: Sequence<FunConstraint>,
}
impl Parse for FunConstraints {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(FunConstraints {
            _when: track!(parser.expect(&Keyword::When))?,
            constraints: track!(parser.parse())?,
        })
    }
}
impl PositionRange for FunConstraints {
    fn start_position(&self) -> Position {
        self._when.start_position()
    }
    fn end_position(&self) -> Position {
        self.constraints.end_position()
    }
}

#[derive(Debug, Clone)]
pub enum FunConstraint {
    Annotated(Annotated),
    IsSubtype(IsSubtype),
}
impl Parse for FunConstraint {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        if let Ok(t) = parser.transaction(|parser| parser.parse()) {
            Ok(FunConstraint::Annotated(t))
        } else {
            let t = track!(parser.parse())?;
            Ok(FunConstraint::IsSubtype(t))
        }
    }
}
impl PositionRange for FunConstraint {
    fn start_position(&self) -> Position {
        match *self {
            FunConstraint::Annotated(ref t) => t.start_position(),
            FunConstraint::IsSubtype(ref t) => t.start_position(),
        }
    }
    fn end_position(&self) -> Position {
        match *self {
            FunConstraint::Annotated(ref t) => t.end_position(),
            FunConstraint::IsSubtype(ref t) => t.end_position(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IsSubtype {
    pub _is_subtype: AtomToken,
    pub _open: SymbolToken,
    pub var: VariableToken,
    pub _comma: SymbolToken,
    pub ty: Type,
    pub _close: SymbolToken,
}
impl Parse for IsSubtype {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(IsSubtype {
            _is_subtype: track!(parser.expect("is_subtype"))?,
            _open: track!(parser.expect(&Symbol::OpenParen))?,
            var: track!(parser.parse())?,
            _comma: track!(parser.expect(&Symbol::Comma))?,
            ty: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseParen))?,
        })
    }
}
impl PositionRange for IsSubtype {
    fn start_position(&self) -> Position {
        self._is_subtype.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Range {
    pub low: Type,
    pub _dot: SymbolToken,
    pub high: Type,
}
impl ParseTail for Range {
    type Head = Type;
    fn parse_tail<T>(parser: &mut Parser<T>, head: Self::Head) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(Range {
            low: head,
            _dot: track!(parser.expect(&Symbol::DoubleDot))?,
            high: track!(parser.parse())?,
        })
    }
}
impl PositionRange for Range {
    fn start_position(&self) -> Position {
        self.low.start_position()
    }
    fn end_position(&self) -> Position {
        self.high.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Union {
    pub left: Type,
    pub _or: SymbolToken,
    pub right: Type,
}
impl ParseTail for Union {
    type Head = Type;
    fn parse_tail<T>(parser: &mut Parser<T>, head: Self::Head) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(Union {
            left: head,
            _or: track!(parser.expect(&Symbol::VerticalBar))?,
            right: track!(parser.parse())?,
        })
    }
}
impl PositionRange for Union {
    fn start_position(&self) -> Position {
        self.left.start_position()
    }
    fn end_position(&self) -> Position {
        self.right.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Annotated {
    pub var: VariableToken,
    pub _colon: SymbolToken,
    pub ty: Type,
}
impl Parse for Annotated {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(Annotated {
            var: track!(parser.parse())?,
            _colon: track!(parser.expect(&Symbol::DoubleColon))?,
            ty: track!(parser.parse())?,
        })
    }
}
impl PositionRange for Annotated {
    fn start_position(&self) -> Position {
        self.var.start_position()
    }
    fn end_position(&self) -> Position {
        self.ty.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct List {
    pub _open: SymbolToken,
    pub element: Option<ListElement>,
    pub _close: SymbolToken,
}
impl Parse for List {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(List {
            _open: track!(parser.expect(&Symbol::OpenSquare))?,
            element: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseSquare))?,
        })
    }
}
impl PositionRange for List {
    fn start_position(&self) -> Position {
        self._open.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
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
pub struct Bits {
    pub _open: SymbolToken,
    pub spec: Option<BitsSpec>,
    pub _close: SymbolToken,
}
impl Parse for Bits {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: TokenRead,
    {
        Ok(Bits {
            _open: track!(parser.expect(&Symbol::DoubleLeftAngle))?,
            spec: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::DoubleRightAngle))?,
        })
    }
}
impl PositionRange for Bits {
    fn start_position(&self) -> Position {
        self._open.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
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

pub type Tuple = collections::Tuple<Type>;
pub type Map = collections::Map<Type>;
pub type Record = collections::Record<Type>;
pub type Parenthesized = building_blocks::Parenthesized<Type>;
pub type LocalCall = building_blocks::LocalCall<AtomToken, Type>;
pub type RemoteCall = building_blocks::RemoteCall<AtomToken, Type>;
pub type UnaryOpCall = building_blocks::UnaryOpCall<Type>;
pub type BinaryOpCall = building_blocks::BinaryOpCall<Type>;
