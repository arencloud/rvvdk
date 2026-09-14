use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("range overflow: offset={offset}, length={length}")]
    RangeOverflow { offset: u64, length: u64 },

    #[error("range outside device bounds: offset={offset}, length={length}, size={size}")]
    OutOfBounds { offset: u64, length: u64, size: u64 },

    #[error("invalid alignment: value={value}, alignment={alignment}")]
    InvalidAlignment { value: u64, alignment: u64 },

    #[error("operation is not supported")]
    Unsupported,

    #[error("corrupt metadata: {0}")]
    CorruptMetadata(String),
}
