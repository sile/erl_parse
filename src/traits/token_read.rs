use erl_tokenize::LexicalToken;

use {Result, ErrorKind};
use traits::Preprocessor;

pub trait TokenRead: Preprocessor {
    fn try_read_token(&mut self) -> Result<Option<LexicalToken>>;
    fn read_token(&mut self) -> Result<LexicalToken> {
        if let Some(token) = track!(self.try_read_token())? {
            Ok(token)
        } else {
            track_panic!(ErrorKind::UnexpectedEos);
        }
    }
    fn unread_token(&mut self, token: LexicalToken);
}
