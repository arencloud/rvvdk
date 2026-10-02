//! Durable RAW ownership with separate fresh publication and explicit cleanup.
use super::*;
mod publication;
pub use publication::{
    OutputLocation, PublicationDirectory, PublicationRecovery, PublicationReport, VerifiedOutput,
};

const OUTPUT_MEMBERS: [&str; 3] = ["disk.raw", "output.json", "owner"];
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct OutputId([u8; 16]);
impl OutputId {
    /// Generated final bundle component; this name alone grants no authority.
    pub fn bundle_name(self) -> String {
        format!("raw-{}", crate::transport::hex_string(&self.0))
    }
    pub fn new(bytes: [u8; 16]) -> Result<Self> {
        if bytes == [0; 16] {
            Err(OwnershipError::Record)
        } else {
            Ok(Self(bytes))
        }
    }
}
impl fmt::Debug for OutputId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OutputId([redacted])")
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputState {
    Prepared,
    StageIntent,
    Staged,
    ConvertIntent,
    Converted,
    VerifyIntent,
    Verified,
    PublishIntent,
    Published,
    CleanupIntent,
    Cleaned,
}
impl OutputState {
    fn sequence(self) -> u64 {
        match self {
            Self::Prepared => 0,
            Self::StageIntent => 1,
            Self::Staged => 2,
            Self::ConvertIntent => 3,
            Self::Converted => 4,
            Self::VerifyIntent => 5,
            Self::Verified => 6,
            Self::PublishIntent => 7,
            Self::Published => 8,
            Self::CleanupIntent | Self::Cleaned => u64::MAX,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct OutputRecovery {
    pub state: OutputState,
    pub sequence: u64,
    pub pending_transaction: bool,
}
#[derive(Debug, Default, Serialize)]
pub struct OutputReport {
    pub primary_error: Option<OwnershipError>,
    pub assessment_error: Option<OwnershipError>,
    pub recovery: Option<OutputRecovery>,
    pub conversion_flushed: bool,
    pub logical_bytes_verified: u64,
    pub metadata_durable: bool,
}
impl OutputReport {
    pub fn is_success(&self) -> bool {
        self.primary_error.is_none()
            && self.assessment_error.is_none()
            && self.conversion_flushed
            && self.metadata_durable
            && self
                .recovery
                .as_ref()
                .is_some_and(|r| r.state == OutputState::Verified && !r.pending_transaction)
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OutputRecord {
    version: u32,
    store: Identity,
    output: [u8; 16],
    artifact: [u8; 16],
    source: [u8; 32],
    container_contract: [u8; 32],
    operation: [u8; 16],
    logical_bytes: u64,
    state: OutputState,
    sequence: u64,
    stage: Option<Stage>,
    raw_sha256: Option<[u8; 32]>,
    // Absent fields preserve the canonical v1 checksum representation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    publication_parent: Option<Identity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    cleanup_from: Option<OutputState>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct OutputEnvelope {
    record: OutputRecord,
    sha256: [u8; 32],
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct OutputMetadata {
    version: u32,
    format: RawFormat,
    output: [u8; 16],
    artifact: [u8; 16],
    source: [u8; 32],
    container_contract: [u8; 32],
    logical_bytes: u64,
    raw_sha256: [u8; 32],
    verification: OutputVerification,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum RawFormat {
    Raw,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum OutputVerification {
    LogicalSourceReadback,
}
fn name(id: &[u8; 16]) -> String {
    format!("output-{}.json", crate::transport::hex_string(id))
}
fn txn(id: &[u8; 16]) -> String {
    format!("output-txn-{}.json", crate::transport::hex_string(id))
}
fn stage(id: &[u8; 16]) -> String {
    format!("raw-stage-{}", crate::transport::hex_string(id))
}
fn check_stage(store: &JobStore, record: &OutputRecord) -> Result<File> {
    store.ready()?;
    let stamp = record.stage.as_ref().ok_or(OwnershipError::Identity)?;
    let dir = check_named(
        &store.directory,
        &stage(&record.operation),
        stamp.directory,
        true,
    )?;
    for (i, name) in OUTPUT_MEMBERS.iter().enumerate() {
        let mut file = check_named(&dir, name, stamp.members[i], false)?;
        if i == 2 && bounded_read(&mut file, 16)? != record.operation {
            return Err(OwnershipError::Identity);
        }
    }
    Ok(dir)
}
impl JobStore {
    /// Read-only assessment. Never returns a writer, publication or cleanup capability.
    pub fn assess_output(
        &self,
        id: OutputId,
        artifact: ArtifactId,
        source: &SourceSelection,
    ) -> Result<OutputRecovery> {
        let r = self.read_output(id, artifact, source)?;
        if r.stage.is_some() && r.publication_parent.is_none() && r.cleanup_from.is_none() {
            drop(check_stage(self, &r)?);
        }
        self.output_recovery(&r)
    }
    fn output_recovery(&self, r: &OutputRecord) -> Result<OutputRecovery> {
        Ok(OutputRecovery {
            state: r.state,
            sequence: r.sequence,
            pending_transaction: exists(&self.directory, &txn(&r.output))?,
        })
    }
    fn read_output(
        &self,
        id: OutputId,
        artifact: ArtifactId,
        source: &SourceSelection,
    ) -> Result<OutputRecord> {
        self.ready()?;
        let mut file = open_at(&self.directory, &name(&id.0), false, false, false)?;
        private(&file, false)?;
        let envelope: OutputEnvelope =
            serde_json::from_slice(&bounded_read(&mut file, MAX_RECORD)?)
                .map_err(|_| OwnershipError::Record)?;
        let r = envelope.record;
        let data = serde_json::to_vec(&r).map_err(|_| OwnershipError::Record)?;
        if envelope.sha256 != <[u8; 32]>::from(Sha256::digest(data))
            || r.output != id.0
            || r.artifact != *artifact.as_bytes()
            || r.store != self.identity
            || r.source != source.ownership_binding()
            || r.logical_bytes != source.logical_bytes()
            || r.operation == [0; 16]
            || !publication::valid_record(&r)
        {
            return Err(OwnershipError::Record);
        }
        Ok(r)
    }
}
impl RetainedArtifact {
    /// Consume the retained source into a fresh private owned RAW output. The
    /// source stays locked through conversion, full logical readback and journal
    /// acknowledgment. Stages/records are retained on every outcome; no publication,
    /// cleanup, restart or remote capability is produced. Await blocking completion.
    pub fn convert_owned<O: CopyObserver + ?Sized>(
        mut self,
        id: OutputId,
        copy: CopyOptions,
        options: RetainedOptions,
        observer: &O,
    ) -> OutputReport {
        self.convert_owned_hook(id, copy, options, observer, |_, _| Ok(()))
    }
    fn convert_owned_hook<O: CopyObserver + ?Sized>(
        &mut self,
        id: OutputId,
        copy: CopyOptions,
        options: RetainedOptions,
        observer: &O,
        hook: impl FnMut(OutputState, &str) -> Result<()>,
    ) -> OutputReport {
        let mut report = OutputReport::default();
        let result = (|| {
            let control = Control::new(&options)?;
            self.store.ready()?;
            if exists(&self.store.directory, &name(&id.0))?
                || exists(&self.store.directory, &txn(&id.0))?
            {
                return Err(OwnershipError::Exists);
            }
            let mut operation = [0; 16];
            random(&mut operation)?;
            let container_contract = Sha256::digest(
                self.metadata
                    .to_json()
                    .map_err(|_| OwnershipError::Content)?,
            )
            .into();
            let record = OutputRecord {
                version: 1,
                store: self.store.identity,
                output: id.0,
                artifact: *self.metadata.id().as_bytes(),
                source: self.source.ownership_binding(),
                container_contract,
                operation,
                logical_bytes: self.logical_bytes(),
                state: OutputState::Prepared,
                sequence: 0,
                stage: None,
                raw_sha256: None,
                publication_parent: None,
                cleanup_from: None,
            };
            let mut owner = Owner {
                artifact: self,
                record,
                hook,
                poisoned: false,
            };
            owner.commit(true)?;
            owner.prepare(&control)?;
            owner.advance(OutputState::ConvertIntent)?;
            control.check()?;
            let dir = check_stage(&owner.artifact.store, &owner.record)?;
            let file = open_at(&dir, OUTPUT_MEMBERS[0], true, false, false)?;
            private(&file, false)?;
            if identity(&file)? != owner.record.stage.as_ref().unwrap().members[0] {
                return Err(OwnershipError::Identity);
            }
            let dest = RawDisk::new(
                LocalFileBlockDevice::from_buffered_file(file)
                    .map_err(|_| OwnershipError::Conversion)?,
            );
            let copy_options = RetainedOptions {
                cancellation: options.cancellation.clone(),
                timeout: control.deadline.saturating_duration_since(Instant::now()),
            };
            control.check()?;
            owner
                .artifact
                .convert_to(&dest, copy, copy_options, observer)?;
            // No writer survives Converted, verification, or return.
            drop(dest);
            drop(dir);
            report.conversion_flushed = true;
            owner.point("conversion_drained")?;
            control.check()?;
            drop(check_stage(&owner.artifact.store, &owner.record)?);
            owner.advance(OutputState::Converted)?;
            owner.advance(OutputState::VerifyIntent)?;
            owner.verify(&control, &mut report)?;
            owner.advance(OutputState::Verified)?;
            control.check()
        })();
        report.primary_error = result.err();
        // Assessment may inspect a surviving transaction after uncertain persistence;
        // it never clears poison or authorizes a subsequent resource operation.
        match self
            .store
            .assess_output(id, self.metadata.id(), &self.source)
        {
            Ok(r) => report.recovery = Some(r),
            Err(e) => report.assessment_error = Some(e),
        }
        report
    }
}
struct Owner<'a, H> {
    artifact: &'a mut RetainedArtifact,
    record: OutputRecord,
    hook: H,
    poisoned: bool,
}
impl<H: FnMut(OutputState, &str) -> Result<()>> Owner<'_, H> {
    fn point(&mut self, phase: &str) -> Result<()> {
        (self.hook)(self.record.state, phase)
    }
    fn advance(&mut self, state: OutputState) -> Result<()> {
        if self.poisoned || state.sequence() != self.record.sequence + 1 {
            return Err(OwnershipError::Transition);
        }
        self.record.state = state;
        self.record.sequence = state.sequence();
        self.commit(false)
    }
    fn commit(&mut self, initial: bool) -> Result<()> {
        if self.poisoned {
            return Err(OwnershipError::Uncertain);
        }
        let result = commit_output(&self.artifact.store, &self.record, initial, &mut self.hook);
        if result.is_err() {
            self.poisoned = true;
            return Err(OwnershipError::Uncertain);
        }
        Ok(())
    }
    fn prepare(&mut self, control: &Control<'_>) -> Result<()> {
        control.check()?;
        self.advance(OutputState::StageIntent)?;
        control.check()?;
        let raw = cstr(&stage(&self.record.operation))?;
        // SAFETY: pinned private directory and generated component.
        if unsafe {
            libc::mkdirat(
                self.artifact.store.directory.as_raw_fd(),
                raw.as_ptr(),
                0o700,
            )
        } != 0
        {
            return Err(OwnershipError::Io);
        }
        self.point("stage_created")?;
        let dir = open_at(
            &self.artifact.store.directory,
            &stage(&self.record.operation),
            false,
            false,
            true,
        )?;
        private(&dir, true)?;
        let mut members = [Identity { dev: 0, ino: 0 }; 3];
        for (i, name) in OUTPUT_MEMBERS.iter().enumerate() {
            let mut f = open_at(&dir, name, true, true, false)?;
            if i == 0 {
                f.set_len(self.record.logical_bytes)
                    .map_err(|_| OwnershipError::Io)?;
            }
            if i == 2 {
                f.write_all(&self.record.operation)
                    .map_err(|_| OwnershipError::Io)?;
            }
            f.sync_all().map_err(|_| OwnershipError::Io)?;
            members[i] = identity(&f)?;
        }
        dir.sync_all().map_err(|_| OwnershipError::Io)?;
        self.artifact
            .store
            .directory
            .sync_all()
            .map_err(|_| OwnershipError::Io)?;
        self.point("stage_durable")?;
        self.record.stage = Some(Stage {
            directory: identity(&dir)?,
            members,
        });
        self.advance(OutputState::Staged)
    }
    fn verify(&mut self, control: &Control<'_>, report: &mut OutputReport) -> Result<()> {
        control.check()?;
        self.point("verification_started")?;
        let dir = check_stage(&self.artifact.store, &self.record)?;
        let stamp = self.record.stage.as_ref().unwrap().clone();
        let mut raw = check_named(&dir, OUTPUT_MEMBERS[0], stamp.members[0], false)?;
        if raw.metadata().map_err(|_| OwnershipError::Io)?.len() != self.record.logical_bytes {
            return Err(OwnershipError::Content);
        }
        let mut expected = vec![0; 1 << 20];
        let mut observed = vec![0; 1 << 20];
        let mut hash = Sha256::new();
        let mut offset = 0;
        while offset < self.record.logical_bytes {
            control.check()?;
            let n = (self.record.logical_bytes - offset).min(expected.len() as u64) as usize;
            self.artifact
                .disk
                .as_ref()
                .ok_or(OwnershipError::Transition)?
                .read_exact_at(offset, &mut expected[..n])
                .map_err(|_| OwnershipError::Content)?;
            raw.read_exact(&mut observed[..n])
                .map_err(|_| OwnershipError::Io)?;
            if expected[..n] != observed[..n] {
                return Err(OwnershipError::Content);
            }
            hash.update(&observed[..n]);
            offset += n as u64;
            report.logical_bytes_verified = offset;
            self.point("verified_chunk")?;
        }
        if raw.metadata().map_err(|_| OwnershipError::Io)?.len() != self.record.logical_bytes {
            return Err(OwnershipError::Content);
        }
        drop(raw);
        drop(dir);
        check_current(
            &self.artifact.store,
            &self.artifact.record,
            &self.artifact.source,
            &self.artifact.metadata,
            self.artifact.journal,
            control,
        )?;
        drop(check_stage(&self.artifact.store, &self.record)?);
        let digest = hash.finalize().into();
        let metadata = OutputMetadata {
            version: 1,
            format: RawFormat::Raw,
            output: self.record.output,
            artifact: self.record.artifact,
            source: self.record.source,
            container_contract: self.record.container_contract,
            logical_bytes: self.record.logical_bytes,
            raw_sha256: digest,
            verification: OutputVerification::LogicalSourceReadback,
        };
        let bytes = serde_json::to_vec(&metadata).map_err(|_| OwnershipError::Content)?;
        if bytes.len() > MAX_METADATA_BYTES {
            return Err(OwnershipError::Content);
        }
        control.check()?;
        let dir = check_stage(&self.artifact.store, &self.record)?;
        let mut file = open_at(&dir, OUTPUT_MEMBERS[1], true, false, false)?;
        private(&file, false)?;
        if identity(&file)? != stamp.members[1]
            || file.metadata().map_err(|_| OwnershipError::Io)?.len() != 0
        {
            return Err(OwnershipError::Identity);
        }
        let mid = bytes.len() / 2;
        file.write_all(&bytes[..mid])
            .map_err(|_| OwnershipError::Io)?;
        self.point("metadata_partial")?;
        file.write_all(&bytes[mid..])
            .map_err(|_| OwnershipError::Io)?;
        self.point("metadata_written")?;
        file.sync_all().map_err(|_| OwnershipError::Io)?;
        drop(file);
        self.point("metadata_synced")?;
        let mut file = check_named(&dir, OUTPUT_MEMBERS[1], stamp.members[1], false)?;
        let seen = bounded_read(&mut file, MAX_METADATA_BYTES as u64)?;
        if seen != bytes {
            return Err(OwnershipError::Content);
        }
        let parsed: OutputMetadata =
            serde_json::from_slice(&seen).map_err(|_| OwnershipError::Content)?;
        if parsed != metadata {
            return Err(OwnershipError::Content);
        }
        drop(check_stage(&self.artifact.store, &self.record)?);
        control.check()?;
        report.metadata_durable = true;
        self.record.raw_sha256 = Some(digest);
        Ok(())
    }
}
fn commit_output(
    store: &JobStore,
    record: &OutputRecord,
    initial: bool,
    hook: &mut impl FnMut(OutputState, &str) -> Result<()>,
) -> Result<()> {
    (|| {
        store.ready()?;
        let data = serde_json::to_vec(&record).map_err(|_| OwnershipError::Record)?;
        let envelope = OutputEnvelope {
            record: record.clone(),
            sha256: Sha256::digest(&data).into(),
        };
        let data = serde_json::to_vec(&envelope).map_err(|_| OwnershipError::Record)?;
        if data.len() as u64 > MAX_RECORD {
            return Err(OwnershipError::Record);
        }
        if !initial {
            private(
                &open_at(&store.directory, &name(&record.output), false, false, false)?,
                false,
            )?;
        }
        let mut file = open_at(&store.directory, &txn(&record.output), true, true, false)?;
        let middle = data.len() / 2;
        file.write_all(&data[..middle])
            .map_err(|_| OwnershipError::Io)?;
        hook(record.state, "record_partial")?;
        file.write_all(&data[middle..])
            .map_err(|_| OwnershipError::Io)?;
        hook(record.state, "record_written")?;
        file.sync_all().map_err(|_| OwnershipError::Io)?;
        hook(record.state, "record_synced")?;
        let from = cstr(&txn(&record.output))?;
        let to = cstr(&name(&record.output))?;
        // SAFETY: pinned directory, generated single-component names; initial record is no-replace.
        if unsafe {
            libc::renameat2(
                store.directory.as_raw_fd(),
                from.as_ptr(),
                store.directory.as_raw_fd(),
                to.as_ptr(),
                if initial { libc::RENAME_NOREPLACE } else { 0 },
            )
        } != 0
        {
            return Err(OwnershipError::Io);
        }
        hook(record.state, "record_renamed")?;
        store.directory.sync_all().map_err(|_| OwnershipError::Io)?;
        hook(record.state, "record_durable")?;
        Ok(())
    })()
    .map_err(|_| OwnershipError::Uncertain)
}

#[cfg(test)]
mod tests;
