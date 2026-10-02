//! Awaited composition. Success retains source and published RAW; no automatic cleanup.
use crate::{
    Credentials, Error, OwnedTransferOptions, OwnedTransferReport,
    contract::{ArtifactId, SourceSelection},
    ownership::{
        JobStore, OutputId, OutputReport, OwnershipError, PublicationDirectory, PublicationReport,
        RetainedArtifact, RetainedOptions, VerifiedOutput,
    },
    transfer_owned_artifact,
};
use rvvdk_datamover::{CopyEvent, CopyOptions};
use serde::Serialize;
use std::time::{Duration, Instant};
#[derive(Clone)]
pub struct PipelineOptions {
    pub transfer: OwnedTransferOptions,
    /// One shared budget for admission, conversion and publication after export.
    pub local_timeout: Duration,
    pub copy: CopyOptions,
}
impl Default for PipelineOptions {
    fn default() -> Self {
        Self {
            transfer: OwnedTransferOptions::default(),
            local_timeout: Duration::from_secs(3600),
            copy: CopyOptions::default(),
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelinePhase {
    #[default]
    Preflight,
    Export,
    Conversion,
    PublicationAdmission,
    Publication,
    LocalOutcomeUnknown,
    Completed,
}
#[derive(Debug, Default, Serialize)]
pub struct PipelineReport {
    pub phase: PipelinePhase,
    pub remote_error: Option<Error>,
    pub local_error: Option<OwnershipError>,
    pub blocking_worker_failed: bool,
    pub transfer: Option<OwnedTransferReport>,
    pub conversion: Option<OutputReport>,
    pub publication: Option<PublicationReport>,
    pub export_ms: f64,
    pub conversion_ms: f64,
    pub publication_admission_ms: f64,
    pub publication_ms: f64,
    pub elapsed_ms: f64,
}
impl PipelineReport {
    pub fn is_success(&self) -> bool {
        self.phase == PipelinePhase::Completed
            && self.remote_error.is_none()
            && self.local_error.is_none()
            && !self.blocking_worker_failed
            && self
                .transfer
                .as_ref()
                .is_some_and(OwnedTransferReport::is_artifact_success)
            && self
                .conversion
                .as_ref()
                .is_some_and(OutputReport::is_success)
            && self
                .publication
                .as_ref()
                .is_some_and(PublicationReport::is_success)
    }
}
/// Run one explicit export through private conversion and durable publication.
/// Await to completion: dropping this future cannot guarantee remote cleanup or
/// cancel an accepted blocking task. No retry, power change, rollback or deletion.
/// Destination stays locked; the original store inode is pinned across phase gaps.
pub async fn run_export_pipeline(
    source: SourceSelection,
    credentials: &Credentials,
    store: JobStore,
    destination: PublicationDirectory,
    artifact: ArtifactId,
    output: OutputId,
    options: PipelineOptions,
) -> PipelineReport {
    let started = Instant::now();
    let mut report = PipelineReport::default();
    if options.local_timeout.is_zero()
        || options.local_timeout > Duration::from_secs(21600)
        || options.transfer.transfer_timeout.is_zero()
        || options.transfer.transfer_timeout > Duration::from_secs(21600)
        || options.transfer.max_encoded_bytes < 512
        || options.transfer.max_encoded_bytes > 1 << 40
    {
        report.remote_error = Some(Error::InvalidInput);
        return report;
    }
    if options.transfer.cancellation.check().is_err() {
        report.remote_error = Some(Error::Cancelled);
        return report;
    }
    let preflight = tokio::task::spawn_blocking(move || {
        destination.preflight(&store, output)?;
        let anchor = store.anchor()?;
        Ok::<_, OwnershipError>((store, destination, anchor))
    })
    .await;
    let (store, destination, anchor) = match preflight {
        Ok(Ok(v)) => v,
        Ok(Err(e)) => {
            report.local_error = Some(e);
            report.elapsed_ms = started.elapsed().as_secs_f64() * 1000.;
            return report;
        }
        Err(_) => {
            report.blocking_worker_failed = true;
            report.elapsed_ms = started.elapsed().as_secs_f64() * 1000.;
            return report;
        }
    };
    report.phase = PipelinePhase::Export;
    let now = Instant::now();
    match transfer_owned_artifact(
        source.clone(),
        credentials,
        store,
        artifact,
        options.transfer.clone(),
    )
    .await
    {
        Ok(r) => report.transfer = Some(r),
        Err(e) => report.remote_error = Some(e),
    }
    report.export_ms = now.elapsed().as_secs_f64() * 1000.;
    if !report
        .transfer
        .as_ref()
        .is_some_and(OwnedTransferReport::is_artifact_success)
    {
        report.elapsed_ms = started.elapsed().as_secs_f64() * 1000.;
        return report;
    }
    // Export's worker has drained and released its store lock. All accepted local
    // work is in one blocking closure, awaited without an outer cancellation race.
    let local = tokio::task::spawn_blocking(move || {
        let mut local = PipelineReport::default();
        let deadline = Instant::now() + options.local_timeout;
        let remaining = || -> Result<RetainedOptions, OwnershipError> {
            if options.transfer.cancellation.check().is_err() {
                return Err(OwnershipError::Cancelled);
            }
            let timeout = deadline.saturating_duration_since(Instant::now());
            if timeout.is_zero() {
                return Err(OwnershipError::Deadline);
            }
            Ok(RetainedOptions {
                cancellation: options.transfer.cancellation.clone(),
                timeout,
            })
        };
        let result = (|| {
            local.phase = PipelinePhase::Conversion;
            let now = Instant::now();
            let conversion = (|| {
                let retained =
                    RetainedArtifact::open(anchor.open()?, artifact, source.clone(), remaining()?)?;
                Ok::<_, OwnershipError>(retained.convert_owned(
                    output,
                    options.copy,
                    remaining()?,
                    &|_: &CopyEvent| {},
                ))
            })();
            local.conversion_ms = now.elapsed().as_secs_f64() * 1000.;
            local.conversion = Some(conversion?);
            if !local.conversion.as_ref().unwrap().is_success() {
                return Ok(());
            }
            local.phase = PipelinePhase::PublicationAdmission;
            let now = Instant::now();
            let verified =
                VerifiedOutput::open(anchor.open()?, output, artifact, source, remaining()?);
            local.publication_admission_ms = now.elapsed().as_secs_f64() * 1000.;
            let verified = verified?;
            local.phase = PipelinePhase::Publication;
            let now = Instant::now();
            local.publication = Some(verified.publish(destination, remaining()?));
            local.publication_ms = now.elapsed().as_secs_f64() * 1000.;
            if local.publication.as_ref().unwrap().is_success() {
                local.phase = PipelinePhase::Completed;
            }
            Ok::<_, OwnershipError>(())
        })();
        local.local_error = result.err();
        local
    })
    .await;
    match local {
        Ok(mut local) => {
            local.transfer = report.transfer;
            local.export_ms = report.export_ms;
            report = local;
        }
        Err(_) => {
            report.phase = PipelinePhase::LocalOutcomeUnknown;
            report.blocking_worker_failed = true;
        }
    }
    report.elapsed_ms = started.elapsed().as_secs_f64() * 1000.;
    report
}
