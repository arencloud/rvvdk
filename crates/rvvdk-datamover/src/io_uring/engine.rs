use std::collections::HashMap;
use std::os::fd::RawFd;

use io_uring::{IoUring, opcode, types};

use rvvdk_core::{BufferGuard, Error, Result};

use super::operation::{CompletedOperation, InFlightOperation, IoUringOperationKind};

pub struct IoUringEngine {
    ring: Option<IoUring>,
    queue_depth: u32,
    next_user_data: u64,

    // Ownership-safe asynchronous operations.
    //
    // Each entry owns the BufferGuard referenced by its SQE.
    in_flight: HashMap<u64, InFlightOperation>,

    // M17B compatibility path.
    //
    // Borrowed read_at()/write_at() never permit more than one
    // outstanding borrowed operation.
    borrowed_in_flight: bool,
}

impl IoUringEngine {
    pub fn new(queue_depth: u32) -> Result<Self> {
        if queue_depth == 0 {
            return Err(Error::InvalidIoUringQueueDepth);
        }

        let ring = IoUring::new(queue_depth).map_err(Error::Io)?;

        Ok(Self {
            ring: Some(ring),
            queue_depth,
            next_user_data: 1,
            in_flight: HashMap::with_capacity(queue_depth as usize),
            borrowed_in_flight: false,
        })
    }

    pub const fn queue_depth(&self) -> u32 {
        self.queue_depth
    }

    pub fn in_flight(&self) -> usize {
        self.in_flight.len() + usize::from(self.borrowed_in_flight)
    }

    pub fn read_at(&mut self, fd: RawFd, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        let user_data = self.submit_borrowed_read(fd, offset, buffer)?;

        self.submit()?;

        let completion = self.wait_borrowed_completion()?;

        if completion.user_data != user_data {
            return Err(Error::IoUringUnexpectedCompletion {
                expected: user_data,
                actual: completion.user_data,
            });
        }

        completion_result_to_bytes(completion.result)
    }

    pub fn write_at(&mut self, fd: RawFd, offset: u64, buffer: &[u8]) -> Result<usize> {
        let user_data = self.submit_borrowed_write(fd, offset, buffer)?;

        self.submit()?;

        let completion = self.wait_borrowed_completion()?;

        if completion.user_data != user_data {
            return Err(Error::IoUringUnexpectedCompletion {
                expected: user_data,
                actual: completion.user_data,
            });
        }

        completion_result_to_bytes(completion.result)
    }

    pub fn submit_owned_read(
        &mut self,
        fd: RawFd,
        offset: u64,
        length: usize,
        mut buffer: BufferGuard,
    ) -> Result<u64> {
        if self.borrowed_in_flight {
            return Err(Error::IoUringOperationModeConflict);
        }

        let available = buffer.as_slice().len();

        if length > available {
            return Err(Error::BufferTooSmall {
                requested: length,
                available,
            });
        }

        let length_u32 = u32::try_from(length).map_err(|_| Error::RangeOverflow {
            offset,
            length: length as u64,
        })?;

        let user_data = self.next_user_data();

        let pointer = buffer.as_mut_slice().as_mut_ptr();

        let entry = opcode::Read::new(types::Fd(fd), pointer, length_u32)
            .offset(offset)
            .build()
            .user_data(user_data);

        // SAFETY:
        //
        // The SQE contains a pointer into `buffer`.
        //
        // After the SQE is successfully pushed, `buffer` is moved
        // into `InFlightOperation` and then into `self.in_flight`.
        //
        // The BufferGuard therefore remains alive and its allocation
        // cannot return to the BufferPool until the corresponding CQE
        // is consumed and the resulting CompletedOperation is dropped.
        let queue_depth = self.queue_depth;

        // SAFETY:
        //
        // The safety invariant depends on the operation being submitted.
        // See the operation-specific comment above this block.
        unsafe {
            self.ring_mut()?
                .submission()
                .push(&entry)
                .map_err(|_| Error::IoUringQueueFull { queue_depth })?;
        }

        let operation = InFlightOperation::new(
            user_data,
            IoUringOperationKind::Read,
            offset,
            length,
            buffer,
        );

        let previous = self.in_flight.insert(user_data, operation);

        debug_assert!(previous.is_none(), "io_uring user_data collision");

        Ok(user_data)
    }

    pub fn submit_owned_write(
        &mut self,
        fd: RawFd,
        offset: u64,
        length: usize,
        buffer: BufferGuard,
    ) -> Result<u64> {
        if self.borrowed_in_flight {
            return Err(Error::IoUringOperationModeConflict);
        }

        let available = buffer.as_slice().len();

        if length > available {
            return Err(Error::BufferTooSmall {
                requested: length,
                available,
            });
        }

        let length_u32 = u32::try_from(length).map_err(|_| Error::RangeOverflow {
            offset,
            length: length as u64,
        })?;

        let user_data = self.next_user_data();

        let pointer = buffer.as_slice().as_ptr();

        let entry = opcode::Write::new(types::Fd(fd), pointer, length_u32)
            .offset(offset)
            .build()
            .user_data(user_data);

        // SAFETY:
        //
        // The SQE contains a pointer into `buffer`.
        //
        // Once pushed, the BufferGuard is transferred into the
        // in-flight operation table and remains alive until the CQE
        // has been consumed.
        let queue_depth = self.queue_depth;

        // SAFETY:
        //
        // The safety invariant depends on the operation being submitted.
        // See the operation-specific comment above this block.
        unsafe {
            self.ring_mut()?
                .submission()
                .push(&entry)
                .map_err(|_| Error::IoUringQueueFull { queue_depth })?;
        }

        let operation = InFlightOperation::new(
            user_data,
            IoUringOperationKind::Write,
            offset,
            length,
            buffer,
        );

        let previous = self.in_flight.insert(user_data, operation);

        debug_assert!(previous.is_none(), "io_uring user_data collision");

        Ok(user_data)
    }

    pub fn submit(&mut self) -> Result<usize> {
        self.ring_mut()?.submit().map_err(Error::Io)
    }

    pub fn wait_owned_completion(&mut self) -> Result<CompletedOperation> {
        if self.in_flight.is_empty() {
            return Err(Error::IoUringNoInFlight);
        }

        self.ring_mut()?.submit_and_wait(1).map_err(Error::Io)?;

        let (user_data, result) = self.next_completion()?;

        let operation = self
            .in_flight
            .remove(&user_data)
            .ok_or(Error::IoUringUnknownCompletion { user_data })?;

        let bytes_transferred = completion_result_to_bytes(result)?;

        Ok(operation.complete(bytes_transferred))
    }

    pub fn drain(&mut self) -> Result<Vec<CompletedOperation>> {
        if self.borrowed_in_flight {
            return Err(Error::IoUringOperationModeConflict);
        }

        let mut completed = Vec::with_capacity(self.in_flight.len());

        while !self.in_flight.is_empty() {
            completed.push(self.wait_owned_completion()?);
        }

        Ok(completed)
    }

    fn submit_borrowed_read(&mut self, fd: RawFd, offset: u64, buffer: &mut [u8]) -> Result<u64> {
        if !self.in_flight.is_empty() || self.borrowed_in_flight {
            return Err(Error::IoUringOperationModeConflict);
        }

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
        // This is the synchronous borrowed compatibility path.
        // `read_at()` does not return until the corresponding CQE has
        // been consumed, so the caller's mutable buffer remains
        // borrowed for the complete kernel access interval.
        let queue_depth = self.queue_depth;

        // SAFETY:
        //
        // The safety invariant depends on the operation being submitted.
        // See the operation-specific comment above this block.
        unsafe {
            self.ring_mut()?
                .submission()
                .push(&entry)
                .map_err(|_| Error::IoUringQueueFull { queue_depth })?;
        }

        self.borrowed_in_flight = true;

        Ok(user_data)
    }

    fn submit_borrowed_write(&mut self, fd: RawFd, offset: u64, buffer: &[u8]) -> Result<u64> {
        if !self.in_flight.is_empty() || self.borrowed_in_flight {
            return Err(Error::IoUringOperationModeConflict);
        }

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
        // `write_at()` waits for the CQE before returning, keeping the
        // immutable borrow alive for the entire kernel access
        // interval.
        let queue_depth = self.queue_depth;

        // SAFETY:
        //
        // The safety invariant depends on the operation being submitted.
        // See the operation-specific comment above this block.
        unsafe {
            self.ring_mut()?
                .submission()
                .push(&entry)
                .map_err(|_| Error::IoUringQueueFull { queue_depth })?;
        }

        self.borrowed_in_flight = true;

        Ok(user_data)
    }

    fn wait_borrowed_completion(&mut self) -> Result<BorrowedCompletion> {
        if !self.borrowed_in_flight {
            return Err(Error::IoUringNoInFlight);
        }

        self.ring_mut()?.submit_and_wait(1).map_err(Error::Io)?;

        let (user_data, result) = self.next_completion()?;

        self.borrowed_in_flight = false;

        Ok(BorrowedCompletion { user_data, result })
    }

    fn next_completion(&mut self) -> Result<(u64, i32)> {
        let mut completion_queue = self.ring_mut()?.completion();

        completion_queue
            .next()
            .map(|entry| (entry.user_data(), entry.result()))
            .ok_or(Error::IoUringCompletionMissing)
    }

    fn next_user_data(&mut self) -> u64 {
        loop {
            let value = self.next_user_data;

            self.next_user_data = self.next_user_data.wrapping_add(1);

            if self.next_user_data == 0 {
                self.next_user_data = 1;
            }

            if value != 0 && !self.in_flight.contains_key(&value) {
                return value;
            }
        }
    }

    fn ring_mut(&mut self) -> Result<&mut IoUring> {
        self.ring.as_mut().ok_or(Error::IoUringEngineShutDown)
    }

    fn drain_for_drop(&mut self) {
        if self.borrowed_in_flight {
            /*
             * A borrowed operation is only created and completed
             * within read_at()/write_at(). Reaching Drop with one
             * outstanding indicates an interrupted/error path.
             *
             * Keep the ring alive while attempting to consume its CQE.
             */
            if let Some(ring) = self.ring.as_mut()
                && ring.submit_and_wait(1).is_ok()
            {
                let mut queue = ring.completion();

                if queue.next().is_some() {
                    self.borrowed_in_flight = false;
                }
            }
        }

        while !self.in_flight.is_empty() {
            let wait_result = match self.ring.as_mut() {
                Some(ring) => ring.submit_and_wait(1),

                None => break,
            };

            if wait_result.is_err() {
                break;
            }

            let user_data = match self.ring.as_mut() {
                Some(ring) => {
                    let mut queue = ring.completion();

                    queue.next().map(|entry| entry.user_data())
                }

                None => None,
            };

            let Some(user_data) = user_data else {
                break;
            };

            self.in_flight.remove(&user_data);
        }
    }
}

impl Drop for IoUringEngine {
    fn drop(&mut self) {
        self.drain_for_drop();

        /*
         * Drop order is explicit:
         *
         * 1. attempt to consume outstanding CQEs;
         * 2. destroy the io_uring instance;
         * 3. only then allow any remaining InFlightOperation values
         *    and their BufferGuards to be dropped.
         *
         * Keeping the ring in Option<IoUring> lets us force this
         * ordering rather than depending on struct field drop order.
         */
        drop(self.ring.take());

        self.in_flight.clear();
    }
}

struct BorrowedCompletion {
    user_data: u64,
    result: i32,
}

fn completion_result_to_bytes(result: i32) -> Result<usize> {
    if result < 0 {
        return Err(Error::Io(std::io::Error::from_raw_os_error(-result)));
    }

    usize::try_from(result)
        .map_err(|_| Error::CorruptMetadata("io_uring completion result does not fit usize".into()))
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
    fn owned_completion_without_submission_fails() {
        let mut engine = IoUringEngine::new(8).unwrap();

        let result = engine.wait_owned_completion();

        assert!(matches!(result, Err(Error::IoUringNoInFlight)));
    }
}
