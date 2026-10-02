use super::Result;
use crate::ownership::{Job, JobState, OwnershipError};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
};
pub(crate) const CHUNK_BYTES: usize = 1024 * 1024;
#[derive(Clone, Copy, Default)]
pub(crate) struct Progress {
    pub(crate) written: u64,
    pub(crate) durable: u64,
    pub(crate) verified: bool,
}
pub(crate) struct Expected {
    pub(crate) bytes: u64,
    pub(crate) sha256: [u8; 32],
    pub(crate) sha1: [u8; 20],
}
#[derive(Default)]
pub(super) struct Payload {
    file: Option<File>,
    started: bool,
    limit: u64,
    pub(super) progress: Progress,
}
impl Payload {
    pub(super) fn open(&mut self, job: &Job<'_>, limit: u64) -> Result<()> {
        if self.started || job.state() != JobState::LeaseHeld || limit == 0 || limit > 1 << 40 {
            return Err(OwnershipError::Transition);
        }
        job.validate_stage()?;
        self.started = true;
        let file = job.payload_file()?;
        if file.metadata().map_err(|_| OwnershipError::Io)?.len() != 0 {
            return Err(OwnershipError::Identity);
        }
        self.file = Some(file);
        self.limit = limit;
        Ok(())
    }
    pub(super) fn write(&mut self, data: &[u8]) -> Result<()> {
        if data.is_empty() || data.len() > CHUNK_BYTES || self.progress.verified {
            return Err(OwnershipError::Transition);
        }
        if self
            .progress
            .written
            .checked_add(data.len() as u64)
            .is_none_or(|n| n > self.limit)
        {
            return Err(OwnershipError::Content);
        }
        let file = self.file.as_mut().ok_or(OwnershipError::Transition)?;
        let mut remaining = data;
        while !remaining.is_empty() {
            match file.write(remaining) {
                Ok(0) => return Err(OwnershipError::Io),
                Ok(n) => {
                    self.progress.written += n as u64;
                    remaining = &remaining[n..];
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => return Err(OwnershipError::Io),
            }
        }
        Ok(())
    }
    pub(super) fn seal(&mut self, job: &Job<'_>, expected: &Expected) -> Result<()> {
        // Taking the only writer closes it on every exit, including sync/readback failure.
        let file = self.file.take().ok_or(OwnershipError::Transition)?;
        if job.state() != JobState::LeaseHeld
            || expected.bytes < 512
            || expected.bytes != self.progress.written
        {
            return Err(OwnershipError::Content);
        }
        file.sync_all().map_err(|_| OwnershipError::Io)?;
        self.progress.durable = self.progress.written;
        drop(file);
        job.validate_stage()?;
        let mut reader = job.payload_file()?; // fresh descriptor, checked stage/member identity
        if reader.metadata().map_err(|_| OwnershipError::Io)?.len() != expected.bytes {
            return Err(OwnershipError::Content);
        }
        let mut buffer = vec![0; CHUNK_BYTES];
        let mut left = expected.bytes;
        let mut sha256 = Sha256::new();
        let mut sha1 = Sha1::new();
        while left > 0 {
            let n = left.min(buffer.len() as u64) as usize;
            reader
                .read_exact(&mut buffer[..n])
                .map_err(|_| OwnershipError::Io)?;
            if left == expected.bytes && &buffer[..4] != b"KDMV" {
                return Err(OwnershipError::Content);
            }
            sha256.update(&buffer[..n]);
            sha1.update(&buffer[..n]);
            left -= n as u64;
        }
        if reader.metadata().map_err(|_| OwnershipError::Io)?.len() != expected.bytes
            || <[u8; 32]>::from(sha256.finalize()) != expected.sha256
            || <[u8; 20]>::from(sha1.finalize()) != expected.sha1
        {
            return Err(OwnershipError::Content);
        }
        // Recheck the namespace/marker after readback as well as before it.
        job.validate_stage()?;
        drop(job.payload_file()?);
        self.progress.verified = true;
        Ok(())
    }
    pub(super) fn close(&mut self) {
        self.file = None;
    }
}
