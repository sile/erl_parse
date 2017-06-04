use {Result, TokenReader, Parse, TokenRange};
use cst::Type;
use cst::keywords;
use cst::primitives::{Args, Atom, Seq2, Variable, Int};
use cst::symbols;

#[derive(Debug, Clone)]
pub struct Function<'token, 'text: 'token> {
    pub args: Args<Type<'token, 'text>>,
    pub allow: symbols::RightArrow,
    pub return_type: Type<'token, 'text>,
    // TODO: pub constraints: Constraints<'token, 'text>,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for Function<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        Ok(Function {
               args: track_try!(Parse::parse(reader)),
               allow: track_try!(Parse::parse(reader)),
               return_type: track_try!(Parse::parse(reader)),
           })
    }
}
impl<'token, 'text: 'token> TokenRange for Function<'token, 'text> {
    fn token_start(&self) -> usize {
        self.args.token_start()
    }
    fn token_end(&self) -> usize {
        self.return_type.token_end()
    }
}

// #[derive(Debug, Clone)]
// pub struct Constraints<'token, 'text: 'token> {
//     _a: &'token (),
//     _b: &'text (),
// }

#[derive(Debug, Clone)]
pub struct LocalType<'token, 'text: 'token> {
    pub name: Atom<'token, 'text>,
    pub args: Args<Type<'token, 'text>>,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for LocalType<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        Ok(LocalType {
               name: track_try!(reader.parse_next()),
               args: track_try!(Parse::parse(reader)),
           })
    }
}
impl<'token, 'text: 'token> TokenRange for LocalType<'token, 'text> {
    fn token_start(&self) -> usize {
        self.name.token_start()
    }
    fn token_end(&self) -> usize {
        self.args.token_end()
    }
}

#[derive(Debug, Clone)]
pub struct RemoteType<'token, 'text: 'token> {
    pub module_name: Atom<'token, 'text>,
    pub _colon: symbols::Colon,
    pub type_name: Atom<'token, 'text>,
    pub args: Args<Type<'token, 'text>>,
}
derive_parse!(RemoteType, module_name, _colon, type_name, args);
derive_token_range!(RemoteType, module_name, args);

#[derive(Debug, Clone)]
pub struct Union<'token, 'text: 'token> {
    pub head: Type<'token, 'text>,
    pub tail: Vec<UnionElem<'token, 'text>>,
}
derive_parse!(Union, head, tail);
impl<'token, 'text: 'token> TokenRange for Union<'token, 'text> {
    fn token_start(&self) -> usize {
        self.head.token_start()
    }
    fn token_end(&self) -> usize {
        self.tail
            .last()
            .map_or(self.head.token_end(), |t| t.token_end())
    }
}

#[derive(Debug, Clone)]
pub struct UnionElem<'token, 'text: 'token> {
    pub bar: symbols::VerticalBar,
    pub ty: Type<'token, 'text>,
}
derive_parse!(UnionElem, bar, ty);
derive_token_range!(UnionElem, bar, ty);

#[derive(Debug, Clone)]
pub struct List<'token, 'text: 'token> {
    pub _open: symbols::OpenSquare,
    pub element_type: Type<'token, 'text>,
    pub _close: symbols::CloseSquare,
}
derive_parse!(List, _open, element_type, _close);
derive_token_range!(List, _open, _close);

#[derive(Debug, Clone)]
pub struct Tuple<'token, 'text: 'token> {
    pub _open: symbols::OpenBrace,
    pub elements: Seq2<Type<'token, 'text>, symbols::Comma>,
    pub _close: symbols::CloseBrace,
}
derive_parse!(Tuple, _open, elements, _close);
derive_token_range!(Tuple, _open, _close);

#[derive(Debug, Clone)]
pub struct Annotated<'token, 'text: 'token> {
    pub variable: Variable<'token, 'text>,
    pub _double_colon: symbols::DoubleColon,
    pub ann_type: Type<'token, 'text>,
}
derive_parse!(Annotated, variable, _double_colon, ann_type);
derive_token_range!(Annotated, variable, ann_type);

#[derive(Debug, Clone)]
pub struct IntRange<'token, 'text: 'token> {
    pub low: Int<'token, 'text>,
    pub _double_dot: symbols::DoubleDot,
    pub high: Int<'token, 'text>,
}
derive_parse!(IntRange, low, _double_dot, high);
derive_token_range!(IntRange, low, high);

#[derive(Debug, Clone)]
pub struct Parenthesized<'token, 'text: 'token> {
    pub _open: symbols::OpenParen,
    pub ty: Type<'token, 'text>,
    pub _close: symbols::CloseParen,
}
derive_parse!(Parenthesized, _open, ty, _close);
derive_token_range!(Parenthesized, _open, _close);

#[derive(Debug, Clone)]
pub struct AnyArgFun<'token, 'text: 'token> {
    pub _fun: keywords::Fun,
    pub _open: symbols::OpenParen,

    // TODO: AnyArg
    pub _open_args: symbols::OpenParen,
    pub _any: symbols::TripleDot,
    pub _close_args: symbols::OpenParen,

    pub _arrow: symbols::RightArrow,
    pub return_type: Type<'token, 'text>,
    pub _close: symbols::CloseParen,
}
derive_parse!(AnyArgFun,
              _fun,
              _open,
              _open_args,
              _any,
              _close_args,
              _arrow,
              return_type,
              _close);
derive_token_range!(AnyArgFun, _fun, _close);

#[derive(Debug, Clone)]
pub struct Fun<'token, 'text: 'token> {
    pub _fun: keywords::Fun,
    pub _open: symbols::OpenParen,
    pub args: Args<Type<'token, 'text>>,
    pub _arrow: symbols::RightArrow,
    pub return_type: Type<'token, 'text>,
    // TODO: guard
    pub _close: symbols::CloseParen,
}
derive_parse!(Fun, _fun, _open, args, _arrow, return_type, _close);
derive_token_range!(Fun, _fun, _close);
