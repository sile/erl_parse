use erl_tokenize::Token;

pub mod forms;
pub mod primitives;
pub mod symbols;

#[derive(Debug)]
pub struct ModuleDecl<'token, 'text: 'token> {
    pub forms: Vec<Form<'token, 'text>>,
    pub trailings: &'token [Token<'text>],
}
impl<'token, 'text: 'token> ModuleDecl<'token, 'text> {}

#[derive(Debug)]
pub enum Form<'token, 'text: 'token> {
    ModuleAttr(forms::ModuleAttr<'token, 'text>),
    ExportAttr(forms::ExportAttr<'token, 'text>),
}
