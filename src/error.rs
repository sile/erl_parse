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
        match *f.kind() {
            erl_tokenize::ErrorKind::InvalidInput => ErrorKind::InvalidInput.takes_over(f).into(),
            erl_tokenize::ErrorKind::UnexpectedEos => ErrorKind::UnexpectedEos.takes_over(f).into(),
        }
    }
}
impl From<erl_pp::Error> for Error {
    fn from(f: erl_pp::Error) -> Self {
        match f.kind().clone() {
            erl_pp::ErrorKind::InvalidInput => ErrorKind::InvalidInput.takes_over(f).into(),
            erl_pp::ErrorKind::UnexpectedToken(t) => {
                ErrorKind::UnexpectedToken(t).takes_over(f).into()
            }
            erl_pp::ErrorKind::UnexpectedEos => ErrorKind::UnexpectedEos.takes_over(f).into(),
        }
    }
}

/// The list of the possible error kinds
#[derive(Debug, Clone)]
pub enum ErrorKind {
    InvalidInput,
    UnexpectedToken(LexicalToken),
    UnexpectedEos,
    Other,
}
impl TrackableErrorKind for ErrorKind {}
