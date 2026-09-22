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

    #[error("unexpected end of input at offset={offset}, remaining={remaining}")]
    UnexpectedEof { offset: u64, remaining: usize },

    #[error("write made no progress at offset={offset}, remaining={remaining}")]
    WriteZero { offset: u64, remaining: usize },

    #[error("invalid buffer alignment: alignment={alignment}")]
    InvalidBufferAlignment { alignment: usize },

    #[error("buffer allocation failed: size={size}, alignment={alignment}")]
    BufferAllocation { size: usize, alignment: usize },

    #[error("buffer pool capacity must be greater than zero")]
    InvalidBufferPoolCapacity,

    #[error("work queue capacity must be greater than zero")]
    InvalidWorkQueueCapacity,

    #[error("work queue closed unexpectedly")]
    WorkQueueClosed,

    #[error(
        "direct I/O alignment violation: offset={offset}, length={length}, alignment={alignment}"
    )]
    DirectIoAlignment {
        offset: u64,
        length: usize,
        alignment: usize,
    },

    #[error("io_uring queue depth must be greater than zero")]
    InvalidIoUringQueueDepth,

    #[error("io_uring submission queue is full: queue_depth={queue_depth}")]
    IoUringQueueFull { queue_depth: u32 },

    #[error("io_uring has no operations in flight")]
    IoUringNoInFlight,

    #[error("io_uring completion was expected but none was available")]
    IoUringCompletionMissing,

    #[error("unexpected io_uring completion: expected user_data={expected}, actual={actual}")]
    IoUringUnexpectedCompletion { expected: u64, actual: u64 },

    #[error("buffer is too small: requested={requested}, available={available}")]
    BufferTooSmall { requested: usize, available: usize },

    #[error("io_uring completion references unknown user_data={user_data}")]
    IoUringUnknownCompletion { user_data: u64 },

    #[error("cannot mix borrowed and owned io_uring operations")]
    IoUringOperationModeConflict,

    #[error("io_uring engine has been shut down")]
    IoUringEngineShutDown,

    #[error("short write at offset {offset}: expected {expected} bytes, wrote {actual}")]
    ShortWrite {
        offset: u64,
        expected: usize,
        actual: usize,
    },

    #[error("native execution strategy was not selected")]
    NativeExecutionNotSelected,

    #[error("selected native execution strategy is unsupported by the backend pair")]
    NativeExecutionUnsupported,

    #[error(
        "native execution does not yet support extent kind {kind} at offset={offset}, length={length}"
    )]
    UnsupportedNativeExtent {
        kind: &'static str,
        offset: u64,
        length: u64,
    },

    #[error("corrupt metadata: {0}")]
    CorruptMetadata(String),
}
