use erl_tokenize::Token;
use erl_tokenize::tokens::AtomToken;

use super::primitives::Export;

#[derive(Debug)]
pub struct ModuleAttr<'a> {
    pub module_name: AtomToken<'a>,
    pub tokens: Vec<Token<'a>>,
}

#[derive(Debug)]
pub struct ExportAttr<'a> {
    pub exports: Vec<Export<'a>>,
    pub tokens: Vec<Token<'a>>,
}
