use erl_tokenize::Token;

pub mod forms;

#[derive(Debug)]
pub struct ModuleDecl<'a> {
    pub forms: Vec<Form<'a>>,
    pub trailing_tokens: Vec<Token<'a>>,
}
impl<'a> ModuleDecl<'a> {
    pub fn new() -> Self {
        ModuleDecl {
            forms: Vec::new(),
            trailing_tokens: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum Form<'a> {
    Module(forms::ModuleAttr<'a>),
    Foo,
}
impl<'a> Form<'a> {
    pub fn as_module_attr(&self) -> Option<&forms::ModuleAttr<'a>> {
        if let Form::Module(ref f) = *self {
            Some(f)
        } else {
            None
        }
    }
}
impl<'a> From<forms::ModuleAttr<'a>> for Form<'a> {
    fn from(f: forms::ModuleAttr<'a>) -> Self {
        Form::Module(f)
    }
}
