use erl_tokenize::{Token, TokenKind, Result as TokenizeResult};
use erl_tokenize::values::Symbol;

use {Result, ErrorKind};
use parse_tree::Form;
use parse_tree::forms;
use parse_tree::primitives::{Arity, Export};
use token_reader::TokenReader;

#[derive(Debug)]
pub struct FormParser<'a, 'text: 'a, I: 'a> {
    reader: &'a mut TokenReader<'text, I>,
}
impl<'a, 'text: 'a, I: 'a> FormParser<'a, 'text, I>
    where I: Iterator<Item = TokenizeResult<Token<'text>>>
{
    pub fn new(reader: &'a mut TokenReader<'text, I>) -> Self {
        FormParser { reader }
    }
    pub fn parse_next(&mut self) -> Result<Option<Form<'text>>> {
        let token = track_try!(self.reader.peek()).map(|t| t.kind());
        match token {
            Some(TokenKind::Atom) => track!(self.parse_function_decl()).map(Some),
            Some(TokenKind::Symbol) => track!(self.parse_decl_or_attr()).map(Some),
            Some(other) => track_panic!(ErrorKind::InvalidInput, "Unrecognized token: {:?}", other),
            None => Ok(None),
        }
    }
    fn parse_function_decl(&mut self) -> Result<Form<'text>> {
        panic!()
    }
    fn parse_decl_or_attr(&mut self) -> Result<Form<'text>> {
        track_try!(self.reader.expect_symbol(Symbol::Hyphen));

        let name = track_try!(self.reader.read_atom()).clone();
        match name.value() {
            "module" => track!(self.parse_module_attr()),
            "export" => track!(self.parse_export_attr()),
            _ => panic!("{:?}", name),
        }
    }
    fn parse_module_attr(&mut self) -> Result<Form<'text>> {
        track_try!(self.reader.expect_symbol(Symbol::OpenParen));
        let name = track_try!(self.reader.read_atom()).clone();
        track_try!(self.reader.expect_symbol(Symbol::CloseParen));
        track_try!(self.reader.expect_symbol(Symbol::Dot));
        let form = forms::ModuleAttr {
            module_name: name,
            tokens: self.reader.take_read_tokens(),
        };
        Ok(form.into())
    }
    fn parse_export_attr(&mut self) -> Result<Form<'text>> {
        track_try!(self.reader.expect_symbol(Symbol::OpenParen));
        let result = self.reader
            .read_list(|r| {
                           let name = track_try!(r.read_atom()).clone();
                           track_try!(r.expect_symbol(Symbol::Slash));
                           let arity = track_try!(r.read_integer());
                           let arity = track_try!(Arity::from_integer(arity));
                           Ok(Export::new(name, arity))
                       });
        let list = track_try!(result);
        track_try!(self.reader.expect_symbol(Symbol::CloseParen));
        track_try!(self.reader.expect_symbol(Symbol::Dot));
        let form = forms::ExportAttr {
            exports: list,
            tokens: self.reader.take_read_tokens(),
        };
        Ok(form.into())
    }
}
