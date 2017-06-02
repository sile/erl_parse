use std::ops::Range;
use erl_tokenize::Token;

use {Result, TokenReader, Parse, TokenRange, ErrorKind};
use self::primitives::Atom;
use self::symbols::Hyphen;

pub mod atoms;
pub mod clauses;
pub mod exprs;
pub mod forms;
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
        } else {
            track_panic!(ErrorKind::InvalidInput,
                         "Unrecognized pattern: next={:?}",
                         reader.read());
        }
    }
}
impl<'token, 'text: 'token> TokenRange for Pattern<'token, 'text> {
    fn token_start(&self) -> usize {
        match *self {
            Pattern::Integer(ref t) => t.token_start(),
            Pattern::Atom(ref t) => t.token_start(),
            Pattern::Variable(ref t) => t.token_start(),
        }
    }
    fn token_end(&self) -> usize {
        match *self {
            Pattern::Integer(ref t) => t.token_end(),
            Pattern::Atom(ref t) => t.token_end(),
            Pattern::Variable(ref t) => t.token_end(),
        }
    }
}

#[derive(Debug)]
pub enum Expr<'token, 'text: 'token> {
    Integer(primitives::Integer<'token, 'text>),
    Atom(primitives::Atom<'token, 'text>),
    Variable(primitives::Variable<'token, 'text>),
    LocalCall(exprs::LocalCall<'token, 'text>),
    BinaryOpCall(Box<exprs::BinaryOpCall<'token, 'text>>),
    List(Box<exprs::List<'token, 'text>>),
    Try(Box<expr::Try<'token, 'text>>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Expr<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        // TODO: improve
        let expr = if let Some(t) = reader.try_parse_next() {
            Expr::Integer(t)
        } else if let Some(t) = reader.try_parse_next() {
            Expr::LocalCall(t)
        } else if let Some(t) = reader.try_parse_next() {
            Expr::Variable(t)
        } else if let Some(t) = reader.try_parse_next() {
            Expr::Atom(t)
        } else if let Some(t) = reader.try_parse_next() {
            Expr::List(Box::new(t))
        } else if let Some(t) = reader.try_parse_next() {
            Expr::Try(Box::new(t))
        } else {
            // reader.skip_hidden_tokens();
            track_panic!(ErrorKind::InvalidInput,
                         "Unrecognized expression: next={:?}",
                         reader.read());
        };
        if let Some(op) = exprs::BinaryOp::try_parse(reader) {
            let right = track_try!(reader.parse_next());
            let expr = exprs::BinaryOpCall {
                left: expr,
                op,
                right,
            };
            Ok(Expr::BinaryOpCall(Box::new(expr)))
        } else {
            Ok(expr)
        }
    }
}
impl<'token, 'text: 'token> TokenRange for Expr<'token, 'text> {
    fn token_range(&self) -> Range<usize> {
        match *self {
            Expr::Integer(ref t) => t.token_range(),
            Expr::Atom(ref t) => t.token_range(),
            Expr::Variable(ref t) => t.token_range(),
            Expr::LocalCall(ref t) => t.token_range(),
            Expr::BinaryOpCall(ref t) => t.token_range(),
            Expr::List(ref t) => t.token_range(),
            Expr::Try(ref t) => t.token_range(),
        }
    }
}

// XXX: name => NonUnionType (?)
#[derive(Debug)]
enum NonRecursiveType<'token, 'text: 'token> {
    Local(types::LocalType<'token, 'text>),
    Remote(types::RemoteType<'token, 'text>),
    Atom(primitives::Atom<'token, 'text>),
    Int(primitives::Int<'token, 'text>),
    IntRange(types::IntRange<'token, 'text>),
    List(types::List<'token, 'text>),
    Tuple(types::Tuple<'token, 'text>),
    Annotated(types::Annotated<'token, 'text>),
    Parenthesized(types::Parenthesized<'token, 'text>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for NonRecursiveType<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        // TODO: improve
        if let Some(t) = types::LocalType::try_parse(reader) {
            Ok(NonRecursiveType::Local(t))
        } else if let Some(t) = types::RemoteType::try_parse(reader) {
            Ok(NonRecursiveType::Remote(t))
        } else if let Some(t) = primitives::Atom::try_parse(reader) {
            Ok(NonRecursiveType::Atom(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(NonRecursiveType::IntRange(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(NonRecursiveType::Int(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(NonRecursiveType::List(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(NonRecursiveType::Tuple(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(NonRecursiveType::Annotated(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(NonRecursiveType::Parenthesized(t))
        } else {
            track_panic!(ErrorKind::InvalidInput, "Unrecognized type");
        }
    }
}
impl<'token, 'text: 'token> From<NonRecursiveType<'token, 'text>> for Type<'token, 'text> {
    fn from(f: NonRecursiveType<'token, 'text>) -> Self {
        match f {
            NonRecursiveType::Local(t) => Type::Local(t),
            NonRecursiveType::Remote(t) => Type::Remote(t),
            NonRecursiveType::Atom(t) => Type::Atom(t),
            NonRecursiveType::Int(t) => Type::Int(t),
            NonRecursiveType::IntRange(t) => Type::IntRange(t),
            NonRecursiveType::List(t) => Type::List(Box::new(t)),
            NonRecursiveType::Tuple(t) => Type::Tuple(Box::new(t)),
            NonRecursiveType::Annotated(t) => Type::Annotated(Box::new(t)),
            NonRecursiveType::Parenthesized(t) => Type::Parenthesized(Box::new(t)),
        }
    }
}

#[derive(Debug)]
pub enum Type<'token, 'text: 'token> {
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
        if let Some(t) = types::Union::try_parse(reader) {
            Ok(Type::Union(Box::new(t)))
        } else if let Some(t) = NonRecursiveType::try_parse(reader) {
            Ok(t.into())
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
