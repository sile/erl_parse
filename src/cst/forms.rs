use {Result, TokenReader, Parse, TokenRange};
use super::atoms;
use super::clauses;
use super::primitives::{Atom, List, Export, ModuleAtom, Clauses};
use super::symbols;
use super::types;

#[derive(Debug)]
pub struct ModuleAttr<'token, 'text: 'token> {
    pub hyphen: symbols::Hyphen,
    pub attr_name: atoms::Module,
    pub open: symbols::OpenParen,
    pub module_name: Atom<'token, 'text>,
    pub close: symbols::CloseParen,
    pub dot: symbols::Dot,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for ModuleAttr<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        Ok(ModuleAttr {
               hyphen: try_parse!(reader),
               attr_name: try_parse!(reader),
               open: try_parse!(reader),
               module_name: try_parse!(reader),
               close: try_parse!(reader),
               dot: try_parse!(reader),
           })
    }
}
impl<'token, 'text: 'token> TokenRange for ModuleAttr<'token, 'text> {
    fn token_start(&self) -> usize {
        self.hyphen.token_start()
    }
    fn token_end(&self) -> usize {
        self.dot.token_end()
    }
}

// #[derive(Debug)]
// pub struct BehaviourAttr<'token, 'text: 'token> {
//     pub hyphen: symbols::Hyphen,
//     pub attr_name: atoms::Module,
//     pub open: symbols::OpenParen,
//     pub behaviour_name: Atom<'token, 'text>,
//     pub close: symbols::CloseParen,
//     pub dot: symbols::Dot,
// }
// impl<'token, 'text: 'token> Parse<'token, 'text> for BehaviourAttr<'token, 'text> {
//     fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
//         Ok(BehaviourAttr {
//                hyphen: try_parse!(reader),
//                attr_name: try_parse!(reader),
//                open: try_parse!(reader),
//                behaviour_name: try_parse!(reader),
//                close: try_parse!(reader),
//                dot: try_parse!(reader),
//            })
//     }
// }
// impl<'token, 'text: 'token> TokenRange for BehaviourAttr<'token, 'text> {
//     fn token_start(&self) -> usize {
//         self.hyphen.token_start()
//     }
//     fn token_end(&self) -> usize {
//         self.dot.token_end()
//     }
// }

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
