use erl_tokenize;
use trackable::error::{TrackableError, IntoTrackableError};
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt};

/// This crate specific error type.
pub type Error = TrackableError<ErrorKind>;

/// The list of the possible error kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    TokenizationFailed,
    Other,
}

impl TrackableErrorKind for ErrorKind {}
impl IntoTrackableError<erl_tokenize::Error> for ErrorKind {
    fn into_trackable_error(e: erl_tokenize::Error) -> Error {
        ErrorKind::TokenizationFailed.cause(e)
    }
}
