use erl_tokenize::{LexicalToken, Position, PositionRange};
use erl_tokenize::tokens::{SymbolToken, VariableToken, IntegerToken, AtomToken, StringToken};
use erl_tokenize::values::Symbol;

use {Result, Parser, Preprocessor, Parse};
use cst::{Type, Expr};
use cst::building_blocks::{Args, Sequence};
use cst::clauses::{Clauses, SpecClause, NamedFunClause};

#[derive(Debug, Clone)]
pub struct ModuleAttr {
    pub _hyphen: SymbolToken,
    pub _module: AtomToken,
    pub _open: SymbolToken,
    pub module_name: AtomToken,
    pub _close: SymbolToken,
    pub _dot: SymbolToken,
}
impl Parse for ModuleAttr {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(ModuleAttr {
            _hyphen: track!(parser.expect(&Symbol::Hyphen))?,
            _module: track!(parser.expect("module"))?,
            _open: track!(parser.expect(&Symbol::OpenParen))?,
            module_name: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseParen))?,
            _dot: track!(parser.expect(&Symbol::Dot))?,
        })
    }
}
impl PositionRange for ModuleAttr {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct List<T> {
    pub _open: SymbolToken,
    pub elements: Option<Sequence<T>>,
    pub _close: SymbolToken,
}
impl<T: Parse> Parse for List<T> {
    fn parse<U>(parser: &mut Parser<U>) -> Result<Self>
    where
        U: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(List {
            _open: track!(parser.expect(&Symbol::OpenSquare))?,
            elements: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseSquare))?,
        })
    }
}
impl<T> PositionRange for List<T> {
    fn start_position(&self) -> Position {
        self._open.start_position()
    }
    fn end_position(&self) -> Position {
        self._close.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct ExportAttr {
    pub _hyphen: SymbolToken,
    pub _export: AtomToken,
    pub _open: SymbolToken,
    pub exports: List<Export>,
    pub _close: SymbolToken,
    pub _dot: SymbolToken,
}
impl Parse for ExportAttr {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(ExportAttr {
            _hyphen: track!(parser.expect(&Symbol::Hyphen))?,
            _export: track!(parser.expect("export"))?,
            _open: track!(parser.expect(&Symbol::OpenParen))?,
            exports: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseParen))?,
            _dot: track!(parser.expect(&Symbol::Dot))?,
        })
    }
}
impl PositionRange for ExportAttr {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Export {
    pub name: AtomToken,
    pub _slash: SymbolToken,
    pub arity: IntegerToken,
}
impl Parse for Export {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Export {
            name: track!(parser.parse())?,
            _slash: track!(parser.expect(&Symbol::Slash))?,
            arity: track!(parser.parse())?,
        })
    }
}
impl PositionRange for Export {
    fn start_position(&self) -> Position {
        self.name.start_position()
    }
    fn end_position(&self) -> Position {
        self.arity.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct Import {
    pub name: AtomToken,
    pub _slash: SymbolToken,
    pub arity: IntegerToken,
}
impl Parse for Import {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(Import {
            name: track!(parser.parse())?,
            _slash: track!(parser.expect(&Symbol::Slash))?,
            arity: track!(parser.parse())?,
        })
    }
}
impl PositionRange for Import {
    fn start_position(&self) -> Position {
        self.name.start_position()
    }
    fn end_position(&self) -> Position {
        self.arity.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct ExportTypeAttr {
    pub _hyphen: SymbolToken,
    pub _export_type: AtomToken,
    pub _open: SymbolToken,
    pub exports: List<Export>,
    pub _close: SymbolToken,
    pub _dot: SymbolToken,
}
impl Parse for ExportTypeAttr {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(ExportTypeAttr {
            _hyphen: track!(parser.expect(&Symbol::Hyphen))?,
            _export_type: track!(parser.expect("export_type"))?,
            _open: track!(parser.expect(&Symbol::OpenParen))?,
            exports: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseParen))?,
            _dot: track!(parser.expect(&Symbol::Dot))?,
        })
    }
}
impl PositionRange for ExportTypeAttr {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct ImportAttr {
    pub _hyphen: SymbolToken,
    pub _import: AtomToken,
    pub _open: SymbolToken,
    pub module_name: AtomToken,
    pub _comma: SymbolToken,
    pub imports: List<Import>,
    pub _close: SymbolToken,
    pub _dot: SymbolToken,
}
impl Parse for ImportAttr {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(ImportAttr {
            _hyphen: track!(parser.expect(&Symbol::Hyphen))?,
            _import: track!(parser.expect("import"))?,
            _open: track!(parser.expect(&Symbol::OpenParen))?,
            module_name: track!(parser.parse())?,
            _comma: track!(parser.expect(&Symbol::Comma))?,
            imports: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseParen))?,
            _dot: track!(parser.expect(&Symbol::Dot))?,
        })
    }
}
impl PositionRange for ImportAttr {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct FileAttr {
    pub _hyphen: SymbolToken,
    pub _file: AtomToken,
    pub _open: SymbolToken,
    pub file_name: StringToken,
    pub _comma: SymbolToken,
    pub line_num: IntegerToken,
    pub _close: SymbolToken,
    pub _dot: SymbolToken,
}
impl Parse for FileAttr {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(FileAttr {
            _hyphen: track!(parser.expect(&Symbol::Hyphen))?,
            _file: track!(parser.expect("file"))?,
            _open: track!(parser.expect(&Symbol::OpenParen))?,
            file_name: track!(parser.parse())?,
            _comma: track!(parser.expect(&Symbol::Comma))?,
            line_num: track!(parser.parse())?,
            _close: track!(parser.expect(&Symbol::CloseParen))?,
            _dot: track!(parser.expect(&Symbol::Dot))?,
        })
    }
}
impl PositionRange for FileAttr {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct WildAttr {
    pub _hyphen: SymbolToken,
    pub attr_name: AtomToken,
    pub _open: SymbolToken,
    pub attr_value: Vec<LexicalToken>,
    pub _close: SymbolToken,
    pub _dot: SymbolToken,
}
impl Parse for WildAttr {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        let _hyphen = track!(parser.expect(&Symbol::Hyphen))?;
        let attr_name = track!(parser.parse())?;
        let _open = track!(parser.expect(&Symbol::OpenParen))?;

        let count = parser.peek(|parser| {
            for i in 0.. {
                let v = track!(parser.read_token())?.as_symbol_token().map(
                    |t| t.value(),
                );
                if v == Some(Symbol::Dot) {
                    use std::cmp;
                    return Ok(cmp::max(i, 1) - 1);
                }
            }
            unreachable!()
        });
        let attr_value = (0..track!(count)?)
            .map(|_| parser.read_token().expect("Never fails"))
            .collect();
        Ok(WildAttr {
            _hyphen,
            attr_name,
            _open,
            attr_value,
            _close: track!(parser.expect(&Symbol::CloseParen))?,
            _dot: track!(parser.expect(&Symbol::Dot))?,
        })
    }
}
impl PositionRange for WildAttr {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct FunSpec {
    pub _hyphen: SymbolToken,
    pub _spec: AtomToken,
    pub module: Option<ModulePrefix>,
    pub fun_name: AtomToken,
    pub clauses: Clauses<SpecClause>,
    pub _dot: SymbolToken,
}
impl Parse for FunSpec {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(FunSpec {
            _hyphen: track!(parser.expect(&Symbol::Hyphen))?,
            _spec: track!(parser.expect("spec"))?,
            module: track!(parser.parse())?,
            fun_name: track!(parser.parse())?,
            clauses: track!(parser.parse())?,
            _dot: track!(parser.expect(&Symbol::Dot))?,
        })
    }
}
impl PositionRange for FunSpec {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct CallbackSpec {
    pub _hyphen: SymbolToken,
    pub _spec: AtomToken,
    pub callback_name: AtomToken,
    pub clauses: Clauses<SpecClause>,
    pub _dot: SymbolToken,
}
impl Parse for CallbackSpec {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(CallbackSpec {
            _hyphen: track!(parser.expect(&Symbol::Hyphen))?,
            _spec: track!(parser.expect("callback"))?,
            callback_name: track!(parser.parse())?,
            clauses: track!(parser.parse())?,
            _dot: track!(parser.expect(&Symbol::Dot))?,
        })
    }
}
impl PositionRange for CallbackSpec {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct ModulePrefix {
    pub name: AtomToken,
    pub _colon: SymbolToken,
}
impl Parse for ModulePrefix {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(ModulePrefix {
            name: track!(parser.parse())?,
            _colon: track!(parser.expect(&Symbol::Colon))?,
        })
    }
}
impl PositionRange for ModulePrefix {
    fn start_position(&self) -> Position {
        self.name.start_position()
    }
    fn end_position(&self) -> Position {
        self._colon.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct FunDecl {
    pub clauses: Clauses<NamedFunClause>,
    pub _dot: SymbolToken,
}
impl Parse for FunDecl {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(FunDecl {
            clauses: track!(parser.parse())?,
            _dot: track!(parser.expect(&Symbol::Dot))?,
        })
    }
}
impl PositionRange for FunDecl {
    fn start_position(&self) -> Position {
        self.clauses.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct RecordDecl {
    pub _hyphen: SymbolToken,
    pub _record: AtomToken,
    pub _open: SymbolToken,
    pub record_name: AtomToken,
    pub _comma: SymbolToken,
    pub _fields_start: SymbolToken,
    pub fields: Option<Sequence<RecordField>>,
    pub _fields_end: SymbolToken,
    pub _close: SymbolToken,
    pub _dot: SymbolToken,
}
impl Parse for RecordDecl {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(RecordDecl {
            _hyphen: track!(parser.expect(&Symbol::Hyphen))?,
            _record: track!(parser.expect("record"))?,
            _open: track!(parser.expect(&Symbol::OpenParen))?,
            record_name: track!(parser.parse())?,
            _comma: track!(parser.expect(&Symbol::Comma))?,
            _fields_start: track!(parser.expect(&Symbol::OpenBrace))?,
            fields: track!(parser.parse())?,
            _fields_end: track!(parser.expect(&Symbol::CloseBrace))?,
            _close: track!(parser.expect(&Symbol::CloseParen))?,
            _dot: track!(parser.expect(&Symbol::Dot))?,
        })
    }
}
impl PositionRange for RecordDecl {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct RecordField {
    pub field_name: AtomToken,
    pub field_default: Option<RecordFieldDefault>,
    pub field_type: Option<RecordFieldType>,
}
impl Parse for RecordField {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(RecordField {
            field_name: track!(parser.parse())?,
            field_default: track!(parser.parse())?,
            field_type: track!(parser.parse())?,
        })
    }
}
impl PositionRange for RecordField {
    fn start_position(&self) -> Position {
        self.field_name.start_position()
    }
    fn end_position(&self) -> Position {
        self.field_type
            .as_ref()
            .map(|t| t.end_position())
            .or_else(|| self.field_default.as_ref().map(|t| t.end_position()))
            .unwrap_or_else(|| self.field_name.end_position())
    }
}

#[derive(Debug, Clone)]
pub struct RecordFieldDefault {
    pub _match: SymbolToken,
    pub value: Expr,
}
impl Parse for RecordFieldDefault {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(RecordFieldDefault {
            _match: track!(parser.expect(&Symbol::Match))?,
            value: track!(parser.parse())?,
        })
    }
}
impl PositionRange for RecordFieldDefault {
    fn start_position(&self) -> Position {
        self._match.start_position()
    }
    fn end_position(&self) -> Position {
        self.value.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct RecordFieldType {
    pub _double_colon: SymbolToken,
    pub field_type: Type,
}
impl Parse for RecordFieldType {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(RecordFieldType {
            _double_colon: track!(parser.expect(&Symbol::DoubleColon))?,
            field_type: track!(parser.parse())?,
        })
    }
}
impl PositionRange for RecordFieldType {
    fn start_position(&self) -> Position {
        self._double_colon.start_position()
    }
    fn end_position(&self) -> Position {
        self.field_type.end_position()
    }
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub _hyphen: SymbolToken,
    pub type_kind: AtomToken,
    pub type_name: AtomToken,
    pub variables: Args<VariableToken>,
    pub _double_colon: SymbolToken,
    pub ty: Type,
    pub _dot: SymbolToken,
}
impl Parse for TypeDecl {
    fn parse<T>(parser: &mut Parser<T>) -> Result<Self>
    where
        T: Iterator<Item = Result<LexicalToken>> + Preprocessor,
    {
        Ok(TypeDecl {
            _hyphen: track!(parser.expect(&Symbol::Hyphen))?,
            type_kind: track!(parser.expect_any(&["type", "opaque"]))?,
            type_name: track!(parser.parse())?,
            variables: track!(parser.parse())?,
            _double_colon: track!(parser.expect(&Symbol::DoubleColon))?,
            ty: track!(parser.parse())?,
            _dot: track!(parser.expect(&Symbol::Dot))?,
        })
    }
}
impl PositionRange for TypeDecl {
    fn start_position(&self) -> Position {
        self._hyphen.start_position()
    }
    fn end_position(&self) -> Position {
        self._dot.end_position()
    }
}
