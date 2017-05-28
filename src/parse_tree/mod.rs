pub mod forms;

#[derive(Debug)]
pub struct ModuleDecl {
    pub forms: Vec<Form>,
}

#[derive(Debug)]
pub enum Form {
    Module(forms::ModuleAttr),
}
