use cst::Type;
use cst::commons;
use cst::literals;

#[derive(Debug, Clone)]
pub struct Annotated<'token> {
    pub var: commons::Var<'token>,
    pub _double_colon: literals::S_DOUBLE_COLON,
    pub ty: Type<'token>,
}
derive_parse!(Annotated, var, _double_colon, ty);
derive_token_range!(Annotated, var, ty);

#[derive(Debug, Clone)]
pub struct List<'token> {
    pub _open: literals::S_OPEN_SQUARE,
    pub elem: Option<Type<'token>>,
    pub _close: literals::S_CLOSE_SQUARE,
}
derive_parse!(List, _open, elem, _close);
derive_token_range!(List, _open, _close);

#[derive(Debug, Clone)]
pub struct AnyArgFun<'token> {
    pub _fun: literals::K_FUN,
    pub _open: literals::S_OPEN_PAREN,
    pub _args_open: literals::S_OPEN_PAREN,
    pub _args: literals::S_TRIPLE_DOT,
    pub _args_close: literals::S_CLOSE_PAREN,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub return_type: Type<'token>,
    pub _close: literals::S_CLOSE_PAREN,
}
derive_parse!(AnyArgFun,
              _fun,
              _open,
              _args_open,
              _args,
              _args_close,
              _arrow,
              return_type,
              _close);
derive_token_range!(AnyArgFun, _fun, _close);

#[derive(Debug, Clone)]
pub struct Fun<'token> {
    pub _fun: literals::K_FUN,
    pub _open: literals::S_OPEN_PAREN,
    pub args: commons::Args<Type<'token>>,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub return_type: Type<'token>,
    pub constraints: Option<FunConstraints<'token>>,
    pub _close: literals::S_CLOSE_PAREN,
}
derive_parse!(Fun,
              _fun,
              _open,
              args,
              _arrow,
              return_type,
              constraints,
              _close);
derive_token_range!(Fun, _fun, _close);

#[derive(Debug, Clone)]
pub struct FunConstraints<'token> {
    pub _when: literals::K_WHEN,
    pub constraints: commons::NonEmptySeq<FunConstraint<'token>, literals::S_COMMA>,
}
derive_parse!(FunConstraints, _when, constraints);
derive_token_range!(FunConstraints, _when, constraints);

#[derive(Debug, Clone)]
pub enum FunConstraint<'token> {
    Annotated(Annotated<'token>),
    IsSubtype(IsSubtype<'token>),
}
derive_traits_for_enum!(FunConstraint, Annotated, IsSubtype);

#[derive(Debug, Clone)]
pub struct IsSubtype<'token> {
    pub _is_subtype: literals::A_IS_SUBTYPE,
    pub _open: literals::S_OPEN_PAREN,
    pub var: commons::Var<'token>,
    pub _comma: literals::S_COMMA,
    pub ty: Type<'token>,
    pub _close: literals::S_CLOSE_PAREN,
}
derive_parse!(IsSubtype, _is_subtype, _open, var, _comma, ty, _close);
derive_token_range!(IsSubtype, _is_subtype, _close);

#[derive(Debug, Clone)]
pub struct BitStr<'token> {
    pub _open: literals::S_DOUBLE_LEFT_ANGLE,
    pub spec: BitStrSpec<'token>,
    pub _close: literals::S_DOUBLE_RIGHT_ANGLE,
}
derive_parse!(BitStr, _open, spec, _close);
derive_token_range!(BitStr, _open, _close);

#[derive(Debug, Clone)]
pub enum BitStrSpec<'token> {
    BytesAndBits(BytesAndBitsSpec<'token>),
    Bytes(BytesSpec<'token>),
    Bits(BitsSpec<'token>),
    Empty(commons::Void),
}
derive_traits_for_enum!(BitStrSpec, BytesAndBits, Bytes, Bits, Empty);

#[derive(Debug, Clone)]
pub struct BytesSpec<'token> {
    pub _underscore: literals::V_ANY,
    pub _colon: literals::S_COLON,
    pub bytes: literals::Int<'token>,
}
derive_parse!(BytesSpec, _underscore, _colon, bytes);
derive_token_range!(BytesSpec, _underscore, bytes);

#[derive(Debug, Clone)]
pub struct BitsSpec<'token> {
    pub _underscore0: literals::V_ANY,
    pub _colon: literals::S_COLON,
    pub _underscore1: literals::V_ANY,
    pub _asterisk: literals::S_MULTIPLY,
    pub bits: literals::Int<'token>,
}
derive_parse!(BitsSpec,
              _underscore0,
              _colon,
              _underscore1,
              _asterisk,
              bits);
derive_token_range!(BitsSpec, _underscore0, bits);

#[derive(Debug, Clone)]
pub struct BytesAndBitsSpec<'token> {
    pub bytes: BytesSpec<'token>,
    pub _delim: literals::S_COMMA,
    pub bits: BitsSpec<'token>,
}
derive_parse!(BytesAndBitsSpec, bytes, _delim, bits);
derive_token_range!(BytesAndBitsSpec, bytes, bits);

#[derive(Debug, Clone)]
pub struct IntRange<'token> {
    pub min: IntType<'token>,
    pub _delim: literals::S_DOUBLE_DOT,
    pub max: IntType<'token>,
}
derive_parse!(IntRange, min, _delim, max);
derive_token_range!(IntRange, min, max);

#[derive(Debug, Clone)]
pub enum IntType<'token> {
    BinaryOpCall(Box<BinaryOpCall<'token>>),
    UnaryOpCall(Box<UnaryOpCall<'token>>),
    Paren(Box<commons::Parenthesized<IntType<'token>>>),
    Int(literals::Int<'token>),
}
derive_traits_for_enum!(IntType, BinaryOpCall, UnaryOpCall, Paren, Int);

#[derive(Debug, Clone)]
pub enum LeftIntType<'token> {
    UnaryOpCall(Box<UnaryOpCall<'token>>),
    Paren(Box<commons::Parenthesized<IntType<'token>>>),
    Int(literals::Int<'token>),
}
derive_traits_for_enum!(LeftIntType, UnaryOpCall, Paren, Int);

#[derive(Debug, Clone)]
pub struct BinaryOpCall<'token> {
    pub left: LeftIntType<'token>,
    pub op: BinaryOp,
    pub right: IntType<'token>,
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
derive_traits_for_enum!(BinaryOp<>, Add, Sub, Mul, Div, Rem);

#[derive(Debug, Clone)]
pub struct UnaryOpCall<'token> {
    pub op: UnaryOp,
    pub value: IntType<'token>,
}
derive_parse!(UnaryOpCall, op, value);
derive_token_range!(UnaryOpCall, op, value);

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Plus(literals::S_PLUS),
    Minus(literals::S_HYPHEN),
}
derive_traits_for_enum!(UnaryOp<>, Plus, Minus);

#[derive(Debug, Clone)]
pub struct Union<'token> {
    pub left: LeftType<'token>,
    pub _or: literals::S_VERTICAL_BAR,
    pub right: Type<'token>,
}
derive_parse!(Union, left, _or, right);
derive_token_range!(Union, left, right);

#[derive(Debug, Clone)]
pub enum LeftType<'token> {
    IntRange(Box<IntRange<'token>>),
    Int(IntType<'token>),
    BitStr(Box<BitStr<'token>>),
    AnyArgFun(Box<AnyArgFun<'token>>),
    Fun(Box<Fun<'token>>),
    RemoteCall(Box<RemoteCall<'token>>),
    LocalCall(Box<LocalCall<'token>>),
    Record(Box<Record<'token>>),
    Map(Box<Map<'token>>),
    Tuple(Box<Tuple<'token>>),
    Annotated(Box<Annotated<'token>>),
    Paren(Box<Parenthesized<'token>>),
    List(Box<List<'token>>),
    Var(commons::Var<'token>),
    Atom(literals::Atom<'token>),
    Char(literals::Char<'token>),
    Float(literals::Float<'token>),
    Str(literals::Str<'token>),
}
derive_traits_for_enum!(LeftType,
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
                        Str);

pub type Tuple<'token> = commons::Tuple<Type<'token>>;
pub type Map<'token> = commons::Map<Type<'token>>;
pub type Record<'token> = commons::Record<'token, Type<'token>>;
pub type LocalCall<'token> = commons::LocalCall<literals::Atom<'token>, Type<'token>>;
pub type RemoteCall<'token> = commons::RemoteCall<literals::Atom<'token>,
                                                  literals::Atom<'token>,
                                                  Type<'token>>;
pub type Parenthesized<'token> = commons::Parenthesized<Type<'token>>;
