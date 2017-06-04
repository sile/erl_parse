use cst::{LeftExpr, Expr, Pattern};
use cst::commons;
use cst::clauses;
use cst::literals;

#[derive(Debug, Clone)]
pub struct Block<'token, 'text: 'token> {
    pub _begin: literals::K_BEGIN,
    pub body: Body<'token, 'text>,
    pub _end: literals::K_END,
}
derive_parse!(Block, _begin, body, _end);
derive_token_range!(Block, _begin, _end);

// TODO: `catch`か`after`のいずれかは必須
#[derive(Debug, Clone)]
pub struct Try<'token, 'text: 'token> {
    pub _try: literals::K_TRY,
    pub body: Body<'token, 'text>,
    pub branch: Option<TryOf<'token, 'text>>,
    pub catch: Option<TryCatch<'token, 'text>>,
    pub after: Option<TryAfter<'token, 'text>>,
    pub _end: literals::K_END,
}
derive_parse!(Try, _try, body, branch, catch, after, _end);
derive_token_range!(Try, _try, _end);

#[derive(Debug, Clone)]
pub struct TryOf<'token, 'text: 'token> {
    pub _of: literals::K_OF,
    pub clauses: commons::NonEmptySeq<clauses::CaseClause<'token, 'text>, literals::S_SEMICOLON>,
}
derive_parse!(TryOf, _of, clauses);
derive_token_range!(TryOf, _of, clauses);

#[derive(Debug, Clone)]
pub struct TryCatch<'token, 'text: 'token> {
    pub _catch: literals::K_CATCH,
    pub clauses: commons::NonEmptySeq<clauses::CatchClause<'token, 'text>, literals::S_SEMICOLON>,
}
derive_parse!(TryCatch, _catch, clauses);
derive_token_range!(TryCatch, _catch, clauses);

#[derive(Debug, Clone)]
pub struct TryAfter<'token, 'text: 'token> {
    pub _after: literals::K_AFTER,
    pub body: Body<'token, 'text>,
}
derive_parse!(TryAfter, _after, body);
derive_token_range!(TryAfter, _after, body);

#[derive(Debug, Clone)]
pub struct Receive<'token, 'text: 'token> {
    pub _receive: literals::K_RECEIVE,
    pub clauses: commons::NonEmptySeq<clauses::CaseClause<'token, 'text>, literals::S_SEMICOLON>,
    pub timeout: Option<Timeout<'token, 'text>>,
    pub _end: literals::K_END,
}
derive_parse!(Receive, _receive, clauses, timeout, _end);
derive_token_range!(Receive, _receive, _end);

#[derive(Debug, Clone)]
pub struct Timeout<'token, 'text: 'token> {
    pub _after: literals::K_AFTER,
    pub duration: Expr<'token, 'text>,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub body: Body<'token, 'text>,
}
derive_parse!(Timeout, _after, duration, _arrow, body);
derive_token_range!(Timeout, _after, body);

#[derive(Debug, Clone)]
pub struct Case<'token, 'text: 'token> {
    pub _case: literals::K_CASE,
    pub value: Expr<'token, 'text>,
    pub _of: literals::K_OF,
    pub clauses: commons::NonEmptySeq<clauses::CaseClause<'token, 'text>, literals::S_SEMICOLON>,
    pub _end: literals::K_END,
}
derive_parse!(Case, _case, value, _of, clauses, _end);
derive_token_range!(Case, _case, _end);

#[derive(Debug, Clone)]
pub struct If<'token, 'text: 'token> {
    pub _if: literals::K_IF,
    pub clauses: commons::NonEmptySeq<clauses::IfClause<'token, 'text>, literals::S_SEMICOLON>,
    pub _end: literals::K_END,
}
derive_parse!(If, _if, clauses, _end);
derive_token_range!(If, _if, _end);

#[derive(Debug, Clone)]
pub struct Body<'token, 'text: 'token> {
    pub exprs: commons::NonEmptySeq<Expr<'token, 'text>, literals::S_COMMA>,
}
derive_parse!(Body, exprs);
derive_token_range!(Body, exprs, exprs);

#[derive(Debug, Clone)]
pub struct Catch<'token, 'text: 'token> {
    pub _catch: literals::K_CATCH,
    pub expr: Expr<'token, 'text>,
}
derive_parse!(Catch, _catch, expr);
derive_token_range!(Catch, _catch, expr);

#[derive(Debug, Clone)]
pub struct LocalFun<'token, 'text: 'token> {
    pub _fun: literals::K_FUN,
    pub fun_name: literals::Atom<'token, 'text>,
    pub _slash: literals::S_SLASH,
    pub arity: literals::Int<'token, 'text>,
}
derive_parse!(LocalFun, _fun, fun_name, _slash, arity);
derive_token_range!(LocalFun, _fun, arity);

#[derive(Debug, Clone)]
pub struct AnonymousFun<'token, 'text: 'token> {
    pub _fun: literals::K_FUN,
    pub clauses: commons::NonEmptySeq<clauses::FunClause<'token, 'text, commons::Void>,
                                      literals::S_SEMICOLON>,
    pub _end: literals::K_END,
}
derive_parse!(AnonymousFun, _fun, clauses, _end);
derive_token_range!(AnonymousFun, _fun, _end);

#[derive(Debug, Clone)]
pub struct NamedFun<'token, 'text: 'token> {
    pub _fun: literals::K_FUN,
    pub clauses: commons::NonEmptySeq<clauses::FunClause<'token,
                                                         'text,
                                                         commons::Var<'token, 'text>>,
                                      literals::S_SEMICOLON>,
    pub _end: literals::K_END,
}
derive_parse!(NamedFun, _fun, clauses, _end);
derive_token_range!(NamedFun, _fun, _end);

#[derive(Debug, Clone)]
pub struct RemoteFun<'token, 'text: 'token> {
    pub _fun: literals::K_FUN,
    pub module_name: commons::VarOrAtom<'token, 'text>,
    pub _colon: literals::S_COLON,
    pub fun_name: commons::VarOrAtom<'token, 'text>,
    pub _slash: literals::S_SLASH,
    pub arity: commons::VarOrInt<'token, 'text>,
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
pub struct MapUpdate<'token, 'text: 'token> {
    pub map: NonLeftRecurExpr<'token, 'text>,
    pub _sharp: literals::S_SHARP,
    pub _open: literals::S_OPEN_BRACE,
    pub fields: commons::Seq<commons::MapField<Expr<'token, 'text>>, literals::S_COMMA>,
    pub _close: literals::S_CLOSE_BRACE,
}
derive_parse!(MapUpdate, map, _sharp, _open, fields, _close);
derive_token_range!(MapUpdate, map, _close);

#[derive(Debug, Clone)]
pub struct RecordUpdate<'token, 'text: 'token> {
    pub record: NonLeftRecurExpr<'token, 'text>,
    pub _sharp: literals::S_SHARP,
    pub record_name: literals::Atom<'token, 'text>,
    pub _open: literals::S_OPEN_BRACE,
    pub fields:
        commons::Seq<commons::RecordField<'token, 'text, Expr<'token, 'text>>, literals::S_COMMA>,
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
pub struct ListComprehension<'token, 'text: 'token> {
    pub _open: literals::S_OPEN_SQUARE,
    pub elem: Expr<'token, 'text>,
    pub _bar: literals::S_DOUBLE_VERTICAL_BAR,
    pub qualifiers: commons::NonEmptySeq<Qualifier<'token, 'text>, literals::S_COMMA>,
    pub _close: literals::S_CLOSE_SQUARE,
}
derive_parse!(ListComprehension, _open, elem, _bar, qualifiers, _close);
derive_token_range!(ListComprehension, _open, _close);

#[derive(Debug, Clone)]
pub struct BitStrComprehension<'token, 'text: 'token> {
    pub _open: literals::S_DOUBLE_LEFT_ANGLE,
    pub elem: Expr<'token, 'text>,
    pub _bar: literals::S_DOUBLE_VERTICAL_BAR,
    pub qualifiers: commons::NonEmptySeq<Qualifier<'token, 'text>, literals::S_COMMA>,
    pub _close: literals::S_DOUBLE_RIGHT_ANGLE,
}
derive_parse!(BitStrComprehension, _open, elem, _bar, qualifiers, _close);
derive_token_range!(BitStrComprehension, _open, _close);

#[derive(Debug, Clone)]
pub enum Qualifier<'token, 'text: 'token> {
    Generator(Generator<'token, 'text>),
    BitStrGenerator(BitStrGenerator<'token, 'text>),
    Filter(Expr<'token, 'text>),
}
derive_traits_for_enum!(Qualifier, Generator, BitStrGenerator, Filter);

#[derive(Debug, Clone)]
pub struct Generator<'token, 'text: 'token> {
    pub pattern: Pattern<'token, 'text>,
    pub _arrow: literals::S_LEFT_ARROW,
    pub list: Expr<'token, 'text>,
}
derive_parse!(Generator, pattern, _arrow, list);
derive_token_range!(Generator, pattern, list);

#[derive(Debug, Clone)]
pub struct BitStrGenerator<'token, 'text: 'token> {
    pub pattern: Pattern<'token, 'text>,
    pub _arrow: literals::S_DOUBLE_LEFT_ARROW,
    pub bitstring: Expr<'token, 'text>,
}
derive_parse!(BitStrGenerator, pattern, _arrow, bitstring);
derive_token_range!(BitStrGenerator, pattern, bitstring);

#[derive(Debug, Clone)]
pub enum NonLeftRecurExpr<'token, 'text: 'token> {
    NamedFun(Box<NamedFun<'token, 'text>>),
    AnonymousFun(Box<AnonymousFun<'token, 'text>>),
    RemoteFun(Box<RemoteFun<'token, 'text>>),
    LocalFun(Box<LocalFun<'token, 'text>>),
    UnaryOpCall(Box<UnaryOpCall<'token, 'text>>),
    Catch(Box<Catch<'token, 'text>>),
    Paren(Box<Parenthesized<'token, 'text>>),
    Try(Box<Try<'token, 'text>>),
    Receive(Box<Receive<'token, 'text>>),
    Case(Box<Case<'token, 'text>>),
    If(Box<If<'token, 'text>>),
    Block(Box<Block<'token, 'text>>),
    BitStr(Box<BitStr<'token, 'text>>),
    BitStrComprehension(Box<BitStrComprehension<'token, 'text>>),
    Record(Box<Record<'token, 'text>>),
    RecordFieldIndex(RecordFieldIndex<'token, 'text>),
    Map(Box<Map<'token, 'text>>),
    List(Box<List<'token, 'text>>),
    TailConsList(Box<TailConsList<'token, 'text>>),
    ListComprehension(Box<ListComprehension<'token, 'text>>),
    Tuple(Box<Tuple<'token, 'text>>),
    Var(commons::Var<'token, 'text>),
    Atom(literals::Atom<'token, 'text>),
    Char(literals::Char<'token, 'text>),
    Float(literals::Float<'token, 'text>),
    Int(literals::Int<'token, 'text>),
    Str(literals::Str<'token, 'text>),
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

pub type Parenthesized<'token, 'text> = commons::Parenthesized<Expr<'token, 'text>>;
pub type BitStr<'token, 'text> = commons::BitStr<'token,
                                                 'text,
                                                 Expr<'token, 'text>,
                                                 NonLeftRecurExpr<'token, 'text>>;
pub type Tuple<'token, 'text> = commons::Tuple<Expr<'token, 'text>>;
pub type Map<'token, 'text> = commons::Map<Expr<'token, 'text>>;
pub type Record<'token, 'text> = commons::Record<'token, 'text, Expr<'token, 'text>>;
pub type RecordFieldIndex<'token, 'text> = commons::RecordFieldIndex<'token, 'text>;
pub type RecordFieldAccess<'token, 'text> = commons::RecordFieldAccess<'token,
                                                                       'text,
                                                                       NonLeftRecurExpr<'token,
                                                                                        'text>>;
pub type List<'token, 'text> = commons::List<Expr<'token, 'text>>;
pub type TailConsList<'token, 'text> = commons::TailConsList<Expr<'token, 'text>>;
pub type UnaryOpCall<'token, 'text> = commons::UnaryOpCall<Expr<'token, 'text>>;
pub type BinaryOpCall<'token, 'text> = commons::BinaryOpCall<LeftExpr<'token, 'text>,
                                                             Expr<'token, 'text>>;
pub type LocalCall<'token, 'text> = commons::LocalCall<NonLeftRecurExpr<'token, 'text>,
                                                       Expr<'token, 'text>>;
pub type RemoteCall<'token, 'text> = commons::RemoteCall<NonLeftRecurExpr<'token, 'text>,
                                                         NonLeftRecurExpr<'token, 'text>,
                                                         Expr<'token, 'text>>;
pub type Match<'token, 'text> = commons::Match<'token, 'text, Expr<'token, 'text>>;
