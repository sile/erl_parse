use std::ops::Range;

use {Result, ErrorKind, Parse, TokenRange, TokenReader};
use super::{Term, Type, Expr};
use super::atoms;
use super::clauses;
use super::primitives::{Atom, List, Export, Import, ModuleAtom, Clauses, ExportType, Str, Integer,
                        Args, Variable, Tuple};
use super::symbols;
use super::types;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct FunDecl<'token, 'text: 'token> {
    pub clauses: Clauses<clauses::FunctionClause<'token, 'text>>,
    pub _dot: symbols::Dot,
}
derive_parse!(FunDecl, clauses, _dot);
derive_token_range!(FunDecl, clauses, _dot);

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct RecordFieldValue<'token, 'text: 'token> {
    pub _bind: symbols::Match,
    pub value: Expr<'token, 'text>,
}
derive_parse!(RecordFieldValue, _bind, value);
derive_token_range!(RecordFieldValue, _bind, value);

// TODO: 共通化 (=> TypeAnotated)
#[derive(Debug, Clone)]
pub struct RecordFieldType<'token, 'text: 'token> {
    pub _double_colon: symbols::DoubleColon,
    pub field_type: Type<'token, 'text>,
}
derive_parse!(RecordFieldType, _double_colon, field_type);
derive_token_range!(RecordFieldType, _double_colon, field_type);

#[derive(Debug, Clone)]
pub struct MacroDecl<'token, 'text: 'token> {
    pub _hyphen: symbols::Hyphen,
    pub _define: atoms::Define,
    pub _open: symbols::OpenParen,
    pub macro_name: MacroName<'token, 'text>,
    pub args: Option<Args<Variable<'token, 'text>>>,
    pub _comma: symbols::Comma,
    pub replacement: MacroReplacement,
    pub _close: symbols::CloseParen,
    pub _dot: symbols::Dot,
}
derive_parse!(MacroDecl,
              _hyphen,
              _define,
              _open,
              macro_name,
              args,
              _comma,
              replacement,
              _close,
              _dot);
derive_token_range!(MacroDecl, _hyphen, _dot);

#[derive(Debug, Clone)]
pub enum MacroName<'token, 'text: 'token> {
    Atom(Atom<'token, 'text>),
    Var(Variable<'token, 'text>),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for MacroName<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        if let Some(t) = reader.try_parse_next() {
            Ok(MacroName::Atom(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(MacroName::Var(t))
        } else {
            track_panic!(ErrorKind::InvalidInput,
                         "Unrecognized macro name: next={:?}",
                         reader.read());
        }
    }
}
impl<'token, 'text: 'token> TokenRange for MacroName<'token, 'text> {
    fn token_range(&self) -> Range<usize> {
        match *self {
            MacroName::Atom(ref t) => t.token_range(),
            MacroName::Var(ref t) => t.token_range(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MacroReplacement {
    token_start: usize,
    token_end: usize,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for MacroReplacement {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let token_start = reader.position();
        while let Err(_) = reader.peek::<(symbols::CloseParen, symbols::Dot)>() {
            track_try!(reader.read());
        }
        let token_end = reader.position();
        Ok(MacroReplacement {
               token_start,
               token_end,
           })
    }
}
impl TokenRange for MacroReplacement {
    fn token_range(&self) -> Range<usize> {
        Range {
            start: self.token_start,
            end: self.token_end,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MacroDirective<'token, 'text: 'token> {
    Undef(MacroUndef<'token, 'text>),
    Ifdef(MacroIfdef<'token, 'text>),
    Ifndef(MacroIfndef<'token, 'text>),
    Else(MacroElse),
    Endif(MacroEndif),
}
impl<'token, 'text: 'token> Parse<'token, 'text> for MacroDirective<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        if let Some(t) = reader.try_parse_next() {
            Ok(MacroDirective::Undef(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(MacroDirective::Ifdef(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(MacroDirective::Ifndef(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(MacroDirective::Else(t))
        } else if let Some(t) = reader.try_parse_next() {
            Ok(MacroDirective::Endif(t))
        } else {
            track_panic!(ErrorKind::InvalidInput,
                         "Unrecognized macro name: next={:?}",
                         reader.read());
        }
    }
}
impl<'token, 'text: 'token> TokenRange for MacroDirective<'token, 'text> {
    fn token_range(&self) -> Range<usize> {
        match *self {
            MacroDirective::Undef(ref t) => t.token_range(),
            MacroDirective::Ifdef(ref t) => t.token_range(),
            MacroDirective::Ifndef(ref t) => t.token_range(),
            MacroDirective::Else(ref t) => t.token_range(),
            MacroDirective::Endif(ref t) => t.token_range(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MacroUndef<'token, 'text: 'token> {
    pub _hyphen: symbols::Hyphen,
    pub _undef: atoms::Undef,
    pub _open: symbols::OpenParen,
    pub macro_name: MacroName<'token, 'text>,
    pub _close: symbols::CloseParen,
    pub _dot: symbols::Dot,
}
derive_parse!(MacroUndef, _hyphen, _undef, _open, macro_name, _close, _dot);
derive_token_range!(MacroUndef, _hyphen, _dot);

#[derive(Debug, Clone)]
pub struct MacroIfdef<'token, 'text: 'token> {
    pub _hyphen: symbols::Hyphen,
    pub _ifdef: atoms::Ifdef,
    pub _open: symbols::OpenParen,
    pub macro_name: MacroName<'token, 'text>,
    pub _close: symbols::CloseParen,
    pub _dot: symbols::Dot,
}
derive_parse!(MacroIfdef, _hyphen, _ifdef, _open, macro_name, _close, _dot);
derive_token_range!(MacroIfdef, _hyphen, _dot);

#[derive(Debug, Clone)]
pub struct MacroIfndef<'token, 'text: 'token> {
    pub _hyphen: symbols::Hyphen,
    pub _ifndef: atoms::Ifndef,
    pub _open: symbols::OpenParen,
    pub macro_name: MacroName<'token, 'text>,
    pub _close: symbols::CloseParen,
    pub _dot: symbols::Dot,
}
derive_parse!(MacroIfndef,
              _hyphen,
              _ifndef,
              _open,
              macro_name,
              _close,
              _dot);
derive_token_range!(MacroIfndef, _hyphen, _dot);

#[derive(Debug, Clone)]
pub struct MacroElse {
    pub _hyphen: symbols::Hyphen,
    pub _else: atoms::Else,
    pub _dot: symbols::Dot,
}
derive_parse0!(MacroElse, _hyphen, _else, _dot);
derive_token_range0!(MacroElse, _hyphen, _dot);

#[derive(Debug, Clone)]
pub struct MacroEndif {
    pub _hyphen: symbols::Hyphen,
    pub _endif: atoms::Endif,
    pub _dot: symbols::Dot,
}
derive_parse0!(MacroEndif, _hyphen, _endif, _dot);
derive_token_range0!(MacroEndif, _hyphen, _dot);
