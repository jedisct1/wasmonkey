pub use anyhow::{anyhow, bail, ensure, Error};
use parity_wasm::elements;
use std::io;

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum WError {
    #[error("Internal error: {0}")]
    InternalError(&'static str),
    #[error("Incorrect usage: {0}")]
    UsageError(&'static str),
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    WAsmError(#[from] elements::Error),
    #[error("Parse error")]
    ParseError,
    #[error("Unsupported")]
    Unsupported,
}
