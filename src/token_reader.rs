use erl_tokenize::{Token, Result as TokenizeResult};

#[derive(Debug)]
pub struct TokenReader<'a, I> {
    tokens: I,
    unread: Vec<Token<'a>>,
}
impl<'a, I> TokenReader<'a, I>
    where I: Iterator<Item = TokenizeResult<Token<'a>>>
{
    pub fn new(tokens: I) -> Self {
        TokenReader {
            tokens,
            unread: Vec::new(),
        }
    }
}
