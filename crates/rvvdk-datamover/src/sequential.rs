use std::time::Instant;

use rvvdk_core::{BufferPool, CopyOperation, CopyProgress, Error, Extent, Result, VirtualDisk};

use crate::policy::{self, Operation};
use crate::{CopyOptions, CopyPlan, CopyStats, ExecutionBackend, ProgressObserver, ProgressState};

const PROGRESS_INTERVAL_BYTES: u64 = 64 * 1024 * 1024;

/// Statically dispatched hooks keep observer work out of unobserved copies.
pub(crate) trait Progress {
    fn completed(&mut self, operation: Operation, length: u64);
    fn extent_completed(&mut self);
}

impl Progress for () {
    fn completed(&mut self, _: Operation, _: u64) {}
    fn extent_completed(&mut self) {}
}

pub(crate) struct Observed<'a, O: ?Sized> {
    observer: &'a O,
    state: ProgressState,
    started: Instant,
    last_emitted: u64,
}

impl<'a, O: ProgressObserver + ?Sized> Observed<'a, O> {
    pub(crate) fn new(plan: &CopyPlan, observer: &'a O, started: Instant) -> Self {
        Self {
            observer,
            state: ProgressState::from_plan(plan),
            started,
            last_emitted: 0,
        }
    }

    fn emit(&mut self, force: bool) {
        let completed = self.state.completed().logical_bytes_completed();
        if completed == self.last_emitted
            || (!force && completed.saturating_sub(self.last_emitted) < PROGRESS_INTERVAL_BYTES)
        {
            return;
        }
        self.observer.on_progress(
            &self
                .state
                .snapshot(ExecutionBackend::Threaded, self.started.elapsed()),
        );
        self.last_emitted = completed;
    }
}

impl<O: ProgressObserver + ?Sized> Progress for Observed<'_, O> {
    fn completed(&mut self, operation: Operation, length: u64) {
        match operation {
            Operation::Copy => self.state.complete_data(length),
            Operation::Zero => self.state.complete_zero(length),
            Operation::Discard => self.state.complete_discard(length),
            Operation::WriteZero => self.state.complete_fallback_write(length),
        }
        // Preserve byte-threshold callbacks for payload/fallback writes only.
        if matches!(operation, Operation::Copy | Operation::WriteZero) {
            self.emit(false);
        }
    }

    fn extent_completed(&mut self) {
        self.state.complete_extent();
        if self.state.completed().logical_bytes_completed() < self.state.totals().logical_bytes() {
            self.emit(true);
        }
    }
}

#[derive(Default)]
struct Stats {
    read: u64,
    written: u64,
    zeroed: u64,
    discarded: u64,
    blocks: u64,
    extents: u64,
}

impl Stats {
    fn progress(&self) -> CopyProgress {
        CopyProgress {
            bytes_read: self.read,
            bytes_written: self.written,
            bytes_zeroed: self.zeroed,
            bytes_discarded: self.discarded,
            blocks_completed: self.blocks,
            extents_completed: Some(self.extents),
            unconfirmed_io: false,
        }
    }
    fn checkpoint(&self, control: &impl crate::control::Checkpoint) -> Result<()> {
        control.check(self.progress()).map_err(|e| {
            crate::failure::execution("threaded", CopyOperation::Cancel, None, self.progress(), e)
        })
    }

    #[cold]
    fn failure(&self, operation: CopyOperation, range: Option<(u64, u64)>, error: Error) -> Error {
        crate::failure::execution(
            "threaded",
            operation,
            range,
            CopyProgress {
                bytes_read: self.read,
                bytes_written: self.written,
                bytes_zeroed: self.zeroed,
                bytes_discarded: self.discarded,
                blocks_completed: self.blocks,
                extents_completed: Some(self.extents),
                unconfirmed_io: !matches!(
                    operation,
                    CopyOperation::Allocate | CopyOperation::Flush
                ),
            },
            error,
        )
    }
}

/// One sequential lifecycle for direct copy and observed/unobserved plans.
/// The caller validates extents/endpoints and owns initial/final notifications.
pub(crate) fn execute<S, D, P>(
    source: &S,
    destination: &D,
    extents: &[Extent],
    options: CopyOptions,
    started: Instant,
    progress: &mut P,
) -> Result<CopyStats>
where
    S: VirtualDisk + ?Sized,
    D: VirtualDisk + ?Sized,
    P: Progress,
{
    execute_controlled(
        source,
        destination,
        extents,
        options,
        started,
        progress,
        &(),
    )
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_controlled<
    S: VirtualDisk + ?Sized,
    D: VirtualDisk + ?Sized,
    P: Progress,
    C: crate::control::Checkpoint,
>(
    source: &S,
    destination: &D,
    extents: &[Extent],
    options: CopyOptions,
    started: Instant,
    progress: &mut P,
    control: &C,
) -> Result<CopyStats> {
    let mut stats = Stats::default();
    let pool = BufferPool::new(
        options.buffer_count(),
        options.block_size(),
        options.buffer_alignment(),
    )
    .map_err(|e| stats.failure(CopyOperation::Allocate, None, e))?;
    let mut buffer = pool.acquire();
    for extent in extents {
        stats.checkpoint(control)?;
        match policy::select(extent.kind(), || destination.capabilities()) {
            Operation::Copy => transfer::<true, _, _, _, _>(
                source,
                destination,
                *extent,
                buffer.as_mut_slice(),
                options.block_size(),
                &mut stats,
                progress,
                control,
            )?,
            Operation::WriteZero => {
                buffer.as_mut_slice().fill(0);
                transfer::<false, _, _, _, _>(
                    source,
                    destination,
                    *extent,
                    buffer.as_mut_slice(),
                    options.block_size(),
                    &mut stats,
                    progress,
                    control,
                )?;
            }
            Operation::Zero => {
                destination
                    .write_zero_at(extent.offset(), extent.length())
                    .map_err(|e| {
                        stats.failure(
                            CopyOperation::WriteZero,
                            Some((extent.offset(), extent.length())),
                            e,
                        )
                    })?;
                stats.zeroed += extent.length();
                progress.completed(Operation::Zero, extent.length());
            }
            Operation::Discard => {
                destination
                    .discard(extent.offset(), extent.length())
                    .map_err(|e| {
                        stats.failure(
                            CopyOperation::Discard,
                            Some((extent.offset(), extent.length())),
                            e,
                        )
                    })?;
                stats.discarded += extent.length();
                progress.completed(Operation::Discard, extent.length());
            }
        }
        stats.extents += 1;
        progress.extent_completed();
        stats.checkpoint(control)?;
    }
    control.flushing(stats.progress()).map_err(|e| {
        crate::failure::execution("threaded", CopyOperation::Cancel, None, stats.progress(), e)
    })?;
    destination
        .flush()
        .map_err(|e| stats.failure(CopyOperation::Flush, None, e))?;
    Ok(CopyStats::new(
        stats.read,
        stats.written,
        stats.zeroed,
        stats.discarded,
        stats.blocks,
        extents.len() as u64,
        started.elapsed(),
    ))
}

#[allow(clippy::too_many_arguments)]
fn transfer<const READ: bool, S, D, P, C: crate::control::Checkpoint>(
    source: &S,
    destination: &D,
    extent: Extent,
    buffer: &mut [u8],
    block_size: usize,
    stats: &mut Stats,
    progress: &mut P,
    control: &C,
) -> Result<()>
where
    S: VirtualDisk + ?Sized,
    D: VirtualDisk + ?Sized,
    P: Progress,
{
    let mut offset = extent.offset();
    while offset < extent.end() {
        stats.checkpoint(control)?;
        let length = (extent.end() - offset).min(block_size as u64);
        let size = usize::try_from(length).map_err(|_| Error::RangeOverflow { offset, length })?;
        let bytes = &mut buffer[..size];
        if READ {
            source
                .read_exact_at(offset, bytes)
                .map_err(|e| stats.failure(CopyOperation::Read, Some((offset, length)), e))?;
            stats.read += length;
            stats.checkpoint(control)?;
        }
        destination
            .write_all_at(offset, bytes)
            .map_err(|e| stats.failure(CopyOperation::Write, Some((offset, length)), e))?;
        offset = offset
            .checked_add(length)
            .ok_or(Error::RangeOverflow { offset, length })?;
        stats.written += length;
        stats.blocks += 1;
        progress.completed(
            if READ {
                Operation::Copy
            } else {
                Operation::WriteZero
            },
            length,
        );
        stats.checkpoint(control)?;
    }
    Ok(())
}
