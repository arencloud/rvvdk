use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use rvvdk_core::{BufferPool, Capabilities, Result, VirtualDisk};

use crate::work::{WorkItem, WorkKind};

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct WorkerStats {
    pub(crate) bytes_read: u64,
    pub(crate) bytes_written: u64,
    pub(crate) bytes_zeroed: u64,
    pub(crate) bytes_discarded: u64,
    pub(crate) blocks_copied: u64,
}

pub(crate) fn execute<S, D>(
    source: &S,
    destination: &D,
    work: Vec<WorkItem>,
    worker_count: usize,
    pool: &BufferPool,
) -> Result<WorkerStats>
where
    S: VirtualDisk,
    D: VirtualDisk,
{
    let queue = Arc::new(Mutex::new(VecDeque::from(work)));

    let results = Mutex::new(Vec::<Result<WorkerStats>>::new());

    std::thread::scope(|scope| {
        for _ in 0..worker_count {
            let queue = Arc::clone(&queue);

            let results = &results;

            scope.spawn(move || {
                let result = run_worker(source, destination, &queue, pool);

                results
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .push(result);
            });
        }
    });

    let results = results
        .into_inner()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let mut total = WorkerStats::default();

    for result in results {
        let stats = result?;

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
    queue: &Mutex<VecDeque<WorkItem>>,
    pool: &BufferPool,
) -> Result<WorkerStats>
where
    S: VirtualDisk,
    D: VirtualDisk,
{
    let mut stats = WorkerStats::default();

    loop {
        let work = {
            let mut queue = queue
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            queue.pop_front()
        };

        let Some(work) = work else {
            break;
        };

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
    S: VirtualDisk,
    D: VirtualDisk,
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
    D: VirtualDisk,
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
    D: VirtualDisk,
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
    D: VirtualDisk,
{
    let mut buffer = pool.acquire();

    let buffer = &mut buffer.as_mut_slice()[..work.length()];

    buffer.fill(0);

    destination.write_all_at(work.offset(), buffer)?;

    stats.bytes_written += work.length() as u64;

    stats.blocks_copied += 1;

    Ok(())
}
