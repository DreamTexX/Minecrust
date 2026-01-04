use std::string::FromUtf8Error;

use bytes::TryGetError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("packets received are out of order")]
    OutOfOrder,
    #[error("input buffer does not contain enough data")]
    UnexpectedEof,
    #[error("input buffer contains to much data")]
    Overflow,
    #[error("{0}")]
    Custom(&'static str),
    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),
    #[error(transparent)]
    TryGetError(#[from] TryGetError),
}
