//! Fresh local publication capabilities and bounded explicit cleanup.
use super::*;

/// A locked, caller-selected private destination directory. Ancestors must remain
/// trusted and stable. Publication requires the source filesystem and a directory
/// outside the job store. Opening this does not create directories.
pub struct PublicationDirectory(JobStore);
impl PublicationDirectory {
    pub fn open(path: &Path) -> Result<Self> {
        JobStore::open(path).map(Self)
    }
    fn validate(&self, store: &JobStore) -> Result<()> {
        self.0.ready()?;
        if self.0.identity.dev != store.identity.dev {
            return Err(OwnershipError::Identity);
        }
        // Reject the store and all descendants, including source/output stages.
        let mut dir = open_at(&self.0.directory, ".", false, false, true)?;
        for _ in 0..128 {
            let current = identity(&dir)?;
            if current == store.identity {
                return Err(OwnershipError::Identity);
            }
            let parent = open_at(&dir, "..", false, false, true)?;
            if identity(&parent)? == current {
                return Ok(());
            }
            dir = parent;
        }
        Err(OwnershipError::Identity)
    }
}
impl fmt::Debug for PublicationDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PublicationDirectory([private])")
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputLocation {
    Absent,
    Owned,
    ForeignOrInvalid,
}
/// Namespace/identity observations only, never proof of data or rename durability.
#[derive(Debug, Serialize)]
pub struct PublicationRecovery {
    pub output: OutputRecovery,
    pub staged: OutputLocation,
    pub published: OutputLocation,
}
#[derive(Debug, Default, Serialize)]
pub struct PublicationReport {
    pub primary_error: Option<OwnershipError>,
    pub assessment_error: Option<OwnershipError>,
    pub recovery: Option<PublicationRecovery>,
    pub rename_observed: bool,
    pub directories_synced: bool,
}
impl PublicationReport {
    pub fn is_success(&self) -> bool {
        self.primary_error.is_none()
            && self.assessment_error.is_none()
            && self.rename_observed
            && self.directories_synced
            && self.recovery.as_ref().is_some_and(|r| {
                r.output.state == OutputState::Published
                    && !r.output.pending_transaction
                    && r.staged == OutputLocation::Absent
                    && r.published == OutputLocation::Owned
            })
    }
}
/// Holds the source lock and a freshly verified local output binding. No writable
/// handle or remote capability escapes. Publication consumes this capability.
pub struct VerifiedOutput {
    artifact: RetainedArtifact,
    record: OutputRecord,
    journal: Identity,
}
impl fmt::Debug for VerifiedOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("VerifiedOutput([private])")
    }
}
fn cleanup_allowed(state: OutputState) -> bool {
    matches!(
        state,
        OutputState::Staged
            | OutputState::ConvertIntent
            | OutputState::Converted
            | OutputState::VerifyIntent
            | OutputState::Verified
            | OutputState::Published
    )
}
pub(super) fn valid_record(r: &OutputRecord) -> bool {
    let cleaning = matches!(r.state, OutputState::CleanupIntent | OutputState::Cleaned);
    let origin = if cleaning {
        let Some(origin) = r.cleanup_from else {
            return false;
        };
        if !cleanup_allowed(origin) {
            return false;
        }
        origin
    } else {
        if r.cleanup_from.is_some() {
            return false;
        }
        r.state
    };
    let published = matches!(origin, OutputState::PublishIntent | OutputState::Published);
    if match r.version {
        1 => cleaning || published || r.publication_parent.is_some(),
        2 => !cleaning && !published,
        _ => true,
    } {
        return false;
    }
    let sequence = origin.sequence()
        + if cleaning {
            if r.state == OutputState::Cleaned {
                2
            } else {
                1
            }
        } else {
            0
        };
    if r.sequence != sequence
        || r.stage.is_some() != (origin.sequence() >= 2)
        || r.raw_sha256.is_some() != (origin.sequence() >= 6)
        || r.publication_parent.is_some() != published
    {
        return false;
    }
    if let Some(parent) = r.publication_parent
        && (parent.dev != r.store.dev || parent.ino == 0 || parent == r.store)
    {
        return false;
    }
    if let Some(stage) = &r.stage {
        if stage.directory.dev != r.store.dev || stage.directory.ino == 0 {
            return false;
        }
        for (i, member) in stage.members.iter().enumerate() {
            if member.dev != r.store.dev
                || member.ino == 0
                || *member == stage.directory
                || stage.members[..i].contains(member)
            {
                return false;
            }
        }
    }
    true
}
/// Bounded descriptor-relative directory enumeration. Reject unknown entries even
/// during partial cleanup; never adopt or recursively remove them.
fn only_members(dir: &File) -> Result<()> {
    use std::os::fd::IntoRawFd;
    let fd = open_at(dir, ".", false, false, true)?.into_raw_fd();
    // SAFETY: owned directory descriptor; fdopendir takes ownership on success.
    let stream = unsafe { libc::fdopendir(fd) };
    if stream.is_null() {
        // SAFETY: fdopendir did not take ownership on failure.
        unsafe { libc::close(fd) };
        return Err(OwnershipError::Io);
    }
    let result = (|| {
        let mut count = 0;
        loop {
            // SAFETY: thread-local errno and a live directory stream. d_name is a
            // NUL-terminated name valid until the next readdir on this stream.
            let entry = unsafe {
                *libc::__errno_location() = 0;
                libc::readdir(stream)
            };
            if entry.is_null() {
                // SAFETY: reading this thread's errno immediately after readdir.
                return if unsafe { *libc::__errno_location() } == 0 {
                    Ok(())
                } else {
                    Err(OwnershipError::Io)
                };
            }
            count += 1;
            if count > 5 {
                return Err(OwnershipError::Identity);
            }
            // SAFETY: readdir returned a live dirent as described above.
            let n = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes();
            if n != b"." && n != b".." && !OUTPUT_MEMBERS.iter().any(|v| v.as_bytes() == n) {
                return Err(OwnershipError::Identity);
            }
        }
    })();
    // SAFETY: unique live DIR, closes its owned descriptor exactly once.
    unsafe { libc::closedir(stream) };
    result
}
fn bundle(parent: &File, component: &str, r: &OutputRecord, partial: bool) -> Result<Option<File>> {
    if !exists(parent, component)? {
        return if partial {
            Ok(None)
        } else {
            Err(OwnershipError::Identity)
        };
    }
    let stamp = r.stage.as_ref().ok_or(OwnershipError::Record)?;
    let dir = check_named(parent, component, stamp.directory, true)?;
    only_members(&dir)?;
    for (i, n) in OUTPUT_MEMBERS.iter().enumerate() {
        if partial && !exists(&dir, n)? {
            continue;
        }
        let mut f = check_named(&dir, n, stamp.members[i], false)?;
        if i == 2 && bounded_read(&mut f, 16)? != r.operation {
            return Err(OwnershipError::Identity);
        }
    }
    Ok(Some(dir))
}
fn location(parent: &File, component: &str, r: &OutputRecord) -> Result<OutputLocation> {
    if !exists(parent, component)? {
        return Ok(OutputLocation::Absent);
    }
    Ok(if bundle(parent, component, r, false).is_ok() {
        OutputLocation::Owned
    } else {
        OutputLocation::ForeignOrInvalid
    })
}
impl VerifiedOutput {
    /// Requires the original retained source to remain valid and CompletedLease.
    /// Freshly compares every logical byte and checks bounded metadata and hashes.
    pub fn open(
        store: JobStore,
        output: OutputId,
        artifact: ArtifactId,
        source: SourceSelection,
        options: RetainedOptions,
    ) -> Result<Self> {
        let control = Control::new(&options)?;
        let record = store.read_output(output, artifact, &source)?;
        if record.state != OutputState::Verified {
            return Err(OwnershipError::Transition);
        }
        if exists(&store.directory, &txn(&output.0))? {
            return Err(OwnershipError::Uncertain);
        }
        let journal = identity(&open_at(
            &store.directory,
            &name(&output.0),
            false,
            false,
            false,
        )?)?;
        let remaining = RetainedOptions {
            cancellation: options.cancellation.clone(),
            timeout: control.deadline.saturating_duration_since(Instant::now()),
        };
        control.check()?;
        let artifact = RetainedArtifact::open(store, artifact, source, remaining)?;
        let value = Self {
            artifact,
            record,
            journal,
        };
        value.validate(&control)?;
        Ok(value)
    }
    fn current(&self) -> Result<()> {
        let store = &self.artifact.store;
        if exists(&store.directory, &txn(&self.record.output))? {
            return Err(OwnershipError::Uncertain);
        }
        if store.read_output(
            OutputId(self.record.output),
            self.artifact.metadata.id(),
            &self.artifact.source,
        )? != self.record
            || identity(&open_at(
                &store.directory,
                &name(&self.record.output),
                false,
                false,
                false,
            )?)? != self.journal
        {
            return Err(OwnershipError::Identity);
        }
        Ok(())
    }
    fn validate(&self, control: &Control<'_>) -> Result<()> {
        control.check()?;
        self.current()?;
        let contract: [u8; 32] = Sha256::digest(
            self.artifact
                .metadata
                .to_json()
                .map_err(|_| OwnershipError::Content)?,
        )
        .into();
        if contract != self.record.container_contract {
            return Err(OwnershipError::Content);
        }
        let dir = bundle(
            &self.artifact.store.directory,
            &stage(&self.record.operation),
            &self.record,
            false,
        )?
        .unwrap();
        let stamp = self.record.stage.as_ref().unwrap();
        let mut meta = check_named(&dir, OUTPUT_MEMBERS[1], stamp.members[1], false)?;
        let bytes = bounded_read(&mut meta, MAX_METADATA_BYTES as u64)?;
        let parsed: OutputMetadata =
            serde_json::from_slice(&bytes).map_err(|_| OwnershipError::Content)?;
        let expected = OutputMetadata {
            version: 1,
            format: RawFormat::Raw,
            output: self.record.output,
            artifact: self.record.artifact,
            source: self.record.source,
            container_contract: contract,
            logical_bytes: self.record.logical_bytes,
            raw_sha256: self.record.raw_sha256.ok_or(OwnershipError::Content)?,
            verification: OutputVerification::LogicalSourceReadback,
        };
        if parsed != expected {
            return Err(OwnershipError::Content);
        }
        let mut raw = check_named(&dir, OUTPUT_MEMBERS[0], stamp.members[0], false)?;
        if raw.metadata().map_err(|_| OwnershipError::Io)?.len() != self.record.logical_bytes {
            return Err(OwnershipError::Content);
        }
        let mut input = vec![0; 1 << 20];
        let mut output = vec![0; 1 << 20];
        let mut offset = 0;
        let mut hash = Sha256::new();
        while offset < self.record.logical_bytes {
            control.check()?;
            let n = (self.record.logical_bytes - offset).min(input.len() as u64) as usize;
            self.artifact
                .disk
                .as_ref()
                .ok_or(OwnershipError::Transition)?
                .read_exact_at(offset, &mut input[..n])
                .map_err(|_| OwnershipError::Content)?;
            raw.read_exact(&mut output[..n])
                .map_err(|_| OwnershipError::Io)?;
            if input[..n] != output[..n] {
                return Err(OwnershipError::Content);
            }
            hash.update(&output[..n]);
            offset += n as u64;
        }
        if <[u8; 32]>::from(hash.finalize()) != expected.raw_sha256
            || raw.metadata().map_err(|_| OwnershipError::Io)?.len() != self.record.logical_bytes
        {
            return Err(OwnershipError::Content);
        }
        drop(raw);
        drop(meta);
        let mut meta = check_named(&dir, OUTPUT_MEMBERS[1], stamp.members[1], false)?;
        if bounded_read(&mut meta, MAX_METADATA_BYTES as u64)? != bytes {
            return Err(OwnershipError::Content);
        }
        check_current(
            &self.artifact.store,
            &self.artifact.record,
            &self.artifact.source,
            &self.artifact.metadata,
            self.artifact.journal,
            control,
        )?;
        self.current()?;
        drop(bundle(
            &self.artifact.store.directory,
            &stage(&self.record.operation),
            &self.record,
            false,
        )?);
        control.check()
    }
    pub fn publish(
        self,
        destination: PublicationDirectory,
        options: RetainedOptions,
    ) -> PublicationReport {
        self.publish_hook(destination, options, |_, _| Ok(()))
    }
    fn publish_hook(
        mut self,
        destination: PublicationDirectory,
        options: RetainedOptions,
        mut hook: impl FnMut(OutputState, &str) -> Result<()>,
    ) -> PublicationReport {
        let mut report = PublicationReport::default();
        let result = (|| {
            let control = Control::new(&options)?;
            destination.validate(&self.artifact.store)?;
            let component = OutputId(self.record.output).bundle_name();
            if exists(&destination.0.directory, &component)? {
                return Err(OwnershipError::Exists);
            }
            self.validate(&control)?;
            hook(self.record.state, "validated")?;
            self.current()?;
            let dir = bundle(
                &self.artifact.store.directory,
                &stage(&self.record.operation),
                &self.record,
                false,
            )?
            .unwrap();
            for (i, n) in OUTPUT_MEMBERS.iter().enumerate() {
                check_named(
                    &dir,
                    n,
                    self.record.stage.as_ref().unwrap().members[i],
                    false,
                )?
                .sync_all()
                .map_err(|_| OwnershipError::Io)?;
                hook(self.record.state, "member_synced")?;
            }
            dir.sync_all().map_err(|_| OwnershipError::Io)?;
            drop(dir);
            hook(self.record.state, "bundle_synced")?;
            control.check()?;
            destination.validate(&self.artifact.store)?;
            self.record.version = 2;
            self.record.publication_parent = Some(destination.0.identity);
            self.record.state = OutputState::PublishIntent;
            self.record.sequence = 7;
            commit_output(&self.artifact.store, &self.record, false, &mut hook)?;
            control.check()?;
            hook(self.record.state, "before_rename")?;
            control.check()?;
            destination.validate(&self.artifact.store)?;
            drop(bundle(
                &self.artifact.store.directory,
                &stage(&self.record.operation),
                &self.record,
                false,
            )?);
            let from = cstr(&stage(&self.record.operation))?;
            let to = cstr(&component)?;
            // SAFETY: locked pinned parent descriptors, generated components;
            // atomic whole-bundle rename rejects every existing target type.
            if unsafe {
                libc::renameat2(
                    self.artifact.store.directory.as_raw_fd(),
                    from.as_ptr(),
                    destination.0.directory.as_raw_fd(),
                    to.as_ptr(),
                    libc::RENAME_NOREPLACE,
                )
            } != 0
            {
                return Err(
                    if std::io::Error::last_os_error().raw_os_error() == Some(libc::EEXIST) {
                        OwnershipError::Exists
                    } else {
                        OwnershipError::Uncertain
                    },
                );
            }
            report.rename_observed = true;
            hook(self.record.state, "bundle_renamed")?;
            // After rename, finish both directory syncs before observing cancellation.
            self.artifact
                .store
                .directory
                .sync_all()
                .map_err(|_| OwnershipError::Uncertain)?;
            hook(self.record.state, "source_directory_synced")?;
            destination
                .0
                .directory
                .sync_all()
                .map_err(|_| OwnershipError::Uncertain)?;
            report.directories_synced = true;
            hook(self.record.state, "destination_directory_synced")?;
            control.check()?;
            if exists(
                &self.artifact.store.directory,
                &stage(&self.record.operation),
            )? {
                return Err(OwnershipError::Identity);
            }
            drop(bundle(
                &destination.0.directory,
                &component,
                &self.record,
                false,
            )?);
            self.record.state = OutputState::Published;
            self.record.sequence = 8;
            commit_output(&self.artifact.store, &self.record, false, &mut hook)?;
            control.check()
        })();
        report.primary_error = result.err();
        match self.artifact.store.assess_publication(
            OutputId(self.record.output),
            self.artifact.metadata.id(),
            &self.artifact.source,
            &destination,
        ) {
            Ok(r) => report.recovery = Some(r),
            Err(e) => report.assessment_error = Some(e),
        }
        report
    }
}
impl JobStore {
    /// Namespace observations require the original destination identity once intent
    /// exists. No content verification or mutating authority is returned.
    pub fn assess_publication(
        &self,
        output: OutputId,
        artifact: ArtifactId,
        source: &SourceSelection,
        destination: &PublicationDirectory,
    ) -> Result<PublicationRecovery> {
        destination.validate(self)?;
        let r = self.read_output(output, artifact, source)?;
        if r.publication_parent
            .is_some_and(|p| p != destination.0.identity)
        {
            return Err(OwnershipError::Identity);
        }
        Ok(PublicationRecovery {
            output: self.output_recovery(&r)?,
            staged: location(&self.directory, &stage(&r.operation), &r)?,
            published: location(&destination.0.directory, &output.bundle_name(), &r)?,
        })
    }
    /// Explicitly delete only stamped local output members. Uncertain publication,
    /// pending transactions and unstamped stages are deliberately ineligible.
    /// Published cleanup requires the original locked destination; source bytes need
    /// not still exist. No recursive removal, remote action or automatic Drop cleanup.
    pub fn cleanup_output(
        &mut self,
        output: OutputId,
        artifact: ArtifactId,
        source: &SourceSelection,
        destination: Option<&PublicationDirectory>,
        options: RetainedOptions,
    ) -> Result<()> {
        self.cleanup_output_hook(
            output,
            artifact,
            source,
            destination,
            options,
            |_, _| Ok(()),
        )
    }
    fn cleanup_output_hook(
        &mut self,
        output: OutputId,
        artifact: ArtifactId,
        source: &SourceSelection,
        destination: Option<&PublicationDirectory>,
        options: RetainedOptions,
        mut hook: impl FnMut(OutputState, &str) -> Result<()>,
    ) -> Result<()> {
        let result = (|| {
            let control = Control::new(&options)?;
            let mut r = self.read_output(output, artifact, source)?;
            if exists(&self.directory, &txn(&r.output))? {
                return Err(OwnershipError::Uncertain);
            }
            if !cleanup_allowed(r.state)
                && !matches!(r.state, OutputState::CleanupIntent | OutputState::Cleaned)
            {
                return Err(OwnershipError::Transition);
            }
            let parent = if let Some(expected) = r.publication_parent {
                let dest = destination.ok_or(OwnershipError::Identity)?;
                dest.validate(self)?;
                if dest.0.identity != expected || exists(&self.directory, &stage(&r.operation))? {
                    return Err(OwnershipError::Identity);
                }
                &dest.0.directory
            } else {
                if destination.is_some() {
                    return Err(OwnershipError::Identity);
                }
                &self.directory
            };
            let component = if r.publication_parent.is_some() {
                output.bundle_name()
            } else {
                stage(&r.operation)
            };
            if r.state == OutputState::Cleaned {
                return if exists(parent, &component)? {
                    Err(OwnershipError::Identity)
                } else {
                    Ok(())
                };
            }
            let partial = r.state == OutputState::CleanupIntent;
            let directory = bundle(parent, &component, &r, partial)?;
            control.check()?;
            if !partial {
                r.version = 2;
                r.cleanup_from = Some(r.state);
                r.state = OutputState::CleanupIntent;
                r.sequence += 1;
                commit_output(self, &r, false, &mut hook)?;
            }
            if let Some(dir) = directory {
                let stamp = r.stage.as_ref().unwrap();
                for (i, n) in OUTPUT_MEMBERS.iter().enumerate() {
                    control.check()?;
                    check_named(parent, &component, stamp.directory, true)?;
                    only_members(&dir)?;
                    if exists(&dir, n)? {
                        let mut file = check_named(&dir, n, stamp.members[i], false)?;
                        if i == 2 && bounded_read(&mut file, 16)? != r.operation {
                            return Err(OwnershipError::Identity);
                        }
                        drop(file);
                        unlink(&dir, n, false)?;
                        hook(
                            r.state,
                            match i {
                                0 => "cleanup_raw",
                                1 => "cleanup_metadata",
                                _ => "cleanup_marker",
                            },
                        )?;
                    }
                }
                dir.sync_all().map_err(|_| OwnershipError::Uncertain)?;
                hook(r.state, "cleanup_members_synced")?;
                control.check()?;
                check_named(parent, &component, stamp.directory, true)?;
                unlink(parent, &component, true)?;
                hook(r.state, "cleanup_directory_removed")?;
            }
            // Also sync after recovering an already absent directory: visibility
            // after process loss is insufficient to acknowledge durable removal.
            parent.sync_all().map_err(|_| OwnershipError::Uncertain)?;
            hook(r.state, "cleanup_parent_synced")?;
            control.check()?;
            r.state = OutputState::Cleaned;
            r.sequence += 1;
            commit_output(self, &r, false, &mut hook)
        })();
        if matches!(result, Err(OwnershipError::Io | OwnershipError::Uncertain)) {
            self.poisoned = true;
        }
        result
    }
}
#[cfg(test)]
mod tests;
