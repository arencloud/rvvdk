use std::os::fd::BorrowedFd;

use rvvdk_core::{BufferPool, CopyOperation, CopyProgress, Error, Result};

use super::validation::{validate_copy_configuration, validate_file_range};
use super::{CompletedOperation, IoUringEngine, IoUringFile, IoUringOperationKind};

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
    pub(crate) fn progress(&self) -> CopyProgress {
        CopyProgress {
            bytes_read: self.bytes_read,
            bytes_written: self.bytes_written,
            blocks_completed: self.blocks_completed,
            extents_completed: Some(0),
            ..CopyProgress::default()
        }
    }

    #[cold]
    fn failure(&self, operation: CopyOperation, range: Option<(u64, u64)>, error: Error) -> Error {
        if error.copy_failure().is_some() {
            return error;
        }
        let mut progress = self.progress();
        progress.unconfirmed_io = true;
        crate::failure::execution("io_uring", operation, range, progress, error)
    }

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
    source_fd: IoUringFile,
    destination_fd: IoUringFile,
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

/// Copy an explicit file range. Configuration and range validation run before
/// allocation or I/O, even for empty ranges. This function does not flush.
pub fn copy_file_range(
    source_fd: BorrowedFd<'_>,
    destination_fd: BorrowedFd<'_>,
    offset: u64,
    length: u64,
    block_size: usize,
    queue_depth: u32,
    alignment: usize,
) -> Result<IoUringCopyStats> {
    let options =
        crate::IoUringExecutionOptions::new(queue_depth).ok_or(Error::InvalidIoUringQueueDepth)?;

    copy_file_range_with_options(
        source_fd,
        destination_fd,
        offset,
        length,
        block_size,
        alignment,
        options,
    )
}

/// Like [`copy_file_range`], with a validated queue depth and read window.
/// Blocks must fit a u32 SQE length; alignment must describe a valid allocation.
/// Offsets and the exclusive range end must fit the nonnegative i64 domain.
pub fn copy_file_range_with_options(
    source_fd: BorrowedFd<'_>,
    destination_fd: BorrowedFd<'_>,
    offset: u64,
    length: u64,
    block_size: usize,
    alignment: usize,
    options: crate::IoUringExecutionOptions,
) -> Result<IoUringCopyStats> {
    validate_copy_configuration(block_size, alignment)?;
    validate_file_range(offset, length)?;

    if length == 0 {
        return Ok(IoUringCopyStats::default());
    }

    crate::preflight::files(source_fd, destination_fd, offset, length)?;
    copy_file_range_preflighted(
        source_fd,
        destination_fd,
        offset,
        length,
        block_size,
        alignment,
        options,
    )
}

// The caller validated configuration and the entire endpoint range. Extent
// executors use this to avoid repeating fstat/fcntl for every Data extent.
pub(super) fn copy_file_range_preflighted(
    source_fd: BorrowedFd<'_>,
    destination_fd: BorrowedFd<'_>,
    offset: u64,
    length: u64,
    block_size: usize,
    alignment: usize,
    options: crate::IoUringExecutionOptions,
) -> Result<IoUringCopyStats> {
    let queue_depth = options.queue_depth();

    let setup = |operation, error| {
        crate::failure::execution(
            "io_uring",
            operation,
            Some((offset, length)),
            CopyProgress {
                extents_completed: Some(0),
                ..CopyProgress::default()
            },
            error,
        )
    };
    let pool = BufferPool::new(queue_depth as usize, block_size, alignment)
        .map_err(|e| setup(CopyOperation::Allocate, e))?;

    let mut engine =
        IoUringEngine::new(queue_depth).map_err(|e| setup(CopyOperation::NativeSetup, e))?;

    let context = CopyContext {
        source_fd: IoUringFile::new(source_fd).map_err(|e| setup(CopyOperation::NativeSetup, e))?,
        destination_fd: IoUringFile::new(destination_fd)
            .map_err(|e| setup(CopyOperation::NativeSetup, e))?,
        start_offset: offset,
        length,
        block_size,
        queue_depth,
        read_window: options.read_window(),
    };

    let result = copy_with_engine(&mut engine, &pool, &context);
    let cleanup = engine.shutdown();
    match (result, cleanup) {
        (Err(original), Err(Error::IoUringShutdownUnconfirmed { operations })) => {
            Err(Error::IoUringCleanup {
                original: Box::new(original),
                operations,
            })
        }
        (Err(error), _) => Err(error),
        (Ok(stats), Err(error)) => {
            Err(stats.failure(CopyOperation::NativeShutdown, Some((offset, length)), error))
        }
        (Ok(stats), Ok(())) => Ok(stats),
    }
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

    let result = (|| {
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
                    let range = Some((completed.offset(), completed.length() as u64));
                    process_read_completion(engine, context, &mut state, &mut stats, completed)
                        .map_err(|e| stats.failure(CopyOperation::Read, range, e))?;
                }

                IoUringOperationKind::Write => {
                    let range = Some((completed.offset(), completed.length() as u64));
                    process_write_completion(&mut state, &mut stats, completed)
                        .map_err(|e| stats.failure(CopyOperation::Write, range, e))?;

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

        Ok(())
    })();
    result.map_err(|e| {
        stats.failure(
            CopyOperation::NativeCompletion,
            Some((context.start_offset, context.length)),
            e,
        )
    })?;
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

    stats.bytes_read = stats
        .bytes_read
        .checked_add(actual as u64)
        .ok_or(Error::RangeOverflow {
            offset: stats.bytes_read,
            length: actual as u64,
        })?;

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

    /*
     * Transfer ownership of the exact same BufferGuard from the
     * completed source read into the destination write.
     *
     * No userspace data copy occurs here.
     */
    let buffer = completed.into_buffer();

    engine
        .submit_owned_write(&context.destination_fd, offset, actual, buffer)
        .map_err(|e| stats.failure(CopyOperation::Write, Some((offset, actual as u64)), e))?;

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

    stats.bytes_written =
        stats
            .bytes_written
            .checked_add(actual as u64)
            .ok_or(Error::RangeOverflow {
                offset: stats.bytes_written,
                length: actual as u64,
            })?;

    if actual != expected {
        return Err(Error::ShortWrite {
            offset,
            expected,
            actual,
        });
    }

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
    source_fd: &IoUringFile,
    offset: u64,
    length: usize,
    state: &mut PipelineState,
    stats: &mut IoUringCopyStats,
) -> Result<()> {
    let buffer = pool.acquire();

    engine
        .submit_owned_read(source_fd, offset, length, buffer)
        .map_err(|e| stats.failure(CopyOperation::Read, Some((offset, length as u64)), e))?;

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
            &context.source_fd,
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
    fn pipeline_error_preserves_confirmed_blocks_and_short_reads() {
        use std::os::fd::AsFd;
        use std::os::unix::fs::FileExt;
        for source_size in [1024, 4096] {
            let path = std::env::temp_dir().join(format!(
                "rvvdk-r15-native-{}-{source_size}",
                std::process::id()
            ));
            std::fs::write(&path, vec![0x5a; source_size]).unwrap();
            let source = std::fs::File::open(&path).unwrap();
            std::fs::remove_file(&path).unwrap();
            std::fs::write(&path, [0xa5; 8192]).unwrap();
            let destination = std::fs::File::options()
                .read(true)
                .write(true)
                .open(&path)
                .unwrap();
            std::fs::remove_file(path).unwrap();
            let pool = BufferPool::new(1, 4096, 4096).unwrap();
            let mut engine = IoUringEngine::new(1).unwrap();
            let context = CopyContext {
                source_fd: IoUringFile::new(source.as_fd()).unwrap(),
                destination_fd: IoUringFile::new(destination.as_fd()).unwrap(),
                start_offset: 0,
                length: 8192,
                block_size: 4096,
                queue_depth: 1,
                read_window: 1,
            };
            // Deliberately bypass public capacity preflight to exercise a runtime EOF.
            let error = copy_with_engine(&mut engine, &pool, &context).unwrap_err();
            engine.shutdown().unwrap();
            assert_eq!(pool.available(), 1);
            let f = error.copy_failure().unwrap();
            assert_eq!(f.operation, CopyOperation::Read);
            assert_eq!(f.progress.bytes_read, source_size as u64);
            assert_eq!(
                f.progress.bytes_written,
                if source_size == 4096 { 4096 } else { 0 }
            );
            assert_eq!(f.progress.blocks_completed, u64::from(source_size == 4096));
            assert!(f.progress.unconfirmed_io);
            assert!(matches!(f.cause, Error::UnexpectedEof { .. }));
            let mut tail = [0; 4096];
            destination.read_exact_at(&mut tail, 4096).unwrap();
            assert_eq!(tail, [0xa5; 4096]);
        }
    }

    #[test]
    fn short_write_cqe_counts_confirmed_bytes_without_counting_a_complete_block() {
        use super::super::operation::InFlightOperation;
        use std::os::fd::AsFd;
        let file = std::fs::File::open("/dev/null").unwrap();
        let file = IoUringFile::new(file.as_fd()).unwrap();
        let pool = BufferPool::new(1, 4096, 4096).unwrap();
        let completed = InFlightOperation::new(
            file,
            1,
            IoUringOperationKind::Write,
            4096,
            4096,
            pool.acquire(),
        )
        .complete(256);
        let mut state = PipelineState {
            writes_in_flight: 1,
            ..PipelineState::default()
        };
        let mut stats = IoUringCopyStats {
            bytes_written: 4096,
            blocks_completed: 1,
            ..IoUringCopyStats::default()
        };
        assert!(matches!(
            process_write_completion(&mut state, &mut stats, completed),
            Err(Error::ShortWrite { actual: 256, .. })
        ));
        assert_eq!(stats.bytes_written, 4352);
        assert_eq!(stats.blocks_completed, 1);
        assert_eq!(pool.available(), 1);
    }

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
