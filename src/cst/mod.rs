macro_rules! derive_traits_for_enum {
    ($name:ident <>, $($variant:ident),*) => {
        impl<'token, 'text: 'token> ::Parse<'token, 'text> for $name {
            fn parse(reader: &mut ::TokenReader<'token, 'text>) -> ::Result<Self> {
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
        impl<'token, 'text: 'token, $($param),*> ::Parse<'token, 'text> for $name<$($param),*>
            where $($param: ::Parse<'token,'text>),* {
            fn parse(reader: &mut ::TokenReader<'token, 'text>) -> ::Result<Self> {
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
        impl<'token, 'text: 'token> ::Parse<'token, 'text> for $name<'token, 'text> {
            fn parse(reader: &mut ::TokenReader<'token, 'text>) -> ::Result<Self> {
                $(if let Some(t) = ::Parse::try_parse(reader) {
                    return Ok($name::$variant(t));
                })*
                track_panic!(::ErrorKind::Other,
                             "Unrecognized token for `{}`: token={:?}",
                             stringify!($name),
                             reader.read());
            }
        }
        impl<'token, 'text: 'token> ::TokenRange for $name<'token, 'text> {
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
    ($name:ident <'token, 'text, $($param:ident),* >, $($field:ident),*) => {
        impl <'token, 'text: 'token, $($param),*> ::Parse<'token, 'text> for
            $name <'token, 'text, $($param),* >
            where $($param: ::Parse<'token, 'text>),* {
            fn parse(reader: &mut ::TokenReader<'token, 'text>) -> ::Result<Self> {
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
        impl <'token, 'text: 'token, $($param),*> ::Parse<'token, 'text> for $name < $($param),* >
            where $($param: ::Parse<'token, 'text>),* {
            fn parse(reader: &mut ::TokenReader<'token, 'text>) -> ::Result<Self> {
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
        impl <'token, 'text: 'token> ::Parse<'token, 'text> for $name<'token, 'text> {
            fn parse(reader: &mut ::TokenReader<'token, 'text>) -> ::Result<Self> {
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
    ($name:ident <'token, 'text, $($param:ident),* >, $first:ident, $last:ident) => {
        impl <'token, 'text: 'token, $($param),*> ::TokenRange for $name <'token, 'text, $($param),* >
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
        impl <'token, 'text:'token> ::TokenRange for $name<'token, 'text> {
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
        impl<'token, 'text: 'token> Deref for $name<'token, 'text> {
            type Target = $token<'text>;
            fn deref(&self) -> &Self::Target {
                self.value
            }
        }
        impl<'token, 'text: 'token> ::Parse<'token, 'text> for $name<'token, 'text> {
            fn parse(reader: &mut ::TokenReader<'token, 'text>) -> ::Result<Self> {
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
        impl<'token, 'text: 'token> ::TokenRange for $name<'token, 'text> {
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
pub mod guard_tests;
pub mod literals;
pub mod patterns;

#[derive(Debug, Clone)]
pub enum Expr<'token, 'text: 'token> {
    // TODO: remote call
    // TODO: local call
    // TODO: binary op
    // TODO: map update
    // TODO: record update, field access
    
    // TODO: try
    // TOOD: receive
    // TODO: match
    // TODO: list comprehension
    // TODO: bitstring comprehension
    // TODO: if
    // TOOD: named fun
    // TODO: anonymous fun
    RemoteFun(Box<exprs::RemoteFun<'token, 'text>>),
    LocalFun(Box<exprs::LocalFun<'token, 'text>>),
    UnaryOpCall(Box<exprs::UnaryOpCall<'token, 'text>>),
    Catch(Box<exprs::Catch<'token, 'text>>),
    Paren(Box<exprs::Parenthesized<'token, 'text>>),
    // Case(Box<exprs::Case<'token, 'text>>),
    Block(Box<exprs::Block<'token, 'text>>),
    BitStr(Box<exprs::BitStr<'token, 'text>>),
    Record(Box<exprs::Record<'token, 'text>>),
    RecordFieldIndex(exprs::RecordFieldIndex<'token, 'text>),
    Map(Box<exprs::Map<'token, 'text>>),
    List(Box<exprs::List<'token, 'text>>),
    TailConsList(Box<exprs::TailConsList<'token, 'text>>),
    Tuple(Box<exprs::Tuple<'token, 'text>>),
    Var(commons::Var<'token, 'text>),
    Atom(literals::Atom<'token, 'text>),
    Char(literals::Char<'token, 'text>),
    Float(literals::Float<'token, 'text>),
    Int(literals::Int<'token, 'text>),
    Str(literals::Str<'token, 'text>),
}
derive_traits_for_enum!(Expr,
                        RemoteFun,
                        LocalFun,
                        UnaryOpCall,
                        Catch,
                        Paren,
                        // Case,
                        Block,
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

#[derive(Debug, Clone)]
pub enum LeftExpr<'token, 'text: 'token> {
    RemoteFun(Box<exprs::RemoteFun<'token, 'text>>),
    LocalFun(Box<exprs::LocalFun<'token, 'text>>),
    UnaryOpCall(Box<exprs::UnaryOpCall<'token, 'text>>),
    Catch(Box<exprs::Catch<'token, 'text>>),
    Paren(Box<exprs::Parenthesized<'token, 'text>>),
    Block(Box<exprs::Block<'token, 'text>>),
    BitStr(Box<exprs::BitStr<'token, 'text>>),
    Record(Box<exprs::Record<'token, 'text>>),
    RecordFieldIndex(exprs::RecordFieldIndex<'token, 'text>),
    Map(Box<exprs::Map<'token, 'text>>),
    List(Box<exprs::List<'token, 'text>>),
    TailConsList(Box<exprs::TailConsList<'token, 'text>>),
    Tuple(Box<exprs::Tuple<'token, 'text>>),
    Var(commons::Var<'token, 'text>),
    Atom(literals::Atom<'token, 'text>),
    Char(literals::Char<'token, 'text>),
    Float(literals::Float<'token, 'text>),
    Int(literals::Int<'token, 'text>),
    Str(literals::Str<'token, 'text>),
}
derive_traits_for_enum!(LeftExpr,
                        RemoteFun,
                        LocalFun,
                        UnaryOpCall,
                        Catch,
                        Paren,
                        Block,
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

#[derive(Debug, Clone)]
pub enum Pattern<'token, 'text: 'token> {
    Match(Box<patterns::Match<'token, 'text>>),
    BinaryOpCall(Box<patterns::BinaryOpCall<'token, 'text>>),
    UnaryOpCall(Box<patterns::UnaryOpCall<'token,'text>>),
    Paren(Box<patterns::Parenthesized<'token, 'text>>),    
    Record(Box<patterns::Record<'token, 'text>>),
    RecordFieldIndex(patterns::RecordFieldIndex<'token, 'text>),
    Map(Box<patterns::Map<'token, 'text>>),
    Tuple(Box<patterns::Tuple<'token, 'text>>),
    List(Box<patterns::List<'token, 'text>>),
    TailConsList(Box<patterns::TailConsList<'token, 'text>>),
    BitStr(Box<patterns::BitStr<'token, 'text>>),
    Var(commons::Var<'token, 'text>),
    Atom(literals::Atom<'token, 'text>),
    Char(literals::Char<'token, 'text>),
    Float(literals::Float<'token, 'text>),
    Int(literals::Int<'token, 'text>),
    Str(literals::Str<'token, 'text>),
}
derive_traits_for_enum!(Pattern,
                        Match, BinaryOpCall,
                        UnaryOpCall, Paren, Record, RecordFieldIndex,
                        Map, Tuple, List, TailConsList, BitStr,
                        Var, Atom, Char, Float, Int, Str);

#[derive(Debug, Clone)]
pub enum LeftPattern<'token, 'text: 'token> {
    Paren(Box<patterns::Parenthesized<'token, 'text>>),
    UnaryOpCall(Box<patterns::UnaryOpCall<'token,'text>>),
    Record(Box<patterns::Record<'token, 'text>>),
    RecordFieldIndex(patterns::RecordFieldIndex<'token, 'text>),
    Map(Box<patterns::Map<'token, 'text>>),
    Tuple(Box<patterns::Tuple<'token, 'text>>),
    List(Box<patterns::List<'token, 'text>>),
    TailConsList(Box<patterns::TailConsList<'token, 'text>>),
    BitStr(Box<patterns::BitStr<'token, 'text>>),
    Var(commons::Var<'token, 'text>),
    Atom(literals::Atom<'token, 'text>),
    Char(literals::Char<'token, 'text>),
    Float(literals::Float<'token, 'text>),
    Int(literals::Int<'token, 'text>),
    Str(literals::Str<'token, 'text>),
}
derive_traits_for_enum!(LeftPattern, Paren, UnaryOpCall, Record, RecordFieldIndex,
                        Map, Tuple, List, TailConsList, BitStr,
                        Var, Atom, Char, Float, Int, Str);

#[derive(Debug, Clone)]
pub struct GuardSeq<'token, 'text:'token> {
    pub guards: commons::NonEmptySeq<Guard<'token, 'text>, literals::S_SEMICOLON>
}
derive_parse!(GuardSeq, guards);
derive_token_range!(GuardSeq, guards, guards);

#[derive(Debug, Clone)]
pub struct Guard<'token, 'text:'token> {
    pub tests: commons::NonEmptySeq<GuardTest<'token, 'text>, literals::S_COMMA>
}
derive_parse!(Guard, tests);
derive_token_range!(Guard, tests, tests);

#[derive(Debug, Clone)]
pub enum GuardTest<'token, 'text: 'token> {
    // TODO: remote call
    // TODO: local call
    // TODO: binary op
    // TODO: map update
    // TODO: record field access
    UnaryOpCall(Box<guard_tests::UnaryOpCall<'token, 'text>>),
    Paren(Box<guard_tests::Parenthesized<'token, 'text>>),
    BitStr(Box<guard_tests::BitStr<'token, 'text>>),
    Record(Box<guard_tests::Record<'token, 'text>>),
    RecordFieldIndex(guard_tests::RecordFieldIndex<'token, 'text>),
    Map(Box<guard_tests::Map<'token, 'text>>),
    List(Box<guard_tests::List<'token, 'text>>),
    TailConsList(Box<guard_tests::TailConsList<'token, 'text>>),
    Tuple(Box<guard_tests::Tuple<'token, 'text>>),
    Var(commons::Var<'token, 'text>),
    Atom(literals::Atom<'token, 'text>),
    Char(literals::Char<'token, 'text>),
    Float(literals::Float<'token, 'text>),
    Int(literals::Int<'token, 'text>),
    Str(literals::Str<'token, 'text>),
}
derive_traits_for_enum!(GuardTest,
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

#[derive(Debug, Clone)]
pub enum LeftGuardTest<'token, 'text: 'token> {
    // TODO: remote call
    // TODO: local call
    // TODO: binary op
    // TODO: map update
    // TODO: record field access
    UnaryOpCall(Box<guard_tests::UnaryOpCall<'token, 'text>>),
    Paren(Box<guard_tests::Parenthesized<'token, 'text>>),
    BitStr(Box<guard_tests::BitStr<'token, 'text>>),
    Record(Box<guard_tests::Record<'token, 'text>>),
    RecordFieldIndex(guard_tests::RecordFieldIndex<'token, 'text>),
    Map(Box<guard_tests::Map<'token, 'text>>),
    List(Box<guard_tests::List<'token, 'text>>),
    TailConsList(Box<guard_tests::TailConsList<'token, 'text>>),
    Tuple(Box<guard_tests::Tuple<'token, 'text>>),
    Var(commons::Var<'token, 'text>),
    Atom(literals::Atom<'token, 'text>),
    Char(literals::Char<'token, 'text>),
    Float(literals::Float<'token, 'text>),
    Int(literals::Int<'token, 'text>),
    Str(literals::Str<'token, 'text>),
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
