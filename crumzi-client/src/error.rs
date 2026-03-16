use std::io;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("protocol error: {0}")]
    Proto(#[from] ProtoError),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("server error: {0}")]
    Server(#[from] AckError),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ProtoError {
    #[error("expected OK terminator")]
    NotOk,

    #[error("bad banner: {0:?}")]
    BadBanner(String),

    #[error("unexpected EOF")]
    UnexpectedEof,

    #[error("expected `Key: Value` pair line, got: {0:?}")]
    NotPair(String),

    #[error("missing field: {0}")]
    MissingField(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("ACK [{code}@{command_idx}] {{{command}}} {message}")]
pub struct AckError {
    pub code: u32,
    pub command_idx: u32,
    pub command: String,
    pub message: String,
}
