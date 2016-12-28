use std::iter::Peekable;
use num::traits::ToPrimitive;

use token::{Token, Keyword, Symbol};
use ast;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedToken(Token),
    UnexpectedEos,
}

pub struct Parser<T: Iterator> {
    line: usize,
    tokens: Peekable<T>,
}
impl<T> Parser<T>
    where T: Iterator<Item = Token>
{
    pub fn new(tokens: T) -> Self {
        Parser {
            line: 1,
            tokens: tokens.peekable(),
        }
    }
    pub fn parse_module(&mut self) -> ParseResult<ast::Module> {
        let mut forms = Vec::new();
        while let Some(token) = self.read_token_if_exists() {
            match token {
                Token::Symbol(Symbol::Hyphen) => {
                    match self.read_token()? {
                        Token::Atom(name) => {
                            // See: http://erlang.org/doc/reference_manual/modules.html
                            let form = match name.as_str() {
                                "module" => self.parse_module_decl()?,
                                "export" => self.parse_export_attr()?,
                                "export_type" => self.parse_export_attr()?,                                
                                "import" => unimplemented!(),
                                "compile" => unimplemented!(),
                                "vsn" => unimplemented!(),
                                "on_load" => unimplemented!(),
                                "include" => unimplemented!(),
                                "include_lib" => unimplemented!(),
                                "behaviour" | "behavior" => unimplemented!(),
                                "spec" => unimplemented!(),
                                "type" => unimplemented!(),
                                "callback" => unimplemented!(),
                                "record" => unimplemented!(),
                                "file" => unimplemented!(),
                                name => panic!("# user defined: {}", name),
                            };
                            forms.push(form);
                        }
                        token => Err(ParseError::UnexpectedToken(token))?,
                    }
                }
                Token::Atom(function_name) => {
                    //
                    unimplemented!()
                }
                Token::Comment(_) => {
                    // TODO:
                }
                Token::LineNum(_) => unreachable!(),
                _ => Err(ParseError::UnexpectedToken(token))?,
            }
        }
        Ok(ast::Module { forms: forms })
    }
    fn parse_export_attr(&mut self) -> ParseResult<ast::Form> {
        self.expect_token(Symbol::OpenParen)?;
        let exports = self.parse_list_of(|mut this| this.parse_fa())?;
        self.expect_token(Symbol::CloseParen)?;
        self.expect_token(Symbol::Dot)?;
        Ok(From::from(ast::ExportAttr { exports: exports }))
    }
    fn parse_module_decl(&mut self) -> ParseResult<ast::Form> {
        self.expect_token(Symbol::OpenParen)?;
        let name = match self.read_token()? {
            Token::Atom(name) => name,
            other => Err(ParseError::UnexpectedToken(other))?,
        };
        self.expect_token(Symbol::CloseParen)?;
        self.expect_token(Symbol::Dot)?;
        Ok(From::from(ast::ModuleDecl { name: name }))
    }

    fn parse_list_of<F, U: ::std::fmt::Debug>(&mut self, f: F) -> ParseResult<Vec<U>>
        where F: Fn(&mut Self) -> ParseResult<U>
    {
        let mut elements = Vec::new();
        self.expect_token(Symbol::OpenSquare)?;
        loop {
            match self.peek_token()? {
                &Token::Symbol(Symbol::CloseSquare) => {
                    self.consume_token()?;
                    return Ok(elements);
                }
                _ => {}
            }
            if !elements.is_empty() {
                self.expect_token(Symbol::Comma)?;
            }

            elements.push(f(self)?);
        }
    }
    fn parse_fa(&mut self) -> ParseResult<ast::FA> {
        let name = match self.read_token()? {
            Token::Atom(name) => name,
            other => Err(self.unexpected_token(other))?,
        };
        self.expect_token(Symbol::Slash)?;
        let arity = match self.read_token()? {
            Token::Integer(arity) => arity.to_usize().expect("TODO"),
            other => Err(self.unexpected_token(other))?,
        };
        Ok(ast::FA {
            fun_name: name,
            arity: arity,
        })
    }
    fn unexpected_token(&self, token: Token) -> ParseError {
        ParseError::UnexpectedToken(token)
    }
    pub fn parse_expr(&mut self) -> ParseResult<ast::Expr> {
        match self.read_token()? {
            Token::Symbol(x) => self.parse_expr_symbol(x),
            Token::Keyword(x) => self.parse_expr_keyword(x),
            Token::Var(x) => {
                let x = ast::Expr::Var(ast::Var(x));
                self.try_parse_binary_op_expr(x)
            }
            Token::Char(x) => {
                let x = ast::Expr::Literal(ast::Literal::Char(x));
                self.try_parse_binary_op_expr(x)
            }
            Token::Atom(x) => {
                let x = ast::Expr::Literal(ast::Literal::Atom(x));
                self.try_parse_binary_op_expr(x)
            }
            Token::Integer(x) => {
                let x = ast::Expr::Literal(ast::Literal::Integer(x));
                self.try_parse_binary_op_expr(x)
            }
            Token::Float(x) => {
                let x = ast::Expr::Literal(ast::Literal::Float(x));
                self.try_parse_binary_op_expr(x)
            }
            Token::String(x) => {
                let x = ast::Expr::Literal(ast::Literal::String(x));
                self.try_parse_binary_op_expr(x)
            }
            Token::Comment(_) => {
                // TODO:
                self.parse_expr()
            }
            t => unreachable!("Token: {:?}", t),
        }
    }

    fn parse_expr_symbol(&mut self, s: Symbol) -> ParseResult<ast::Expr> {
        match s {
            Symbol::OpenBrace => self.parse_expr_tuple(),
            Symbol::OpenSquare => self.parse_expr_open_square(),
            _ => panic!("[unimplemented] Symbol: {:?}", s),
        }
    }
    fn parse_expr_open_square(&mut self) -> ParseResult<ast::Expr> {
        if *self.peek_token()? == Token::Symbol(Symbol::CloseSquare) {
            self.consume_token()?;
            Ok(From::from(ast::Nil))
        } else {
            let expr0 = self.parse_expr()?;
            if *self.peek_token()? == Token::Symbol(Symbol::DoubleVerticalBar) {
                // list comprehension
                unimplemented!();
            } else {
                let mut head = ast::Cons::last(expr0);
                {
                    let mut cons: &mut ast::Cons<ast::Expr> = &mut head;
                    loop {
                        match self.read_token()? {
                            Token::Symbol(Symbol::CloseSquare) => {
                                break;
                            }
                            Token::Symbol(Symbol::Comma) => {
                                use ast::TryAsMut; // TODO
                                let next = self.parse_expr()?;
                                cons.set_tail(ast::Cons::last(next));
                                cons = {
                                    let temp = cons;
                                    temp.tail.try_as_mut().unwrap()
                                };
                            }
                            Token::Symbol(Symbol::VerticalBar) => {
                                let tail = self.parse_expr()?;
                                cons.set_tail(tail);
                                match self.read_token()? {
                                    Token::Symbol(Symbol::CloseSquare) => break,
                                    token => Err(ParseError::UnexpectedToken(token))?,
                                }
                            }
                            token => Err(ParseError::UnexpectedToken(token))?,
                        }
                    }
                }
                Ok(From::from(head))
            }
        }
    }
    fn parse_expr_tuple(&mut self) -> ParseResult<ast::Expr> {
        let mut elements = Vec::new();
        loop {
            if *self.peek_token()? == Token::Symbol(Symbol::CloseBrace) {
                break;
            }
            if !elements.is_empty() {
                let token = self.read_token()?;
                if token != Token::Symbol(Symbol::Comma) {
                    Err(ParseError::UnexpectedToken(token))?;
                }
            }
            elements.push(self.parse_expr()?);
        }
        Ok(From::from(ast::Tuple(elements)))
    }
    fn parse_expr_keyword(&mut self, k: Keyword) -> ParseResult<ast::Expr> {
        match k {
            _ => panic!("[unimplemented] Keyword: {:?}", k),
        }
    }
    fn consume_and_parse_expr(&mut self) -> ParseResult<ast::Expr> {
        self.consume_token()?;
        self.parse_expr()
    }
    fn try_parse_binary_op_expr(&mut self, left: ast::Expr) -> ParseResult<ast::Expr> {
        // TODO: consider priorities and associativeties
        match self.peek_token_if_exists() {
            Some(&Token::Keyword(k)) => {
                let e = ast::Expr::Op(match k {
                    Keyword::And => ast::And::new(left, self.consume_and_parse_expr()?).into(),
                    Keyword::Andalso => {
                        ast::Andalso::new(left, self.consume_and_parse_expr()?).into()
                    }
                    Keyword::Band => ast::Band::new(left, self.consume_and_parse_expr()?).into(),
                    Keyword::Bor => ast::Bor::new(left, self.consume_and_parse_expr()?).into(),
                    Keyword::Bsl => ast::Bsl::new(left, self.consume_and_parse_expr()?).into(),
                    Keyword::Bsr => ast::Bsr::new(left, self.consume_and_parse_expr()?).into(),
                    Keyword::Bxor => ast::Bxor::new(left, self.consume_and_parse_expr()?).into(),
                    Keyword::Div => ast::Div::new(left, self.consume_and_parse_expr()?).into(),
                    Keyword::Or => ast::Or::new(left, self.consume_and_parse_expr()?).into(),
                    Keyword::Orelse => {
                        ast::Orelse::new(left, self.consume_and_parse_expr()?).into()
                    }
                    Keyword::Rem => ast::Rem::new(left, self.consume_and_parse_expr()?).into(),
                    Keyword::Xor => ast::Xor::new(left, self.consume_and_parse_expr()?).into(),
                    _ => return Ok(left),
                });
                Ok(e)
            }
            Some(&Token::Symbol(s)) => {
                let e = ast::Expr::Op(match s {
                    Symbol::Slash => {
                        ast::DivFloat::new(left, self.consume_and_parse_expr()?).into()
                    }
                    Symbol::Match => unimplemented!(),
                    Symbol::Hyphen => ast::Sub::new(left, self.consume_and_parse_expr()?).into(),
                    Symbol::MinusMinus => {
                        ast::ListSub::new(left, self.consume_and_parse_expr()?).into()
                    }
                    Symbol::Plus => ast::Add::new(left, self.consume_and_parse_expr()?).into(),
                    Symbol::PlusPlus => {
                        ast::ListAdd::new(left, self.consume_and_parse_expr()?).into()
                    }
                    Symbol::Multiply => ast::Mul::new(left, self.consume_and_parse_expr()?).into(),
                    Symbol::Eq => ast::Eq::new(left, self.consume_and_parse_expr()?).into(),
                    Symbol::ExactEq => {
                        ast::ExactEq::new(left, self.consume_and_parse_expr()?).into()
                    }
                    Symbol::NotEq => ast::NotEq::new(left, self.consume_and_parse_expr()?).into(),
                    Symbol::ExactNotEq => {
                        ast::ExactNotEq::new(left, self.consume_and_parse_expr()?).into()
                    }
                    Symbol::Greater => {
                        ast::Greater::new(left, self.consume_and_parse_expr()?).into()
                    }
                    Symbol::GreaterEq => {
                        ast::GreaterEq::new(left, self.consume_and_parse_expr()?).into()
                    }
                    Symbol::Less => ast::Less::new(left, self.consume_and_parse_expr()?).into(),
                    Symbol::LessEq => ast::LessEq::new(left, self.consume_and_parse_expr()?).into(),
                    _ => return Ok(left),
                });
                Ok(e)
            }
            _ => Ok(left),
        }
    }

    fn expect_token<U>(&mut self, expected: U) -> ParseResult<()>
        where Token: From<U>
    {
        let token = self.read_token()?;
        if token == Token::from(expected) {
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(token))
        }
    }
    fn read_token_if_exists(&mut self) -> Option<Token> {
        match self.tokens.next() {
            None => None,
            Some(Token::LineNum(n)) => {
                self.line = n;
                self.read_token_if_exists()
            }
            Some(token) => Some(token),
        }
    }
    fn peek_token_if_exists(&mut self) -> Option<&Token> {
        while let Some(&Token::LineNum(n)) = self.tokens.peek() {
            self.line = n;
            self.tokens.next();
        }
        self.tokens.peek()
    }

    fn peek_token(&mut self) -> ParseResult<&Token> {
        self.peek_token_if_exists().ok_or(ParseError::UnexpectedEos)
    }
    fn consume_token(&mut self) -> ParseResult<()> {
        self.read_token()?;
        Ok(())
    }
    fn read_token(&mut self) -> ParseResult<Token> {
        self.read_token_if_exists().ok_or(ParseError::UnexpectedEos)
    }
}

#[cfg(test)]
mod test {
    use ast::{self, Expr, Var, Cons};
    use lexer::Lexer;
    use super::*;

    // fn parse(text: &str) -> Result<Ast, Box<::std::error::Error>> {
    fn parse_expr(text: &str) -> Result<ast::Expr, String> {
        let tokens = Lexer::new(text).tokenize().map_err(|e| format!("{:?}", e))?;
        let expr = Parser::new(tokens.into_iter()).parse_expr().map_err(|e| format!("{:?}", e))?;
        Ok(expr)
    }

    #[test]
    fn parse_expr_works() {
        assert_eq!(parse_expr("1").unwrap(), Expr::from(1));
        assert_eq!(parse_expr("$a").unwrap(), Expr::from('a'));
        assert_eq!(parse_expr("1 + 2").unwrap(),
                   Expr::from(ast::Add::new(1, 2)));
        assert_eq!(parse_expr("A div 2").unwrap(),
                   Expr::from(ast::Div::new(Var("A".to_string()), 2)));
        assert_eq!(parse_expr("{1, 2, 3}").unwrap(),
                   Expr::from(ast::Tuple::from((1, 2, 3))));
        assert_eq!(parse_expr("[]").unwrap(), Expr::from(ast::Nil));
        assert_eq!(parse_expr("[1, 2, 3]").unwrap(),
                   Expr::from(Cons::cons(1, Cons::cons(2, Cons::last(3)))));
        assert_eq!(parse_expr("[1, 2 | 3]").unwrap(),
                   Expr::from(Cons::cons(1, Cons::cons(2, 3))));
        // assert_eq!(parse_expr("[X || X <- Y]").unwrap(),
        //            Expr::from(ast::Tuple::from((1, 2, 3))));
    }

    #[test]
    fn parse_module_works() {
        let source = include_str!("../testdata/jsone.erl");
        let lexer = Lexer::new(&source);
        let tokens = lexer.tokenize().expect("Failed to tokenize");
        let _module = Parser::new(tokens.into_iter())
            .parse_module()
            .expect("Failed to parse module");
    }
}
