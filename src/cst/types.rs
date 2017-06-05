use cst::Type;
use cst::commons;
use cst::literals;

#[derive(Debug, Clone)]
pub struct Annotated<'token, 'text: 'token> {
    pub var: commons::Var<'token, 'text>,
    pub _double_colon: literals::S_DOUBLE_COLON,
    pub ty: Type<'token, 'text>,
}
derive_parse!(Annotated, var, _double_colon, ty);
derive_token_range!(Annotated, var, ty);

#[derive(Debug, Clone)]
pub struct List<'token, 'text: 'token> {
    pub _open: literals::S_OPEN_SQUARE,
    pub elem: Option<Type<'token, 'text>>,
    pub _close: literals::S_CLOSE_SQUARE,
}
derive_parse!(List, _open, elem, _close);
derive_token_range!(List, _open, _close);

#[derive(Debug, Clone)]
pub struct AnyArgFun<'token, 'text: 'token> {
    pub _fun: literals::K_FUN,
    pub _open: literals::S_OPEN_PAREN,
    pub _args_open: literals::S_OPEN_PAREN,
    pub _args: literals::S_TRIPLE_DOT,
    pub _args_close: literals::S_CLOSE_PAREN,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub return_type: Type<'token, 'text>,
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
pub struct Fun<'token, 'text: 'token> {
    pub _fun: literals::K_FUN,
    pub _open: literals::S_OPEN_PAREN,
    pub args: commons::Args<Type<'token, 'text>>,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub return_type: Type<'token, 'text>,
    pub constraints: Option<FunConstraints<'token, 'text>>,
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
pub struct FunConstraints<'token, 'text: 'token> {
    pub _when: literals::K_WHEN,
    pub constraints: commons::NonEmptySeq<FunConstraint<'token, 'text>, literals::S_COMMA>,
}
derive_parse!(FunConstraints, _when, constraints);
derive_token_range!(FunConstraints, _when, constraints);

#[derive(Debug, Clone)]
pub enum FunConstraint<'token, 'text: 'token> {
    Annotated(Annotated<'token, 'text>),
    IsSubtype(IsSubtype<'token, 'text>),
}
derive_traits_for_enum!(FunConstraint, Annotated, IsSubtype);

#[derive(Debug, Clone)]
pub struct IsSubtype<'token, 'text: 'token> {
    pub _is_subtype: literals::A_IS_SUBTYPE,
    pub _open: literals::S_OPEN_PAREN,
    pub var: commons::Var<'token, 'text>,
    pub _comma: literals::S_COMMA,
    pub ty: Type<'token, 'text>,
    pub _close: literals::S_CLOSE_PAREN,
}
derive_parse!(IsSubtype, _is_subtype, _open, var, _comma, ty, _close);
derive_token_range!(IsSubtype, _is_subtype, _close);

#[derive(Debug, Clone)]
pub struct BitStr<'token, 'text: 'token> {
    pub _open: literals::S_DOUBLE_LEFT_ANGLE,
    pub spec: BitStrSpec<'token, 'text>,
    pub _close: literals::S_DOUBLE_RIGHT_ANGLE,
}
derive_parse!(BitStr, _open, spec, _close);
derive_token_range!(BitStr, _open, _close);

#[derive(Debug, Clone)]
pub enum BitStrSpec<'token, 'text: 'token> {
    BytesAndBits(BytesAndBitsSpec<'token, 'text>),
    Bytes(BytesSpec<'token, 'text>),
    Bits(BitsSpec<'token, 'text>),
    Empty(commons::Void),
}
derive_traits_for_enum!(BitStrSpec, BytesAndBits, Bytes, Bits, Empty);

#[derive(Debug, Clone)]
pub struct BytesSpec<'token, 'text: 'token> {
    pub _underscore: literals::V_ANY,
    pub _colon: literals::S_COLON,
    pub bytes: literals::Int<'token, 'text>,
}
derive_parse!(BytesSpec, _underscore, _colon, bytes);
derive_token_range!(BytesSpec, _underscore, bytes);

#[derive(Debug, Clone)]
pub struct BitsSpec<'token, 'text: 'token> {
    pub _underscore0: literals::V_ANY,
    pub _colon: literals::S_COLON,
    pub _underscore1: literals::V_ANY,
    pub _asterisk: literals::S_MULTIPLY,
    pub bits: literals::Int<'token, 'text>,
}
derive_parse!(BitsSpec,
              _underscore0,
              _colon,
              _underscore1,
              _asterisk,
              bits);
derive_token_range!(BitsSpec, _underscore0, bits);

#[derive(Debug, Clone)]
pub struct BytesAndBitsSpec<'token, 'text: 'token> {
    pub bytes: BytesSpec<'token, 'text>,
    pub _delim: literals::S_COMMA,
    pub bits: BitsSpec<'token, 'text>,
}
derive_parse!(BytesAndBitsSpec, bytes, _delim, bits);
derive_token_range!(BytesAndBitsSpec, bytes, bits);

#[derive(Debug, Clone)]
pub struct IntRange<'token, 'text: 'token> {
    pub min: IntType<'token, 'text>,
    pub _delim: literals::S_DOUBLE_DOT,
    pub max: IntType<'token, 'text>,
}
derive_parse!(IntRange, min, _delim, max);
derive_token_range!(IntRange, min, max);

#[derive(Debug, Clone)]
pub enum IntType<'token, 'text: 'token> {
    BinaryOpCall(Box<BinaryOpCall<'token, 'text>>),
    UnaryOpCall(Box<UnaryOpCall<'token, 'text>>),
    Paren(Box<commons::Parenthesized<IntType<'token, 'text>>>),
    Int(literals::Int<'token, 'text>),
}
derive_traits_for_enum!(IntType, BinaryOpCall, UnaryOpCall, Paren, Int);

#[derive(Debug, Clone)]
pub enum LeftIntType<'token, 'text: 'token> {
    UnaryOpCall(Box<UnaryOpCall<'token, 'text>>),
    Paren(Box<commons::Parenthesized<IntType<'token, 'text>>>),
    Int(literals::Int<'token, 'text>),
}
derive_traits_for_enum!(LeftIntType, UnaryOpCall, Paren, Int);

#[derive(Debug, Clone)]
pub struct BinaryOpCall<'token, 'text: 'token> {
    pub left: LeftIntType<'token, 'text>,
    pub op: BinaryOp,
    pub right: IntType<'token, 'text>,
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
pub struct UnaryOpCall<'token, 'text: 'token> {
    pub op: UnaryOp,
    pub value: IntType<'token, 'text>,
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
pub struct Union<'token, 'text: 'token> {
    pub left: LeftType<'token, 'text>,
    pub _or: literals::S_VERTICAL_BAR,
    pub right: Type<'token, 'text>,
}
derive_parse!(Union, left, _or, right);
derive_token_range!(Union, left, right);

#[derive(Debug, Clone)]
pub enum LeftType<'token, 'text: 'token> {
    IntRange(Box<IntRange<'token, 'text>>),
    Int(IntType<'token, 'text>),
    BitStr(Box<BitStr<'token, 'text>>),
    AnyArgFun(Box<AnyArgFun<'token, 'text>>),
    Fun(Box<Fun<'token, 'text>>),
    RemoteCall(Box<RemoteCall<'token, 'text>>),
    LocalCall(Box<LocalCall<'token, 'text>>),
    Record(Box<Record<'token, 'text>>),
    Map(Box<Map<'token, 'text>>),
    Tuple(Box<Tuple<'token, 'text>>),
    Annotated(Box<Annotated<'token, 'text>>),
    Paren(Box<Parenthesized<'token, 'text>>),
    List(Box<List<'token, 'text>>),
    Var(commons::Var<'token, 'text>),
    Atom(literals::Atom<'token, 'text>),
    Char(literals::Char<'token, 'text>),
    Float(literals::Float<'token, 'text>),
    Str(literals::Str<'token, 'text>),
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

pub type Tuple<'token, 'text> = commons::Tuple<Type<'token, 'text>>;
pub type Map<'token, 'text> = commons::Map<Type<'token, 'text>>;
pub type Record<'token, 'text> = commons::Record<'token, 'text, Type<'token, 'text>>;
pub type LocalCall<'token, 'text> = commons::LocalCall<literals::Atom<'token, 'text>,
                                                       Type<'token, 'text>>;
pub type RemoteCall<'token, 'text> = commons::RemoteCall<literals::Atom<'token, 'text>,
                                                         literals::Atom<'token, 'text>,
                                                         Type<'token, 'text>>;
pub type Parenthesized<'token, 'text> = commons::Parenthesized<Type<'token, 'text>>;
