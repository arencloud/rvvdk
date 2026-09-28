use crate::{DiskRange, Error};

/// Operation attempted when a copy failed. Backend method failures may be partial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CopyOperation {
    Allocate,
    Read,
    Write,
    WriteZero,
    Discard,
    Flush,
    Schedule,
    NativeSetup,
    NativeCompletion,
    NativeShutdown,
}

/// Confirmed lower bounds at failure, never a durable or contiguous resume point.
/// Failed backend calls and native cleanup may have performed additional I/O.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CopyProgress {
    pub bytes_read: u64,
    pub bytes_written: u64,
    pub bytes_zeroed: u64,
    pub bytes_discarded: u64,
    pub blocks_completed: u64,
    /// None when a concurrent executor cannot attribute work to whole extents.
    pub extents_completed: Option<u64>,
    /// Additional I/O may have occurred outside the confirmed counters.
    pub unconfirmed_io: bool,
}

/// Execution failure with the original error retained in the source chain.
/// Backend labels currently used by the mover are "threaded" and "io_uring".
#[derive(Debug, thiserror::Error)]
#[error("{backend} {operation:?} failed in {range:?}: {cause}")]
pub struct CopyFailure {
    pub backend: &'static str,
    pub operation: CopyOperation,
    /// Attempted range, not necessarily the precise failing byte.
    /// None for whole-job setup, scheduling, flush, or unidentified operations.
    pub range: Option<DiskRange>,
    pub progress: CopyProgress,
    #[source]
    pub cause: Error,
}
