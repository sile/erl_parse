use std::ops::Deref;
use erl_tokenize::Token;
use erl_tokenize::tokens::VariableToken;

use {Result, Parse, TokenRange, TokenReader};
use cst::LeftPattern;
use cst::literals;

#[derive(Debug, Clone)]
pub struct LocalCall<N, A> {
    pub name: N,
    pub args: Args<A>,
}
derive_parse!(LocalCall<N, A>, name, args);
derive_token_range!(LocalCall<N, A>, name, args);

#[derive(Debug, Clone)]
pub struct RemoteCall<M, N, A> {
    pub module_name: M,
    pub _colon: literals::S_COLON,
    pub name: N,
    pub args: Args<A>,
}
derive_parse!(RemoteCall<M, N, A>, module_name, _colon, name, args);
derive_token_range!(RemoteCall<M, N, A>, module_name, args);

#[derive(Debug, Clone)]
pub struct Var {
    position: usize,
    value: VariableToken,
}
derive_traits_for_token!(Var, Variable, VariableToken);

#[derive(Debug, Clone)]
pub struct Void {
    position: usize,
}
impl Parse for Void {
    fn parse(reader: &mut TokenReader) -> Result<Self> {
        Ok(Void { position: reader.position() })
    }
}
impl TokenRange for Void {
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.position
    }
}

#[derive(Debug, Clone)]
pub enum Seq<T, D> {
    NonEmpty(NonEmptySeq<T, D>),
    Empty(Void),
}
derive_traits_for_enum!(Seq<T, D>, NonEmpty, Empty);

#[derive(Debug, Clone)]
pub struct NonEmptySeq<T, D> {
    pub head: T,
    pub tail: Vec<SeqNonHeadElem<T, D>>,
    _position: Void,
}
derive_parse!(NonEmptySeq<T, D>, head, tail, _position);
derive_token_range!(NonEmptySeq<T, D>, head, _position);

#[derive(Debug, Clone)]
pub struct SeqNonHeadElem<T, D> {
    pub delim: D,
    pub elem: T,
}
derive_parse!(SeqNonHeadElem<T, D>, delim, elem);
derive_token_range!(SeqNonHeadElem<T, D>, delim, elem);

#[derive(Debug, Clone)]
pub struct Parenthesized<T> {
    pub _open: literals::S_OPEN_PAREN,
    pub inner: T,
    pub _close: literals::S_CLOSE_PAREN,
}
derive_parse!(Parenthesized<T>, _open, inner, _close);
derive_token_range!(Parenthesized<T>, _open, _close);

#[derive(Debug, Clone)]
pub struct BitStr<T, S> {
    pub _open: literals::S_DOUBLE_LEFT_ANGLE,
    pub elems: Seq<BitStrElem<T, S>, literals::S_COMMA>,
    pub _close: literals::S_DOUBLE_RIGHT_ANGLE,
}
derive_parse!(BitStr<T, S>, _open, elems, _close);
derive_token_range!(BitStr<T, S>, _open, _close);

#[derive(Debug, Clone)]
pub struct BitStrElem<T, S> {
    pub elem: T,
    pub size: Option<BitStrElemSize<S>>,
    pub type_spec_list: Option<BitStrElemTypeSpecs>,
    _position: Void,
}
derive_parse!(BitStrElem<T, S>, elem, size, type_spec_list, _position);
derive_token_range!(BitStrElem<T, S>, elem, _position);

#[derive(Debug, Clone)]
pub struct BitStrElemSize<S> {
    pub _colon: literals::S_COLON,
    pub size: S,
}
derive_parse!(BitStrElemSize<T>, _colon, size);
derive_token_range!(BitStrElemSize<T>, _colon, size);

#[derive(Debug, Clone)]
pub struct BitStrElemTypeSpecs {
    pub _slash: literals::S_SLASH,
    pub specs: NonEmptySeq<BitStrElemTypeSpec, literals::S_HYPHEN>,
}
derive_parse!(BitStrElemTypeSpecs, _slash, specs);
derive_token_range!(BitStrElemTypeSpecs, _slash, specs);

#[derive(Debug, Clone)]
pub enum BitStrElemTypeSpec {
    // Type
    Integer(literals::A_INTEGER),
    Float(literals::A_FLOAT),
    Binary(literals::A_BINARY),
    Bytes(literals::A_BYTES),
    Bitstring(literals::A_BITSTRING),
    Bits(literals::A_BITS),
    Utf8(literals::A_UTF8),
    Utf16(literals::A_UTF16),
    Utf32(literals::A_UTF32),

    // Signedness
    Signed(literals::A_SIGNED),
    Unsigned(literals::A_UNSIGNED),

    // Endianness
    Big(literals::A_BIG),
    Little(literals::A_LITTLE),
    Native(literals::A_NATIVE),

    // Unit
    Unit(BitStrElemTypeSpecUnit),
}
derive_traits_for_enum!(BitStrElemTypeSpec, Integer, Float, Binary,
                        Bytes, Bitstring, Bits, Utf8, Utf16, Utf32,
                        Signed, Unsigned, Big, Little, Native, Unit);

#[derive(Debug, Clone)]
pub struct BitStrElemTypeSpecUnit {
    pub _unit: literals::A_UNIT,
    pub _colon: literals::S_COLON,
    pub unit: literals::Int,
}
derive_parse!(BitStrElemTypeSpecUnit, _unit, _colon, unit);
derive_token_range!(BitStrElemTypeSpecUnit, _unit, unit);

#[derive(Debug, Clone)]
pub struct Tuple<T> {
    pub _open: literals::S_OPEN_BRACE,
    pub elems: Seq<T, literals::S_COMMA>,
    pub _close: literals::S_CLOSE_BRACE,
}
derive_parse!(Tuple<T>, _open, elems, _close);
derive_token_range!(Tuple<T>, _open, _close);

#[derive(Debug, Clone)]
pub struct Args<T> {
    pub _open: literals::S_OPEN_PAREN,
    pub args: Seq<T, literals::S_COMMA>,
    pub _close: literals::S_CLOSE_PAREN,
}
derive_parse!(Args<T>, _open, args, _close);
derive_token_range!(Args<T>, _open, _close);

#[derive(Debug, Clone)]
pub struct Record<T> {
    pub _sharp: literals::S_SHARP,
    pub record_name: literals::Atom,
    pub _open: literals::S_OPEN_BRACE,
    pub fields: Seq<RecordField<T>, literals::S_COMMA>,
    pub _close: literals::S_CLOSE_BRACE,
}
derive_parse!(Record<T>, _sharp, record_name, _open, fields, _close);
derive_token_range!(Record<T>, _sharp, _close);

#[derive(Debug, Clone)]
pub struct RecordFieldIndex {
    pub _sharp: literals::S_SHARP,
    pub record_name: literals::Atom,
    pub _dot: literals::S_DOT,
    pub field_name: literals::Atom,
}
derive_parse!(RecordFieldIndex, _sharp, record_name, _dot, field_name);
derive_token_range!(RecordFieldIndex, _sharp, field_name);

#[derive(Debug, Clone)]
pub struct RecordFieldAccess<T> {
    pub record: T,
    pub _sharp: literals::S_SHARP,
    pub record_name: literals::Atom,
    pub _dot: literals::S_DOT,
    pub field_name: literals::Atom,
}
derive_parse!(RecordFieldAccess<T>, record, _sharp, record_name, _dot, field_name);
derive_token_range!(RecordFieldAccess<T>, record, field_name);

#[derive(Debug, Clone)]
pub struct RecordField<T> {
    // XXX: 実際には全ての変数が許容される訳ではない
    // '_'のみが特定ケースで使用可能になるだけ(TODO)
    pub key: VarOrAtom,
    pub _delim: literals::S_MATCH,
    pub value: T,
}
derive_parse!(RecordField<T>, key, _delim, value);
derive_token_range!(RecordField<T>, key, value);

#[derive(Debug, Clone)]
pub struct Map<T> {
    pub _sharp: literals::S_SHARP,
    pub _open: literals::S_OPEN_BRACE,
    pub fields: Seq<MapField<T>, literals::S_COMMA>,
    pub _close: literals::S_CLOSE_BRACE,
}
derive_parse!(Map<T>, _sharp, _open, fields, _close);
derive_token_range!(Map<T>, _sharp, _close);

// XXX: 常に`:=`と`=>`の両方が許容される訳ではない
#[derive(Debug, Clone)]
pub enum MapField<T> {
    Assoc(MapFieldAssoc<T>),
    Exact(MapFieldExact<T>),
}
derive_traits_for_enum!(MapField<T>, Assoc, Exact);

#[derive(Debug, Clone)]
pub struct MapFieldAssoc<T> {
    pub key: T,
    pub _delim: literals::S_DOUBLE_RIGHT_ARROW,
    pub value: T,
}
derive_parse!(MapFieldAssoc<T>, key, _delim, value);
derive_token_range!(MapFieldAssoc<T>, key, value);

#[derive(Debug, Clone)]
pub struct MapFieldExact<T> {
    pub key: T,
    pub _delim: literals::S_MAP_MATCH,
    pub value: T,
}
derive_parse!(MapFieldExact<T>, key, _delim, value);
derive_token_range!(MapFieldExact<T>, key, value);

#[derive(Debug, Clone)]
pub struct List<T> {
    pub _open: literals::S_OPEN_SQUARE,
    pub elems: Seq<T, literals::S_COMMA>,
    pub _close: literals::S_CLOSE_SQUARE,
}
derive_parse!(List<T>, _open, elems, _close);
derive_token_range!(List<T>, _open, _close);

#[derive(Debug, Clone)]
pub struct TailConsList<T> {
    pub _open: literals::S_OPEN_SQUARE,
    pub head: NonEmptySeq<T, literals::S_COMMA>,
    pub _var: literals::S_VERTICAL_BAR,
    pub tail: T,
    pub _close: literals::S_CLOSE_SQUARE,
}
derive_parse!(TailConsList<T>, _open, head, _var, tail, _close);
derive_token_range!(TailConsList<T>, _open, _close);

#[derive(Debug, Clone)]
pub struct UnaryOpCall<T> {
    pub op: UnaryOp,
    pub operand: T,
}
derive_parse!(UnaryOpCall<T>, op, operand);
derive_token_range!(UnaryOpCall<T>, op, operand);

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Plus(literals::S_PLUS),
    Minus(literals::S_HYPHEN),
    Not(literals::K_NOT),
    Bnot(literals::K_BNOT),
}
derive_traits_for_enum!(UnaryOp, Plus, Minus, Not, Bnot);

#[derive(Debug, Clone)]
pub struct Match<T> {
    pub pattern: LeftPattern,
    pub _match: literals::S_MATCH,
    pub value: T,
}
derive_parse!(Match<T>, pattern, _match, value);
derive_token_range!(Match<T>, pattern, value);

#[derive(Debug, Clone)]
pub struct BinaryOpCall<L, R> {
    pub left: L,
    pub op: BinaryOp,
    pub right: R,
}
derive_parse!(BinaryOpCall<L, R>, left, op, right);
derive_token_range!(BinaryOpCall<L, R>, left, right);

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Plus(literals::S_PLUS),
    Minus(literals::S_HYPHEN),
    Mul(literals::S_MULTIPLY),
    FloatDiv(literals::S_SLASH),
    IntDiv(literals::K_DIV),
    Rem(literals::K_REM),
    Bor(literals::K_BOR),
    Bxor(literals::K_BXOR),
    Bsl(literals::K_BSL),
    Bsr(literals::K_BSR),
    Or(literals::K_OR),
    Xor(literals::K_XOR),
    PlusPlus(literals::S_PLUS_PLUS),
    MinusMinus(literals::S_MINUS_MINUS),
    Eq(literals::S_EQ),
    ExactEq(literals::S_EXACT_EQ),
    NotEq(literals::S_NOT_EQ),
    ExactNotEq(literals::S_EXACT_NOT_EQ),
    Less(literals::S_LESS),
    LessEq(literals::S_LESS_EQ),
    Greater(literals::S_GREATER),
    GreaterEq(literals::S_GREATER_EQ),
    AndAlso(literals::K_AND_ALSO),
    OrElse(literals::K_OR_ELSE),
    Send(literals::S_NOT),
}
derive_traits_for_enum!(BinaryOp, Plus, Minus, Mul,
                        FloatDiv, IntDiv, Rem, Bor, Bxor, Bsl, Bsr, Or, Xor, PlusPlus, MinusMinus,
                        Eq, ExactEq, NotEq, ExactNotEq, Less, LessEq, Greater, GreaterEq,
                        AndAlso, OrElse, Send);

#[derive(Debug, Clone)]
pub struct UnaryNumOpCall<T> {
    pub op: UnaryNumOp,
    pub operand: T,
}
derive_parse!(UnaryNumOpCall<T>, op, operand);
derive_token_range!(UnaryNumOpCall<T>, op, operand);

#[derive(Debug, Clone)]
pub enum UnaryNumOp {
    Plus(literals::S_PLUS),
    Minus(literals::S_HYPHEN),
}
derive_traits_for_enum!(UnaryNumOp, Plus, Minus);

#[derive(Debug, Clone)]
pub enum VarOrAtom {
    Var(Var),
    Atom(literals::Atom),
}
derive_traits_for_enum!(VarOrAtom, Var, Atom);

#[derive(Debug, Clone)]
pub enum VarOrInt {
    Var(Var),
    Int(literals::Int),
}
derive_traits_for_enum!(VarOrInt, Var, Int);
