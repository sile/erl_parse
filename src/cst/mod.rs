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
    ExportAttr(forms::ExportAttr<'token, 'text>),
    FunctionSpec(forms::FunctionSpec<'token, 'text>),
    FunctionDecl(forms::FunctionDecl<'token, 'text>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Form<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let position = reader.position();
        if primitives::Atom::try_parse(reader).is_some() {
            reader.set_position(position);
            track!(reader.parse_next()).map(Form::FunctionDecl)
        } else {
            track_try!(symbols::Hyphen::parse(reader));
            let atom = track_try!(primitives::Atom::parse(reader));
            reader.set_position(position);

            match atom.value() {
                "module" => track!(Parse::parse(reader)).map(Form::ModuleAttr),
                "export" => track!(Parse::parse(reader)).map(Form::ExportAttr),
                "spec" => track!(Parse::parse(reader)).map(Form::FunctionSpec),
                a => panic!("{:?}", a),
            }
        }
    }
}
impl<'token, 'text: 'token> TokenRange for Form<'token, 'text> {
    fn token_start(&self) -> usize {
        match *self {
            Form::ModuleAttr(ref f) => f.token_start(),
            Form::ExportAttr(ref f) => f.token_start(),
            Form::FunctionSpec(ref f) => f.token_start(),
            Form::FunctionDecl(ref f) => f.token_start(),
        }
    }
    fn token_end(&self) -> usize {
        match *self {
            Form::ModuleAttr(ref f) => f.token_end(),
            Form::ExportAttr(ref f) => f.token_end(),
            Form::FunctionSpec(ref f) => f.token_end(),
            Form::FunctionDecl(ref f) => f.token_end(),            
        }
    }
}

#[derive(Debug)]
pub enum Pattern<'token, 'text: 'token> {
    Atom(primitives::Atom<'token, 'text>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Pattern<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        // TODO: improve
        if let Some(t) = primitives::Atom::try_parse(reader) {
            return Ok(Pattern::Atom(t));
        }
        track_panic!(ErrorKind::InvalidInput, "Unrecognized pattern");
    }
}
impl<'token, 'text: 'token> TokenRange for Pattern<'token, 'text> {
    fn token_start(&self) -> usize {
        match *self {
            Pattern::Atom(ref t) => t.token_start(),
        }
    }
    fn token_end(&self) -> usize {
        match *self {
            Pattern::Atom(ref t) => t.token_end(),
        }
    }
}

#[derive(Debug)]
pub enum Expression<'token, 'text: 'token> {
    Atom(primitives::Atom<'token, 'text>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Expression<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        // TODO: improve
        if let Some(t) = primitives::Atom::try_parse(reader) {
            return Ok(Expression::Atom(t));
        }
        track_panic!(ErrorKind::InvalidInput, "Unrecognized expression: next={:?}", reader.read());
    }
}
impl<'token, 'text: 'token> TokenRange for Expression<'token, 'text> {
    fn token_start(&self) -> usize {
        match *self {
            Expression::Atom(ref t) => t.token_start(),
        }
    }
    fn token_end(&self) -> usize {
        match *self {
            Expression::Atom(ref t) => t.token_end(),
        }
    }
}

#[derive(Debug)]
pub enum Type<'token, 'text: 'token> {
    Atom(primitives::Atom<'token, 'text>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Type<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        // TODO: improve
        if let Some(t) = primitives::Atom::try_parse(reader) {
            return Ok(Type::Atom(t));
        }
        track_panic!(ErrorKind::InvalidInput, "Unrecognized type");
    }
}
impl<'token, 'text: 'token> TokenRange for Type<'token, 'text> {
    fn token_start(&self) -> usize {
        match *self {
            Type::Atom(ref t) => t.token_start(),
        }
    }
    fn token_end(&self) -> usize {
        match *self {
            Type::Atom(ref t) => t.token_end(),
        }
    }
}
