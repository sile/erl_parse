use erl_tokenize::Token;

use super::primitives::{Atom, Export};

#[derive(Debug)]
pub struct ModuleAttr<'token, 'text: 'token> {
    pub leadings: &'token [Token<'text>], // Token* '-' Token* 'module' Token* '('
    pub module_name: Atom<'token, 'text>,
    pub trailings: &'token [Token<'text>], // Token* ')' Token* '.'
}

#[derive(Debug)]
pub struct ExportAttr<'token, 'text: 'token> {
    pub leadings: &'token [Token<'text>], // Token* '-' Token* 'export' Token* '('
    pub exports: Vec<Export<'token, 'text>>,
    pub trailings: &'token [Token<'text>], // Token* ')' Token* '.'
}
