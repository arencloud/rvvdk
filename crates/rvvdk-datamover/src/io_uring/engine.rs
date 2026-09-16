use std::os::fd::RawFd;

use io_uring::{IoUring, opcode, types};

use rvvdk_core::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IoUringCompletion {
    user_data: u64,
    result: i32,
}

impl IoUringCompletion {
    pub(crate) const fn user_data(&self) -> u64 {
        self.user_data
    }
    pub(crate) fn bytes_transferred(&self) -> Result<usize> {
        if self.result < 0 {
            return Err(Error::Io(std::io::Error::from_raw_os_error(-self.result)));
        }

        usize::try_from(self.result).map_err(|_| {
            Error::CorruptMetadata("io_uring completion result does not fit usize".into())
        })
    }
}

pub struct IoUringEngine {
    ring: IoUring,
    queue_depth: u32,
    next_user_data: u64,
    in_flight: usize,
}

impl IoUringEngine {
    pub fn new(queue_depth: u32) -> Result<Self> {
        if queue_depth == 0 {
            return Err(Error::InvalidIoUringQueueDepth);
        }

        let ring = IoUring::new(queue_depth).map_err(Error::Io)?;

        Ok(Self {
            ring,
            queue_depth,
            next_user_data: 1,
            in_flight: 0,
        })
    }

    pub const fn queue_depth(&self) -> u32 {
        self.queue_depth
    }

    pub const fn in_flight(&self) -> usize {
        self.in_flight
    }

    fn next_user_data(&mut self) -> u64 {
        let value = self.next_user_data;

        self.next_user_data = self.next_user_data.wrapping_add(1);

        if self.next_user_data == 0 {
            self.next_user_data = 1;
        }

        value
    }

    pub(crate) fn submit_read(&mut self, fd: RawFd, offset: u64, buffer: &mut [u8]) -> Result<u64> {
        let length = u32::try_from(buffer.len()).map_err(|_| Error::RangeOverflow {
            offset,
            length: buffer.len() as u64,
        })?;

        let user_data = self.next_user_data();

        let entry = opcode::Read::new(types::Fd(fd), buffer.as_mut_ptr(), length)
            .offset(offset)
            .build()
            .user_data(user_data);

        // SAFETY:
        //
        // The caller must keep `buffer` alive and must not access or move
        // its backing allocation until the corresponding completion has
        // been consumed.
        //
        // M17B uses this API only with buffers whose lifetime extends
        // through `wait_completion`.
        unsafe {
            self.ring
                .submission()
                .push(&entry)
                .map_err(|_| Error::IoUringQueueFull {
                    queue_depth: self.queue_depth,
                })?;
        }

        self.in_flight += 1;

        Ok(user_data)
    }

    pub(crate) fn submit_write(&mut self, fd: RawFd, offset: u64, buffer: &[u8]) -> Result<u64> {
        let length = u32::try_from(buffer.len()).map_err(|_| Error::RangeOverflow {
            offset,
            length: buffer.len() as u64,
        })?;

        let user_data = self.next_user_data();

        let entry = opcode::Write::new(types::Fd(fd), buffer.as_ptr(), length)
            .offset(offset)
            .build()
            .user_data(user_data);

        // SAFETY:
        //
        // The caller must keep `buffer` alive and immutable until the
        // corresponding completion has been consumed.
        unsafe {
            self.ring
                .submission()
                .push(&entry)
                .map_err(|_| Error::IoUringQueueFull {
                    queue_depth: self.queue_depth,
                })?;
        }

        self.in_flight += 1;

        Ok(user_data)
    }

    pub(crate) fn submit(&mut self) -> Result<usize> {
        self.ring.submit().map_err(Error::Io)
    }

    pub(crate) fn wait_completion(&mut self) -> Result<IoUringCompletion> {
        if self.in_flight == 0 {
            return Err(Error::IoUringNoInFlight);
        }

        self.ring.submit_and_wait(1).map_err(Error::Io)?;

        let completion = {
            let mut completion_queue = self.ring.completion();

            let entry = completion_queue
                .next()
                .ok_or(Error::IoUringCompletionMissing)?;

            IoUringCompletion {
                user_data: entry.user_data(),
                result: entry.result(),
            }
        };

        self.in_flight -= 1;

        Ok(completion)
    }

    pub fn read_at(&mut self, fd: RawFd, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        let user_data = self.submit_read(fd, offset, buffer)?;

        self.submit()?;

        let completion = self.wait_completion()?;

        if completion.user_data() != user_data {
            return Err(Error::IoUringUnexpectedCompletion {
                expected: user_data,
                actual: completion.user_data(),
            });
        }

        completion.bytes_transferred()
    }

    pub fn write_at(&mut self, fd: RawFd, offset: u64, buffer: &[u8]) -> Result<usize> {
        let user_data = self.submit_write(fd, offset, buffer)?;

        self.submit()?;

        let completion = self.wait_completion()?;

        if completion.user_data() != user_data {
            return Err(Error::IoUringUnexpectedCompletion {
                expected: user_data,
                actual: completion.user_data(),
            });
        }

        completion.bytes_transferred()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_engine() {
        let engine = IoUringEngine::new(8).unwrap();

        assert_eq!(engine.queue_depth(), 8,);

        assert_eq!(engine.in_flight(), 0,);
    }

    #[test]
    fn rejects_zero_queue_depth() {
        let result = IoUringEngine::new(0);

        assert!(matches!(result, Err(Error::InvalidIoUringQueueDepth)));
    }

    #[test]
    fn wait_without_submission_fails() {
        let mut engine = IoUringEngine::new(8).unwrap();

        let result = engine.wait_completion();

        assert!(matches!(result, Err(Error::IoUringNoInFlight)));
    }
}
