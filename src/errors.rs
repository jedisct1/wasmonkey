#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum WError {
    #[error("Internal error: {0}")]
    InternalError(&'static str),
    #[error("Incorrect usage: {0}")]
    UsageError(&'static str),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    WAsmError(#[from] parity_wasm::elements::Error),
    #[error("Parse error")]
    ParseError,
    #[error("Unsupported")]
    Unsupported,
}
