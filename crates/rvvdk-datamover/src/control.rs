//! Cooperative cancellation and coordinator-owned copy lifecycle notifications.
use crate::ExecutionBackend;
use rvvdk_core::{CopyProgress, Error, Result};
use std::{
    cell::Cell,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

/// A cancellation request is sticky for the lifetime of an invocation. Implementors
/// must be cheap and must not panic. Blocking backend calls cannot be interrupted.
pub trait Cancellation: Sync {
    fn is_cancelled(&self) -> bool;
}
#[derive(Clone, Default)]
pub struct CancellationToken(Arc<AtomicBool>);
impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}
impl Cancellation for CancellationToken {
    fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}
#[derive(Default)]
pub struct NoCancellation;
impl Cancellation for NoCancellation {
    fn is_cancelled(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CopyPhase {
    Preparing,
    Started,
    Transferring,
    Flushing,
    Completed,
    Cancelled,
    Failed,
}
/// Confirmed counters are lower bounds, never a contiguous or durable resume point.
/// Completed means the engine's flush succeeded; external verification/publication
/// are owned by the caller. Callbacks run on the invoking coordinator, not workers.
#[derive(Debug, Clone, Copy)]
pub struct CopyEvent {
    pub phase: CopyPhase,
    pub backend: Option<ExecutionBackend>,
    pub logical_bytes: u64,
    pub progress: CopyProgress,
    pub elapsed: Duration,
}
pub trait CopyObserver {
    fn on_event(&self, event: &CopyEvent);
}
impl<F: Fn(&CopyEvent)> CopyObserver for F {
    fn on_event(&self, event: &CopyEvent) {
        self(event);
    }
}

pub(crate) trait Checkpoint {
    const ENABLED: bool;
    fn cancellation(&self) -> &dyn Cancellation;
    fn check(&self, progress: CopyProgress) -> Result<()>;
    fn flushing(&self, progress: CopyProgress) -> Result<()> {
        self.check(progress)
    }
}
impl Checkpoint for () {
    const ENABLED: bool = false;
    fn cancellation(&self) -> &dyn Cancellation {
        &NoCancellation
    }
    fn check(&self, _: CopyProgress) -> Result<()> {
        Ok(())
    }
}
pub(crate) struct Lifecycle<'a, C: ?Sized, O: ?Sized> {
    pub cancellation: &'a C,
    pub observer: &'a O,
    pub logical_bytes: u64,
    pub backend: Cell<Option<ExecutionBackend>>,
    started: Instant,
    last_bytes: Cell<u64>,
    last_checked_bytes: Cell<u64>,
    last_time: Cell<Duration>,
    pub progress: Cell<CopyProgress>,
}
impl<'a, C: Cancellation + ?Sized, O: CopyObserver + ?Sized> Lifecycle<'a, C, O> {
    pub fn new(cancellation: &'a C, observer: &'a O, logical_bytes: u64) -> Self {
        Self {
            cancellation,
            observer,
            logical_bytes,
            backend: Cell::new(None),
            started: Instant::now(),
            last_bytes: Cell::new(0),
            last_checked_bytes: Cell::new(0),
            last_time: Cell::new(Duration::ZERO),
            progress: Cell::new(CopyProgress::default()),
        }
    }
    pub fn emit(&self, phase: CopyPhase, progress: CopyProgress) {
        self.progress.set(progress);
        self.observer.on_event(&CopyEvent {
            phase,
            backend: self.backend.get(),
            logical_bytes: self.logical_bytes,
            progress,
            elapsed: self.started.elapsed(),
        });
    }
    pub fn cancelled(&self) -> Result<()> {
        if self.cancellation.is_cancelled() {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
}
impl<C: Cancellation, O: CopyObserver + ?Sized> Checkpoint for Lifecycle<'_, C, O> {
    const ENABLED: bool = true;
    fn cancellation(&self) -> &dyn Cancellation {
        self.cancellation
    }
    fn flushing(&self, progress: CopyProgress) -> Result<()> {
        self.check(progress)?;
        self.emit(CopyPhase::Flushing, progress);
        self.cancelled()
    }
    fn check(&self, progress: CopyProgress) -> Result<()> {
        self.progress.set(progress);
        self.cancelled()?;
        let bytes = progress.bytes_written + progress.bytes_zeroed + progress.bytes_discarded;
        if bytes == self.last_checked_bytes.replace(bytes) {
            return Ok(());
        }
        let elapsed = self.started.elapsed();
        if bytes > self.last_bytes.get()
            && (self.last_bytes.get() == 0
                || bytes - self.last_bytes.get() >= 64 * 1024 * 1024
                || elapsed - self.last_time.get() >= Duration::from_millis(100))
        {
            self.last_bytes.set(bytes);
            self.last_time.set(elapsed);
            self.emit(CopyPhase::Transferring, progress);
        }
        self.cancelled()
    }
}
pub(crate) fn sum(mut a: CopyProgress, b: CopyProgress) -> CopyProgress {
    a.bytes_read += b.bytes_read;
    a.bytes_written += b.bytes_written;
    a.bytes_zeroed += b.bytes_zeroed;
    a.bytes_discarded += b.bytes_discarded;
    a.blocks_completed += b.blocks_completed;
    a.unconfirmed_io |= b.unconfirmed_io;
    a
}
