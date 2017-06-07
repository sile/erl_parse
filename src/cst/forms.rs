use cst::{Term, Type, Expr};
use cst::commons;
use cst::clauses;
use cst::literals;
use cst::types;

#[derive(Debug, Clone)]
pub struct ModuleAttr {
    pub _hyphen: literals::S_HYPHEN,
    pub _module: literals::A_MODULE,
    pub _open: literals::S_OPEN_PAREN,
    pub module_name: literals::Atom,
    pub _close: literals::S_CLOSE_PAREN,
    pub _dot: literals::S_DOT,
}
derive_parse!(ModuleAttr,
              _hyphen,
              _module,
              _open,
              module_name,
              _close,
              _dot);
derive_token_range!(ModuleAttr, _hyphen, _dot);

#[derive(Debug, Clone)]
pub struct ExportAttr {
    pub _hyphen: literals::S_HYPHEN,
    pub _export: literals::A_EXPORT,
    pub _open: literals::S_OPEN_PAREN,
    pub exports: commons::List<Export>,
    pub _close: literals::S_CLOSE_PAREN,
    pub _dot: literals::S_DOT,
}
derive_parse!(ExportAttr, _hyphen, _export, _open, exports, _close, _dot);
derive_token_range!(ExportAttr, _hyphen, _dot);

#[derive(Debug, Clone)]
pub struct ExportTypeAttr {
    pub _hyphen: literals::S_HYPHEN,
    pub _export_type: literals::A_EXPORT_TYPE,
    pub _open: literals::S_OPEN_PAREN,
    pub exports: commons::List<Export>,
    pub _close: literals::S_CLOSE_PAREN,
    pub _dot: literals::S_DOT,
}
derive_parse!(ExportTypeAttr,
              _hyphen,
              _export_type,
              _open,
              exports,
              _close,
              _dot);
derive_token_range!(ExportTypeAttr, _hyphen, _dot);

#[derive(Debug, Clone)]
pub struct Export {
    pub name: literals::Atom,
    pub _slash: literals::S_SLASH,
    pub arity: literals::Int,
}
derive_parse!(Export, name, _slash, arity);
derive_token_range!(Export, name, arity);

#[derive(Debug, Clone)]
pub struct ImportAttr {
    pub _hyphen: literals::S_HYPHEN,
    pub _import: literals::A_IMPORT,
    pub _open: literals::S_OPEN_PAREN,
    pub module_name: literals::Atom,
    pub _comma: literals::S_COMMA,
    pub imports: commons::List<Import>,
    pub _close: literals::S_CLOSE_PAREN,
    pub _dot: literals::S_DOT,
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

#[derive(Debug, Clone)]
pub struct Import {
    pub name: literals::Atom,
    pub _slash: literals::S_SLASH,
    pub arity: literals::Int,
}
derive_parse!(Import, name, _slash, arity);
derive_token_range!(Import, name, arity);

#[derive(Debug, Clone)]
pub struct FileAttr {
    pub _hyphen: literals::S_HYPHEN,
    pub _file: literals::A_FILE,
    pub _open: literals::S_OPEN_PAREN,
    pub file_name: literals::Str,
    pub _comma: literals::S_COMMA,
    pub line_num: literals::Int,
    pub _close: literals::S_CLOSE_PAREN,
    pub _dot: literals::S_DOT,
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

#[derive(Debug, Clone)]
pub struct WildAttr {
    pub _hyphen: literals::S_HYPHEN,
    pub attr_name: literals::Atom,
    pub _open: literals::S_OPEN_PAREN,
    pub attr_value: Term,
    pub _close: literals::S_CLOSE_PAREN,
    pub _dot: literals::S_DOT,
}
derive_parse!(WildAttr,
              _hyphen,
              attr_name,
              _open,
              attr_value,
              _close,
              _dot);
derive_token_range!(WildAttr, _hyphen, _dot);

#[derive(Debug, Clone)]
pub struct FunSpec {
    pub _hyphen: literals::S_HYPHEN,
    pub _spec: literals::A_SPEC,
    pub fun_name: literals::Atom,
    pub clauses: commons::NonEmptySeq<FunClause, literals::S_SEMICOLON>,
    pub _dot: literals::S_DOT,
}
derive_parse!(FunSpec, _hyphen, _spec, fun_name, clauses, _dot);
derive_token_range!(FunSpec, _hyphen, _dot);

#[derive(Debug, Clone)]
pub struct RemoteFunSpec {
    pub _hyphen: literals::S_HYPHEN,
    pub _spec: literals::A_SPEC,
    pub module_name: literals::Atom,
    pub _colon: literals::S_COLON,
    pub fun_name: literals::Atom,
    pub clauses: commons::NonEmptySeq<FunClause, literals::S_SEMICOLON>,
    pub _dot: literals::S_DOT,
}
derive_parse!(RemoteFunSpec,
              _hyphen,
              _spec,
              module_name,
              _colon,
              fun_name,
              clauses,
              _dot);
derive_token_range!(RemoteFunSpec, _hyphen, _dot);

#[derive(Debug, Clone)]
pub struct CallbackSpec {
    pub _hyphen: literals::S_HYPHEN,
    pub _callback: literals::A_CALLBACK,
    pub callback_name: literals::Atom,
    pub clauses: commons::NonEmptySeq<FunClause, literals::S_SEMICOLON>,
    pub _dot: literals::S_DOT,
}
derive_parse!(CallbackSpec,
              _hyphen,
              _callback,
              callback_name,
              clauses,
              _dot);
derive_token_range!(CallbackSpec, _hyphen, _dot);

#[derive(Debug, Clone)]
pub struct FunClause {
    pub args: commons::Args<Type>,
    pub _arrow: literals::S_RIGHT_ARROW,
    pub return_type: Type,
    pub constraints: Option<types::FunConstraints>,
    _position: commons::Void,
}
derive_parse!(FunClause, args, _arrow, return_type, constraints, _position);
derive_token_range!(FunClause, args, _position);

#[derive(Debug, Clone)]
pub struct FunDecl {
    pub fun_name: literals::Atom,
    pub clauses: commons::NonEmptySeq<clauses::FunClause<commons::Void>, literals::S_SEMICOLON>,
    pub _dot: literals::S_DOT,
}
derive_parse!(FunDecl, fun_name, clauses, _dot);
derive_token_range!(FunDecl, fun_name, _dot);

#[derive(Debug, Clone)]
pub struct RecordDecl {
    pub _hyphen: literals::S_HYPHEN,
    pub _record: literals::A_RECORD,
    pub _open: literals::S_OPEN_PAREN,
    pub record_name: literals::Atom,
    pub _comma: literals::S_COMMA,
    pub _fields_start: literals::S_OPEN_BRACE,
    pub fields: commons::Seq<RecordField, literals::S_COMMA>,
    pub _fields_end: literals::S_CLOSE_BRACE,
    pub _close: literals::S_CLOSE_PAREN,
    pub _dot: literals::S_DOT,
}
derive_parse!(RecordDecl,
              _hyphen,
              _record,
              _open,
              record_name,
              _comma,
              _fields_start,
              fields,
              _fields_end,
              _close,
              _dot);
derive_token_range!(RecordDecl, _hyphen, _dot);

#[derive(Debug, Clone)]
pub struct RecordField {
    pub field_name: literals::Atom,
    pub field_default: Option<RecordFieldDefault>,
    pub field_type: Option<RecordFieldType>,
    _position: commons::Void,
}
derive_parse!(RecordField,
              field_name,
              field_default,
              field_type,
              _position);
derive_token_range!(RecordField, field_name, _position);

#[derive(Debug, Clone)]
pub struct RecordFieldDefault {
    pub _bind: literals::S_MATCH,
    pub value: Expr,
}
derive_parse!(RecordFieldDefault, _bind, value);
derive_token_range!(RecordFieldDefault, _bind, value);

#[derive(Debug, Clone)]
pub struct RecordFieldType {
    pub _double_colon: literals::S_DOUBLE_COLON,
    pub field_type: Type,
}
derive_parse!(RecordFieldType, _double_colon, field_type);
derive_token_range!(RecordFieldType, _double_colon, field_type);

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub _hyphen: literals::S_HYPHEN,
    pub _type: literals::A_TYPE,
    pub name: literals::Atom,
    pub vars: commons::Args<commons::Var>,
    pub _double_colon: literals::S_DOUBLE_COLON,
    pub ty: Type,
    pub _dot: literals::S_DOT,
}
derive_parse!(TypeDecl,
              _hyphen,
              _type,
              name,
              vars,
              _double_colon,
              ty,
              _dot);
derive_token_range!(TypeDecl, _hyphen, _dot);

#[derive(Debug, Clone)]
pub struct OpaqueDecl {
    pub _hyphen: literals::S_HYPHEN,
    pub _opaque: literals::A_OPAQUE,
    pub name: literals::Atom,
    pub vars: commons::Args<commons::Var>,
    pub _double_colon: literals::S_DOUBLE_COLON,
    pub ty: Type,
    pub _dot: literals::S_DOT,
}
derive_parse!(OpaqueDecl,
              _hyphen,
              _opaque,
              name,
              vars,
              _double_colon,
              ty,
              _dot);
derive_token_range!(OpaqueDecl, _hyphen, _dot);
