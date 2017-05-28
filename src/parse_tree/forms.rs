use erl_tokenize::Token;
use erl_tokenize::tokens::AtomToken;

#[derive(Debug)]
pub struct ModuleAttr<'a> {
    pub module_name: AtomToken<'a>,
    pub tokens: Vec<Token<'a>>,
}
