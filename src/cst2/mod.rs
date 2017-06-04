use std::ops::Range;
use erl_tokenize::Token;

use {Result, TokenReader, Parse, TokenRange, ErrorKind};
use self::primitives::Atom;
use self::symbols::Hyphen;

pub mod atoms;
pub mod clauses;
pub mod exprs;
pub mod forms;
pub mod keywords;
pub mod patterns;
pub mod primitives;
pub mod symbols;
pub mod types;

#[derive(Debug)]
pub struct ModuleDecl<'token, 'text: 'token> {
    position: usize,
    pub forms: Vec<Form<'token, 'text>>,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for ModuleDecl<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let position = reader.position();
        let mut forms = Vec::new();
        loop {
            match track!(Form::parse(reader)) {
                Ok(form) => {
                    forms.push(form);
                }
                Err(e) => {
                    let eos = reader
                        .remaining_tokens()
                        .iter()
                        .all(|t| match *t {
                                 Token::Comment(_) |
                                 Token::Whitespace(_) => true,
                                 _ => false,
                             });
                    if eos {
                        break;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        Ok(ModuleDecl { position, forms })
    }
}
impl<'token, 'text: 'token> TokenRange for ModuleDecl<'token, 'text> {
    fn token_start(&self) -> usize {
        self.position
    }
    fn token_end(&self) -> usize {
        self.forms.last().map_or(self.position, |f| f.token_end())
    }
}

#[derive(Debug)]
pub enum Form<'token, 'text: 'token> {
    ModuleAttr(forms::ModuleAttr<'token, 'text>),
    BehaviourAttr(forms::BehaviourAttr<'token, 'text>),
    ExportAttr(forms::ExportAttr<'token, 'text>),
    ImportAttr(forms::ImportAttr<'token, 'text>),
    ExportTypeAttr(forms::ExportTypeAttr<'token, 'text>),
    FileAttr(forms::FileAttr<'token, 'text>),
    WildAttr(forms::WildAttr<'token, 'text>),
    FunSpec(forms::FunSpec<'token, 'text>),
    FunDecl(forms::FunDecl<'token, 'text>),
    TypeDecl(forms::TypeDecl<'token, 'text>),
    RecordDecl(forms::RecordDecl<'token, 'text>),
    MacroDecl(forms::MacroDecl<'token, 'text>),
    MacroDirective(forms::MacroDirective<'token, 'text>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Form<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        if reader.peek::<Atom>().is_ok() {
            parse!(reader).map(Form::FunDecl)
        } else {
            let (_, atom): (Hyphen, Atom) = track_try!(reader.peek());
            match atom.value() {
                "module" => parse!(reader).map(Form::ModuleAttr),
                "behaviour" | "behavior" => parse!(reader).map(Form::BehaviourAttr),
                "export" => parse!(reader).map(Form::ExportAttr),
                "import" => parse!(reader).map(Form::ImportAttr),
                "export_type" => parse!(reader).map(Form::ExportTypeAttr),
                "spec" | "callback" => parse!(reader).map(Form::FunSpec),
                "file" => parse!(reader).map(Form::FileAttr),
                "type" | "opaque" => parse!(reader).map(Form::TypeDecl),
                "record" => parse!(reader).map(Form::RecordDecl),
                "define" => parse!(reader).map(Form::MacroDecl),
                "undef" | "ifdef" | "ifndef" | "else" | "endif" | "warning" | "error" => {
                    parse!(reader).map(Form::MacroDirective)
                }
                _ => parse!(reader).map(Form::WildAttr),
            }
        }
    }
}
impl<'token, 'text: 'token> TokenRange for Form<'token, 'text> {
    fn token_range(&self) -> Range<usize> {
        match *self {
            Form::ModuleAttr(ref f) => f.token_range(),
            Form::BehaviourAttr(ref f) => f.token_range(),
            Form::ExportAttr(ref f) => f.token_range(),
            Form::ImportAttr(ref f) => f.token_range(),
            Form::ExportTypeAttr(ref f) => f.token_range(),
            Form::FileAttr(ref f) => f.token_range(),
            Form::WildAttr(ref f) => f.token_range(),
            Form::FunSpec(ref f) => f.token_range(),
            Form::FunDecl(ref f) => f.token_range(),
            Form::TypeDecl(ref f) => f.token_range(),
            Form::RecordDecl(ref f) => f.token_range(),
            Form::MacroDecl(ref f) => f.token_range(),
            Form::MacroDirective(ref f) => f.token_range(),
        }
    }
}

#[derive(Debug)]
pub enum Pattern<'token, 'text: 'token> {
    Integer(primitives::Integer<'token, 'text>),
    Atom(primitives::Atom<'token, 'text>),
    Variable(primitives::Variable<'token, 'text>),
    Tuple(Box<patterns::Tuple<'token, 'text>>),
    List(Box<patterns::List<'token, 'text>>),
    // TODO: 暫定的な対処
    MacroCall(Box<patterns::MacroCall<'token, 'text>>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Pattern<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        // TODO: improve
        if let Some(t) = reader.try_parse_next() {
            Ok(Pattern::Integer(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Pattern::Atom(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Pattern::Variable(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Pattern::Tuple(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Pattern::List(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Pattern::MacroCall(t))
        } else {
            track_panic!(ErrorKind::InvalidInput,
                         "Unrecognized pattern: next={:?}",
                         reader.read());
        }
    }
}
impl<'token, 'text: 'token> TokenRange for Pattern<'token, 'text> {
    fn token_range(&self) -> Range<usize> {
        match *self {
            Pattern::Integer(ref t) => t.token_range(),
            Pattern::Atom(ref t) => t.token_range(),
            Pattern::Variable(ref t) => t.token_range(),
            Pattern::Tuple(ref t) => t.token_range(),
            Pattern::List(ref t) => t.token_range(),
            Pattern::MacroCall(ref t) => t.token_range(),
        }
    }
}

#[derive(Debug)]
pub enum Expr<'token, 'text: 'token> {
    Integer(primitives::Integer<'token, 'text>),
    Atom(primitives::Atom<'token, 'text>),
    Variable(primitives::Variable<'token, 'text>),
    List(Box<exprs::List<'token, 'text>>),
    Tuple(Box<exprs::Tuple<'token, 'text>>),
    Try(Box<exprs::Try<'token, 'text>>),
    // LocalCall(Box<exprs::LocalCall<'token, 'text>>),
    // RemoteCall(Box<exprs::RemoteCall<'token, 'text>>),
    Call(Box<exprs::Call<'token, 'text>>),
    BinaryOpCall(Box<exprs::BinaryOpCall<'token, 'text>>),
    Match(Box<exprs::Match<'token, 'text>>),
    AnonymousFun(Box<exprs::AnonymousFun<'token, 'text>>),
    LocalFun(Box<exprs::LocalFun<'token, 'text>>),
    RemoteFun(Box<exprs::RemoteFun<'token, 'text>>),
    Case(Box<exprs::Case<'token, 'text>>),

    // TODO: 暫定的な対処
    MacroCall(Box<exprs::MacroCall<'token, 'text>>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Expr<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        if let Some(t) = reader.try_parse_next2(0) {
            Ok(Expr::BinaryOpCall(Box::new(t)))
            // } else if let Some(t) = reader.try_parse_next2(4) {
            //     Ok(Expr::RemoteCall(t))
            // } else if let Some(t) = reader.try_parse_next2(1) {
            //     Ok(Expr::LocalCall(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Expr::AnonymousFun(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Expr::LocalFun(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Expr::RemoteFun(t))
        } else if let Some(t) = reader.try_parse_next2(1) {
            Ok(Expr::Call(t))
        } else if let Some(t) = reader.try_parse_next2(3) {
            Ok(Expr::Match(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Expr::Integer(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Expr::Variable(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Expr::Atom(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Expr::List(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Expr::Tuple(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Expr::Try(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Expr::Case(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Expr::MacroCall(t))
        } else {
            track_panic!(ErrorKind::InvalidInput,
                         "Unrecognized expr: {:?}",
                         reader.read());
        }
    }
}
impl<'token, 'text: 'token> TokenRange for Expr<'token, 'text> {
    fn token_range(&self) -> Range<usize> {
        match *self {
            Expr::Integer(ref t) => t.token_range(),
            Expr::Atom(ref t) => t.token_range(),
            Expr::Variable(ref t) => t.token_range(),
            // Expr::LocalCall(ref t) => t.token_range(),
            // Expr::RemoteCall(ref t) => t.token_range(),
            Expr::Call(ref t) => t.token_range(),
            Expr::BinaryOpCall(ref t) => t.token_range(),
            Expr::List(ref t) => t.token_range(),
            Expr::Tuple(ref t) => t.token_range(),
            Expr::Try(ref t) => t.token_range(),
            Expr::Case(ref t) => t.token_range(),
            Expr::Match(ref t) => t.token_range(),
            Expr::AnonymousFun(ref t) => t.token_range(),
            Expr::LocalFun(ref t) => t.token_range(),
            Expr::RemoteFun(ref t) => t.token_range(),
            Expr::MacroCall(ref t) => t.token_range(),
        }
    }
}

// TODO: rename
#[derive(Debug)]
pub enum IdExpr<'token, 'text: 'token> {
    Atom(primitives::Atom<'token, 'text>),
    Variable(primitives::Variable<'token, 'text>),
    // TODO: parenthesized
}
impl<'token, 'text: 'token> Parse<'token, 'text> for IdExpr<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        if let Some(t) = reader.try_parse_next() {
            Ok(IdExpr::Variable(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(IdExpr::Atom(t))
        } else {
            track_panic!(ErrorKind::InvalidInput,
                         "Unrecognized id expr: {:?}",
                         reader.read());
        }
    }
}
impl<'token, 'text: 'token> TokenRange for IdExpr<'token, 'text> {
    fn token_range(&self) -> Range<usize> {
        match *self {
            IdExpr::Atom(ref t) => t.token_range(),
            IdExpr::Variable(ref t) => t.token_range(),
        }
    }
}

#[derive(Debug)]
pub enum Type<'token, 'text: 'token> {
    AnyArgFun(Box<types::AnyArgFun<'token, 'text>>),
    Fun(Box<types::Fun<'token, 'text>>),
    Local(types::LocalType<'token, 'text>),
    Remote(types::RemoteType<'token, 'text>),
    Atom(primitives::Atom<'token, 'text>),
    Int(primitives::Int<'token, 'text>),
    IntRange(types::IntRange<'token, 'text>),
    List(Box<types::List<'token, 'text>>),
    Tuple(Box<types::Tuple<'token, 'text>>),
    Annotated(Box<types::Annotated<'token, 'text>>),
    Parenthesized(Box<types::Parenthesized<'token, 'text>>),
    Union(Box<types::Union<'token, 'text>>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Type<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        if let Some(t) = reader.try_parse_next2(2) {
            Ok(Type::Union(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Type::AnyArgFun(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Type::Fun(t))
        } else if let Some(t) = types::LocalType::try_parse(reader) {
            Ok(Type::Local(t))
        } else if let Some(t) = types::RemoteType::try_parse(reader) {
            Ok(Type::Remote(t))
        } else if let Some(t) = primitives::Atom::try_parse(reader) {
            Ok(Type::Atom(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Type::IntRange(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Type::Int(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Type::List(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Type::Tuple(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Type::Annotated(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(Type::Parenthesized(t))
        } else {
            track_panic!(ErrorKind::InvalidInput,
                         "Unrecognized type: {:?}",
                         reader.read());
        }
    }
}
impl<'token, 'text: 'token> TokenRange for Type<'token, 'text> {
    fn token_range(&self) -> Range<usize> {
        match *self {
            Type::Local(ref t) => t.token_range(),
            Type::Remote(ref t) => t.token_range(),
            Type::Atom(ref t) => t.token_range(),
            Type::Int(ref t) => t.token_range(),
            Type::IntRange(ref t) => t.token_range(),
            Type::List(ref t) => t.token_range(),
            Type::Tuple(ref t) => t.token_range(),
            Type::Annotated(ref t) => t.token_range(),
            Type::Parenthesized(ref t) => t.token_range(),
            Type::Union(ref t) => t.token_range(),
            Type::Fun(ref t) => t.token_range(),
            Type::AnyArgFun(ref t) => t.token_range(),
        }
    }
}

#[derive(Debug)]
pub enum Term<'token, 'text: 'token> {
    Atom(primitives::Atom<'token, 'text>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Term<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        // TODO: improve
        if let Some(t) = primitives::Atom::try_parse(reader) {
            Ok(Term::Atom(t))
        } else {
            track_panic!(ErrorKind::InvalidInput, "Unrecognized term");
        }
    }
}
impl<'token, 'text: 'token> TokenRange for Term<'token, 'text> {
    fn token_start(&self) -> usize {
        match *self {
            Term::Atom(ref t) => t.token_start(),
        }
    }
    fn token_end(&self) -> usize {
        match *self {
            Term::Atom(ref t) => t.token_end(),
        }
    }
}
