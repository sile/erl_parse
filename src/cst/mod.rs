use std::ops::Range;
use erl_tokenize::Token;

use {Result, TokenReader, Parse, TokenRange, ErrorKind};

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
    //    BehaviourAttr(forms::BehaviourAttr<'token, 'text>),
    ExportAttr(forms::ExportAttr<'token, 'text>),
    FunSpec(forms::FunctionSpec<'token, 'text>),
    FunDecl(forms::FunctionDecl<'token, 'text>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Form<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        use self::symbols::Hyphen;
        use self::primitives::Atom; // TODO
        if reader.peek::<Atom>().is_ok() {
            parse!(reader).map(Form::FunDecl)
        } else {
            let (_, atom): (Hyphen, Atom) = track_try!(reader.peek());
            match atom.value() {
                "module" => parse!(reader).map(Form::ModuleAttr),
                //"behaviour" | "behavior" => try_parse!(reader),
                "export" => parse!(reader).map(Form::ExportAttr),
                "spec" => parse!(reader).map(Form::FunSpec),
                a => panic!("{:?}", a),
            }
        }
    }
}
impl<'token, 'text: 'token> TokenRange for Form<'token, 'text> {
    fn token_range(&self) -> Range<usize> {
        match *self {
            Form::ModuleAttr(ref f) => f.token_range(),
            Form::ExportAttr(ref f) => f.token_range(),
            Form::FunSpec(ref f) => f.token_range(),
            Form::FunDecl(ref f) => f.token_range(),
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
pub enum Expression<'token, 'text: 'token> {
    Integer(primitives::Integer<'token, 'text>),
    Atom(primitives::Atom<'token, 'text>),
    Variable(primitives::Variable<'token, 'text>),
    LocalCall(exprs::LocalCall<'token, 'text>),
    BinaryOpCall(Box<exprs::BinaryOpCall<'token, 'text>>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Expression<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        // TODO: improve
        let expr = if let Some(t) = reader.try_parse_next() {
            Expression::Integer(t)
        } else if let Some(t) = reader.try_parse_next() {
            Expression::LocalCall(t)
        } else if let Some(t) = reader.try_parse_next() {
            Expression::Variable(t)
        } else if let Some(t) = reader.try_parse_next() {
            Expression::Atom(t)
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
            Ok(Expression::BinaryOpCall(Box::new(expr)))
        } else {
            Ok(expr)
        }
    }
}
impl<'token, 'text: 'token> TokenRange for Expression<'token, 'text> {
    fn token_start(&self) -> usize {
        match *self {
            Expression::Integer(ref t) => t.token_start(),
            Expression::Atom(ref t) => t.token_start(),
            Expression::Variable(ref t) => t.token_start(),
            Expression::LocalCall(ref t) => t.token_start(),
            Expression::BinaryOpCall(ref t) => t.token_start(),
        }
    }
    fn token_end(&self) -> usize {
        match *self {
            Expression::Integer(ref t) => t.token_end(),
            Expression::Atom(ref t) => t.token_end(),
            Expression::Variable(ref t) => t.token_end(),
            Expression::LocalCall(ref t) => t.token_end(),
            Expression::BinaryOpCall(ref t) => t.token_end(),
        }
    }
}

#[derive(Debug)]
pub enum Type<'token, 'text: 'token> {
    Local(types::LocalType<'token, 'text>),
    Atom(primitives::Atom<'token, 'text>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Type<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        // TODO: improve
        if let Some(t) = types::LocalType::try_parse(reader) {
            Ok(Type::Local(t))
        } else if let Some(t) = primitives::Atom::try_parse(reader) {
            Ok(Type::Atom(t))
        } else {
            track_panic!(ErrorKind::InvalidInput, "Unrecognized type");
        }
    }
}
impl<'token, 'text: 'token> TokenRange for Type<'token, 'text> {
    fn token_start(&self) -> usize {
        match *self {
            Type::Local(ref t) => t.token_start(),
            Type::Atom(ref t) => t.token_start(),
        }
    }
    fn token_end(&self) -> usize {
        match *self {
            Type::Local(ref t) => t.token_end(),
            Type::Atom(ref t) => t.token_end(),
        }
    }
}
