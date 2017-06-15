use erl_tokenize::Token;
use erl_tokenize::values::Whitespace;

use {Result, ErrorKind};

// TODO: Support macro expansions

#[derive(Debug, Clone)]
pub struct TokenReader<'a> {
    tokens: &'a [Token],
    position: usize,
    line_num: usize,
}
impl<'a> TokenReader<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        let mut this = TokenReader {
            tokens,
            position: 0,
            line_num: 1,
        };
        this.skip_hidden_tokens();
        this
    }
    pub fn line_num(&self) -> usize {
        self.tokens[..self.position]
            .iter()
            .filter(|t| match **t {
                Token::Whitespace(ref t) if t.value() == Whitespace::Newline => true,
                _ => false,
            })
            .count() + 1
    }
    pub fn position(&self) -> usize {
        self.position
    }
    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }
    pub fn read(&mut self) -> Result<&Token> {
        if let Some(token) = self.tokens.get(self.position) {
            self.position += 1;
            self.skip_hidden_tokens();
            Ok(token)
        } else {
            track_panic!(ErrorKind::UnexpectedEos);
        }
    }

    fn skip_hidden_tokens(&mut self) {
        let count = self.tokens
            .iter()
            .skip(self.position)
            .take_while(|&t| match *t {
                Token::Comment(_) |
                Token::Whitespace(_) => true,
                _ => false,
            })
            .count();
        self.position += count;
    }
}
