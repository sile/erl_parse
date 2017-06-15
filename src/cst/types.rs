use cst::Type;
use cst::commons;
use cst::literals;

#[derive(Debug, Clone)]
pub struct Annotated {
    pub var: commons::Var,
    pub _double_colon: literals::S_DOUBLE_COLON,
    pub ty: Type,
}
derive_parse!(Annotated, var, _double_colon, ty);
derive_token_range!(Annotated, var, ty);

#[derive(Debug, Clone)]
pub struct List {
    pub _open: literals::S_OPEN_SQUARE,
    pub elem: Option<Type>,
    pub _close: literals::S_CLOSE_SQUARE,
}
derive_parse!(List, _open, elem, _close);
derive_token_range!(List, _open, _close);

#[derive(Debug, Clone)]
pub struct AnyArgFun {
    pub _fun: literals::K_FUN,
    pub _open: literals::S_OPEN_PAREN,
    pub _args_open: literals::S_OPEN_PAREN,
    pub _args: literals::S_TRIPLE_DOT,
    pub _args_close: literals::S_CLOSE_PAREN,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub return_type: Type,
    pub _close: literals::S_CLOSE_PAREN,
}
derive_parse!(
    AnyArgFun,
    _fun,
    _open,
    _args_open,
    _args,
    _args_close,
    _arrow,
    return_type,
    _close
);
derive_token_range!(AnyArgFun, _fun, _close);

#[derive(Debug, Clone)]
pub struct Fun {
    pub _fun: literals::K_FUN,
    pub _open: literals::S_OPEN_PAREN,
    pub args: commons::Args<Type>,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub return_type: Type,
    pub constraints: Option<FunConstraints>,
    pub _close: literals::S_CLOSE_PAREN,
}
derive_parse!(
    Fun,
    _fun,
    _open,
    args,
    _arrow,
    return_type,
    constraints,
    _close
);
derive_token_range!(Fun, _fun, _close);

#[derive(Debug, Clone)]
pub struct FunConstraints {
    pub _when: literals::K_WHEN,
    pub constraints: commons::NonEmptySeq<FunConstraint, literals::S_COMMA>,
}
derive_parse!(FunConstraints, _when, constraints);
derive_token_range!(FunConstraints, _when, constraints);

#[derive(Debug, Clone)]
pub enum FunConstraint {
    Annotated(Annotated),
    IsSubtype(IsSubtype),
}
derive_traits_for_enum!(FunConstraint, Annotated, IsSubtype);

#[derive(Debug, Clone)]
pub struct IsSubtype {
    pub _is_subtype: literals::A_IS_SUBTYPE,
    pub _open: literals::S_OPEN_PAREN,
    pub var: commons::Var,
    pub _comma: literals::S_COMMA,
    pub ty: Type,
    pub _close: literals::S_CLOSE_PAREN,
}
derive_parse!(IsSubtype, _is_subtype, _open, var, _comma, ty, _close);
derive_token_range!(IsSubtype, _is_subtype, _close);

#[derive(Debug, Clone)]
pub struct BitStr {
    pub _open: literals::S_DOUBLE_LEFT_ANGLE,
    pub spec: BitStrSpec,
    pub _close: literals::S_DOUBLE_RIGHT_ANGLE,
}
derive_parse!(BitStr, _open, spec, _close);
derive_token_range!(BitStr, _open, _close);

#[derive(Debug, Clone)]
pub enum BitStrSpec {
    BytesAndBits(BytesAndBitsSpec),
    Bytes(BytesSpec),
    Bits(BitsSpec),
    Empty(commons::Void),
}
derive_traits_for_enum!(BitStrSpec, BytesAndBits, Bytes, Bits, Empty);

#[derive(Debug, Clone)]
pub struct BytesSpec {
    pub _underscore: literals::V_ANY,
    pub _colon: literals::S_COLON,
    pub bytes: literals::Int,
}
derive_parse!(BytesSpec, _underscore, _colon, bytes);
derive_token_range!(BytesSpec, _underscore, bytes);

#[derive(Debug, Clone)]
pub struct BitsSpec {
    pub _underscore0: literals::V_ANY,
    pub _colon: literals::S_COLON,
    pub _underscore1: literals::V_ANY,
    pub _asterisk: literals::S_MULTIPLY,
    pub bits: literals::Int,
}
derive_parse!(
    BitsSpec,
    _underscore0,
    _colon,
    _underscore1,
    _asterisk,
    bits
);
derive_token_range!(BitsSpec, _underscore0, bits);

#[derive(Debug, Clone)]
pub struct BytesAndBitsSpec {
    pub bytes: BytesSpec,
    pub _delim: literals::S_COMMA,
    pub bits: BitsSpec,
}
derive_parse!(BytesAndBitsSpec, bytes, _delim, bits);
derive_token_range!(BytesAndBitsSpec, bytes, bits);

#[derive(Debug, Clone)]
pub struct IntRange {
    pub min: IntType,
    pub _delim: literals::S_DOUBLE_DOT,
    pub max: IntType,
}
derive_parse!(IntRange, min, _delim, max);
derive_token_range!(IntRange, min, max);

#[derive(Debug, Clone)]
pub enum IntType {
    BinaryOpCall(Box<BinaryOpCall>),
    UnaryOpCall(Box<UnaryOpCall>),
    Paren(Box<commons::Parenthesized<IntType>>),
    Int(literals::Int),
}
derive_traits_for_enum!(IntType, BinaryOpCall, UnaryOpCall, Paren, Int);

#[derive(Debug, Clone)]
pub enum LeftIntType {
    UnaryOpCall(Box<UnaryOpCall>),
    Paren(Box<commons::Parenthesized<IntType>>),
    Int(literals::Int),
}
derive_traits_for_enum!(LeftIntType, UnaryOpCall, Paren, Int);

#[derive(Debug, Clone)]
pub struct BinaryOpCall {
    pub left: LeftIntType,
    pub op: BinaryOp,
    pub right: IntType,
}
derive_parse!(BinaryOpCall, left, op, right);
derive_token_range!(BinaryOpCall, left, right);

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add(literals::S_PLUS),
    Sub(literals::S_HYPHEN),
    Mul(literals::S_MULTIPLY),
    Div(literals::K_DIV),
    Rem(literals::K_REM),
}
derive_traits_for_enum!(BinaryOp, Add, Sub, Mul, Div, Rem);

#[derive(Debug, Clone)]
pub struct UnaryOpCall {
    pub op: UnaryOp,
    pub value: IntType,
}
derive_parse!(UnaryOpCall, op, value);
derive_token_range!(UnaryOpCall, op, value);

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Plus(literals::S_PLUS),
    Minus(literals::S_HYPHEN),
}
derive_traits_for_enum!(UnaryOp, Plus, Minus);

#[derive(Debug, Clone)]
pub struct Union {
    pub left: LeftType,
    pub _or: literals::S_VERTICAL_BAR,
    pub right: Type,
}
derive_parse!(Union, left, _or, right);
derive_token_range!(Union, left, right);

#[derive(Debug, Clone)]
pub enum LeftType {
    IntRange(Box<IntRange>),
    Int(IntType),
    BitStr(Box<BitStr>),
    AnyArgFun(Box<AnyArgFun>),
    Fun(Box<Fun>),
    RemoteCall(Box<RemoteCall>),
    LocalCall(Box<LocalCall>),
    Record(Box<Record>),
    Map(Box<Map>),
    Tuple(Box<Tuple>),
    Annotated(Box<Annotated>),
    Paren(Box<Parenthesized>),
    List(Box<List>),
    Var(commons::Var),
    Atom(literals::Atom),
    Char(literals::Char),
    Float(literals::Float),
    Str(literals::Str),
}
derive_traits_for_enum!(
    LeftType,
    IntRange,
    Int,
    BitStr,
    AnyArgFun,
    Fun,
    RemoteCall,
    LocalCall,
    Record,
    Map,
    Tuple,
    Annotated,
    Paren,
    List,
    Var,
    Atom,
    Char,
    Float,
    Str
);

pub type Tuple = commons::Tuple<Type>;
pub type Map = commons::Map<Type>;
pub type Record = commons::Record<Type>;
pub type LocalCall = commons::LocalCall<literals::Atom, Type>;
pub type RemoteCall = commons::RemoteCall<literals::Atom, literals::Atom, Type>;
pub type Parenthesized = commons::Parenthesized<Type>;
