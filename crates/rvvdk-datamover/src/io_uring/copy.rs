use std::os::fd::RawFd;

use rvvdk_core::{BufferPool, Error, Result};

use super::{CompletedOperation, IoUringEngine, IoUringOperationKind};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IoUringCopyStats {
    bytes_read: u64,
    bytes_written: u64,
    blocks_completed: u64,
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
}

struct CopyContext {
    source_fd: RawFd,
    destination_fd: RawFd,
    start_offset: u64,
    length: u64,
    block_size: usize,
    queue_depth: u32,
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

    let context = CopyContext {
        source_fd,
        destination_fd,
        start_offset: offset,
        length,
        block_size,
        queue_depth,
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
    while next_offset < end && engine.in_flight() < context.queue_depth as usize {
        let request_length = request_length(next_offset, end, context.block_size)?;

        submit_read(engine, pool, context.source_fd, next_offset, request_length)?;

        next_offset = advance_offset(next_offset, request_length)?;
    }

    engine.submit()?;

    /*
     * Every read completion becomes a write using the same
     * BufferGuard.
     *
     * Every write completion releases a buffer and allows another
     * source read to be submitted.
     */
    while engine.in_flight() > 0 {
        let completed = engine.wait_owned_completion()?;

        match completed.kind() {
            IoUringOperationKind::Read => {
                process_read_completion(engine, context, &mut stats, completed)?;
            }

            IoUringOperationKind::Write => {
                process_write_completion(&mut stats, completed)?;

                /*
                 * A write completion returns its BufferGuard to the
                 * pool when CompletedOperation is dropped.
                 *
                 * We can now use that buffer for another source read.
                 */
                if next_offset < end {
                    let request_length = request_length(next_offset, end, context.block_size)?;

                    submit_read(engine, pool, context.source_fd, next_offset, request_length)?;

                    next_offset = advance_offset(next_offset, request_length)?;
                }
            }
        }

        /*
         * Push any SQEs generated while processing this completion.
         */
        engine.submit()?;
    }

    Ok(stats)
}

fn process_read_completion(
    engine: &mut IoUringEngine,
    context: &CopyContext,
    stats: &mut IoUringCopyStats,
    completed: CompletedOperation,
) -> Result<()> {
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

    Ok(())
}

fn process_write_completion(
    stats: &mut IoUringCopyStats,
    completed: CompletedOperation,
) -> Result<()> {
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
) -> Result<()> {
    let buffer = pool.acquire();

    engine.submit_owned_read(source_fd, offset, length, buffer)?;

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
