mod concurrent;
mod execution;
#[cfg(target_os = "linux")]
pub mod io_uring;
mod mover;
mod native;
mod options;
mod planner;
mod stats;
mod work;

pub use execution::{ExecutionBackend, ExecutionStrategy};
pub use mover::DataMover;
pub use options::{
    CopyOptions, DEFAULT_BLOCK_SIZE, DEFAULT_BUFFER_ALIGNMENT, DEFAULT_BUFFER_COUNT,
    DEFAULT_CONCURRENCY, DEFAULT_QUEUE_CAPACITY,
};
pub use stats::{CopyReport, CopyStats};

#[cfg(target_os = "linux")]
pub use execution::IoUringExecutionOptions;

pub use native::{NativeCopyReport, NativeCopyStats};
