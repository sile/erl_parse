use super::{Term, Type, Expression, TokenRange};
use super::atoms;
use super::clauses;
use super::primitives::{Atom, List, Export, Import, ModuleAtom, Clauses, ExportType, Str, Integer,
                        Args, Variable, Tuple};
use super::symbols;
use super::types;

#[derive(Debug)]
pub struct ModuleAttr<'token, 'text: 'token> {
    pub _hyphen: symbols::Hyphen,
    pub _module: atoms::Module,
    pub _open: symbols::OpenParen,
    pub module_name: Atom<'token, 'text>,
    pub _close: symbols::CloseParen,
    pub _dot: symbols::Dot,
}
derive_parse!(ModuleAttr,
              _hyphen,
              _module,
              _open,
              module_name,
              _close,
              _dot);
derive_token_range!(ModuleAttr, _hyphen, _dot);

#[derive(Debug)]
pub struct BehaviourAttr<'token, 'text: 'token> {
    pub _hyphen: symbols::Hyphen,
    pub _behaviour: atoms::Behaviour,
    pub _open: symbols::OpenParen,
    pub behaviour_name: Atom<'token, 'text>,
    pub _close: symbols::CloseParen,
    pub _dot: symbols::Dot,
}
derive_parse!(BehaviourAttr,
              _hyphen,
              _behaviour,
              _open,
              behaviour_name,
              _close,
              _dot);
derive_token_range!(BehaviourAttr, _hyphen, _dot);

#[derive(Debug)]
pub struct ExportAttr<'token, 'text: 'token> {
    pub _hyphen: symbols::Hyphen,
    pub _export: atoms::Export,
    pub _open: symbols::OpenParen,
    pub exports: List<Export<'token, 'text>>,
    pub _close: symbols::CloseParen,
    pub _dot: symbols::Dot,
}
derive_parse!(ExportAttr, _hyphen, _export, _open, exports, _close, _dot);
derive_token_range!(ExportAttr, _hyphen, _dot);

#[derive(Debug)]
pub struct ImportAttr<'token, 'text: 'token> {
    pub _hyphen: symbols::Hyphen,
    pub _import: atoms::Import,
    pub _open: symbols::OpenParen,
    pub module_name: Atom<'token, 'text>,
    pub _comma: symbols::Comma,
    pub imports: List<Import<'token, 'text>>,
    pub _close: symbols::CloseParen,
    pub _dot: symbols::Dot,
}
derive_parse!(ImportAttr,
              _hyphen,
              _import,
              _open,
              module_name,
              _comma,
              imports,
              _close,
              _dot);
derive_token_range!(ImportAttr, _hyphen, _dot);

#[derive(Debug)]
pub struct ExportTypeAttr<'token, 'text: 'token> {
    pub _hyphen: symbols::Hyphen,
    pub _export_type: atoms::ExportType,
    pub _open: symbols::OpenParen,
    pub exports: List<ExportType<'token, 'text>>,
    pub _close: symbols::CloseParen,
    pub _dot: symbols::Dot,
}
derive_parse!(ExportTypeAttr,
              _hyphen,
              _export_type,
              _open,
              exports,
              _close,
              _dot);
derive_token_range!(ExportTypeAttr, _hyphen, _dot);

#[derive(Debug)]
pub struct FileAttr<'token, 'text: 'token> {
    pub _hyphen: symbols::Hyphen,
    pub _file: atoms::File,
    pub _open: symbols::OpenParen,
    pub file_name: Str<'token, 'text>,
    pub _comma: symbols::Comma,
    pub line_num: Integer<'token, 'text>,
    pub _close: symbols::CloseParen,
    pub _dot: symbols::Dot,
}
derive_parse!(FileAttr,
              _hyphen,
              _file,
              _open,
              file_name,
              _comma,
              line_num,
              _close,
              _dot);
derive_token_range!(FileAttr, _hyphen, _dot);

#[derive(Debug)]
pub struct WildAttr<'token, 'text: 'token> {
    pub _hyphen: symbols::Hyphen,
    pub attr_name: Atom<'token, 'text>,
    pub _open: symbols::OpenParen,
    pub attr_value: Term<'token, 'text>,
    pub _close: symbols::CloseParen,
    pub _dot: symbols::Dot,
}
derive_parse!(WildAttr,
              _hyphen,
              attr_name,
              _open,
              attr_value,
              _close,
              _dot);
derive_token_range!(WildAttr, _hyphen, _dot);

// TODO: Split to `FunSpec`, `RemoteFunSpec` and `CallbackSpec`
#[derive(Debug)]
pub struct FunSpec<'token, 'text: 'token> {
    pub _hyphen: symbols::Hyphen,
    pub spec_kind: atoms::Spec,
    pub module_name: Option<ModuleAtom<'token, 'text>>,
    pub fun_name: Atom<'token, 'text>,
    pub fun_types: Clauses<types::Function<'token, 'text>>,
    pub _dot: symbols::Dot,
}
derive_parse!(FunSpec,
              _hyphen,
              spec_kind,
              module_name,
              fun_name,
              fun_types,
              _dot);
derive_token_range!(FunSpec, _hyphen, _dot);

#[derive(Debug)]
pub struct FunDecl<'token, 'text: 'token> {
    pub clauses: Clauses<clauses::FunctionClause<'token, 'text>>,
    pub _dot: symbols::Dot,
}
derive_parse!(FunDecl, clauses, _dot);
derive_token_range!(FunDecl, clauses, _dot);

#[derive(Debug)]
pub struct TypeDecl<'token, 'text: 'token> {
    pub _hyphen: symbols::Hyphen,
    pub type_kind: atoms::Type,
    pub type_name: Atom<'token, 'text>,
    pub vars: Args<Variable<'token, 'text>>,
    pub _double_colon: symbols::DoubleColon,
    pub type_value: Type<'token, 'text>,
    pub _dot: symbols::Dot,
}
derive_parse!(TypeDecl,
              _hyphen,
              type_kind,
              type_name,
              vars,
              _double_colon,
              type_value,
              _dot);
derive_token_range!(TypeDecl, _hyphen, _dot);

#[derive(Debug)]
pub struct RecordDecl<'token, 'text: 'token> {
    pub _hyphen: symbols::Hyphen,
    pub _record: atoms::Record,
    pub record_name: Atom<'token, 'text>,
    pub _comma: symbols::Comma,
    pub record_fields: Tuple<RecordField<'token, 'text>>,
    pub _dot: symbols::Dot,
}
derive_parse!(RecordDecl,
              _hyphen,
              _record,
              record_name,
              _comma,
              record_fields,
              _dot);
derive_token_range!(RecordDecl, _hyphen, _dot);

#[derive(Debug)]
pub struct RecordField<'token, 'text: 'token> {
    pub field_name: Atom<'token, 'text>,
    pub default_value: Option<RecordFieldValue<'token, 'text>>,
    pub field_type: Option<RecordFieldType<'token, 'text>>,
}
derive_parse!(RecordField, field_name, default_value, field_type);
impl<'token, 'text: 'token> TokenRange for RecordField<'token, 'text> {
    fn token_start(&self) -> usize {
        self.field_name.token_start()
    }
    fn token_end(&self) -> usize {
        self.field_type
            .as_ref()
            .map(|t| t.token_end())
            .or_else(|| self.default_value.as_ref().map(|t| t.token_end()))
            .unwrap_or_else(|| self.field_name.token_end())
    }
}

// TODO: 共通化 (=> Bind)
#[derive(Debug)]
pub struct RecordFieldValue<'token, 'text: 'token> {
    pub _bind: symbols::Match,
    pub value: Expression<'token, 'text>,
}
derive_parse!(RecordFieldValue, _bind, value);
derive_token_range!(RecordFieldValue, _bind, value);

// TODO: 共通化 (=> TypeAnotated)
#[derive(Debug)]
pub struct RecordFieldType<'token, 'text: 'token> {
    pub _double_colon: symbols::DoubleColon,
    pub field_type: Type<'token, 'text>,
}
derive_parse!(RecordFieldType, _double_colon, field_type);
derive_token_range!(RecordFieldType, _double_colon, field_type);
