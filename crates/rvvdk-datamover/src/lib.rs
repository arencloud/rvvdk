mod concurrent;
mod execution;
mod extent_validation;
mod failure;
#[cfg(target_os = "linux")]
pub mod io_uring;
mod mover;
mod native;
mod observer;
mod options;
mod plan;
mod planner;
mod policy;
mod preflight;
mod preparation;
mod progress;
mod sequential;
mod stats;
mod work;

pub use execution::{
    ExecutionBackend, ExecutionSelection, ExecutionSelectionReason, ExecutionStrategy,
};
pub use mover::DataMover;
pub use options::{
    CopyOptions, DEFAULT_BLOCK_SIZE, DEFAULT_BUFFER_ALIGNMENT, DEFAULT_BUFFER_COUNT,
    DEFAULT_CONCURRENCY, DEFAULT_QUEUE_CAPACITY,
};
pub use stats::{CopyReport, CopyStats};

#[cfg(target_os = "linux")]
pub use execution::IoUringExecutionOptions;

pub use native::{NativeCopyReport, NativeCopyStats};
pub use observer::{NoopProgressObserver, ProgressObserver};
pub use plan::{CopyPlan, CopyPlanSummary};
pub(crate) use progress::ProgressState;
pub use progress::{ProgressCompleted, ProgressSnapshot, ProgressTotals};
