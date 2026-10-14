//! Finish only an exact intent-7 / Published-8 transaction pair after fresh checks.
use super::*;

#[derive(Debug, Default, Serialize)]
pub struct PublicationAcknowledgmentRecoveryReport {
    pub logical_bytes_verified: u64,
    pub content_verified: bool,
    pub bundle_synced: bool,
    pub intent_synced: bool,
    pub directories_synced: bool,
    pub transaction_synced: bool,
    pub transaction_promoted: bool,
    pub acknowledgment_durable: bool,
    pub bindings_rechecked: bool,
    pub primary_error: Option<OwnershipError>,
    pub assessment_error: Option<OwnershipError>,
    pub recovery: Option<PublicationRecovery>,
}
impl PublicationAcknowledgmentRecoveryReport {
    pub fn is_success(&self) -> bool {
        self.primary_error.is_none()
            && self.assessment_error.is_none()
            && self.content_verified
            && self.bundle_synced
            && self.intent_synced
            && self.directories_synced
            && self.transaction_synced
            && self.transaction_promoted
            && self.acknowledgment_durable
            && self.bindings_rechecked
            && self.recovery.as_ref().is_some_and(|r| {
                r.output.state == OutputState::Published
                    && r.output.sequence == 8
                    && !r.output.pending_transaction
                    && eligible(r.staged, r.published)
            })
    }
}
fn eligible(stage: OutputLocation, final_bundle: OutputLocation) -> bool {
    stage == OutputLocation::Absent && final_bundle == OutputLocation::Owned
}
struct PinnedRecord {
    file: File,
    bytes: Vec<u8>,
    identity: Identity,
}
impl PinnedRecord {
    fn open(store: &JobStore, component: &str) -> Result<Self> {
        let mut file = open_at(&store.directory, component, false, false, false)?;
        private(&file, false)?;
        let identity = identity(&file)?;
        let bytes = bounded_read(&mut file, MAX_RECORD)?;
        Ok(Self {
            file,
            bytes,
            identity,
        })
    }
    fn current(&self, store: &JobStore, component: &str) -> Result<()> {
        let mut file = check_named(&store.directory, component, self.identity, false)?;
        if bounded_read(&mut file, MAX_RECORD)? != self.bytes {
            return Err(OwnershipError::Identity);
        }
        Ok(())
    }
}
struct AdmittedPair {
    old: PinnedRecord,
    candidate: PinnedRecord,
    record: OutputRecord,
}
impl AdmittedPair {
    fn open(
        store: &JobStore,
        output: OutputId,
        artifact: ArtifactId,
        source: &SourceSelection,
    ) -> Result<Self> {
        store.ready()?;
        let old = PinnedRecord::open(store, &name(&output.0))?;
        let record = store.decode_output(&old.bytes, output, artifact, source)?;
        if record.state != OutputState::PublishIntent {
            return Err(OwnershipError::Transition);
        }
        if !exists(&store.directory, &txn(&output.0))? {
            return Err(OwnershipError::Transition);
        }
        let candidate = PinnedRecord::open(store, &txn(&output.0))?;
        let seen = store.decode_output(&candidate.bytes, output, artifact, source)?;
        let mut expected = record.clone();
        expected.state = OutputState::Published;
        expected.sequence = 8;
        if seen != expected || old.identity == candidate.identity {
            return Err(OwnershipError::Record);
        }
        Ok(Self {
            old,
            candidate,
            record,
        })
    }
    fn namespace(&self, store: &JobStore, destination: &PublicationDirectory) -> Result<()> {
        destination.validate(store)?;
        if self.record.publication_parent != Some(destination.0.identity)
            || !eligible(
                location(
                    &store.directory,
                    &stage(&self.record.operation),
                    &self.record,
                )?,
                location(
                    &destination.0.directory,
                    &OutputId(self.record.output).bundle_name(),
                    &self.record,
                )?,
            )
        {
            return Err(OwnershipError::Identity);
        }
        Ok(())
    }
    fn current(
        &self,
        artifact: &RetainedArtifact,
        destination: &PublicationDirectory,
        promoted: bool,
    ) -> Result<()> {
        let store = &artifact.store;
        store.ready()?;
        source_current(artifact)?;
        self.namespace(store, destination)?;
        if promoted {
            self.candidate.current(store, &name(&self.record.output))?;
            if exists(&store.directory, &txn(&self.record.output))? {
                return Err(OwnershipError::Uncertain);
            }
        } else {
            self.old.current(store, &name(&self.record.output))?;
            self.candidate.current(store, &txn(&self.record.output))?;
        }
        Ok(())
    }
}
trait Io {
    fn sync(&mut self, file: &File, point: &'static str) -> std::io::Result<()>;
    fn promote(&mut self, store: &File, from: &str, to: &str) -> Result<()>;
}
struct SystemIo;
impl Io for SystemIo {
    fn sync(&mut self, file: &File, _: &'static str) -> std::io::Result<()> {
        file.sync_all()
    }
    fn promote(&mut self, store: &File, from: &str, to: &str) -> Result<()> {
        let from = cstr(from)?;
        let to = cstr(to)?;
        // SAFETY: pinned store fd and generated components. The admitted pair was
        // checked immediately before this call under the cooperating-writer lock.
        if unsafe {
            libc::renameat2(
                store.as_raw_fd(),
                from.as_ptr(),
                store.as_raw_fd(),
                to.as_ptr(),
                0,
            )
        } != 0
        {
            return Err(OwnershipError::Uncertain);
        }
        Ok(())
    }
}
impl JobStore {
    /// Complete only an exact PublishIntent / Published transaction pair after
    /// fresh source and full RAW verification. Consumes both original locks.
    /// No bundle rename, byte rewrite, cleanup, rollback or automatic retry.
    /// Await synchronous completion; the report never proves earlier success.
    pub fn recover_publication_acknowledgment(
        self,
        output: OutputId,
        artifact: ArtifactId,
        source: SourceSelection,
        destination: PublicationDirectory,
        options: RetainedOptions,
    ) -> PublicationAcknowledgmentRecoveryReport {
        self.acknowledgment_hook(
            output,
            artifact,
            source,
            destination,
            options,
            &mut SystemIo,
            |_, _| Ok(()),
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn acknowledgment_hook(
        self,
        output: OutputId,
        artifact: ArtifactId,
        source: SourceSelection,
        destination: PublicationDirectory,
        options: RetainedOptions,
        io: &mut impl Io,
        mut hook: impl FnMut(OutputState, &str) -> Result<()>,
    ) -> PublicationAcknowledgmentRecoveryReport {
        let mut report = PublicationAcknowledgmentRecoveryReport::default();
        let control = Control::new(&options);
        let mut store = Some(self);
        let mut admitted = None;
        let mut pair = None;
        let result = (|| {
            let control = control.as_ref().map_err(|e| *e)?;
            let s = store.as_ref().unwrap();
            pair = Some(AdmittedPair::open(s, output, artifact, &source)?);
            let pair = pair.as_ref().unwrap();
            pair.namespace(s, &destination)?;
            hook(OutputState::PublishIntent, "before_admission")?;
            let (record, metadata, disk, journal) = admit(s, artifact, &source, control)?;
            admitted = Some(RetainedArtifact {
                disk: Some(disk),
                store: store.take().unwrap(),
                record,
                source: source.clone(),
                metadata,
                journal,
            });
            let a = admitted.as_ref().unwrap();
            hook(OutputState::PublishIntent, "admitted")?;
            let current = || pair.current(a, &destination, false);
            current()?;
            let parent = &destination.0.directory;
            let component = output.bundle_name();
            CheckedOutput {
                artifact: a,
                record: &pair.record,
                journal: pair.old.identity,
            }
            .validate_bound_at(
                parent,
                &component,
                control,
                &mut report.logical_bytes_verified,
                &mut hook,
                &current,
            )?;
            current()?;
            report.content_verified = true;
            hook(OutputState::PublishIntent, "validated")?;
            current()?;
            let dir = bundle(parent, &component, &pair.record, false)?.unwrap();
            for (i, point) in ["raw_synced", "metadata_synced", "marker_synced"]
                .into_iter()
                .enumerate()
            {
                drop(bundle(parent, &component, &pair.record, false)?);
                let file = check_named(
                    &dir,
                    OUTPUT_MEMBERS[i],
                    pair.record.stage.as_ref().unwrap().members[i],
                    false,
                )?;
                control.check()?;
                io.sync(&file, point).map_err(|_| OwnershipError::Io)?;
                hook(OutputState::PublishIntent, point)?;
            }
            drop(bundle(parent, &component, &pair.record, false)?);
            control.check()?;
            io.sync(&dir, "bundle_synced")
                .map_err(|_| OwnershipError::Io)?;
            drop(dir);
            report.bundle_synced = true;
            hook(OutputState::PublishIntent, "bundle_synced")?;
            current()?;
            control.check()?;
            io.sync(&pair.old.file, "intent_file_synced")
                .map_err(|_| OwnershipError::Io)?;
            hook(OutputState::PublishIntent, "intent_file_synced")?;
            io.sync(&a.store.directory, "intent_directory_synced")
                .map_err(|_| OwnershipError::Uncertain)?;
            report.intent_synced = true;
            hook(OutputState::PublishIntent, "intent_directory_synced")?;
            control.check()?;
            current()?;
            control.check()?;
            io.sync(&a.store.directory, "store_namespace_synced")
                .map_err(|_| OwnershipError::Uncertain)?;
            hook(OutputState::PublishIntent, "store_namespace_synced")?;
            io.sync(parent, "destination_synced")
                .map_err(|_| OwnershipError::Uncertain)?;
            report.directories_synced = true;
            hook(OutputState::PublishIntent, "destination_synced")?;
            control.check()?;
            hook(OutputState::PublishIntent, "transaction_checked")?;
            current()?;
            control.check()?;
            io.sync(&pair.candidate.file, "transaction_synced")
                .map_err(|_| OwnershipError::Io)?;
            report.transaction_synced = true;
            hook(OutputState::PublishIntent, "transaction_synced")?;
            hook(OutputState::PublishIntent, "before_promotion")?;
            current()?;
            control.check()?;
            // No callback or control gap after the final pair check. After rename,
            // complete the store sync before observing cancellation (unless I/O fails).
            io.promote(&a.store.directory, &txn(&output.0), &name(&output.0))
                .map_err(|_| OwnershipError::Uncertain)?;
            report.transaction_promoted = true;
            hook(OutputState::Published, "transaction_promoted")?;
            io.sync(&a.store.directory, "acknowledgment_directory_synced")
                .map_err(|_| OwnershipError::Uncertain)?;
            report.acknowledgment_durable = true;
            hook(OutputState::Published, "acknowledgment_directory_synced")?;
            control.check()?;
            pair.current(a, &destination, true)?;
            control.check()
        })();
        report.primary_error = result.err();
        let s = store
            .as_ref()
            .or_else(|| admitted.as_ref().map(|a| &a.store))
            .unwrap();
        let assessment = (|| {
            hook(OutputState::Published, "before_assessment")?;
            s.assess_publication(output, artifact, &source, &destination)
        })();
        match assessment {
            Ok(value) => report.recovery = Some(value),
            Err(error) => report.assessment_error = Some(error),
        }
        if report.primary_error.is_none() {
            report.primary_error = (|| {
                hook(OutputState::Published, "after_assessment")?;
                pair.as_ref()
                    .unwrap()
                    .current(admitted.as_ref().unwrap(), &destination, true)?;
                report.bindings_rechecked = true;
                hook(OutputState::Published, "bindings_rechecked")?;
                control.as_ref().map_err(|e| *e)?.check()
            })()
            .err();
        }
        report
    }
}
#[cfg(test)]
mod tests;
