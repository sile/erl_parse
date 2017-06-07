use cst::{LeftExpr, Expr, Pattern};
use cst::commons;
use cst::clauses;
use cst::literals;

#[derive(Debug, Clone)]
pub struct Block<'token> {
    pub _begin: literals::K_BEGIN,
    pub body: Body<'token>,
    pub _end: literals::K_END,
}
derive_parse!(Block, _begin, body, _end);
derive_token_range!(Block, _begin, _end);

// TODO: `catch`か`after`のいずれかは必須
#[derive(Debug, Clone)]
pub struct Try<'token> {
    pub _try: literals::K_TRY,
    pub body: Body<'token>,
    pub branch: Option<TryOf<'token>>,
    pub catch: Option<TryCatch<'token>>,
    pub after: Option<TryAfter<'token>>,
    pub _end: literals::K_END,
}
derive_parse!(Try, _try, body, branch, catch, after, _end);
derive_token_range!(Try, _try, _end);

#[derive(Debug, Clone)]
pub struct TryOf<'token> {
    pub _of: literals::K_OF,
    pub clauses: commons::NonEmptySeq<clauses::CaseClause<'token>, literals::S_SEMICOLON>,
}
derive_parse!(TryOf, _of, clauses);
derive_token_range!(TryOf, _of, clauses);

#[derive(Debug, Clone)]
pub struct TryCatch<'token> {
    pub _catch: literals::K_CATCH,
    pub clauses: commons::NonEmptySeq<clauses::CatchClause<'token>, literals::S_SEMICOLON>,
}
derive_parse!(TryCatch, _catch, clauses);
derive_token_range!(TryCatch, _catch, clauses);

#[derive(Debug, Clone)]
pub struct TryAfter<'token> {
    pub _after: literals::K_AFTER,
    pub body: Body<'token>,
}
derive_parse!(TryAfter, _after, body);
derive_token_range!(TryAfter, _after, body);

#[derive(Debug, Clone)]
pub struct Receive<'token> {
    pub _receive: literals::K_RECEIVE,
    pub clauses: commons::NonEmptySeq<clauses::CaseClause<'token>, literals::S_SEMICOLON>,
    pub timeout: Option<Timeout<'token>>,
    pub _end: literals::K_END,
}
derive_parse!(Receive, _receive, clauses, timeout, _end);
derive_token_range!(Receive, _receive, _end);

#[derive(Debug, Clone)]
pub struct Timeout<'token> {
    pub _after: literals::K_AFTER,
    pub duration: Expr<'token>,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub body: Body<'token>,
}
derive_parse!(Timeout, _after, duration, _arrow, body);
derive_token_range!(Timeout, _after, body);

#[derive(Debug, Clone)]
pub struct Case<'token> {
    pub _case: literals::K_CASE,
    pub value: Expr<'token>,
    pub _of: literals::K_OF,
    pub clauses: commons::NonEmptySeq<clauses::CaseClause<'token>, literals::S_SEMICOLON>,
    pub _end: literals::K_END,
}
derive_parse!(Case, _case, value, _of, clauses, _end);
derive_token_range!(Case, _case, _end);

#[derive(Debug, Clone)]
pub struct If<'token> {
    pub _if: literals::K_IF,
    pub clauses: commons::NonEmptySeq<clauses::IfClause<'token>, literals::S_SEMICOLON>,
    pub _end: literals::K_END,
}
derive_parse!(If, _if, clauses, _end);
derive_token_range!(If, _if, _end);

#[derive(Debug, Clone)]
pub struct Body<'token> {
    pub exprs: commons::NonEmptySeq<Expr<'token>, literals::S_COMMA>,
}
derive_parse!(Body, exprs);
derive_token_range!(Body, exprs, exprs);

#[derive(Debug, Clone)]
pub struct Catch<'token> {
    pub _catch: literals::K_CATCH,
    pub expr: Expr<'token>,
}
derive_parse!(Catch, _catch, expr);
derive_token_range!(Catch, _catch, expr);

#[derive(Debug, Clone)]
pub struct LocalFun<'token> {
    pub _fun: literals::K_FUN,
    pub fun_name: literals::Atom<'token>,
    pub _slash: literals::S_SLASH,
    pub arity: literals::Int<'token>,
}
derive_parse!(LocalFun, _fun, fun_name, _slash, arity);
derive_token_range!(LocalFun, _fun, arity);

#[derive(Debug, Clone)]
pub struct AnonymousFun<'token> {
    pub _fun: literals::K_FUN,
    pub clauses:
        commons::NonEmptySeq<clauses::FunClause<'token, commons::Void>, literals::S_SEMICOLON>,
    pub _end: literals::K_END,
}
derive_parse!(AnonymousFun, _fun, clauses, _end);
derive_token_range!(AnonymousFun, _fun, _end);

#[derive(Debug, Clone)]
pub struct NamedFun<'token> {
    pub _fun: literals::K_FUN,
    pub clauses: commons::NonEmptySeq<clauses::FunClause<'token, commons::Var<'token>>,
                                      literals::S_SEMICOLON>,
    pub _end: literals::K_END,
}
derive_parse!(NamedFun, _fun, clauses, _end);
derive_token_range!(NamedFun, _fun, _end);

#[derive(Debug, Clone)]
pub struct RemoteFun<'token> {
    pub _fun: literals::K_FUN,
    pub module_name: commons::VarOrAtom<'token>,
    pub _colon: literals::S_COLON,
    pub fun_name: commons::VarOrAtom<'token>,
    pub _slash: literals::S_SLASH,
    pub arity: commons::VarOrInt<'token>,
}
derive_parse!(RemoteFun,
              _fun,
              module_name,
              _colon,
              fun_name,
              _slash,
              arity);
derive_token_range!(RemoteFun, _fun, arity);

#[derive(Debug, Clone)]
pub struct MapUpdate<'token> {
    pub map: NonLeftRecurExpr<'token>,
    pub _sharp: literals::S_SHARP,
    pub _open: literals::S_OPEN_BRACE,
    pub fields: commons::Seq<commons::MapField<Expr<'token>>, literals::S_COMMA>,
    pub _close: literals::S_CLOSE_BRACE,
}
derive_parse!(MapUpdate, map, _sharp, _open, fields, _close);
derive_token_range!(MapUpdate, map, _close);

#[derive(Debug, Clone)]
pub struct RecordUpdate<'token> {
    pub record: NonLeftRecurExpr<'token>,
    pub _sharp: literals::S_SHARP,
    pub record_name: literals::Atom<'token>,
    pub _open: literals::S_OPEN_BRACE,
    pub fields: commons::Seq<commons::RecordField<'token, Expr<'token>>, literals::S_COMMA>,
    pub _close: literals::S_CLOSE_BRACE,
}
derive_parse!(RecordUpdate,
              record,
              _sharp,
              record_name,
              _open,
              fields,
              _close);
derive_token_range!(RecordUpdate, record, _close);

#[derive(Debug, Clone)]
pub struct ListComprehension<'token> {
    pub _open: literals::S_OPEN_SQUARE,
    pub elem: Expr<'token>,
    pub _bar: literals::S_DOUBLE_VERTICAL_BAR,
    pub qualifiers: commons::NonEmptySeq<Qualifier<'token>, literals::S_COMMA>,
    pub _close: literals::S_CLOSE_SQUARE,
}
derive_parse!(ListComprehension, _open, elem, _bar, qualifiers, _close);
derive_token_range!(ListComprehension, _open, _close);

#[derive(Debug, Clone)]
pub struct BitStrComprehension<'token> {
    pub _open: literals::S_DOUBLE_LEFT_ANGLE,
    pub elem: Expr<'token>,
    pub _bar: literals::S_DOUBLE_VERTICAL_BAR,
    pub qualifiers: commons::NonEmptySeq<Qualifier<'token>, literals::S_COMMA>,
    pub _close: literals::S_DOUBLE_RIGHT_ANGLE,
}
derive_parse!(BitStrComprehension, _open, elem, _bar, qualifiers, _close);
derive_token_range!(BitStrComprehension, _open, _close);

#[derive(Debug, Clone)]
pub enum Qualifier<'token> {
    Generator(Generator<'token>),
    BitStrGenerator(BitStrGenerator<'token>),
    Filter(Expr<'token>),
}
derive_traits_for_enum!(Qualifier, Generator, BitStrGenerator, Filter);

#[derive(Debug, Clone)]
pub struct Generator<'token> {
    pub pattern: Pattern<'token>,
    pub _arrow: literals::S_LEFT_ARROW,
    pub list: Expr<'token>,
}
derive_parse!(Generator, pattern, _arrow, list);
derive_token_range!(Generator, pattern, list);

#[derive(Debug, Clone)]
pub struct BitStrGenerator<'token> {
    pub pattern: Pattern<'token>,
    pub _arrow: literals::S_DOUBLE_LEFT_ARROW,
    pub bitstring: Expr<'token>,
}
derive_parse!(BitStrGenerator, pattern, _arrow, bitstring);
derive_token_range!(BitStrGenerator, pattern, bitstring);

#[derive(Debug, Clone)]
pub enum NonLeftRecurExpr<'token> {
    NamedFun(Box<NamedFun<'token>>),
    AnonymousFun(Box<AnonymousFun<'token>>),
    RemoteFun(Box<RemoteFun<'token>>),
    LocalFun(Box<LocalFun<'token>>),
    UnaryOpCall(Box<UnaryOpCall<'token>>),
    Catch(Box<Catch<'token>>),
    Paren(Box<Parenthesized<'token>>),
    Try(Box<Try<'token>>),
    Receive(Box<Receive<'token>>),
    Case(Box<Case<'token>>),
    If(Box<If<'token>>),
    Block(Box<Block<'token>>),
    BitStr(Box<BitStr<'token>>),
    BitStrComprehension(Box<BitStrComprehension<'token>>),
    Record(Box<Record<'token>>),
    RecordFieldIndex(RecordFieldIndex<'token>),
    Map(Box<Map<'token>>),
    List(Box<List<'token>>),
    TailConsList(Box<TailConsList<'token>>),
    ListComprehension(Box<ListComprehension<'token>>),
    Tuple(Box<Tuple<'token>>),
    Var(commons::Var<'token>),
    Atom(literals::Atom<'token>),
    Char(literals::Char<'token>),
    Float(literals::Float<'token>),
    Int(literals::Int<'token>),
    Str(literals::Str<'token>),
}
derive_traits_for_enum!(NonLeftRecurExpr,
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
                        Str);

pub type Parenthesized<'token> = commons::Parenthesized<Expr<'token>>;
pub type BitStr<'token> = commons::BitStr<'token, Expr<'token>, NonLeftRecurExpr<'token>>;
pub type Tuple<'token> = commons::Tuple<Expr<'token>>;
pub type Map<'token> = commons::Map<Expr<'token>>;
pub type Record<'token> = commons::Record<'token, Expr<'token>>;
pub type RecordFieldIndex<'token> = commons::RecordFieldIndex<'token>;
pub type RecordFieldAccess<'token> = commons::RecordFieldAccess<'token, NonLeftRecurExpr<'token>>;
pub type List<'token> = commons::List<Expr<'token>>;
pub type TailConsList<'token> = commons::TailConsList<Expr<'token>>;
pub type UnaryOpCall<'token> = commons::UnaryOpCall<Expr<'token>>;
pub type BinaryOpCall<'token> = commons::BinaryOpCall<LeftExpr<'token>, Expr<'token>>;
pub type LocalCall<'token> = commons::LocalCall<NonLeftRecurExpr<'token>, Expr<'token>>;
pub type RemoteCall<'token> = commons::RemoteCall<NonLeftRecurExpr<'token>,
                                                  NonLeftRecurExpr<'token>,
                                                  Expr<'token>>;
pub type Match<'token> = commons::Match<'token, Expr<'token>>;
