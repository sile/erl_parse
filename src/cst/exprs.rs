use erl_tokenize::values::Symbol;

use {Result, TokenReader, Parse, TokenRange, ErrorKind};
use cst::Expression;
use cst::primitives::{Args, Atom};

#[derive(Debug)]
pub struct LocalCall<'token, 'text: 'token> {
    pub function_name: Atom<'token, 'text>,
    pub args: Args<Expression<'token, 'text>>,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for LocalCall<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        Ok(LocalCall {
               function_name: track_try!(reader.parse_next()),
               args: track_try!(reader.parse_next()),
           })
    }
}
impl<'token, 'text: 'token> TokenRange for LocalCall<'token, 'text> {
    fn token_start(&self) -> usize {
        self.function_name.token_start()
    }
    fn token_end(&self) -> usize {
        self.args.token_end()
    }
}

#[derive(Debug, Copy,Clone)]
pub enum BinaryOp {
    Add { position: usize },
    Sub { position: usize },
}
impl<'token, 'text: 'token> Parse<'token, 'text> for BinaryOp {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        // reader.skip_hidden_tokens();
        let position = reader.position();
        let symbol = track_try!(reader.read_symbol());
        match symbol.value() {
            Symbol::Plus => Ok(BinaryOp::Add { position }),
            Symbol::Hyphen => Ok(BinaryOp::Sub { position }),
            _ => {
                track_panic!(ErrorKind::InvalidInput,
                             "Not a binary operator: {:?}",
                             symbol)
            }
        }
    }
}
impl<'token, 'text: 'token> TokenRange for BinaryOp {
    fn token_start(&self) -> usize {
        match *self {
            BinaryOp::Add { position } => position,
            BinaryOp::Sub { position } => position,
        }
    }
    fn token_end(&self) -> usize {
        match *self {
            BinaryOp::Add { position } => position + 1,
            BinaryOp::Sub { position } => position + 1,
        }
    }
}

#[derive(Debug)]
pub struct BinaryOpCall<'token, 'text: 'token> {
    pub left: Expression<'token, 'text>,
    pub op: BinaryOp,
    pub right: Expression<'token, 'text>,
}
impl<'token, 'text: 'token> Parse<'token, 'text> for BinaryOpCall<'token, 'text> {
    fn parse(reader: &mut TokenReader<'token, 'text>) -> Result<Self> {
        Ok(BinaryOpCall {
               left: track_try!(reader.parse_next()),
               op: track_try!(reader.parse_next()),
               right: track_try!(reader.parse_next()),
           })
    }
}
impl<'token, 'text: 'token> TokenRange for BinaryOpCall<'token, 'text> {
    fn token_start(&self) -> usize {
        self.left.token_start()
    }
    fn token_end(&self) -> usize {
        self.right.token_end()
    }
}
