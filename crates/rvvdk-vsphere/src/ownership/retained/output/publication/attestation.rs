//! Fresh verification and persistence observations for Published-at-entry.
//! The existing journal is never rewritten and no earlier success is inferred.
use super::*;

#[derive(Debug, Default, Serialize)]
pub struct PublicationAttestationReport {
    pub logical_bytes_verified: u64,
    pub content_verified: bool,
    pub bundle_synced: bool,
    pub directories_synced: bool,
    pub published_record_synced: bool,
    pub bindings_rechecked: bool,
    pub primary_error: Option<OwnershipError>,
    pub assessment_error: Option<OwnershipError>,
    pub recovery: Option<PublicationRecovery>,
}
impl PublicationAttestationReport {
    pub fn is_success(&self) -> bool {
        self.primary_error.is_none()
            && self.assessment_error.is_none()
            && self.content_verified
            && self.bundle_synced
            && self.directories_synced
            && self.published_record_synced
            && self.bindings_rechecked
            && self.recovery.as_ref().is_some_and(|r| {
                r.output.state == OutputState::Published
                    && r.output.sequence == 8
                    && !r.output.pending_transaction
                    && eligible(r.staged, r.published)
            })
    }
}
fn eligible(staged: OutputLocation, published: OutputLocation) -> bool {
    staged == OutputLocation::Absent && published == OutputLocation::Owned
}
trait Io {
    fn sync(&mut self, file: &File, point: &'static str) -> std::io::Result<()>;
}
struct SystemIo;
impl Io for SystemIo {
    fn sync(&mut self, file: &File, _: &'static str) -> std::io::Result<()> {
        file.sync_all()
    }
}
impl CheckedOutput<'_> {
    fn attest_current(&self, destination: &PublicationDirectory) -> Result<()> {
        self.current()?;
        source_current(self.artifact)?;
        destination.validate(&self.artifact.store)?;
        if self.record.publication_parent != Some(destination.0.identity)
            || !eligible(
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
            )
        {
            return Err(OwnershipError::Identity);
        }
        Ok(())
    }
}
impl JobStore {
    /// Freshly verify and sync only an unambiguous Published output. Requires the
    /// original retained source and parents, with no source/output transaction.
    /// Consumes both locks through final checks; synchronous callers must await it.
    /// Does not rewrite the journal, infer earlier success, rename, repair or delete.
    pub fn attest_publication(
        self,
        output: OutputId,
        artifact: ArtifactId,
        source: SourceSelection,
        destination: PublicationDirectory,
        options: RetainedOptions,
    ) -> PublicationAttestationReport {
        self.attest_hook(
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
    fn attest_hook(
        self,
        output: OutputId,
        artifact: ArtifactId,
        source: SourceSelection,
        destination: PublicationDirectory,
        options: RetainedOptions,
        io: &mut impl Io,
        mut hook: impl FnMut(OutputState, &str) -> Result<()>,
    ) -> PublicationAttestationReport {
        let mut report = PublicationAttestationReport::default();
        let control = Control::new(&options);
        // Failed borrowed admission must still retain the locked store for assessment.
        let mut store = Some(self);
        let mut admitted = None;
        let mut binding = None;
        let result = (|| {
            let control = control.as_ref().map_err(|e| *e)?;
            let s = store.as_ref().unwrap();
            let record = s.read_output(output, artifact, &source)?;
            if record.state != OutputState::Published {
                return Err(OwnershipError::Transition);
            }
            if exists(&s.directory, &txn(&record.output))? {
                return Err(OwnershipError::Uncertain);
            }
            let observed = s.assess_publication(output, artifact, &source, &destination)?;
            if !eligible(observed.staged, observed.published) {
                return Err(OwnershipError::Identity);
            }
            let journal = identity(&open_at(
                &s.directory,
                &name(&record.output),
                false,
                false,
                false,
            )?)?;
            hook(record.state, "before_admission")?;
            let (source_record, metadata, disk, source_journal) =
                admit(s, artifact, &source, control)?;
            admitted = Some(RetainedArtifact {
                disk: Some(disk),
                store: store.take().unwrap(),
                record: source_record,
                source: source.clone(),
                metadata,
                journal: source_journal,
            });
            binding = Some((record, journal));
            let a = admitted.as_ref().unwrap();
            let (record, journal) = binding.as_ref().unwrap();
            let checked = CheckedOutput {
                artifact: a,
                record,
                journal: *journal,
            };
            hook(record.state, "admitted")?;
            checked.attest_current(&destination)?;
            let parent = &destination.0.directory;
            let component = output.bundle_name();
            checked.validate_at(
                parent,
                &component,
                control,
                &mut report.logical_bytes_verified,
                &mut hook,
            )?;
            checked.attest_current(&destination)?;
            report.content_verified = true;
            hook(record.state, "validated")?;
            checked.attest_current(&destination)?;
            control.check()?;
            let dir = bundle(parent, &component, record, false)?.unwrap();
            for (i, point) in ["raw_synced", "metadata_synced", "marker_synced"]
                .into_iter()
                .enumerate()
            {
                drop(bundle(parent, &component, record, false)?);
                let f = check_named(
                    &dir,
                    OUTPUT_MEMBERS[i],
                    record.stage.as_ref().unwrap().members[i],
                    false,
                )?;
                control.check()?;
                io.sync(&f, point).map_err(|_| OwnershipError::Io)?;
                hook(record.state, point)?;
            }
            drop(bundle(parent, &component, record, false)?);
            control.check()?;
            io.sync(&dir, "bundle_synced")
                .map_err(|_| OwnershipError::Io)?;
            drop(dir);
            report.bundle_synced = true;
            hook(record.state, "bundle_synced")?;
            checked.attest_current(&destination)?;
            hook(record.state, "parents_checked")?;
            control.check()?;
            // Paired syncs defer cancellation; an actual I/O failure still stops.
            io.sync(&a.store.directory, "store_namespace_synced")
                .map_err(|_| OwnershipError::Uncertain)?;
            hook(record.state, "store_namespace_synced")?;
            io.sync(parent, "destination_synced")
                .map_err(|_| OwnershipError::Uncertain)?;
            report.directories_synced = true;
            hook(record.state, "destination_synced")?;
            control.check()?;
            checked.attest_current(&destination)?;
            let file = check_named(&a.store.directory, &name(&record.output), *journal, false)?;
            hook(record.state, "record_checked")?;
            control.check()?;
            io.sync(&file, "published_file_synced")
                .map_err(|_| OwnershipError::Uncertain)?;
            hook(record.state, "published_file_synced")?;
            io.sync(&a.store.directory, "published_directory_synced")
                .map_err(|_| OwnershipError::Uncertain)?;
            report.published_record_synced = true;
            hook(record.state, "published_directory_synced")?;
            control.check()?;
            checked.attest_current(&destination)?;
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
        // Assessment cannot detect a late source transaction or journal replacement.
        // Recheck both bindings even when assessment failed, without hiding its error.
        if report.primary_error.is_none() {
            let final_check = (|| {
                hook(OutputState::Published, "after_assessment")?;
                let (record, journal) = binding.as_ref().unwrap();
                CheckedOutput {
                    artifact: admitted.as_ref().unwrap(),
                    record,
                    journal: *journal,
                }
                .attest_current(&destination)?;
                report.bindings_rechecked = true;
                hook(OutputState::Published, "bindings_rechecked")?;
                control.as_ref().map_err(|e| *e)?.check()
            })();
            report.primary_error = final_check.err();
        }
        report
    }
}
#[cfg(test)]
mod tests;
