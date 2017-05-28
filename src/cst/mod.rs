use erl_tokenize::Token;

use self::primitives::HiddenToken;

pub mod forms;
pub mod primitives;

#[derive(Debug)]
pub struct ModuleDecl<'token, 'text: 'token> {
    pub forms: Vec<Form<'token, 'text>>,
    pub trailings: &'token [HiddenToken<'text>],
}
impl<'token, 'text: 'token> ModuleDecl<'token, 'text> {}

#[derive(Debug)]
pub enum Form<'token, 'text: 'token> {
    ModuleAttr(forms::ModuleAttr<'token, 'text>),
    ExportAttr(forms::ExportAttr<'token, 'text>),
}
