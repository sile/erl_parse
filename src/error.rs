use erl_pp;
use erl_tokenize;
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
        // TODO: takes_over
        ErrorKind::Other.cause(f.to_string()).into()
    }
}

/// The list of the possible error kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    TokenizationFailed,
    InvalidInput,
    UnexpectedEos,
    Other,
}
impl TrackableErrorKind for ErrorKind {}
