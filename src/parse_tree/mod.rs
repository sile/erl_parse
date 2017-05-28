use erl_tokenize::Token;

pub mod forms;
pub mod primitives;

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
    ModuleAttr(forms::ModuleAttr<'a>),
    ExportAttr(forms::ExportAttr<'a>),
}
impl<'a> Form<'a> {
    pub fn as_module_attr(&self) -> Option<&forms::ModuleAttr<'a>> {
        if let Form::ModuleAttr(ref f) = *self {
            Some(f)
        } else {
            None
        }
    }
    pub fn as_export_attr(&self) -> Option<&forms::ExportAttr<'a>> {
        if let Form::ExportAttr(ref f) = *self {
            Some(f)
        } else {
            None
        }
    }
}
impl<'a> From<forms::ModuleAttr<'a>> for Form<'a> {
    fn from(f: forms::ModuleAttr<'a>) -> Self {
        Form::ModuleAttr(f)
    }
}
impl<'a> From<forms::ExportAttr<'a>> for Form<'a> {
    fn from(f: forms::ExportAttr<'a>) -> Self {
        Form::ExportAttr(f)
    }
}
