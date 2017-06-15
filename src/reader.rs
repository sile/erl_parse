use erl_tokenize::LexicalToken;

use {Result, ErrorKind};

// TODO: Support macro expansions

#[derive(Debug, Clone)]
pub struct TokenReader<'a> {
    tokens: &'a [LexicalToken],
    position: usize,
    line_num: usize,
}
impl<'a> TokenReader<'a> {
    pub fn new(tokens: &'a [LexicalToken]) -> Self {
        TokenReader {
            tokens,
            position: 0,
            line_num: 1,
        }
    }
    pub fn position(&self) -> usize {
        self.position
    }
    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }
    pub fn read(&mut self) -> Result<&LexicalToken> {
        if let Some(token) = self.tokens.get(self.position) {
            self.position += 1;
            Ok(token)
        } else {
            track_panic!(ErrorKind::UnexpectedEos);
        }
    }
}
