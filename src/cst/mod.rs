macro_rules! derive_traits_for_enum {
    ($name:ident <>, $($variant:ident),*) => {
        impl<'token> ::Parse<'token> for $name {
            fn parse(reader: &mut ::TokenReader<'token>) -> ::Result<Self> {
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
        impl<'token, $($param),*> ::Parse<'token> for $name<$($param),*>
            where $($param: ::Parse<'token,>),* {
            fn parse(reader: &mut ::TokenReader<'token>) -> ::Result<Self> {
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
        impl<'token> ::Parse<'token> for $name<'token> {
            fn parse(reader: &mut ::TokenReader<'token>) -> ::Result<Self> {
                $(if let Some(t) = ::Parse::try_parse(reader) {
                    return Ok($name::$variant(t));
                })*
                track_panic!(::ErrorKind::Other,
                             "Unrecognized token for `{}`: token={:?}",
                             stringify!($name),
                             reader.read());
            }
        }
        impl<'token> ::TokenRange for $name<'token> {
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
    ($name:ident <'token, $($param:ident),* >, $($field:ident),*) => {
        impl <'token, $($param),*> ::Parse<'token> for
            $name <'token, $($param),* >
            where $($param: ::Parse<'token>),* {
            fn parse(reader: &mut ::TokenReader<'token>) -> ::Result<Self> {
                Ok($name {
                    $($field: track_try!(::Parse::parse(reader),
                                         "struct={}, field={}",
                                         stringify!($name),
                                         stringify!($field))),*
                })
            }
        }
    };
    ($name:ident < $($param:ident),* >, $($field:ident),*) => {
        impl <'token, $($param),*> ::Parse<'token> for $name < $($param),* >
            where $($param: ::Parse<'token>),* {
            fn parse(reader: &mut ::TokenReader<'token>) -> ::Result<Self> {
                Ok($name {
                    $($field: track_try!(::Parse::parse(reader),
                                         "struct={}, field={}",
                                         stringify!($name),
                                         stringify!($field))),*
                })
            }
        }
    };
    ($name:ident, $($field:ident),*) => {
        impl <'token> ::Parse<'token> for $name<'token> {
            fn parse(reader: &mut ::TokenReader<'token>) -> ::Result<Self> {
                Ok($name {
                    $($field: track_try!(::Parse::parse(reader),
                                         "struct={}, field={}",
                                         stringify!($name),
                                         stringify!($field))),*
                })
            }
        }
    }
}
macro_rules! derive_token_range {
    ($name:ident <'token, $($param:ident),* >, $first:ident, $last:ident) => {
        impl <'token, $($param),*> ::TokenRange for $name <'token, $($param),* >
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
        impl <'token:'token> ::TokenRange for $name<'token> {
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
        impl<'token> Deref for $name<'token> {
            type Target = $token<>;
            fn deref(&self) -> &Self::Target {
                self.value
            }
        }
        impl<'token> ::Parse<'token> for $name<'token> {
            fn parse(reader: &mut ::TokenReader<'token>) -> ::Result<Self> {
                let position = reader.position();
                let token = track_try!(reader.read());
                if let Token::$variant(ref value) = *token {
                    Ok($name { position, value })
                 } else {
                    track_panic!(::ErrorKind::Other,
                                 "An `{}` is expected: actual={:?}",
                                 stringify!($token),
                                 token);
                }
            }
        }
        impl<'token> ::TokenRange for $name<'token> {
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
pub struct ModuleDecl<'token: 'token> {
    _start: commons::Void,
    pub forms: Vec<Form<'token>>,
    _end: commons::Void,
}
derive_parse!(ModuleDecl, _start, forms, _end);
derive_token_range!(ModuleDecl, _start, _end);

#[derive(Debug, Clone)]
pub enum Term<'token> {
    Paren(Box<terms::Parenthesized<'token>>),
    BitStr(Box<terms::BitStr<'token>>),
    Record(Box<terms::Record<'token>>),
    Map(Box<terms::Map<'token>>),
    List(Box<terms::List<'token>>),
    TailConsList(Box<terms::TailConsList<'token>>),
    Tuple(Box<terms::Tuple<'token>>),
    Atom(literals::Atom<'token>),
    Char(literals::Char<'token>),
    Float(literals::Float<'token>),
    Int(literals::Int<'token>),
    Str(literals::Str<'token>),
}
derive_traits_for_enum!(Term,
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
                        Str);

#[derive(Debug, Clone)]
pub enum Form<'token> {
    ModuleAttr(forms::ModuleAttr<'token>),
    ExportAttr(forms::ExportAttr<'token>),
    ExportTypeAttr(forms::ExportTypeAttr<'token>),
    ImportAttr(forms::ImportAttr<'token>),
    FileAttr(forms::FileAttr<'token>),
    WildAttr(forms::WildAttr<'token>),
    FunSpec(forms::FunSpec<'token>),
    RemoteFunSpec(forms::RemoteFunSpec<'token>),
    CallbackSpec(forms::CallbackSpec<'token>),
    FunDecl(forms::FunDecl<'token>),
    RecordDecl(forms::RecordDecl<'token>),
    TypeDecl(forms::TypeDecl<'token>),
    OpaqueDecl(forms::OpaqueDecl<'token>),
}
derive_traits_for_enum!(Form,
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
                        OpaqueDecl);

#[derive(Debug, Clone)]
pub enum Type<'token> {
    Union(Box<types::Union<'token>>),
    IntRange(Box<types::IntRange<'token>>),
    Int(types::IntType<'token>),
    BitStr(Box<types::BitStr<'token>>),
    AnyArgFun(Box<types::AnyArgFun<'token>>),
    Fun(Box<types::Fun<'token>>),
    RemoteCall(Box<types::RemoteCall<'token>>),
    LocalCall(Box<types::LocalCall<'token>>),
    Record(Box<types::Record<'token>>),
    Map(Box<types::Map<'token>>),
    Tuple(Box<types::Tuple<'token>>),
    Annotated(Box<types::Annotated<'token>>),
    Paren(Box<types::Parenthesized<'token>>),
    List(Box<types::List<'token>>),
    Var(commons::Var<'token>),
    Atom(literals::Atom<'token>),
    Char(literals::Char<'token>),
    Float(literals::Float<'token>),
    Str(literals::Str<'token>),
}
derive_traits_for_enum!(Type,
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
                        Str);

#[derive(Debug, Clone)]
pub enum Expr<'token> {
    BinaryOpCall(Box<exprs::BinaryOpCall<'token>>),
    RemoteCall(Box<exprs::RemoteCall<'token>>),
    LocalCall(Box<exprs::LocalCall<'token>>),
    Match(Box<exprs::Match<'token>>),
    MapUpdate(Box<exprs::MapUpdate<'token>>),
    RecordUpdate(Box<exprs::RecordUpdate<'token>>),
    RecordFieldAccess(Box<exprs::RecordFieldAccess<'token>>),
    NamedFun(Box<exprs::NamedFun<'token>>),
    AnonymousFun(Box<exprs::AnonymousFun<'token>>),
    RemoteFun(Box<exprs::RemoteFun<'token>>),
    LocalFun(Box<exprs::LocalFun<'token>>),
    UnaryOpCall(Box<exprs::UnaryOpCall<'token>>),
    Catch(Box<exprs::Catch<'token>>),
    Paren(Box<exprs::Parenthesized<'token>>),
    Try(Box<exprs::Try<'token>>),
    Receive(Box<exprs::Receive<'token>>),
    Case(Box<exprs::Case<'token>>),
    If(Box<exprs::If<'token>>),
    Block(Box<exprs::Block<'token>>),
    BitStr(Box<exprs::BitStr<'token>>),
    BitStrComprehension(Box<exprs::BitStrComprehension<'token>>),
    Record(Box<exprs::Record<'token>>),
    RecordFieldIndex(exprs::RecordFieldIndex<'token>),
    Map(Box<exprs::Map<'token>>),
    List(Box<exprs::List<'token>>),
    TailConsList(Box<exprs::TailConsList<'token>>),
    ListComprehension(Box<exprs::ListComprehension<'token>>),
    Tuple(Box<exprs::Tuple<'token>>),
    Var(commons::Var<'token>),
    Atom(literals::Atom<'token>),
    Char(literals::Char<'token>),
    Float(literals::Float<'token>),
    Int(literals::Int<'token>),
    Str(literals::Str<'token>),
}
derive_traits_for_enum!(Expr,
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
                        Str);

#[derive(Debug, Clone)]
pub enum LeftExpr<'token> {
    RemoteCall(Box<exprs::RemoteCall<'token>>),
    LocalCall(Box<exprs::LocalCall<'token>>),
    MapUpdate(Box<exprs::MapUpdate<'token>>),
    RecordUpdate(Box<exprs::RecordUpdate<'token>>),
    RecordFieldAccess(Box<exprs::RecordFieldAccess<'token>>),
    NamedFun(Box<exprs::NamedFun<'token>>),
    AnonymousFun(Box<exprs::AnonymousFun<'token>>),
    RemoteFun(Box<exprs::RemoteFun<'token>>),
    LocalFun(Box<exprs::LocalFun<'token>>),
    UnaryOpCall(Box<exprs::UnaryOpCall<'token>>),
    Catch(Box<exprs::Catch<'token>>),
    Paren(Box<exprs::Parenthesized<'token>>),
    Try(Box<exprs::Try<'token>>),
    Receive(Box<exprs::Receive<'token>>),
    Case(Box<exprs::Case<'token>>),
    If(Box<exprs::If<'token>>),
    Block(Box<exprs::Block<'token>>),
    BitStr(Box<exprs::BitStr<'token>>),
    BitStrComprehension(Box<exprs::BitStrComprehension<'token>>),
    Record(Box<exprs::Record<'token>>),
    RecordFieldIndex(exprs::RecordFieldIndex<'token>),
    Map(Box<exprs::Map<'token>>),
    List(Box<exprs::List<'token>>),
    TailConsList(Box<exprs::TailConsList<'token>>),
    ListComprehension(Box<exprs::ListComprehension<'token>>),
    Tuple(Box<exprs::Tuple<'token>>),
    Var(commons::Var<'token>),
    Atom(literals::Atom<'token>),
    Char(literals::Char<'token>),
    Float(literals::Float<'token>),
    Int(literals::Int<'token>),
    Str(literals::Str<'token>),
}
derive_traits_for_enum!(LeftExpr,
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
                        Str);

#[derive(Debug, Clone)]
pub enum Pattern<'token> {
    Match(Box<patterns::Match<'token>>),
    BinaryOpCall(Box<patterns::BinaryOpCall<'token>>),
    UnaryOpCall(Box<patterns::UnaryOpCall<'token>>),
    Paren(Box<patterns::Parenthesized<'token>>),
    Record(Box<patterns::Record<'token>>),
    RecordFieldIndex(patterns::RecordFieldIndex<'token>),
    Map(Box<patterns::Map<'token>>),
    Tuple(Box<patterns::Tuple<'token>>),
    List(Box<patterns::List<'token>>),
    TailConsList(Box<patterns::TailConsList<'token>>),
    BitStr(Box<patterns::BitStr<'token>>),
    Var(commons::Var<'token>),
    Atom(literals::Atom<'token>),
    Char(literals::Char<'token>),
    Float(literals::Float<'token>),
    Int(literals::Int<'token>),
    Str(literals::Str<'token>),
}
derive_traits_for_enum!(Pattern,
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
                        Str);

#[derive(Debug, Clone)]
pub enum LeftPattern<'token> {
    Paren(Box<patterns::Parenthesized<'token>>),
    UnaryOpCall(Box<patterns::UnaryOpCall<'token>>),
    Record(Box<patterns::Record<'token>>),
    RecordFieldIndex(patterns::RecordFieldIndex<'token>),
    Map(Box<patterns::Map<'token>>),
    Tuple(Box<patterns::Tuple<'token>>),
    List(Box<patterns::List<'token>>),
    TailConsList(Box<patterns::TailConsList<'token>>),
    BitStr(Box<patterns::BitStr<'token>>),
    Var(commons::Var<'token>),
    Atom(literals::Atom<'token>),
    Char(literals::Char<'token>),
    Float(literals::Float<'token>),
    Int(literals::Int<'token>),
    Str(literals::Str<'token>),
}
derive_traits_for_enum!(LeftPattern,
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
                        Str);

#[derive(Debug, Clone)]
pub struct GuardSeq<'token: 'token> {
    pub guards: commons::NonEmptySeq<Guard<'token>, literals::S_SEMICOLON>,
}
derive_parse!(GuardSeq, guards);
derive_token_range!(GuardSeq, guards, guards);

#[derive(Debug, Clone)]
pub struct Guard<'token: 'token> {
    pub tests: commons::NonEmptySeq<GuardTest<'token>, literals::S_COMMA>,
}
derive_parse!(Guard, tests);
derive_token_range!(Guard, tests, tests);

#[derive(Debug, Clone)]
pub enum GuardTest<'token> {
    BinaryOpCall(Box<guard_tests::BinaryOpCall<'token>>),
    RemoteCall(Box<guard_tests::RemoteCall<'token>>),
    LocalCall(Box<guard_tests::LocalCall<'token>>),
    RecordFieldAccess(guard_tests::RecordFieldAccess<'token>),
    UnaryOpCall(Box<guard_tests::UnaryOpCall<'token>>),
    Paren(Box<guard_tests::Parenthesized<'token>>),
    BitStr(Box<guard_tests::BitStr<'token>>),
    Record(Box<guard_tests::Record<'token>>),
    RecordFieldIndex(guard_tests::RecordFieldIndex<'token>),
    Map(Box<guard_tests::Map<'token>>),
    List(Box<guard_tests::List<'token>>),
    TailConsList(Box<guard_tests::TailConsList<'token>>),
    Tuple(Box<guard_tests::Tuple<'token>>),
    Var(commons::Var<'token>),
    Atom(literals::Atom<'token>),
    Char(literals::Char<'token>),
    Float(literals::Float<'token>),
    Int(literals::Int<'token>),
    Str(literals::Str<'token>),
}
derive_traits_for_enum!(GuardTest,
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
                        Str);

#[derive(Debug, Clone)]
pub enum LeftGuardTest<'token> {
    UnaryOpCall(Box<guard_tests::UnaryOpCall<'token>>),
    Paren(Box<guard_tests::Parenthesized<'token>>),
    BitStr(Box<guard_tests::BitStr<'token>>),
    Record(Box<guard_tests::Record<'token>>),
    RecordFieldIndex(guard_tests::RecordFieldIndex<'token>),
    Map(Box<guard_tests::Map<'token>>),
    List(Box<guard_tests::List<'token>>),
    TailConsList(Box<guard_tests::TailConsList<'token>>),
    Tuple(Box<guard_tests::Tuple<'token>>),
    Var(commons::Var<'token>),
    Atom(literals::Atom<'token>),
    Char(literals::Char<'token>),
    Float(literals::Float<'token>),
    Int(literals::Int<'token>),
    Str(literals::Str<'token>),
}
derive_traits_for_enum!(LeftGuardTest,
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
                        Str);
