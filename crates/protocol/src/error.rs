use std::string::FromUtf8Error;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("parsing a var int failed: data suggests there are more bytes but int is overflowing")]
    VarIntOverflow,
    #[error(
        "parsing a var long failed: data suggests there are more bytes but long is overflowing"
    )]
    VarLongOverflow,
    #[error("unknown packet id {0} in version {1}")]
    UnknownPacket(i32, i32),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8(#[from] FromUtf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;
