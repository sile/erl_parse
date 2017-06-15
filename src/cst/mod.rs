macro_rules! derive_traits_for_enum {
    ($name:ident , $($variant:ident),*) => {
        impl ::Parse for $name {
            fn parse(reader: &mut ::TokenReader) -> ::Result<Self> {
                $(if let Some(t) = ::Parse::try_parse(reader) {
                    return Ok($name::$variant(t));
                })*
                track_panic!(::ErrorKind::Other,
                             "Unrecognized token for `{}`: token={:?}",
                             stringify!($name),
                             reader.read());
            }
        }
        impl ::TokenRange for $name {
            fn token_start(&self) -> usize {
                match *self {
                    $($name::$variant(ref t) => t.token_start()),*
                }
            }
            fn token_end(&self) -> usize {
                match *self {
                    $($name::$variant(ref t) => t.token_end()),*
                }
            }
        }
    };
    ($name:ident <$($param:ident),*>, $($variant:ident),*) => {
        impl< $($param),*> ::Parse for $name<$($param),*>
            where $($param: ::Parse),* {
            fn parse(reader: &mut ::TokenReader) -> ::Result<Self> {
                $(if let Some(t) = ::Parse::try_parse(reader) {
                    return Ok($name::$variant(t));
                })*
                track_panic!(::ErrorKind::Other,
                             "Unrecognized token for `{}`: token={:?}",
                             stringify!($name),
                             reader.read());
            }
        }
        impl<$($param),*> ::TokenRange for $name<$($param),*>
            where $($param: ::TokenRange),* {
            fn token_start(&self) -> usize {
                match *self {
                    $($name::$variant(ref t) => t.token_start()),*
                }
            }
            fn token_end(&self) -> usize {
                match *self {
                    $($name::$variant(ref t) => t.token_end()),*
                }
            }
        }
    };
    ($name:ident, $($variant:ident),*) => {
        impl ::Parse for $name {
            fn parse(reader: &mut ::TokenReader) -> ::Result<Self> {
                $(if let Some(t) = ::Parse::try_parse(reader) {
                    return Ok($name::$variant(t));
                })*
                track_panic!(::ErrorKind::Other,
                             "Unrecognized token for `{}`: token={:?}",
                             stringify!($name),
                             reader.read());
            }
        }
        impl ::TokenRange for $name {
            fn token_start(&self) -> usize {
                match *self {
                    $($name::$variant(ref t) => t.token_start()),*
                }
            }
            fn token_end(&self) -> usize {
                match *self {
                    $($name::$variant(ref t) => t.token_end()),*
                }
            }
        }
    }
}
macro_rules! derive_parse {
    ($name:ident < $($param:ident),* >, $($field:ident),*) => {
        impl < $($param),*> ::Parse for
            $name < $($param),* >
            where $($param: ::Parse),* {
            fn parse(reader: &mut ::TokenReader) -> ::Result<Self> {
                Ok($name {
                    $($field: track!(::Parse::parse(reader),
                                     "struct={}, field={}",
                                     stringify!($name),
                                     stringify!($field))?),*
                })
            }
        }
    };
    ($name:ident < $($param:ident),* >, $($field:ident),*) => {
        impl < $($param),*> ::Parse for $name < $($param),* >
            where $($param: ::Parse),* {
            fn parse(reader: &mut ::TokenReader) -> ::Result<Self> {
                Ok($name {
                    $($field: track!(::Parse::parse(reader),
                                     "struct={}, field={}",
                                     stringify!($name),
                                     stringify!($field))?),*
                })
            }
        }
    };
    ($name:ident, $($field:ident),*) => {
        impl ::Parse for $name {
            fn parse(reader: &mut ::TokenReader) -> ::Result<Self> {
                Ok($name {
                    $($field: track!(::Parse::parse(reader),
                                     "struct={}, field={}",
                                     stringify!($name),
                                     stringify!($field))?),*
                })
            }
        }
    }
}
macro_rules! derive_token_range {
    ($name:ident < $($param:ident),* >, $first:ident, $last:ident) => {
        impl < $($param),*> ::TokenRange for $name < $($param),* >
            where $($param: ::TokenRange),* {
            fn token_start(&self) -> usize {
                self.$first.token_start()
            }
            fn token_end(&self) -> usize {
                self.$last.token_end()
            }
        }
    };
    ($name:ident < $($param:ident),* >, $first:ident, $last:ident) => {
        impl <$($param),*> ::TokenRange for $name < $($param),* >
            where $($param: ::TokenRange),* {
            fn token_start(&self) -> usize {
                self.$first.token_start()
            }
            fn token_end(&self) -> usize {
                self.$last.token_end()
            }
        }
    };
    ($name:ident, $first:ident, $last:ident) => {
        impl ::TokenRange for $name {
            fn token_start(&self) -> usize {
                self.$first.token_start()
            }
            fn token_end(&self) -> usize {
                self.$last.token_end()
            }
        }
    }
}
macro_rules! derive_traits_for_token {
    ($name:ident, $variant:ident, $token:ident) => {
        impl Deref for $name {
            type Target = $token;
            fn deref(&self) -> &Self::Target {
                &self.value
            }
        }
        impl ::Parse for $name {
            fn parse(reader: &mut ::TokenReader) -> ::Result<Self> {
                let position = reader.position();
                let token = track!(reader.read())?;
                if let Token::$variant(ref value) = *token {
                    Ok($name { position, value: value.clone() })
                 } else {
                    track_panic!(::ErrorKind::Other,
                                 "An `{}` is expected: actual={:?}",
                                 stringify!($token),
                                 token);
                }
            }
        }
        impl ::TokenRange for $name {
            fn token_start(&self) -> usize {
                self.position
            }
            fn token_end(&self) -> usize {
                self.position + 1
            }
        }
    }
}

pub mod commons;
pub mod clauses;
pub mod exprs;
pub mod forms;
pub mod guard_tests;
pub mod literals;
pub mod patterns;
pub mod terms;
pub mod types;

#[derive(Debug, Clone)]
pub struct ModuleDecl {
    _start: commons::Void,
    pub forms: Vec<Form>,
    _end: commons::Void,
}
derive_parse!(ModuleDecl, _start, forms, _end);
derive_token_range!(ModuleDecl, _start, _end);

#[derive(Debug, Clone)]
pub enum Term {
    Paren(Box<terms::Parenthesized>),
    BitStr(Box<terms::BitStr>),
    Record(Box<terms::Record>),
    Map(Box<terms::Map>),
    List(Box<terms::List>),
    TailConsList(Box<terms::TailConsList>),
    Tuple(Box<terms::Tuple>),
    Atom(literals::Atom),
    Char(literals::Char),
    Float(literals::Float),
    Int(literals::Int),
    Str(literals::Str),
}
derive_traits_for_enum!(
    Term,
    Paren,
    BitStr,
    Record,
    Map,
    List,
    TailConsList,
    Tuple,
    Atom,
    Char,
    Float,
    Int,
    Str
);

#[derive(Debug, Clone)]
pub enum Form {
    ModuleAttr(forms::ModuleAttr),
    ExportAttr(forms::ExportAttr),
    ExportTypeAttr(forms::ExportTypeAttr),
    ImportAttr(forms::ImportAttr),
    FileAttr(forms::FileAttr),
    WildAttr(forms::WildAttr),
    FunSpec(forms::FunSpec),
    RemoteFunSpec(forms::RemoteFunSpec),
    CallbackSpec(forms::CallbackSpec),
    FunDecl(forms::FunDecl),
    RecordDecl(forms::RecordDecl),
    TypeDecl(forms::TypeDecl),
    OpaqueDecl(forms::OpaqueDecl),
}
derive_traits_for_enum!(
    Form,
    ModuleAttr,
    ExportAttr,
    ExportTypeAttr,
    ImportAttr,
    FileAttr,
    WildAttr,
    FunSpec,
    RemoteFunSpec,
    CallbackSpec,
    FunDecl,
    RecordDecl,
    TypeDecl,
    OpaqueDecl
);

#[derive(Debug, Clone)]
pub enum Type {
    Union(Box<types::Union>),
    IntRange(Box<types::IntRange>),
    Int(types::IntType),
    BitStr(Box<types::BitStr>),
    AnyArgFun(Box<types::AnyArgFun>),
    Fun(Box<types::Fun>),
    RemoteCall(Box<types::RemoteCall>),
    LocalCall(Box<types::LocalCall>),
    Record(Box<types::Record>),
    Map(Box<types::Map>),
    Tuple(Box<types::Tuple>),
    Annotated(Box<types::Annotated>),
    Paren(Box<types::Parenthesized>),
    List(Box<types::List>),
    Var(commons::Var),
    Atom(literals::Atom),
    Char(literals::Char),
    Float(literals::Float),
    Str(literals::Str),
}
derive_traits_for_enum!(
    Type,
    Union,
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

#[derive(Debug, Clone)]
pub enum Expr {
    BinaryOpCall(Box<exprs::BinaryOpCall>),
    RemoteCall(Box<exprs::RemoteCall>),
    LocalCall(Box<exprs::LocalCall>),
    Match(Box<exprs::Match>),
    MapUpdate(Box<exprs::MapUpdate>),
    RecordUpdate(Box<exprs::RecordUpdate>),
    RecordFieldAccess(Box<exprs::RecordFieldAccess>),
    NamedFun(Box<exprs::NamedFun>),
    AnonymousFun(Box<exprs::AnonymousFun>),
    RemoteFun(Box<exprs::RemoteFun>),
    LocalFun(Box<exprs::LocalFun>),
    UnaryOpCall(Box<exprs::UnaryOpCall>),
    Catch(Box<exprs::Catch>),
    Paren(Box<exprs::Parenthesized>),
    Try(Box<exprs::Try>),
    Receive(Box<exprs::Receive>),
    Case(Box<exprs::Case>),
    If(Box<exprs::If>),
    Block(Box<exprs::Block>),
    BitStr(Box<exprs::BitStr>),
    BitStrComprehension(Box<exprs::BitStrComprehension>),
    Record(Box<exprs::Record>),
    RecordFieldIndex(exprs::RecordFieldIndex),
    Map(Box<exprs::Map>),
    List(Box<exprs::List>),
    TailConsList(Box<exprs::TailConsList>),
    ListComprehension(Box<exprs::ListComprehension>),
    Tuple(Box<exprs::Tuple>),
    Var(commons::Var),
    Atom(literals::Atom),
    Char(literals::Char),
    Float(literals::Float),
    Int(literals::Int),
    Str(literals::Str),
}
derive_traits_for_enum!(
    Expr,
    BinaryOpCall,
    RemoteCall,
    LocalCall,
    Match,
    MapUpdate,
    RecordUpdate,
    RecordFieldAccess,
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

#[derive(Debug, Clone)]
pub enum LeftExpr {
    RemoteCall(Box<exprs::RemoteCall>),
    LocalCall(Box<exprs::LocalCall>),
    MapUpdate(Box<exprs::MapUpdate>),
    RecordUpdate(Box<exprs::RecordUpdate>),
    RecordFieldAccess(Box<exprs::RecordFieldAccess>),
    NamedFun(Box<exprs::NamedFun>),
    AnonymousFun(Box<exprs::AnonymousFun>),
    RemoteFun(Box<exprs::RemoteFun>),
    LocalFun(Box<exprs::LocalFun>),
    UnaryOpCall(Box<exprs::UnaryOpCall>),
    Catch(Box<exprs::Catch>),
    Paren(Box<exprs::Parenthesized>),
    Try(Box<exprs::Try>),
    Receive(Box<exprs::Receive>),
    Case(Box<exprs::Case>),
    If(Box<exprs::If>),
    Block(Box<exprs::Block>),
    BitStr(Box<exprs::BitStr>),
    BitStrComprehension(Box<exprs::BitStrComprehension>),
    Record(Box<exprs::Record>),
    RecordFieldIndex(exprs::RecordFieldIndex),
    Map(Box<exprs::Map>),
    List(Box<exprs::List>),
    TailConsList(Box<exprs::TailConsList>),
    ListComprehension(Box<exprs::ListComprehension>),
    Tuple(Box<exprs::Tuple>),
    Var(commons::Var),
    Atom(literals::Atom),
    Char(literals::Char),
    Float(literals::Float),
    Int(literals::Int),
    Str(literals::Str),
}
derive_traits_for_enum!(
    LeftExpr,
    RemoteCall,
    LocalCall,
    MapUpdate,
    RecordUpdate,
    RecordFieldAccess,
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

#[derive(Debug, Clone)]
pub enum Pattern {
    Match(Box<patterns::Match>),
    BinaryOpCall(Box<patterns::BinaryOpCall>),
    UnaryOpCall(Box<patterns::UnaryOpCall>),
    Paren(Box<patterns::Parenthesized>),
    Record(Box<patterns::Record>),
    RecordFieldIndex(patterns::RecordFieldIndex),
    Map(Box<patterns::Map>),
    Tuple(Box<patterns::Tuple>),
    List(Box<patterns::List>),
    TailConsList(Box<patterns::TailConsList>),
    BitStr(Box<patterns::BitStr>),
    Var(commons::Var),
    Atom(literals::Atom),
    Char(literals::Char),
    Float(literals::Float),
    Int(literals::Int),
    Str(literals::Str),
}
derive_traits_for_enum!(
    Pattern,
    Match,
    BinaryOpCall,
    UnaryOpCall,
    Paren,
    Record,
    RecordFieldIndex,
    Map,
    Tuple,
    List,
    TailConsList,
    BitStr,
    Var,
    Atom,
    Char,
    Float,
    Int,
    Str
);

#[derive(Debug, Clone)]
pub enum LeftPattern {
    Paren(Box<patterns::Parenthesized>),
    UnaryOpCall(Box<patterns::UnaryOpCall>),
    Record(Box<patterns::Record>),
    RecordFieldIndex(patterns::RecordFieldIndex),
    Map(Box<patterns::Map>),
    Tuple(Box<patterns::Tuple>),
    List(Box<patterns::List>),
    TailConsList(Box<patterns::TailConsList>),
    BitStr(Box<patterns::BitStr>),
    Var(commons::Var),
    Atom(literals::Atom),
    Char(literals::Char),
    Float(literals::Float),
    Int(literals::Int),
    Str(literals::Str),
}
derive_traits_for_enum!(
    LeftPattern,
    Paren,
    UnaryOpCall,
    Record,
    RecordFieldIndex,
    Map,
    Tuple,
    List,
    TailConsList,
    BitStr,
    Var,
    Atom,
    Char,
    Float,
    Int,
    Str
);

#[derive(Debug, Clone)]
pub struct GuardSeq {
    pub guards: commons::NonEmptySeq<Guard, literals::S_SEMICOLON>,
}
derive_parse!(GuardSeq, guards);
derive_token_range!(GuardSeq, guards, guards);

#[derive(Debug, Clone)]
pub struct Guard {
    pub tests: commons::NonEmptySeq<GuardTest, literals::S_COMMA>,
}
derive_parse!(Guard, tests);
derive_token_range!(Guard, tests, tests);

#[derive(Debug, Clone)]
pub enum GuardTest {
    BinaryOpCall(Box<guard_tests::BinaryOpCall>),
    RemoteCall(Box<guard_tests::RemoteCall>),
    LocalCall(Box<guard_tests::LocalCall>),
    RecordFieldAccess(guard_tests::RecordFieldAccess),
    UnaryOpCall(Box<guard_tests::UnaryOpCall>),
    Paren(Box<guard_tests::Parenthesized>),
    BitStr(Box<guard_tests::BitStr>),
    Record(Box<guard_tests::Record>),
    RecordFieldIndex(guard_tests::RecordFieldIndex),
    Map(Box<guard_tests::Map>),
    List(Box<guard_tests::List>),
    TailConsList(Box<guard_tests::TailConsList>),
    Tuple(Box<guard_tests::Tuple>),
    Var(commons::Var),
    Atom(literals::Atom),
    Char(literals::Char),
    Float(literals::Float),
    Int(literals::Int),
    Str(literals::Str),
}
derive_traits_for_enum!(
    GuardTest,
    BinaryOpCall,
    RemoteCall,
    LocalCall,
    UnaryOpCall,
    Paren,
    BitStr,
    Record,
    RecordFieldIndex,
    RecordFieldAccess,
    Map,
    List,
    TailConsList,
    Tuple,
    Var,
    Atom,
    Char,
    Float,
    Int,
    Str
);

#[derive(Debug, Clone)]
pub enum LeftGuardTest {
    UnaryOpCall(Box<guard_tests::UnaryOpCall>),
    Paren(Box<guard_tests::Parenthesized>),
    BitStr(Box<guard_tests::BitStr>),
    Record(Box<guard_tests::Record>),
    RecordFieldIndex(guard_tests::RecordFieldIndex),
    Map(Box<guard_tests::Map>),
    List(Box<guard_tests::List>),
    TailConsList(Box<guard_tests::TailConsList>),
    Tuple(Box<guard_tests::Tuple>),
    Var(commons::Var),
    Atom(literals::Atom),
    Char(literals::Char),
    Float(literals::Float),
    Int(literals::Int),
    Str(literals::Str),
}
derive_traits_for_enum!(
    LeftGuardTest,
    UnaryOpCall,
    Paren,
    BitStr,
    Record,
    RecordFieldIndex,
    Map,
    List,
    TailConsList,
    Tuple,
    Var,
    Atom,
    Char,
    Float,
    Int,
    Str
);
