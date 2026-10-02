//! Fresh local consumption; no recovered Job, remote lease, or publication.
use super::*;
use crate::{
    Cancellation,
    contract::{ExportArtifact, MAX_METADATA_BYTES},
};
use rvvdk_core::{EndpointIdentity, RawDisk, VirtualDisk};
use rvvdk_datamover::{CopyObserver, CopyOptions, CopyReport, DataMover};
use rvvdk_local::LocalFileBlockDevice;
use rvvdk_vmdk::{StreamDisk, StreamDiskLimits};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct RetainedOptions {
    pub cancellation: Cancellation,
    pub timeout: Duration,
}
impl Default for RetainedOptions {
    fn default() -> Self {
        Self {
            cancellation: Cancellation::default(),
            timeout: Duration::from_secs(3600),
        }
    }
}
mod output;
pub use output::{
    OutputId, OutputLocation, OutputRecovery, OutputReport, OutputState, PublicationDirectory,
    PublicationRecovery, PublicationReport, VerifiedOutput,
};

struct Control<'a> {
    options: &'a RetainedOptions,
    deadline: Instant,
}
impl<'a> Control<'a> {
    fn new(options: &'a RetainedOptions) -> Result<Self> {
        if options.timeout.is_zero() || options.timeout > Duration::from_secs(21600) {
            return Err(OwnershipError::Content);
        }
        let c = Self {
            options,
            deadline: Instant::now() + options.timeout,
        };
        c.check()?;
        Ok(c)
    }
    fn check(&self) -> Result<()> {
        self.options
            .cancellation
            .check()
            .map_err(|_| OwnershipError::Cancelled)?;
        if Instant::now() >= self.deadline {
            return Err(OwnershipError::Deadline);
        }
        Ok(())
    }
}
impl rvvdk_datamover::Cancellation for Control<'_> {
    fn is_cancelled(&self) -> bool {
        self.check().is_err()
    }
}
/// Owns its store lock and confined read-only native source. No handle or Job
/// escapes. Requires cooperating writers and trusted ancestors throughout use.
/// Conversion is synchronous; call from a blocking context and await its end.
pub struct RetainedArtifact {
    // Field drop order matters: release source descriptors before unlocking store.
    disk: Option<StreamDisk>,
    store: JobStore,
    record: Record,
    source: SourceSelection,
    metadata: ExportArtifact,
    journal: Identity,
}
impl fmt::Debug for RetainedArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RetainedArtifact([private])")
    }
}
impl RetainedArtifact {
    /// Admit only an acknowledged CompletedLease with no pending transaction.
    /// Metadata claims never substitute for fresh native/content checks.
    pub fn open(
        store: JobStore,
        id: ArtifactId,
        source: SourceSelection,
        options: RetainedOptions,
    ) -> Result<Self> {
        let control = Control::new(&options)?;
        let (record, metadata, disk, journal) = admit(&store, id, &source, &control)?;
        Ok(Self {
            disk: Some(disk),
            store,
            record,
            source,
            metadata,
            journal,
        })
    }
    pub fn logical_bytes(&self) -> u64 {
        self.metadata.logical_bytes()
    }
    /// Convert into a caller-owned, already-sized buffered RAW file. Re-admit the
    /// retained source before any destination writes; keep the lock through worker
    /// drain, destination flush and final source checks. No path is created/published.
    /// On error the destination can be partial; observer counters are lower bounds.
    pub fn convert_to<O: CopyObserver + ?Sized>(
        &mut self,
        destination: &RawDisk<LocalFileBlockDevice>,
        copy: CopyOptions,
        options: RetainedOptions,
        observer: &O,
    ) -> Result<CopyReport> {
        let control = Control::new(&options)?;
        if destination.device().is_direct_io() {
            return Err(OwnershipError::Conversion);
        }
        // Release the previous map/decoder before admission to bound peak memory.
        self.disk = None;
        let (record, metadata, disk, journal) =
            admit(&self.store, self.metadata.id(), &self.source, &control)?;
        if record != self.record || metadata != self.metadata || journal != self.journal {
            return Err(OwnershipError::Identity);
        }
        self.disk = Some(disk);
        let disk = self.disk.as_ref().ok_or(OwnershipError::Transition)?;
        let endpoint = destination
            .copy_endpoint()
            .map_err(|_| OwnershipError::Conversion)?;
        // Reject every owned member and the journal, not only the container.
        let Some(EndpointIdentity::LocalFile { device, inode }) = endpoint.identity else {
            return Err(OwnershipError::Identity);
        };
        let identity = Identity {
            dev: device,
            ino: inode,
        };
        if self
            .record
            .stage
            .as_ref()
            .ok_or(OwnershipError::Record)?
            .members
            .contains(&identity)
            || identity == self.journal
        {
            return Err(OwnershipError::Identity);
        }
        if endpoint.size != disk.size() {
            return Err(OwnershipError::Conversion);
        }
        control.check()?;
        let mover = DataMover::new(copy);
        let plan = mover
            .plan_with_destination(disk, destination)
            .map_err(|_| OwnershipError::Conversion)?;
        control.check()?;
        let copied = mover.execute_plan_controlled(&plan, disk, destination, &control, observer);
        control.check()?;
        let report = copied.map_err(|_| OwnershipError::Conversion)?;
        // Engine completion means flush, not final validation/publication. Reject
        // source drift even if destination flush already succeeded.
        check_current(
            &self.store,
            &self.record,
            &self.source,
            &self.metadata,
            self.journal,
            &control,
        )?;
        disk.revalidate().map_err(|_| OwnershipError::Content)?;
        let now = destination
            .copy_endpoint()
            .map_err(|_| OwnershipError::Conversion)?;
        if now.identity != endpoint.identity
            || now.size != endpoint.size
            || now.capabilities != endpoint.capabilities
        {
            return Err(OwnershipError::Identity);
        }
        control.check()?;
        Ok(report)
    }
}
fn members(
    store: &JobStore,
    id: ArtifactId,
    source: &SourceSelection,
) -> Result<(Record, ExportArtifact, File, Identity)> {
    store.ready()?;
    if exists(&store.directory, &transaction_name(id.as_bytes()))? {
        return Err(OwnershipError::Uncertain);
    }
    let record = store.read(id, source)?;
    if record.state != JobState::CompletedLease {
        return Err(OwnershipError::Transition);
    }
    let journal = open_at(
        &store.directory,
        &record_name(id.as_bytes()),
        false,
        false,
        false,
    )?;
    private(&journal, false)?;
    let journal = identity(&journal)?;
    let dir = store
        .check_stage(&record, false)?
        .ok_or(OwnershipError::Identity)?;
    let stamp = record.stage.as_ref().ok_or(OwnershipError::Record)?;
    let mut file = check_named(&dir, MEMBERS[1], stamp.members[1], false)?;
    let metadata = ExportArtifact::from_json(&bounded_read(&mut file, MAX_METADATA_BYTES as u64)?)
        .map_err(|_| OwnershipError::Content)?;
    if metadata.id() != id {
        return Err(OwnershipError::Identity);
    }
    metadata
        .check_source(source)
        .map_err(|_| OwnershipError::Identity)?;
    if metadata.completeness() != crate::contract::Completeness::Complete {
        return Err(OwnershipError::Content);
    }
    let payload = check_named(&dir, MEMBERS[0], stamp.members[0], false)?;
    if payload.metadata().map_err(|_| OwnershipError::Io)?.len() != metadata.container_bytes() {
        return Err(OwnershipError::Content);
    }
    Ok((record, metadata, payload, journal))
}
fn verify_bytes(mut file: File, metadata: &ExportArtifact, control: &Control<'_>) -> Result<()> {
    let mut left = metadata.container_bytes();
    let mut buffer = vec![0; 1 << 20];
    let mut hash = Sha256::new();
    while left > 0 {
        control.check()?;
        let n = left.min(buffer.len() as u64) as usize;
        file.read_exact(&mut buffer[..n])
            .map_err(|_| OwnershipError::Io)?;
        hash.update(&buffer[..n]);
        left -= n as u64;
    }
    let size = file.metadata().map_err(|_| OwnershipError::Io)?.len();
    metadata
        .check_container(size, &hash.finalize().into())
        .map_err(|_| OwnershipError::Content)?;
    control.check()
}
fn check_current(
    store: &JobStore,
    record: &Record,
    source: &SourceSelection,
    metadata: &ExportArtifact,
    journal: Identity,
    control: &Control<'_>,
) -> Result<()> {
    let (now, m, file, j) = members(store, metadata.id(), source)?;
    if &now != record || &m != metadata || j != journal {
        return Err(OwnershipError::Identity);
    }
    verify_bytes(file, metadata, control)?;
    let (now, m, _, j) = members(store, metadata.id(), source)?;
    if &now != record || &m != metadata || j != journal {
        return Err(OwnershipError::Identity);
    }
    control.check()
}
fn admit(
    store: &JobStore,
    id: ArtifactId,
    source: &SourceSelection,
    control: &Control<'_>,
) -> Result<(Record, ExportArtifact, StreamDisk, Identity)> {
    control.check()?;
    let (record, metadata, file, journal) = members(store, id, source)?;
    let device =
        LocalFileBlockDevice::from_buffered_file(file).map_err(|_| OwnershipError::Content)?;
    let disk = StreamDisk::load(Arc::new(device), StreamDiskLimits::default())
        .map_err(|_| OwnershipError::Content)?;
    if disk.size() != source.logical_bytes() {
        return Err(OwnershipError::Content);
    }
    let mut buffer = vec![0; 65536];
    for grain in disk.map().grains() {
        control.check()?;
        let n = (disk.size() - grain.logical_offset()).min(buffer.len() as u64) as usize;
        disk.read_exact_at(grain.logical_offset(), &mut buffer[..n])
            .map_err(|_| OwnershipError::Content)?;
    }
    disk.revalidate().map_err(|_| OwnershipError::Content)?;
    check_current(store, &record, source, &metadata, journal, control)?;
    Ok((record, metadata, disk, journal))
}
#[cfg(test)]
mod tests;
