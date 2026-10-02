//! Native validation and private metadata persistence while the live Job owns its stage.
use super::{Result, payload::Payload};
use crate::{
    Cancellation,
    contract::{
        ArtifactId, ArtifactObservation, Completeness, ExportArtifact, MAX_METADATA_BYTES,
        SourceSelection, ValidationClaim,
    },
    ownership::{Job, JobState, OwnershipError},
};
use rvvdk_core::VirtualDisk;
use rvvdk_local::LocalFileBlockDevice;
use rvvdk_vmdk::{StreamDisk, StreamDiskLimits};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    sync::Arc,
    time::Instant,
};

fn check(cancel: &Cancellation, deadline: Instant) -> Result<()> {
    if cancel.check().is_err() {
        return Err(OwnershipError::Cancelled);
    }
    if Instant::now() >= deadline {
        return Err(OwnershipError::Deadline);
    }
    Ok(())
}
impl Payload {
    pub(super) fn admit(
        &mut self,
        job: &Job<'_>,
        id: ArtifactId,
        source: &SourceSelection,
        cancel: &Cancellation,
        deadline: Instant,
    ) -> Result<()> {
        self.admit_with(job, id, source, cancel, deadline, |_| Ok(()))
    }
    #[allow(clippy::too_many_arguments)] // A static no-op hook permits precise persistence fault tests.
    fn admit_with(
        &mut self,
        job: &Job<'_>,
        id: ArtifactId,
        source: &SourceSelection,
        cancel: &Cancellation,
        deadline: Instant,
        mut hook: impl FnMut(&str) -> Result<()>,
    ) -> Result<()> {
        check(cancel, deadline)?;
        if job.state() != JobState::LeaseHeld
            || !self.progress.verified
            || self.progress.metadata_durable
        {
            return Err(OwnershipError::Transition);
        }
        let expected = self.expected.as_ref().ok_or(OwnershipError::Transition)?;
        job.validate_stage()?;
        let device = LocalFileBlockDevice::from_buffered_file(job.payload_reader()?)
            .map_err(|_| OwnershipError::Content)?;
        let disk = StreamDisk::load(Arc::new(device), StreamDiskLimits::default())
            .map_err(|_| OwnershipError::Content)?;
        if disk.size() != source.logical_bytes() {
            return Err(OwnershipError::Content);
        }
        check(cancel, deadline)?;
        // Loading validates the bounded map. Read each present grain once to check
        // its complete zlib record/checksum, including padded tail bytes. Hole runs
        // are structurally admitted zeros; no O(logical capacity) zero scan is needed.
        let mut grain = vec![0; 65536];
        for g in disk.map().grains() {
            check(cancel, deadline)?;
            let n = (disk.size() - g.logical_offset()).min(grain.len() as u64) as usize;
            disk.read_exact_at(g.logical_offset(), &mut grain[..n])
                .map_err(|_| OwnershipError::Content)?;
            self.progress.grains_verified += 1;
            hook("grain")?;
        }
        disk.revalidate().map_err(|_| OwnershipError::Content)?;
        drop(disk);
        hook("native_checked")?;
        // Bind the native observations back to the sealed container. Cooperating
        // writers/source quiescence remain required throughout; this is no snapshot.
        job.validate_stage()?;
        let mut reader = job.payload_reader()?;
        if reader.metadata().map_err(|_| OwnershipError::Io)?.len() != expected.bytes {
            return Err(OwnershipError::Content);
        }
        let mut buffer = vec![0; super::CHUNK_BYTES];
        let mut left = expected.bytes;
        let mut hash = Sha256::new();
        while left > 0 {
            check(cancel, deadline)?;
            let n = left.min(buffer.len() as u64) as usize;
            reader
                .read_exact(&mut buffer[..n])
                .map_err(|_| OwnershipError::Io)?;
            hash.update(&buffer[..n]);
            left -= n as u64;
        }
        if <[u8; 32]>::from(hash.finalize()) != expected.sha256
            || reader.metadata().map_err(|_| OwnershipError::Io)?.len() != expected.bytes
        {
            return Err(OwnershipError::Content);
        }
        drop(reader);
        job.validate_stage()?;
        self.progress.native_verified = true;
        let metadata = ExportArtifact::new(
            id,
            source,
            ArtifactObservation {
                container_bytes: expected.bytes,
                completeness: Completeness::Complete,
                validation: ValidationClaim::ContainerDigestVerified,
                container_sha256: Some(expected.sha256),
            },
        )
        .map_err(|_| OwnershipError::Content)?;
        let bytes = metadata.to_json().map_err(|_| OwnershipError::Content)?;
        if bytes.len() > MAX_METADATA_BYTES {
            return Err(OwnershipError::Content);
        }
        check(cancel, deadline)?;
        let mut file = job.metadata_file()?;
        if file.metadata().map_err(|_| OwnershipError::Io)?.len() != 0 {
            return Err(OwnershipError::Identity);
        }
        // The member inode is already owned and directory-synced. Preserve it;
        // a torn write is untrusted metadata and never advances TransferComplete.
        let middle = bytes.len() / 2;
        file.write_all(&bytes[..middle])
            .map_err(|_| OwnershipError::Io)?;
        hook("metadata_partial")?;
        file.write_all(&bytes[middle..])
            .map_err(|_| OwnershipError::Io)?;
        hook("metadata_written")?;
        file.sync_all().map_err(|_| OwnershipError::Io)?;
        drop(file);
        hook("metadata_synced")?;
        check(cancel, deadline)?;
        job.validate_stage()?;
        let mut file = job.metadata_file()?;
        if file.metadata().map_err(|_| OwnershipError::Io)?.len() != bytes.len() as u64 {
            return Err(OwnershipError::Content);
        }
        let mut observed = vec![0; bytes.len()];
        file.read_exact(&mut observed)
            .map_err(|_| OwnershipError::Io)?;
        if observed != bytes
            || file.metadata().map_err(|_| OwnershipError::Io)?.len() != bytes.len() as u64
        {
            return Err(OwnershipError::Content);
        }
        let parsed = ExportArtifact::from_json(&observed).map_err(|_| OwnershipError::Content)?;
        if parsed != metadata {
            return Err(OwnershipError::Content);
        }
        job.validate_stage()?;
        check(cancel, deadline)?;
        self.progress.metadata_durable = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
