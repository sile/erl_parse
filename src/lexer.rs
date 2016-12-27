use std::str::Chars;
use std::iter::Peekable;

use token::Token;

pub type TokenizeResult<T> = Result<T, TokenizeError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizeError {
    pub line: usize,
    pub column: usize,
    pub reason: ErrorReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorReason {
    IllegalChar(char),
    IllegalRadix(String),
    IllegalHexadecimal(String),
    IllegalFloat(String),
    IllegalUnicode(u32),
    UnexpectedEof,
}

pub struct Lexer<'a> {
    tokens: Vec<Token>,
    text: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
}
impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Self {
        Lexer {
            tokens: Vec::new(),
            text: text.chars().peekable(),
            line: 1,
            column: 1,
        }
    }
    fn error(&self, reason: ErrorReason) -> TokenizeError {
        TokenizeError {
            line: self.line,
            column: self.column,
            reason: reason,
        }
    }
    pub fn tokenize(mut self) -> TokenizeResult<Vec<Token>> {
        while let Some(c) = self.peek_char() {
            match c {
                ' ' | '\t' | '\r' | '\n' => self.consume_char()?,
                'a'...'z' => self.scan_atom_or_keyword()?,
                'A'...'Z' | '_' => self.scan_variable()?,
                '0'...'9' => self.scan_number()?,
                '$' => self.scan_character()?,
                '"' => self.scan_string()?,
                '\'' => self.scan_quoted_atom()?,
                '%' => self.scan_comment()?,
                _ => self.scan_symbol()?,
            }
        }
        Ok(self.tokens)
    }
    fn scan_atom_or_keyword(&mut self) -> TokenizeResult<()> {
        fn is_atom_non_head_char(c: char) -> bool {
            match c {
                'a'...'z' | 'A'...'Z' | '@' | '_' | '0'...'9' => true,
                _ => false,
            }
        }
        let name = self.read_while(is_atom_non_head_char)?;
        if let Some(k) = name.parse().ok() {
            self.tokens.push(Token::Keyword(k));
        } else {
            self.tokens.push(Token::Atom(name));
        }
        Ok(())
    }
    fn scan_variable(&mut self) -> TokenizeResult<()> {
        fn is_var_char(c: char) -> bool {
            match c {
                'a'...'z' | 'A'...'Z' | '@' | '_' | '0'...'9' => true,
                _ => false,
            }
        }
        let var = self.read_while(is_var_char)?;
        self.tokens.push(Token::Var(var));
        Ok(())
    }
    fn scan_character(&mut self) -> TokenizeResult<()> {
        self.consume_char()?;
        let c = self.read_char()?;
        self.tokens.push(Token::Char(c));
        Ok(())
    }
    fn scan_string(&mut self) -> TokenizeResult<()> {
        // See: http://erlang.org/doc/reference_manual/data_types.html#id76742
        self.consume_char()?;
        let mut buf = String::new();
        loop {
            let c = match self.read_char()? {
                '\\' => self.read_escaped_char()?,
                '"' => break,
                c => c,
            };
            buf.push(c);
        }
        self.tokens.push(Token::String(buf));
        Ok(())
    }
    fn scan_quoted_atom(&mut self) -> TokenizeResult<()> {
        self.consume_char()?;
        let mut buf = String::new();
        loop {
            let c = match self.read_char()? {
                '\\' => self.read_escaped_char()?,
                '\'' => break,
                c => c,
            };
            buf.push(c);
        }
        self.tokens.push(Token::Atom(buf));
        Ok(())
    }
    fn scan_comment(&mut self) -> TokenizeResult<()> {
        let line = self.read_while(|c| c != '\n')?;
        let _ = self.consume_char();
        self.tokens.push(Token::Comment(line));
        Ok(())
    }
    fn scan_symbol(&mut self) -> TokenizeResult<()> {
        use token::Symbol;
        let symbol = match self.read_char()? {
            '[' => Symbol::OpenSquare,
            ']' => Symbol::CloseSquare,
            '(' => Symbol::OpenParen,
            ')' => Symbol::CloseParen,
            '{' => Symbol::OpenBrace,
            '}' => Symbol::CloseBrace,
            '#' => Symbol::Sharp,
            '.' => Symbol::Dot,
            ',' => Symbol::Comma,
            ';' => Symbol::Semicolon,
            '?' => Symbol::Question,
            '!' => Symbol::Not,
            '*' => Symbol::Multiply,
            '<' => {
                match self.read_char_if("-=<") {
                    Some('-') => Symbol::LeftAllow,
                    Some('=') => Symbol::DoubleLeftAllow,
                    Some('<') => Symbol::DoubleLeftAngle,
                    _ => Symbol::Less,
                }
            }
            '>' => {
                match self.read_char_if(">=") {
                    Some('>') => Symbol::DoubleRightAngle,
                    Some('=') => Symbol::GreaterEq,
                    _ => Symbol::Greater,
                }
            }
            '/' => {
                if self.read_char_if("=").is_some() {
                    Symbol::NotEq
                } else {
                    Symbol::Slash
                }
            }
            '=' => {
                match self.read_char_if("<>=:/") {
                    Some('<') => Symbol::LessEq,
                    Some('>') => Symbol::DoubleRightAllow,
                    Some('=') => Symbol::Eq,
                    Some(':') => {
                        if self.read_char_if("=").is_some() {
                            Symbol::ExactEq
                        } else {
                            self.tokens.push(Token::Symbol(Symbol::Match));
                            Symbol::Colon
                        }
                    }
                    Some('/') => {
                        if self.read_char_if("=").is_some() {
                            Symbol::ExactNotEq
                        } else {
                            self.tokens.push(Token::Symbol(Symbol::Match));
                            Symbol::Slash
                        }
                    }
                    _ => Symbol::Match,
                }
            }
            ':' => {
                match self.read_char_if("=") {
                    Some('=') => Symbol::MapMatch,
                    _ => Symbol::Colon,
                }
            }
            '|' => {
                match self.read_char_if("|") {
                    Some('|') => Symbol::DoubleVerticalBar,
                    _ => Symbol::VerticalBar,
                }
            }
            '-' => {
                match self.read_char_if("->") {
                    Some('-') => Symbol::MinusMinus,
                    Some('>') => Symbol::RightAllow,
                    _ => Symbol::Hyphen,
                }
            }
            '+' => {
                match self.read_char_if("+") {
                    Some('+') => Symbol::PlusPlus,
                    _ => Symbol::Plus,
                }
            }
            c => Err(self.error(ErrorReason::IllegalChar(c)))?,
        };
        self.tokens.push(Token::Symbol(symbol));
        Ok(())
    }
    fn scan_integer(&mut self, radix: u32) -> TokenizeResult<()> {
        use num::Num;
        let buf = self.read_while(|c| c.is_digit(radix))?;
        let n = Num::from_str_radix(&buf, radix).unwrap();
        self.tokens.push(Token::Integer(n));
        Ok(())
    }
    fn scan_float(&mut self, mut buf: String) -> TokenizeResult<()> {
        buf.push_str(&self.read_while(|c| c.is_digit(10))?);
        if self.read_char_if("eE").is_some() {
            buf.push('e');
            if let Some(sign) = self.read_char_if("-+") {
                buf.push(sign);
            }
            buf.push_str(&self.read_while(|c| c.is_digit(10))?);
        }
        let n = buf.parse().map_err(|_| self.error(ErrorReason::IllegalFloat(buf)))?;
        self.tokens.push(Token::Float(n));
        Ok(())
    }
    fn scan_number(&mut self) -> TokenizeResult<()> {
        // See: http://erlang.org/doc/reference_manual/data_types.html#id65900
        let mut buf = String::new();
        while let Some(c) = self.peek_char() {
            match c {
                '0'...'9' => {
                    self.consume_char()?;
                    buf.push(c);
                }
                '.' => {
                    buf.push(self.read_char()?);
                    return self.scan_float(buf);
                }
                '#' => {
                    self.consume_char()?;
                    let radix = buf.parse()
                        .ok()
                        .and_then(|radix| {
                            if 1 < radix && radix < 37 {
                                Some(radix)
                            } else {
                                None
                            }
                        })
                        .ok_or_else(|| self.error(ErrorReason::IllegalRadix(buf)))?;
                    return self.scan_integer(radix);
                }
                _ => break,
            }
        }
        let n = buf.parse().unwrap();
        self.tokens.push(Token::Integer(n));
        Ok(())
    }
    fn read_while<F>(&mut self, f: F) -> TokenizeResult<String>
        where F: Fn(char) -> bool
    {
        let mut buf = String::new();
        while let Some(c) = self.peek_char() {
            if !f(c) {
                break;
            }
            self.consume_char()?;
            buf.push(c);
        }
        Ok(buf)
    }
    fn read_escaped_char(&mut self) -> TokenizeResult<char> {
        use std::char;
        use num::Num;
        let c = self.read_char()?;
        Ok(match c {
            'b' => 8 as char, // Back Space
            'd' => 127 as char, // Delete
            'e' => 27 as char, // Escape,
            'f' => 12 as char, // Form Feed
            'n' => '\n',
            'r' => '\r',
            's' => ' ',
            't' => '\t',
            'v' => 11 as char, // Vertical Tabulation
            '^' => {
                match self.read_char()? {
                    c @ 'a'...'z' => ((c as u8) - ('a' as u8) + 1) as char,
                    c @ 'A'...'Z' => ((c as u8) - ('A' as u8) + 1) as char,
                    c => Err(self.error(ErrorReason::IllegalChar(c)))?,
                }
            }
            'x' => {
                let c = self.read_char()?;
                let buf = if c == '{' {
                    let buf = self.read_while(|c| c != '}')?;
                    self.consume_char()?;
                    buf
                } else {
                    let mut buf = String::with_capacity(2);
                    buf.push(c);
                    buf.push(self.read_char()?);
                    buf
                };
                let code: u32 = Num::from_str_radix(&buf, 16)
                    .map_err(|_| self.error(ErrorReason::IllegalHexadecimal(buf)))?;
                char::from_u32(code).ok_or_else(|| self.error(ErrorReason::IllegalUnicode(code)))?
            }
            c @ '0'...'7' => {
                let mut n = c.to_digit(8).unwrap();
                if let Some(c) = self.read_char_if("012345677") {
                    n = (n * 8) + c.to_digit(8).unwrap();
                }
                if let Some(c) = self.read_char_if("012345677") {
                    n = (n * 8) + c.to_digit(8).unwrap();
                }
                char::from_u32(n).unwrap()
            }
            _ => c,
        })
    }
    fn read_char_if(&mut self, expects: &str) -> Option<char> {
        self.peek_char().and_then(|c| if expects.contains(c) {
            self.consume_char().unwrap();
            Some(c)
        } else {
            None
        })
    }
    fn read_char(&mut self) -> TokenizeResult<char> {
        if let Some(c) = self.text.next() {
            match c {
                '\n' => {
                    self.line += 1;
                    self.tokens.push(Token::LineNum(self.line));
                    self.column = 1;
                }
                _ => {
                    self.column += 1;
                }
            }
            Ok(c)
        } else {
            Err(self.error(ErrorReason::UnexpectedEof))
        }
    }
    fn peek_char(&mut self) -> Option<char> {
        self.text.peek().cloned()
    }
    fn consume_char(&mut self) -> TokenizeResult<()> {
        self.read_char()?;
        Ok(())
    }
}
