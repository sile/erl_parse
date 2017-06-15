use cst::{LeftExpr, Expr, Pattern};
use cst::commons;
use cst::clauses;
use cst::literals;

#[derive(Debug, Clone)]
pub struct Block {
    pub _begin: literals::K_BEGIN,
    pub body: Body,
    pub _end: literals::K_END,
}
derive_parse!(Block, _begin, body, _end);
derive_token_range!(Block, _begin, _end);

// TODO: `catch`か`after`のいずれかは必須
#[derive(Debug, Clone)]
pub struct Try {
    pub _try: literals::K_TRY,
    pub body: Body,
    pub branch: Option<TryOf>,
    pub catch: Option<TryCatch>,
    pub after: Option<TryAfter>,
    pub _end: literals::K_END,
}
derive_parse!(Try, _try, body, branch, catch, after, _end);
derive_token_range!(Try, _try, _end);

#[derive(Debug, Clone)]
pub struct TryOf {
    pub _of: literals::K_OF,
    pub clauses: commons::NonEmptySeq<clauses::CaseClause, literals::S_SEMICOLON>,
}
derive_parse!(TryOf, _of, clauses);
derive_token_range!(TryOf, _of, clauses);

#[derive(Debug, Clone)]
pub struct TryCatch {
    pub _catch: literals::K_CATCH,
    pub clauses: commons::NonEmptySeq<clauses::CatchClause, literals::S_SEMICOLON>,
}
derive_parse!(TryCatch, _catch, clauses);
derive_token_range!(TryCatch, _catch, clauses);

#[derive(Debug, Clone)]
pub struct TryAfter {
    pub _after: literals::K_AFTER,
    pub body: Body,
}
derive_parse!(TryAfter, _after, body);
derive_token_range!(TryAfter, _after, body);

#[derive(Debug, Clone)]
pub struct Receive {
    pub _receive: literals::K_RECEIVE,
    pub clauses: commons::NonEmptySeq<clauses::CaseClause, literals::S_SEMICOLON>,
    pub timeout: Option<Timeout>,
    pub _end: literals::K_END,
}
derive_parse!(Receive, _receive, clauses, timeout, _end);
derive_token_range!(Receive, _receive, _end);

#[derive(Debug, Clone)]
pub struct Timeout {
    pub _after: literals::K_AFTER,
    pub duration: Expr,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub body: Body,
}
derive_parse!(Timeout, _after, duration, _arrow, body);
derive_token_range!(Timeout, _after, body);

#[derive(Debug, Clone)]
pub struct Case {
    pub _case: literals::K_CASE,
    pub value: Expr,
    pub _of: literals::K_OF,
    pub clauses: commons::NonEmptySeq<clauses::CaseClause, literals::S_SEMICOLON>,
    pub _end: literals::K_END,
}
derive_parse!(Case, _case, value, _of, clauses, _end);
derive_token_range!(Case, _case, _end);

#[derive(Debug, Clone)]
pub struct If {
    pub _if: literals::K_IF,
    pub clauses: commons::NonEmptySeq<clauses::IfClause, literals::S_SEMICOLON>,
    pub _end: literals::K_END,
}
derive_parse!(If, _if, clauses, _end);
derive_token_range!(If, _if, _end);

#[derive(Debug, Clone)]
pub struct Body {
    pub exprs: commons::NonEmptySeq<Expr, literals::S_COMMA>,
}
derive_parse!(Body, exprs);
derive_token_range!(Body, exprs, exprs);

#[derive(Debug, Clone)]
pub struct Catch {
    pub _catch: literals::K_CATCH,
    pub expr: Expr,
}
derive_parse!(Catch, _catch, expr);
derive_token_range!(Catch, _catch, expr);

#[derive(Debug, Clone)]
pub struct LocalFun {
    pub _fun: literals::K_FUN,
    pub fun_name: literals::Atom,
    pub _slash: literals::S_SLASH,
    pub arity: literals::Int,
}
derive_parse!(LocalFun, _fun, fun_name, _slash, arity);
derive_token_range!(LocalFun, _fun, arity);

#[derive(Debug, Clone)]
pub struct AnonymousFun {
    pub _fun: literals::K_FUN,
    pub clauses: commons::NonEmptySeq<clauses::FunClause<commons::Void>, literals::S_SEMICOLON>,
    pub _end: literals::K_END,
}
derive_parse!(AnonymousFun, _fun, clauses, _end);
derive_token_range!(AnonymousFun, _fun, _end);

#[derive(Debug, Clone)]
pub struct NamedFun {
    pub _fun: literals::K_FUN,
    pub clauses: commons::NonEmptySeq<clauses::FunClause<commons::Var>, literals::S_SEMICOLON>,
    pub _end: literals::K_END,
}
derive_parse!(NamedFun, _fun, clauses, _end);
derive_token_range!(NamedFun, _fun, _end);

#[derive(Debug, Clone)]
pub struct RemoteFun {
    pub _fun: literals::K_FUN,
    pub module_name: commons::VarOrAtom,
    pub _colon: literals::S_COLON,
    pub fun_name: commons::VarOrAtom,
    pub _slash: literals::S_SLASH,
    pub arity: commons::VarOrInt,
}
derive_parse!(
    RemoteFun,
    _fun,
    module_name,
    _colon,
    fun_name,
    _slash,
    arity
);
derive_token_range!(RemoteFun, _fun, arity);

#[derive(Debug, Clone)]
pub struct MapUpdate {
    pub map: NonLeftRecurExpr,
    pub _sharp: literals::S_SHARP,
    pub _open: literals::S_OPEN_BRACE,
    pub fields: commons::Seq<commons::MapField<Expr>, literals::S_COMMA>,
    pub _close: literals::S_CLOSE_BRACE,
}
derive_parse!(MapUpdate, map, _sharp, _open, fields, _close);
derive_token_range!(MapUpdate, map, _close);

#[derive(Debug, Clone)]
pub struct RecordUpdate {
    pub record: NonLeftRecurExpr,
    pub _sharp: literals::S_SHARP,
    pub record_name: literals::Atom,
    pub _open: literals::S_OPEN_BRACE,
    pub fields: commons::Seq<commons::RecordField<Expr>, literals::S_COMMA>,
    pub _close: literals::S_CLOSE_BRACE,
}
derive_parse!(
    RecordUpdate,
    record,
    _sharp,
    record_name,
    _open,
    fields,
    _close
);
derive_token_range!(RecordUpdate, record, _close);

#[derive(Debug, Clone)]
pub struct ListComprehension {
    pub _open: literals::S_OPEN_SQUARE,
    pub elem: Expr,
    pub _bar: literals::S_DOUBLE_VERTICAL_BAR,
    pub qualifiers: commons::NonEmptySeq<Qualifier, literals::S_COMMA>,
    pub _close: literals::S_CLOSE_SQUARE,
}
derive_parse!(ListComprehension, _open, elem, _bar, qualifiers, _close);
derive_token_range!(ListComprehension, _open, _close);

#[derive(Debug, Clone)]
pub struct BitStrComprehension {
    pub _open: literals::S_DOUBLE_LEFT_ANGLE,
    pub elem: Expr,
    pub _bar: literals::S_DOUBLE_VERTICAL_BAR,
    pub qualifiers: commons::NonEmptySeq<Qualifier, literals::S_COMMA>,
    pub _close: literals::S_DOUBLE_RIGHT_ANGLE,
}
derive_parse!(BitStrComprehension, _open, elem, _bar, qualifiers, _close);
derive_token_range!(BitStrComprehension, _open, _close);

#[derive(Debug, Clone)]
pub enum Qualifier {
    Generator(Generator),
    BitStrGenerator(BitStrGenerator),
    Filter(Expr),
}
derive_traits_for_enum!(Qualifier, Generator, BitStrGenerator, Filter);

#[derive(Debug, Clone)]
pub struct Generator {
    pub pattern: Pattern,
    pub _arrow: literals::S_LEFT_ARROW,
    pub list: Expr,
}
derive_parse!(Generator, pattern, _arrow, list);
derive_token_range!(Generator, pattern, list);

#[derive(Debug, Clone)]
pub struct BitStrGenerator {
    pub pattern: Pattern,
    pub _arrow: literals::S_DOUBLE_LEFT_ARROW,
    pub bitstring: Expr,
}
derive_parse!(BitStrGenerator, pattern, _arrow, bitstring);
derive_token_range!(BitStrGenerator, pattern, bitstring);

#[derive(Debug, Clone)]
pub enum NonLeftRecurExpr {
    NamedFun(Box<NamedFun>),
    AnonymousFun(Box<AnonymousFun>),
    RemoteFun(Box<RemoteFun>),
    LocalFun(Box<LocalFun>),
    UnaryOpCall(Box<UnaryOpCall>),
    Catch(Box<Catch>),
    Paren(Box<Parenthesized>),
    Try(Box<Try>),
    Receive(Box<Receive>),
    Case(Box<Case>),
    If(Box<If>),
    Block(Box<Block>),
    BitStr(Box<BitStr>),
    BitStrComprehension(Box<BitStrComprehension>),
    Record(Box<Record>),
    RecordFieldIndex(RecordFieldIndex),
    Map(Box<Map>),
    List(Box<List>),
    TailConsList(Box<TailConsList>),
    ListComprehension(Box<ListComprehension>),
    Tuple(Box<Tuple>),
    Var(commons::Var),
    Atom(literals::Atom),
    Char(literals::Char),
    Float(literals::Float),
    Int(literals::Int),
    Str(literals::Str),
}
derive_traits_for_enum!(
    NonLeftRecurExpr,
    NamedFun,
    AnonymousFun,
    RemoteFun,
    LocalFun,
    UnaryOpCall,
    Catch,
    Paren,
    Try,
    Receive,
    Case,
    If,
    Block,
    BitStr,
    BitStrComprehension,
    Record,
    RecordFieldIndex,
    Map,
    List,
    TailConsList,
    ListComprehension,
    Tuple,
    Var,
    Atom,
    Char,
    Float,
    Int,
    Str
);

pub type Parenthesized = commons::Parenthesized<Expr>;
pub type BitStr = commons::BitStr<Expr, NonLeftRecurExpr>;
pub type Tuple = commons::Tuple<Expr>;
pub type Map = commons::Map<Expr>;
pub type Record = commons::Record<Expr>;
pub type RecordFieldIndex = commons::RecordFieldIndex;
pub type RecordFieldAccess = commons::RecordFieldAccess<NonLeftRecurExpr>;
pub type List = commons::List<Expr>;
pub type TailConsList = commons::TailConsList<Expr>;
pub type UnaryOpCall = commons::UnaryOpCall<Expr>;
pub type BinaryOpCall = commons::BinaryOpCall<LeftExpr, Expr>;
pub type LocalCall = commons::LocalCall<NonLeftRecurExpr, Expr>;
pub type RemoteCall = commons::RemoteCall<NonLeftRecurExpr, NonLeftRecurExpr, Expr>;
pub type Match = commons::Match<Expr>;
