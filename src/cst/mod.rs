use erl_tokenize::Token;

use {Result, TokenReader, Parse, TokenRange};

pub mod atoms;
pub mod forms;
pub mod primitives;
pub mod symbols;

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
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Form<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let position = reader.position();
        track_try!(symbols::Hyphen::parse(reader));
        let atom = track_try!(primitives::Atom::parse(reader));
        reader.set_position(position);

        match atom.value() {
            "module" => track!(Parse::parse(reader)).map(Form::ModuleAttr),
            "export" => track!(Parse::parse(reader)).map(Form::ExportAttr),
            _ => panic!(),
        }
    }
}
impl<'token, 'text: 'token> TokenRange for Form<'token, 'text> {
    fn token_start(&self) -> usize {
        match *self {
            Form::ModuleAttr(ref f) => f.token_start(),
            Form::ExportAttr(ref f) => f.token_start(),
        }
    }
    fn token_end(&self) -> usize {
        match *self {
            Form::ModuleAttr(ref f) => f.token_end(),
            Form::ExportAttr(ref f) => f.token_end(),
        }
    }
}
