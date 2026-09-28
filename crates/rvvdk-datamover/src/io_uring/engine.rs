use std::collections::HashMap;
use std::os::fd::{AsFd, AsRawFd};

use io_uring::{IoUring, opcode, types};
use rvvdk_core::{BufferGuard, Error, Result};

use super::IoUringFile;
use super::operation::{CompletedOperation, InFlightOperation, IoUringOperationKind};

/// An owned-buffer engine. Every queued SQE retains both its buffer and file.
///
/// Call `shutdown` to observe cleanup errors. Drop performs the same cleanup, but
/// cannot report errors. Unconfirmed operations are permanently retained instead
/// of releasing memory that the kernel may still access.
pub struct IoUringEngine {
    ring: Option<IoUring>,
    queue_depth: u32,
    next_user_data: u64,
    in_flight: HashMap<u64, InFlightOperation>,
    accepting: bool,
    completion_lost: bool,
    quarantined: usize,
    #[cfg(test)]
    faults: faults::Faults,
}

impl IoUringEngine {
    pub fn new(queue_depth: u32) -> Result<Self> {
        if queue_depth == 0 {
            return Err(Error::InvalidIoUringQueueDepth);
        }
        Ok(Self {
            ring: Some(IoUring::new(queue_depth)?),
            queue_depth,
            next_user_data: 1,
            in_flight: HashMap::with_capacity(queue_depth as usize),
            accepting: true,
            completion_lost: false,
            quarantined: 0,
            #[cfg(test)]
            faults: faults::Faults::default(),
        })
    }

    pub const fn queue_depth(&self) -> u32 {
        self.queue_depth
    }

    pub fn in_flight(&self) -> usize {
        self.in_flight.len()
    }

    /// Operations retained permanently after cleanup could not prove completion.
    pub const fn quarantined_operations(&self) -> usize {
        self.quarantined
    }

    pub fn submit_owned_read(
        &mut self,
        file: &IoUringFile,
        offset: u64,
        length: usize,
        buffer: BufferGuard,
    ) -> Result<u64> {
        self.enqueue(file, offset, length, buffer, IoUringOperationKind::Read)
    }

    pub fn submit_owned_write(
        &mut self,
        file: &IoUringFile,
        offset: u64,
        length: usize,
        buffer: BufferGuard,
    ) -> Result<u64> {
        self.enqueue(file, offset, length, buffer, IoUringOperationKind::Write)
    }

    fn enqueue(
        &mut self,
        file: &IoUringFile,
        offset: u64,
        length: usize,
        mut buffer: BufferGuard,
        kind: IoUringOperationKind,
    ) -> Result<u64> {
        self.ensure_accepting()?;
        if length > buffer.as_slice().len() {
            return Err(Error::BufferTooSmall {
                requested: length,
                available: buffer.as_slice().len(),
            });
        }
        let length_u32 = u32::try_from(length).map_err(|_| Error::RangeOverflow {
            offset,
            length: length as u64,
        })?;
        if self.in_flight.len() >= self.queue_depth as usize {
            return Err(Error::IoUringQueueFull {
                queue_depth: self.queue_depth,
            });
        }
        let user_data = self.next_user_data();
        let fd = types::Fd(file.as_fd().as_raw_fd());
        let entry = match kind {
            IoUringOperationKind::Read => {
                opcode::Read::new(fd, buffer.as_mut_slice().as_mut_ptr(), length_u32)
                    .offset(offset)
                    .build()
            }
            IoUringOperationKind::Write => {
                opcode::Write::new(fd, buffer.as_slice().as_ptr(), length_u32)
                    .offset(offset)
                    .build()
            }
        }
        .user_data(user_data);

        // Own every resource BEFORE publishing its pointer in the SQ. A panic
        // or subsequent error cannot leave a published pointer without an owner.
        let operation =
            InFlightOperation::new(file.clone(), user_data, kind, offset, length, buffer);
        let previous = self.in_flight.insert(user_data, operation);
        debug_assert!(previous.is_none());
        let queue_depth = self.queue_depth;
        // SAFETY: the table owns the stable allocation and descriptor. Only a
        // matching, final CQE permits removal. Shutdown retains anything whose
        // completion is unconfirmed. Only single-shot Read/Write SQEs are used.
        let pushed = unsafe {
            self.ring
                .as_mut()
                .expect("accepting engine has a ring")
                .submission()
                .push(&entry)
        };
        if pushed.is_err() {
            // A failed push publishes nothing, so ownership can be returned.
            self.in_flight.remove(&user_data);
            return Err(Error::IoUringQueueFull { queue_depth });
        }
        Ok(user_data)
    }

    pub fn submit(&mut self) -> Result<usize> {
        self.ensure_accepting()?;
        self.enter(0)
    }

    /// Consume one completion, retaining all other operations on error.
    ///
    /// After an I/O error, new submissions are rejected, but remaining known
    /// completions may still be consumed or drained by `shutdown`.
    pub fn wait_owned_completion(&mut self) -> Result<CompletedOperation> {
        self.quarantine_result()?;
        if self.completion_lost {
            return Err(Error::IoUringShutdownUnconfirmed {
                operations: self.in_flight.len(),
            });
        }
        if self.in_flight.is_empty() {
            return Err(Error::IoUringNoInFlight);
        }
        let result = self.wait_one();
        if result.is_err() {
            self.accepting = false;
        }
        result
    }

    fn wait_one(&mut self) -> Result<CompletedOperation> {
        self.enter(1)?;
        let completion = match self.next_completion()? {
            Some(completion) => completion,
            None => {
                self.completion_lost = true;
                return Err(Error::IoUringCompletionMissing);
            }
        };
        let (user_data, result, flags) = completion;
        // These plain single-shot opcodes cannot legitimately return MORE. Do
        // not release an owner if the completion cannot establish finality.
        if io_uring::cqueue::more(flags) || !self.in_flight.contains_key(&user_data) {
            self.completion_lost = true;
            return Err(Error::IoUringUnknownCompletion { user_data });
        }
        let operation = self
            .in_flight
            .remove(&user_data)
            .expect("checked completion id");
        let bytes = completion_result_to_bytes(result)?;
        if bytes > operation.length() {
            return Err(Error::CorruptMetadata(
                "io_uring completion exceeded requested length".into(),
            ));
        }
        Ok(operation.complete(bytes))
    }

    pub fn drain(&mut self) -> Result<Vec<CompletedOperation>> {
        self.quarantine_result()?;
        let mut completed = Vec::with_capacity(self.in_flight.len());
        while !self.in_flight.is_empty() {
            completed.push(self.wait_owned_completion()?);
        }
        Ok(completed)
    }

    /// Drain queued/submitted operations and close the ring. May block on I/O.
    ///
    /// Negative operation CQEs do not prevent draining the remaining requests.
    /// If completion cannot be confirmed, retain outstanding buffers and FDs
    /// permanently, close the ring, and report `IoUringShutdownUnconfirmed`.
    pub fn shutdown(&mut self) -> Result<()> {
        self.accepting = false;
        if self.ring.is_none() {
            return self.quarantine_result();
        }
        let mut first_error = None;
        while !self.in_flight.is_empty() && !self.completion_lost {
            let before = self.in_flight.len();
            if let Err(error) = self.wait_owned_completion() {
                first_error.get_or_insert(error);
                if self.in_flight.len() == before {
                    break;
                }
            }
        }
        if !self.in_flight.is_empty() {
            self.quarantined = self.in_flight.len();
            // Closing a ring requests cancellation, but is not our proof that
            // kernel access has ended. Leak the owners rather than reuse/free
            // their allocations or descriptor numbers without that proof.
            std::mem::forget(std::mem::take(&mut self.in_flight));
        }
        drop(self.ring.take());
        self.quarantine_result()?;
        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }

    fn quarantine_result(&self) -> Result<()> {
        if self.quarantined != 0 {
            Err(Error::IoUringShutdownUnconfirmed {
                operations: self.quarantined,
            })
        } else {
            Ok(())
        }
    }

    fn ensure_accepting(&self) -> Result<()> {
        if !self.accepting {
            return Err(Error::IoUringEngineShutDown);
        }
        Ok(())
    }

    fn enter(&mut self, want: usize) -> Result<usize> {
        loop {
            let result = self.enter_once(want);
            match result {
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) => {
                    self.accepting = false;
                    return Err(Error::Io(error));
                }
                Ok(count) => return Ok(count),
            }
        }
    }

    fn enter_once(&mut self, want: usize) -> std::io::Result<usize> {
        #[cfg(test)]
        if let Some(fault) = self.faults.enter.pop_front() {
            match fault {
                faults::Enter::Before(kind) => return Err(std::io::Error::from(kind)),
                faults::Enter::After(kind) => {
                    self.ring.as_mut().unwrap().submit()?;
                    return Err(std::io::Error::from(kind));
                }
                faults::Enter::PanicAfterSubmit => {
                    self.ring.as_mut().unwrap().submit()?;
                    panic!("injected panic after submission");
                }
                faults::Enter::Partial(fail) => {
                    // SAFETY: test-only entry submits just one already-published
                    // SQE. All its resources are owned; no signal mask or flags.
                    let count = unsafe {
                        self.ring
                            .as_ref()
                            .unwrap()
                            .submitter()
                            .enter(1, 0, 0, None::<&()>)?
                    };
                    return if fail {
                        Err(std::io::Error::other(
                            "injected error after partial submission",
                        ))
                    } else {
                        Ok(count)
                    };
                }
            }
        }
        self.ring
            .as_mut()
            .expect("live operation has a ring")
            .submit_and_wait(want)
    }

    fn next_completion(&mut self) -> Result<Option<(u64, i32, u32)>> {
        #[cfg(test)]
        if self.faults.hide_completions {
            return Ok(None);
        }
        let entry = self
            .ring
            .as_mut()
            .ok_or(Error::IoUringEngineShutDown)?
            .completion()
            .next()
            .map(|cqe| (cqe.user_data(), cqe.result(), cqe.flags()));
        #[cfg(test)]
        let entry = entry.map(|entry| self.faults.transform(entry));
        Ok(entry)
    }

    fn next_user_data(&mut self) -> u64 {
        loop {
            let value = self.next_user_data;
            self.next_user_data = self.next_user_data.wrapping_add(1).max(1);
            if value != 0 && !self.in_flight.contains_key(&value) {
                return value;
            }
        }
    }
}

impl Drop for IoUringEngine {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

fn completion_result_to_bytes(result: i32) -> Result<usize> {
    if result < 0 {
        // checked_neg also handles a malformed i32::MIN CQE without panicking.
        let errno = result.checked_neg().ok_or_else(|| {
            Error::CorruptMetadata("invalid io_uring negative completion result".into())
        })?;
        return Err(Error::Io(std::io::Error::from_raw_os_error(errno)));
    }
    Ok(result as usize)
}

#[cfg(test)]
mod faults;
#[cfg(test)]
mod tests;
