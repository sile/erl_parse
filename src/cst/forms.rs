use {Result, TokenReader, Parse, TokenRange};
use super::atoms;
use super::clauses;
use super::primitives::{Atom, List, Export, ModuleAtom, Clauses};
use super::symbols;
use super::types;

#[derive(Debug)]
pub struct Attribute<N, V> {
    pub hyphen: symbols::Hyphen,
    pub attr_name: N,
    pub open: symbols::OpenParen,
    pub attr_value: V,
    pub close: symbols::CloseParen,
    pub dot: symbols::Dot,
}
impl<'token, 'text: 'token, N, V> Parse<'token, 'text> for Attribute<N, V>
    where N: Parse<'token, 'text>,
          V: Parse<'token, 'text>
{
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        Ok(Attribute {
               hyphen: track_try!(Parse::parse(reader)),
               attr_name: track_try!(Parse::parse(reader)),
               open: track_try!(Parse::parse(reader)),
               attr_value: track_try!(Parse::parse(reader)),
               close: track_try!(Parse::parse(reader)),
               dot: track_try!(Parse::parse(reader)),
           })
    }
}
impl<N, V> TokenRange for Attribute<N, V> {
    fn token_start(&self) -> usize {
        self.hyphen.token_start()
    }
    fn token_end(&self) -> usize {
        self.dot.token_end()
    }
}

#[derive(Debug)]
pub struct ModuleAttr<'token, 'text: 'token>(Attribute<atoms::Module, Atom<'token, 'text>>);
impl<'token, 'text: 'token> ModuleAttr<'token, 'text> {
    pub fn module_name(&self) -> &str {
        self.0.attr_value.value()
    }
    pub fn as_attribute(&self) -> &Attribute<atoms::Module, Atom<'token, 'text>> {
        &self.0
    }
}
impl<'token, 'text: 'token> Parse<'token, 'text> for ModuleAttr<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let inner = track_try!(Parse::parse(reader));
        Ok(ModuleAttr(inner))
    }
}
impl<'token, 'text: 'token> TokenRange for ModuleAttr<'token, 'text> {
    fn token_start(&self) -> usize {
        self.0.token_start()
    }
    fn token_end(&self) -> usize {
        self.0.token_end()
    }
}

#[derive(Debug)]
pub struct ExportAttr<'token, 'text: 'token> {
    inner: Attribute<atoms::Export, List<Export<'token, 'text>>>,
}
impl<'token, 'text: 'token> ExportAttr<'token, 'text> {
    pub fn exports(&self) -> &List<Export<'token, 'text>> {
        &self.inner.attr_value
    }
    pub fn as_attribute(&self) -> &Attribute<atoms::Export, List<Export<'token, 'text>>> {
        &self.inner
    }
}
impl<'token, 'text: 'token> Parse<'token, 'text> for ExportAttr<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        let inner = track_try!(Parse::parse(reader));
        Ok(ExportAttr { inner })
    }
}
impl<'token, 'text: 'token> TokenRange for ExportAttr<'token, 'text> {
    fn token_start(&self) -> usize {
        self.inner.token_start()
    }
    fn token_end(&self) -> usize {
        self.inner.token_end()
    }
}

#[derive(Debug)]
pub struct FunctionSpec<'token, 'text: 'token> {
    pub hyphen: symbols::Hyphen,
    pub spec: atoms::Spec,
    pub module_name: Option<ModuleAtom<'token, 'text>>,
    pub function_name: Atom<'token, 'text>,
    pub function_types: Clauses<types::Function<'token, 'text>>,
    pub dot: symbols::Dot,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for FunctionSpec<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        Ok(FunctionSpec {
               hyphen: track_try!(reader.parse_next()),
               spec: track_try!(reader.parse_next()),
               module_name: reader.try_parse_next(),
               function_name: track_try!(reader.parse_next()),
               function_types: track_try!(reader.parse_next()),
               dot: track_try!(reader.parse_next()),
           })
    }
}
impl<'token, 'text: 'token> TokenRange for FunctionSpec<'token, 'text> {
    fn token_start(&self) -> usize {
        self.hyphen.token_start()
    }
    fn token_end(&self) -> usize {
        self.dot.token_end()
    }
}

#[derive(Debug)]
pub struct FunctionDecl<'token, 'text: 'token> {
    pub clauses: Clauses<clauses::FunctionClause<'token, 'text>>,
    pub dot: symbols::Dot,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for FunctionDecl<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        Ok(FunctionDecl {
               clauses: track_try!(reader.parse_next()),
               dot: track_try!(reader.parse_next()),
           })
    }
}
impl<'token, 'text: 'token> TokenRange for FunctionDecl<'token, 'text> {
    fn token_start(&self) -> usize {
        self.clauses.token_start()
    }
    fn token_end(&self) -> usize {
        self.dot.token_end()
    }
}
