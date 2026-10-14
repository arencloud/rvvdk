//! Explicit local intent reconciliation. Neither assessment nor a saved report
//! supplies authority; every invocation owns both locks and revalidates everything.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconciliationOrigin {
    Staged,
    AlreadyRenamed,
}
#[derive(Debug, Default, Serialize)]
pub struct ReconciliationReport {
    pub origin: Option<ReconciliationOrigin>,
    pub logical_bytes_verified: u64,
    pub content_verified: bool,
    pub bundle_synced: bool,
    pub intent_synced: bool,
    pub rename_observed: bool,
    pub directories_synced: bool,
    pub acknowledgment_durable: bool,
    pub primary_error: Option<OwnershipError>,
    pub assessment_error: Option<OwnershipError>,
    pub recovery: Option<PublicationRecovery>,
}
impl ReconciliationReport {
    pub fn is_success(&self) -> bool {
        self.primary_error.is_none()
            && self.assessment_error.is_none()
            && self.content_verified
            && self.bundle_synced
            && self.intent_synced
            && self.directories_synced
            && self.acknowledgment_durable
            && matches!(
                (self.origin, self.rename_observed),
                (Some(ReconciliationOrigin::Staged), true)
                    | (Some(ReconciliationOrigin::AlreadyRenamed), false)
            )
            && self.recovery.as_ref().is_some_and(|r| {
                r.output.state == OutputState::Published
                    && r.output.sequence == 8
                    && !r.output.pending_transaction
                    && r.staged == OutputLocation::Absent
                    && r.published == OutputLocation::Owned
            })
    }
}
fn origin(staged: OutputLocation, published: OutputLocation) -> Result<ReconciliationOrigin> {
    match (staged, published) {
        (OutputLocation::Owned, OutputLocation::Absent) => Ok(ReconciliationOrigin::Staged),
        (OutputLocation::Absent, OutputLocation::Owned) => Ok(ReconciliationOrigin::AlreadyRenamed),
        _ => Err(OwnershipError::Identity),
    }
}
// A statically dispatched private seam allows tests to return actual syscall-style
// errors before an operation. Production always executes the OS implementation.
trait Io {
    fn sync(&mut self, file: &File, point: &'static str) -> std::io::Result<()>;
    fn rename(&mut self, from: &File, old: &str, to: &File, new: &str) -> Result<()>;
}
struct SystemIo;
impl Io for SystemIo {
    fn sync(&mut self, file: &File, _: &'static str) -> std::io::Result<()> {
        file.sync_all()
    }
    fn rename(&mut self, from: &File, old: &str, to: &File, new: &str) -> Result<()> {
        let old = cstr(old)?;
        let new = cstr(new)?;
        // SAFETY: pinned locked directories and generated single-component names.
        if unsafe {
            libc::renameat2(
                from.as_raw_fd(),
                old.as_ptr(),
                to.as_raw_fd(),
                new.as_ptr(),
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
        Ok(())
    }
}
fn source_current(a: &RetainedArtifact) -> Result<()> {
    let (record, metadata, _, journal) = members(&a.store, a.metadata.id(), &a.source)?;
    if record != a.record || metadata != a.metadata || journal != a.journal {
        return Err(OwnershipError::Identity);
    }
    Ok(())
}
impl CheckedOutput<'_> {
    fn reconcile_current(
        &self,
        destination: &PublicationDirectory,
        expected: ReconciliationOrigin,
    ) -> Result<()> {
        self.current()?;
        source_current(self.artifact)?;
        destination.validate(&self.artifact.store)?;
        if self.record.publication_parent != Some(destination.0.identity) {
            return Err(OwnershipError::Identity);
        }
        let now = origin(
            location(
                &self.artifact.store.directory,
                &stage(&self.record.operation),
                self.record,
            )?,
            location(
                &destination.0.directory,
                &OutputId(self.record.output).bundle_name(),
                self.record,
            )?,
        )?;
        if now != expected {
            return Err(OwnershipError::Identity);
        }
        Ok(())
    }
}
impl JobStore {
    /// Explicitly finish only an unambiguous output PublishIntent. Requires the
    /// original retained source, store and destination, with no pending transaction.
    /// Consumes both locks; synchronous callers must await completion. No deletion,
    /// remote request, automatic retry, transaction repair or Published attestation.
    pub fn reconcile_publication(
        self,
        output: OutputId,
        artifact: ArtifactId,
        source: SourceSelection,
        destination: PublicationDirectory,
        options: RetainedOptions,
    ) -> ReconciliationReport {
        self.reconcile_hook(
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
    fn reconcile_hook(
        self,
        output: OutputId,
        artifact: ArtifactId,
        source: SourceSelection,
        destination: PublicationDirectory,
        options: RetainedOptions,
        io: &mut impl Io,
        mut hook: impl FnMut(OutputState, &str) -> Result<()>,
    ) -> ReconciliationReport {
        let mut report = ReconciliationReport::default();
        // Keep the original locked store available for assessment even when source
        // admission fails. Once admitted, the artifact owns it without reopening.
        let mut store = Some(self);
        let mut admitted = None;
        let result = (|| {
            let control = Control::new(&options)?;
            let s = store.as_ref().unwrap();
            let mut record = s.read_output(output, artifact, &source)?;
            if record.state != OutputState::PublishIntent {
                return Err(OwnershipError::Transition);
            }
            if exists(&s.directory, &txn(&record.output))? {
                return Err(OwnershipError::Uncertain);
            }
            let observed = s.assess_publication(output, artifact, &source, &destination)?;
            let selected = origin(observed.staged, observed.published)?;
            report.origin = Some(selected);
            let journal = identity(&open_at(
                &s.directory,
                &name(&record.output),
                false,
                false,
                false,
            )?)?;
            hook(record.state, "before_admission")?;
            let (source_record, metadata, disk, source_journal) =
                admit(s, artifact, &source, &control)?;
            admitted = Some(RetainedArtifact {
                disk: Some(disk),
                store: store.take().unwrap(),
                record: source_record,
                source: source.clone(),
                metadata,
                journal: source_journal,
            });
            let a = admitted.as_ref().unwrap();
            hook(record.state, "admitted")?;
            let checked = CheckedOutput {
                artifact: a,
                record: &record,
                journal,
            };
            checked.reconcile_current(&destination, selected)?;
            let staged = stage(&record.operation);
            let final_name = output.bundle_name();
            let (parent, component) = match selected {
                ReconciliationOrigin::Staged => (&a.store.directory, &staged),
                ReconciliationOrigin::AlreadyRenamed => (&destination.0.directory, &final_name),
            };
            checked.validate_at(
                parent,
                component,
                &control,
                &mut report.logical_bytes_verified,
                &mut hook,
            )?;
            checked.reconcile_current(&destination, selected)?;
            report.content_verified = true;
            hook(record.state, "validated")?;
            checked.reconcile_current(&destination, selected)?;
            let dir = bundle(parent, component, &record, false)?.unwrap();
            for (i, point) in ["raw_synced", "metadata_synced", "marker_synced"]
                .into_iter()
                .enumerate()
            {
                control.check()?;
                drop(bundle(parent, component, &record, false)?);
                let f = check_named(
                    &dir,
                    OUTPUT_MEMBERS[i],
                    record.stage.as_ref().unwrap().members[i],
                    false,
                )?;
                io.sync(&f, point).map_err(|_| OwnershipError::Io)?;
                hook(record.state, point)?;
            }
            io.sync(&dir, "bundle_synced")
                .map_err(|_| OwnershipError::Io)?;
            drop(dir);
            report.bundle_synced = true;
            hook(record.state, "bundle_synced")?;
            control.check()?;
            checked.reconcile_current(&destination, selected)?;
            let intent = check_named(&a.store.directory, &name(&record.output), journal, false)?;
            io.sync(&intent, "intent_file_synced")
                .map_err(|_| OwnershipError::Io)?;
            hook(record.state, "intent_file_synced")?;
            io.sync(&a.store.directory, "intent_directory_synced")
                .map_err(|_| OwnershipError::Io)?;
            hook(record.state, "intent_directory_synced")?;
            checked.reconcile_current(&destination, selected)?;
            report.intent_synced = true;
            hook(record.state, "intent_checked")?;
            control.check()?;
            hook(record.state, "before_rename")?;
            control.check()?;
            if selected == ReconciliationOrigin::Staged
                && exists(&destination.0.directory, &final_name)?
            {
                return Err(OwnershipError::Exists);
            }
            checked.reconcile_current(&destination, selected)?;
            hook(record.state, "rename_checked")?;
            control.check()?;
            if selected == ReconciliationOrigin::Staged {
                io.rename(
                    &a.store.directory,
                    &staged,
                    &destination.0.directory,
                    &final_name,
                )?;
                report.rename_observed = true;
                hook(record.state, "bundle_renamed")?;
            }
            // No control observation between a successful rename / entry to this
            // phase and completion of both parent syncs. An I/O failure still stops.
            hook(record.state, "before_parent_sync")?;
            io.sync(&a.store.directory, "source_directory_synced")
                .map_err(|_| OwnershipError::Uncertain)?;
            hook(record.state, "source_directory_synced")?;
            io.sync(&destination.0.directory, "destination_directory_synced")
                .map_err(|_| OwnershipError::Uncertain)?;
            report.directories_synced = true;
            hook(record.state, "destination_directory_synced")?;
            control.check()?;
            hook(record.state, "before_acknowledgment")?;
            control.check()?;
            checked.reconcile_current(&destination, ReconciliationOrigin::AlreadyRenamed)?;
            hook(record.state, "acknowledgment_checked")?;
            control.check()?;
            record.state = OutputState::Published;
            record.sequence = 8;
            commit_output(&a.store, &record, false, &mut hook)?;
            report.acknowledgment_durable = true;
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
        report
    }
}
#[cfg(test)]
mod tests;
