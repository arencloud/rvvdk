use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, SyncSender, sync_channel},
};

use rvvdk_core::{BufferPool, Capabilities, Error, Result, VirtualDisk};

use crate::work::{WorkItem, WorkKind};

#[cfg(test)]
mod tests;

#[derive(Default)]
struct Failure {
    stopped: AtomicBool,
    first: Mutex<Option<Error>>,
}

impl Failure {
    fn record(&self, error: Error) {
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
    let (sender, receiver) = sync_channel::<WorkItem>(queue_capacity);

    let receiver = Arc::new(Mutex::new(receiver));

    let results = Mutex::new(Vec::<WorkerStats>::with_capacity(worker_count));
    let failure = Failure::default();

    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            let receiver = Arc::clone(&receiver);

            let results = &results;
            let failure = &failure;

            scope.spawn(
                move || match run_worker(source, destination, &receiver, pool, failure) {
                    Ok(stats) => results
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                        .push(stats),
                    Err(error) => failure.record(error),
                },
            );
        }

        // Only workers retain receivers. Their exit disconnects the channel and
        // wakes a producer blocked in send, even when the queue is full.
        drop(receiver);

        if let Err(error) = producer(&sender) {
            failure.record(error);
        }

        // Wake idle receivers on success or producer error. Scope exit joins all
        // workers before any error or buffer ownership returns to the caller.
        drop(sender);
    });

    if let Some(error) = failure
        .first
        .into_inner()
        .unwrap_or_else(|p| p.into_inner())
    {
        return Err(error);
    }

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

    Ok(total)
}

fn run_worker<S, D>(
    source: &S,
    destination: &D,
    receiver: &Arc<Mutex<Receiver<WorkItem>>>,
    pool: &BufferPool,
    failure: &Failure,
) -> Result<WorkerStats>
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
        process_work(source, destination, work, pool, &mut stats)?;
    }

    Ok(stats)
}

fn process_work<S, D>(
    source: &S,
    destination: &D,
    work: WorkItem,
    pool: &BufferPool,
    stats: &mut WorkerStats,
) -> Result<()>
where
    S: VirtualDisk + ?Sized,
    D: VirtualDisk + ?Sized,
{
    match work.kind() {
        WorkKind::Copy => {
            let mut buffer = pool.acquire();

            let buffer = &mut buffer.as_mut_slice()[..work.length()];

            source.read_exact_at(work.offset(), buffer)?;

            destination.write_all_at(work.offset(), buffer)?;

            let length = work.length() as u64;

            stats.bytes_read += length;
            stats.bytes_written += length;
            stats.blocks_copied += 1;
        }

        WorkKind::Zero => {
            zero_work(destination, work, pool, stats)?;
        }

        WorkKind::Discard => {
            discard_work(destination, work, pool, stats)?;
        }
    }

    Ok(())
}

fn zero_work<D>(
    destination: &D,
    work: WorkItem,
    pool: &BufferPool,
    stats: &mut WorkerStats,
) -> Result<()>
where
    D: VirtualDisk + ?Sized,
{
    let length = work.length() as u64;

    if destination
        .capabilities()
        .contains(Capabilities::WRITE_ZERO)
    {
        destination.write_zero_at(work.offset(), length)?;

        stats.bytes_zeroed += length;

        return Ok(());
    }

    write_zero_fallback(destination, work, pool, stats)
}

fn discard_work<D>(
    destination: &D,
    work: WorkItem,
    pool: &BufferPool,
    stats: &mut WorkerStats,
) -> Result<()>
where
    D: VirtualDisk + ?Sized,
{
    let capabilities = destination.capabilities();

    let length = work.length() as u64;

    if capabilities.contains(Capabilities::DISCARD) {
        destination.discard(work.offset(), length)?;

        stats.bytes_discarded += length;

        return Ok(());
    }

    if capabilities.contains(Capabilities::WRITE_ZERO) {
        destination.write_zero_at(work.offset(), length)?;

        stats.bytes_zeroed += length;

        return Ok(());
    }

    write_zero_fallback(destination, work, pool, stats)
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

    destination.write_all_at(work.offset(), buffer)?;

    stats.bytes_written += work.length() as u64;

    stats.blocks_copied += 1;

    Ok(())
}
