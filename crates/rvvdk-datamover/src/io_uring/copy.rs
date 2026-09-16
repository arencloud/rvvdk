use std::os::fd::RawFd;

use rvvdk_core::{BufferPool, Error, Result};

use super::{CompletedOperation, IoUringEngine, IoUringOperationKind};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IoUringCopyStats {
    bytes_read: u64,
    bytes_written: u64,
    blocks_completed: u64,
    peak_in_flight: usize,
    peak_reads_in_flight: usize,
    peak_writes_in_flight: usize,
    mixed_in_flight_observed: bool,
}

impl IoUringCopyStats {
    pub const fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    pub const fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    pub const fn blocks_completed(&self) -> u64 {
        self.blocks_completed
    }

    pub const fn peak_in_flight(&self) -> usize {
        self.peak_in_flight
    }

    pub const fn peak_reads_in_flight(&self) -> usize {
        self.peak_reads_in_flight
    }

    pub const fn peak_writes_in_flight(&self) -> usize {
        self.peak_writes_in_flight
    }

    pub const fn mixed_in_flight_observed(&self) -> bool {
        self.mixed_in_flight_observed
    }
}

struct CopyContext {
    source_fd: RawFd,
    destination_fd: RawFd,
    start_offset: u64,
    length: u64,
    block_size: usize,
    queue_depth: u32,
    read_window: usize,
}

#[derive(Debug, Default)]
struct PipelineState {
    reads_in_flight: usize,
    writes_in_flight: usize,
}

impl PipelineState {
    fn total_in_flight(&self) -> usize {
        self.reads_in_flight + self.writes_in_flight
    }

    fn read_submitted(&mut self) {
        self.reads_in_flight += 1;
    }

    fn read_completed(&mut self) {
        debug_assert!(self.reads_in_flight > 0);

        self.reads_in_flight -= 1;
    }

    fn write_submitted(&mut self) {
        self.writes_in_flight += 1;
    }

    fn write_completed(&mut self) {
        debug_assert!(self.writes_in_flight > 0);

        self.writes_in_flight -= 1;
    }
}

fn update_peaks(stats: &mut IoUringCopyStats, state: &PipelineState) {
    stats.peak_in_flight = stats.peak_in_flight.max(state.total_in_flight());

    stats.peak_reads_in_flight = stats.peak_reads_in_flight.max(state.reads_in_flight);

    stats.peak_writes_in_flight = stats.peak_writes_in_flight.max(state.writes_in_flight);
    if state.reads_in_flight > 0 && state.writes_in_flight > 0 {
        stats.mixed_in_flight_observed = true;
    }
}

pub fn copy_file_range(
    source_fd: RawFd,
    destination_fd: RawFd,
    offset: u64,
    length: u64,
    block_size: usize,
    queue_depth: u32,
    alignment: usize,
) -> Result<IoUringCopyStats> {
    if block_size == 0 {
        return Err(Error::InvalidAlignment {
            value: 0,
            alignment: 1,
        });
    }

    if queue_depth == 0 {
        return Err(Error::InvalidIoUringQueueDepth);
    }

    if length == 0 {
        return Ok(IoUringCopyStats::default());
    }

    let pool = BufferPool::new(queue_depth as usize, block_size, alignment)?;

    let mut engine = IoUringEngine::new(queue_depth)?;

    let queue_depth_usize = queue_depth as usize;

    let read_window = queue_depth_usize.div_ceil(2).max(1);

    let context = CopyContext {
        source_fd,
        destination_fd,
        start_offset: offset,
        length,
        block_size,
        queue_depth,
        read_window,
    };

    copy_with_engine(&mut engine, &pool, &context)
}

fn copy_with_engine(
    engine: &mut IoUringEngine,
    pool: &BufferPool,
    context: &CopyContext,
) -> Result<IoUringCopyStats> {
    let end = context
        .start_offset
        .checked_add(context.length)
        .ok_or(Error::RangeOverflow {
            offset: context.start_offset,
            length: context.length,
        })?;

    let mut next_offset = context.start_offset;

    let mut stats = IoUringCopyStats::default();

    /*
     * Fill the initial io_uring queue with reads.
     *
     * Each submitted read owns one BufferGuard from the pool.
     */
    let mut state = PipelineState::default();

    refill_reads(
        engine,
        pool,
        context,
        &mut state,
        &mut stats,
        &mut next_offset,
        end,
    )?;

    engine.submit()?;

    while state.total_in_flight() > 0 {
        let completed = engine.wait_owned_completion()?;

        match completed.kind() {
            IoUringOperationKind::Read => {
                process_read_completion(engine, context, &mut state, &mut stats, completed)?;
            }

            IoUringOperationKind::Write => {
                process_write_completion(&mut state, &mut stats, completed)?;

                /*
                 * The completed write has now dropped its BufferGuard,
                 * so one or more buffers may be available for new reads.
                 */
                refill_reads(
                    engine,
                    pool,
                    context,
                    &mut state,
                    &mut stats,
                    &mut next_offset,
                    end,
                )?;
            }
        }
        debug_assert_eq!(state.total_in_flight(), engine.in_flight(),);

        engine.submit()?;
    }

    Ok(stats)
}

fn process_read_completion(
    engine: &mut IoUringEngine,
    context: &CopyContext,
    state: &mut PipelineState,
    stats: &mut IoUringCopyStats,
    completed: CompletedOperation,
) -> Result<()> {
    state.read_completed();
    let offset = completed.offset();

    let expected = completed.length();

    let actual = completed.bytes_transferred();

    if actual != expected {
        let remaining = expected.checked_sub(actual).ok_or(Error::CorruptMetadata(
            "io_uring read completion exceeded requested length".into(),
        ))?;

        let eof_offset = offset
            .checked_add(actual as u64)
            .ok_or(Error::RangeOverflow {
                offset,
                length: actual as u64,
            })?;

        return Err(Error::UnexpectedEof {
            offset: eof_offset,
            remaining,
        });
    }

    stats.bytes_read = stats
        .bytes_read
        .checked_add(actual as u64)
        .ok_or(Error::RangeOverflow {
            offset: stats.bytes_read,
            length: actual as u64,
        })?;

    /*
     * Transfer ownership of the exact same BufferGuard from the
     * completed source read into the destination write.
     *
     * No userspace data copy occurs here.
     */
    let buffer = completed.into_buffer();

    engine.submit_owned_write(context.destination_fd, offset, actual, buffer)?;

    state.write_submitted();

    update_peaks(stats, state);

    Ok(())
}

fn process_write_completion(
    state: &mut PipelineState,
    stats: &mut IoUringCopyStats,
    completed: CompletedOperation,
) -> Result<()> {
    state.write_completed();
    let offset = completed.offset();

    let expected = completed.length();

    let actual = completed.bytes_transferred();

    if actual != expected {
        return Err(Error::ShortWrite {
            offset,
            expected,
            actual,
        });
    }

    stats.bytes_written =
        stats
            .bytes_written
            .checked_add(actual as u64)
            .ok_or(Error::RangeOverflow {
                offset: stats.bytes_written,
                length: actual as u64,
            })?;

    stats.blocks_completed = stats
        .blocks_completed
        .checked_add(1)
        .ok_or(Error::RangeOverflow {
            offset: stats.blocks_completed,
            length: 1,
        })?;

    /*
     * `completed` owns the BufferGuard.
     *
     * It is dropped when this function returns, returning the buffer
     * to BufferPool.
     */
    Ok(())
}

fn submit_read(
    engine: &mut IoUringEngine,
    pool: &BufferPool,
    source_fd: RawFd,
    offset: u64,
    length: usize,
    state: &mut PipelineState,
    stats: &mut IoUringCopyStats,
) -> Result<()> {
    let buffer = pool.acquire();

    engine.submit_owned_read(source_fd, offset, length, buffer)?;

    state.read_submitted();

    update_peaks(stats, state);

    Ok(())
}

fn request_length(offset: u64, end: u64, block_size: usize) -> Result<usize> {
    if offset >= end {
        return Ok(0);
    }

    let remaining = end - offset;

    let length_u64 = remaining.min(block_size as u64);

    usize::try_from(length_u64).map_err(|_| Error::RangeOverflow {
        offset,
        length: length_u64,
    })
}

fn advance_offset(offset: u64, length: usize) -> Result<u64> {
    offset
        .checked_add(length as u64)
        .ok_or(Error::RangeOverflow {
            offset,
            length: length as u64,
        })
}

fn refill_reads(
    engine: &mut IoUringEngine,
    pool: &BufferPool,
    context: &CopyContext,
    state: &mut PipelineState,
    stats: &mut IoUringCopyStats,
    next_offset: &mut u64,
    end: u64,
) -> Result<()> {
    while *next_offset < end
        && state.total_in_flight() < context.queue_depth as usize
        && state.reads_in_flight < context.read_window
        && pool.available() > 0
    {
        let length = request_length(*next_offset, end, context.block_size)?;

        submit_read(
            engine,
            pool,
            context.source_fd,
            *next_offset,
            length,
            state,
            stats,
        )?;

        *next_offset = advance_offset(*next_offset, length)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_length_returns_full_block() {
        let length = request_length(0, 1024 * 1024, 64 * 1024).unwrap();

        assert_eq!(length, 64 * 1024,);
    }

    #[test]
    fn request_length_returns_partial_tail() {
        let length = request_length(1024, 1024 + 123, 4096).unwrap();

        assert_eq!(length, 123,);
    }

    #[test]
    fn request_length_returns_zero_at_end() {
        let length = request_length(4096, 4096, 4096).unwrap();

        assert_eq!(length, 0,);
    }

    #[test]
    fn advances_offset() {
        let offset = advance_offset(4096, 8192).unwrap();

        assert_eq!(offset, 12288,);
    }
}
