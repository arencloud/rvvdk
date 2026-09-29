use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, SyncSender, sync_channel},
};

use rvvdk_core::{BufferPool, CopyOperation, CopyProgress, Error, ExtentKind, Result, VirtualDisk};

use crate::policy::{self, Operation};
use crate::work::{WorkItem, WorkKind};

#[cfg(test)]
mod tests;

#[derive(Default)]
struct Failure {
    stopped: AtomicBool,
    unconfirmed_io: AtomicBool,
    first: Mutex<Option<Error>>,
}

impl Failure {
    fn record(&self, error: Error) {
        if error
            .copy_failure()
            .is_some_and(|f| f.progress.unconfirmed_io)
        {
            self.unconfirmed_io.store(true, Ordering::Relaxed);
        }
        let mut first = self.first.lock().unwrap_or_else(|p| p.into_inner());
        if first.is_none() {
            *first = Some(error);
        }
        // Publish cancellation only after retaining its cause. Channel closure
        // during shutdown must never replace that original error.
        self.stopped.store(true, Ordering::Release);
    }

    fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct WorkerStats {
    pub(crate) bytes_read: u64,
    pub(crate) bytes_written: u64,
    pub(crate) bytes_zeroed: u64,
    pub(crate) bytes_discarded: u64,
    pub(crate) blocks_copied: u64,
}

impl WorkerStats {
    pub(crate) fn progress(&self) -> CopyProgress {
        CopyProgress {
            bytes_read: self.bytes_read,
            bytes_written: self.bytes_written,
            bytes_zeroed: self.bytes_zeroed,
            bytes_discarded: self.bytes_discarded,
            blocks_completed: self.blocks_copied,
            extents_completed: None,
            unconfirmed_io: false,
        }
    }
}

#[cold]
fn work_failure(operation: CopyOperation, work: WorkItem, error: Error) -> Error {
    crate::failure::execution(
        "threaded",
        operation,
        Some((work.offset(), work.length() as u64)),
        CopyProgress {
            unconfirmed_io: true,
            ..CopyProgress::default()
        },
        error,
    )
}

pub(crate) fn execute<S, D, P>(
    source: &S,
    destination: &D,
    worker_count: usize,
    queue_capacity: usize,
    pool: &BufferPool,
    producer: P,
) -> Result<WorkerStats>
where
    S: VirtualDisk + ?Sized,
    D: VirtualDisk + ?Sized,
    P: FnOnce(&SyncSender<WorkItem>) -> Result<()>,
{
    execute_controlled(
        source,
        destination,
        worker_count,
        queue_capacity,
        pool,
        &(),
        |sender, _| producer(sender),
    )
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn execute_controlled<S, D, P, C>(
    source: &S,
    destination: &D,
    worker_count: usize,
    queue_capacity: usize,
    pool: &BufferPool,
    control: &C,
    producer: P,
) -> Result<WorkerStats>
where
    S: VirtualDisk + ?Sized,
    D: VirtualDisk + ?Sized,
    C: crate::control::Checkpoint,
    P: FnOnce(&SyncSender<WorkItem>, &dyn Fn() -> Result<()>) -> Result<()>,
{
    let (sender, receiver) = sync_channel::<WorkItem>(queue_capacity);

    let receiver = Arc::new(Mutex::new(receiver));

    let results = Mutex::new(Vec::<WorkerStats>::with_capacity(worker_count));
    let failure = Failure::default();
    let observed = Mutex::new(WorkerStats::default());
    let cancellation = control.cancellation();

    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            let receiver = Arc::clone(&receiver);

            let results = &results;
            let failure = &failure;
            let observed = &observed;

            scope.spawn(move || {
                let stats = run_worker_controlled::<_, _, C>(
                    source,
                    destination,
                    &receiver,
                    pool,
                    failure,
                    cancellation,
                    observed,
                );
                results
                    .lock()
                    .unwrap_or_else(|p| p.into_inner())
                    .push(stats);
            });
        }

        // Only workers retain receivers. Their exit disconnects the channel and
        // wakes a producer blocked in send, even when the queue is full.
        drop(receiver);

        let checkpoint = || {
            if C::ENABLED {
                let stats = *observed.lock().unwrap_or_else(|p| p.into_inner());
                control.check(stats.progress())
            } else {
                Ok(())
            }
        };
        if let Err(error) = producer(&sender, &checkpoint) {
            failure.record(error);
        }

        // Wake idle receivers on success or producer error. Scope exit joins all
        // workers before any error or buffer ownership returns to the caller.
        drop(sender);
    });

    let results = results
        .into_inner()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let mut total = WorkerStats::default();

    for stats in results {
        total.bytes_read += stats.bytes_read;

        total.bytes_written += stats.bytes_written;

        total.bytes_zeroed += stats.bytes_zeroed;

        total.bytes_discarded += stats.bytes_discarded;

        total.blocks_copied += stats.blocks_copied;
    }

    let mut progress = total.progress();
    progress.unconfirmed_io = failure.unconfirmed_io.load(Ordering::Relaxed);
    if let Some(error) = failure
        .first
        .into_inner()
        .unwrap_or_else(|p| p.into_inner())
    {
        return Err(crate::failure::worker_total(error, progress));
    }
    Ok(total)
}

#[allow(clippy::too_many_arguments)]
fn run_worker_controlled<S, D, C: crate::control::Checkpoint>(
    source: &S,
    destination: &D,
    receiver: &Arc<Mutex<Receiver<WorkItem>>>,
    pool: &BufferPool,
    failure: &Failure,
    cancellation: &dyn crate::Cancellation,
    observed: &Mutex<WorkerStats>,
) -> WorkerStats
where
    S: VirtualDisk + ?Sized,
    D: VirtualDisk + ?Sized,
{
    let mut stats = WorkerStats::default();

    loop {
        let work = {
            let receiver = receiver
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            if failure.is_stopped() {
                break;
            }
            receiver.recv()
        };

        let work = match work {
            Ok(work) => work,
            Err(_) => break,
        };

        if failure.is_stopped() {
            break;
        }
        if C::ENABLED && cancellation.is_cancelled() {
            failure.record(Error::Cancelled);
            break;
        }
        let before = stats;
        let result = process_work_controlled::<S, D, C>(
            source,
            destination,
            work,
            pool,
            &mut stats,
            cancellation,
        );
        if C::ENABLED {
            let mut sum = observed.lock().unwrap_or_else(|p| p.into_inner());
            sum.bytes_read += stats.bytes_read - before.bytes_read;
            sum.bytes_written += stats.bytes_written - before.bytes_written;
            sum.bytes_zeroed += stats.bytes_zeroed - before.bytes_zeroed;
            sum.bytes_discarded += stats.bytes_discarded - before.bytes_discarded;
            sum.blocks_copied += stats.blocks_copied - before.blocks_copied;
        }
        if let Err(error) = result {
            failure.record(error);
            break;
        }
    }

    stats
}

fn process_work_controlled<S, D, C: crate::control::Checkpoint>(
    source: &S,
    destination: &D,
    work: WorkItem,
    pool: &BufferPool,
    stats: &mut WorkerStats,
    cancellation: &dyn crate::Cancellation,
) -> Result<()>
where
    S: VirtualDisk + ?Sized,
    D: VirtualDisk + ?Sized,
{
    let kind = match work.kind() {
        WorkKind::Copy => ExtentKind::Data,
        WorkKind::Zero => ExtentKind::Zero,
        WorkKind::Discard => ExtentKind::Hole,
    };
    match policy::select(kind, || destination.capabilities()) {
        Operation::Copy => {
            let mut buffer = pool.acquire();

            let buffer = &mut buffer.as_mut_slice()[..work.length()];

            source
                .read_exact_at(work.offset(), buffer)
                .map_err(|e| work_failure(CopyOperation::Read, work, e))?;
            stats.bytes_read += work.length() as u64;
            if C::ENABLED && cancellation.is_cancelled() {
                return Err(Error::Cancelled);
            }

            destination
                .write_all_at(work.offset(), buffer)
                .map_err(|e| work_failure(CopyOperation::Write, work, e))?;

            let length = work.length() as u64;

            stats.bytes_written += length;
            stats.blocks_copied += 1;
        }

        Operation::Zero => {
            destination
                .write_zero_at(work.offset(), work.length() as u64)
                .map_err(|e| work_failure(CopyOperation::WriteZero, work, e))?;
            stats.bytes_zeroed += work.length() as u64;
        }
        Operation::Discard => {
            destination
                .discard(work.offset(), work.length() as u64)
                .map_err(|e| work_failure(CopyOperation::Discard, work, e))?;
            stats.bytes_discarded += work.length() as u64;
        }
        Operation::WriteZero => write_zero_fallback(destination, work, pool, stats)?,
    }
    Ok(())
}

fn write_zero_fallback<D>(
    destination: &D,
    work: WorkItem,
    pool: &BufferPool,
    stats: &mut WorkerStats,
) -> Result<()>
where
    D: VirtualDisk + ?Sized,
{
    let mut buffer = pool.acquire();

    let buffer = &mut buffer.as_mut_slice()[..work.length()];

    buffer.fill(0);

    destination
        .write_all_at(work.offset(), buffer)
        .map_err(|e| work_failure(CopyOperation::Write, work, e))?;

    stats.bytes_written += work.length() as u64;

    stats.blocks_copied += 1;

    Ok(())
}
