use erl_pp;
use erl_tokenize::{self, LexicalToken};
use trackable::error::TrackableError;
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt};

/// This crate specific error type.
#[derive(Debug, Clone)]
pub struct Error(TrackableError<ErrorKind>);
derive_traits_for_trackable_error_newtype!(Error, ErrorKind);
impl From<erl_tokenize::Error> for Error {
    fn from(f: erl_tokenize::Error) -> Self {
        ErrorKind::TokenizationFailed.takes_over(f).into()
    }
}
impl From<erl_pp::Error> for Error {
    fn from(f: erl_pp::Error) -> Self {
        ErrorKind::Other.takes_over(f).into()
    }
}

/// The list of the possible error kinds
#[derive(Debug, Clone)]
pub enum ErrorKind {
    TokenizationFailed,
    InvalidInput,
    UnexpectedToken(LexicalToken),
    UnexpectedEos,
    Other,
}
impl TrackableErrorKind for ErrorKind {}
