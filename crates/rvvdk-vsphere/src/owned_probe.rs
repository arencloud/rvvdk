//! Durable acquire/abort integration; no payload transfer, publication or resume.
use crate::{
    About, Cancellation, Cleanup, Credentials, Error, InventoryLimits, LeaseCleanup,
    RequestMeasurement, Result, Session,
    contract::{ArtifactId, SourceSelection},
    inventory::{Kind, Properties, Reference},
    journal_worker::{Client, Command, Worker},
    ownership::{JobState, JobStore, OwnershipError, RecoveryReport},
    session_work,
    transport::Method,
    xml,
};
use serde::Serialize;
use std::time::{Duration, Instant};
use zeroize::Zeroizing;

#[derive(Debug, Serialize)]
pub struct OwnedProbeReport {
    pub primary_error: Option<Error>,
    pub journal_error: Option<OwnershipError>,
    pub recovery: Option<RecoveryReport>,
    pub lease_cleanup: LeaseCleanup,
    pub session_cleanup: Cleanup,
    pub pagination_cleanup_error: Option<Error>,
    pub requests: Vec<RequestMeasurement>,
    pub elapsed_ms: f64,
}
impl Default for OwnedProbeReport {
    fn default() -> Self {
        Self {
            primary_error: None,
            journal_error: None,
            recovery: None,
            lease_cleanup: LeaseCleanup::NotAcquired,
            session_cleanup: Cleanup::NotNeeded,
            pagination_cleanup_error: None,
            requests: Vec::new(),
            elapsed_ms: 0.,
        }
    }
}
impl OwnedProbeReport {
    pub fn is_success(&self) -> bool {
        self.primary_error.is_none()
            && self.journal_error.is_none()
            && self.lease_cleanup == LeaseCleanup::Aborted
            && self.session_cleanup == Cleanup::LoggedOut
            && self.pagination_cleanup_error.is_none()
            && self
                .recovery
                .as_ref()
                .is_some_and(|r| r.state == JobState::AbortedLease && !r.pending_transaction)
    }
}

/// Acquire and abort one explicitly selected powered-off VM's export lease.
/// Consumes a locked private store. The journal-owned empty stage is retained for
/// explicit checked cleanup; this does not download, complete or publish an export.
/// Await to completion and cancel cooperatively. Blocking filesystem operations
/// are drained before return and can outlast network deadlines. Dropping this
/// future cannot guarantee remote cleanup; the worker retains ownership until it
/// finishes its accepted command. Recovery never recreates a lease capability.
pub async fn probe_owned_export(
    source: SourceSelection,
    credentials: &Credentials,
    store: JobStore,
    artifact: ArtifactId,
    cancellation: Cancellation,
) -> Result<OwnedProbeReport> {
    cancellation.check()?;
    let worker = Worker::start(store, artifact, source.clone());
    probe_with_worker(source, credentials, cancellation, worker).await
}
async fn probe_with_worker(
    source: SourceSelection,
    credentials: &Credentials,
    cancellation: Cancellation,
    mut worker: Worker,
) -> Result<OwnedProbeReport> {
    let started = Instant::now();
    let mut report = OwnedProbeReport::default();
    match (&mut worker.ready)
        .await
        .unwrap_or(Err(OwnershipError::Uncertain))
    {
        Err(error) => report.journal_error = Some(error),
        Ok(()) if cancellation.check().is_err() => report.primary_error = Some(Error::Cancelled),
        Ok(()) => {
            let client = worker.client.clone();
            match session_work(
                source.endpoint().connection_policy(),
                credentials,
                move |session, about| {
                    Box::pin(async move {
                        let mut report = OwnedProbeReport::default();
                        if let Err(error) = session
                            .owned_probe(about, &source, &client, &cancellation, &mut report)
                            .await
                        {
                            report.primary_error.get_or_insert(error);
                        }
                        Ok(report)
                    })
                },
            )
            .await
            {
                Ok(outcome) => {
                    match outcome.value {
                        Ok(value) => report = value,
                        Err(error) => report.primary_error = Some(error),
                    }
                    report.session_cleanup = outcome.cleanup;
                    report.pagination_cleanup_error = outcome.pagination_cleanup_error;
                    report.requests = outcome.requests;
                }
                Err(error) => report.primary_error = Some(error),
            }
        }
    }
    match worker.finish().await {
        Ok(recovery) => report.recovery = Some(recovery),
        Err(error) => {
            report.journal_error.get_or_insert(error);
        }
    }
    report.elapsed_ms = started.elapsed().as_secs_f64() * 1000.;
    Ok(report)
}

impl Session {
    async fn owned_probe(
        &mut self,
        about: About,
        source: &SourceSelection,
        client: &Client,
        cancellation: &Cancellation,
        report: &mut OwnedProbeReport,
    ) -> Result<()> {
        cancellation.check()?;
        let inventory = self.inventory(about, InventoryLimits::default()).await?;
        let selected = source
            .select_vm(source.endpoint(), &inventory)
            .map_err(crate::export::selection_error)?;
        let reference = &selected.identity.reference;
        self.check_probe_source(source, reference, cancellation)
            .await?;
        journal_result(client.apply(Command::Prepare).await, report)?;
        cancellation.check()?;
        journal_result(client.apply(Command::Acquire).await, report)?;
        // A slow intent sync must not make the prior observation authoritative.
        self.check_probe_source(source, reference, cancellation)
            .await?;
        let raw = self
            .transport
            .call(Method::ExportVm, reference.element("_this"), self.deadline)
            .await;
        report.lease_cleanup = LeaseCleanup::Unconfirmed;
        // Even a reported server rejection remains AcquireIntent: no unqualified
        // failure-to-no-effect state transition or automatic acquisition retry.
        let raw = raw?;
        let doc = xml::parse(&raw)?;
        let node = xml::required(xml::response(&doc, "ExportVm")?, "returnval")?;
        let lease = Reference::parse(node)?;
        if lease.kind != Kind::HttpNfcLease {
            return Err(Error::LeaseUnconfirmed);
        }
        let lease_reference = Zeroizing::new(xml::text(node)?.to_owned());
        let timeout = match self
            .probe_ready(&lease, reference, source.logical_bytes(), cancellation)
            .await
        {
            Ok(info) => Some(info.timeout),
            Err(error) => {
                report.primary_error.get_or_insert(error);
                None
            }
        };
        // A parsed live lease may be aborted even if readiness failed, but only
        // after its ownership and abort intent have actually become durable.
        self.probe_journal(
            client,
            Command::Held(lease_reference),
            &lease,
            timeout,
            report,
        )
        .await?;
        if let Err(error) = cancellation.check() {
            report.primary_error.get_or_insert(error);
        }
        self.probe_journal(client, Command::Abort, &lease, timeout, report)
            .await?;
        let result = self
            .transport
            .call(
                Method::HttpNfcLeaseAbort,
                lease.element("_this"),
                Instant::now() + self.cleanup_timeout,
            )
            .await;
        result?;
        report.lease_cleanup = LeaseCleanup::Aborted;
        journal_result(client.apply(Command::Aborted).await, report)?;
        if let Err(error) = cancellation.check() {
            report.primary_error.get_or_insert(error);
        }
        Ok(())
    }
    pub(crate) async fn check_probe_source(
        &mut self,
        source: &SourceSelection,
        reference: &Reference,
        cancellation: &Cancellation,
    ) -> Result<()> {
        cancellation.check()?;
        let vm = self.selected_vm(reference).await?;
        cancellation.check()?;
        source
            .check_vm(source.endpoint(), &vm)
            .map_err(crate::export::selection_error)?;
        Ok(())
    }
    pub(crate) async fn probe_ready(
        &mut self,
        lease: &Reference,
        vm: &Reference,
        capacity: u64,
        cancellation: &Cancellation,
    ) -> Result<crate::export::LeaseInfo> {
        loop {
            cancellation.check()?;
            let raw = self.properties(lease).await?;
            let doc = xml::parse(&raw)?;
            let props = Properties::parse(xml::response(&doc, "RetrievePropertiesEx")?, lease)?;
            match xml::text(props.get("state")?)? {
                "ready" => {
                    return self.lease_info(props.get("info")?, lease, vm, capacity);
                }
                "initializing" => {
                    if Instant::now() >= self.deadline {
                        return Err(Error::Deadline);
                    }
                    tokio::time::sleep(Duration::from_millis(250)).await;
                }
                _ => return Err(Error::LeaseState),
            }
        }
    }
    pub(crate) async fn probe_journal(
        &mut self,
        client: &Client,
        command: Command,
        lease: &Reference,
        timeout: Option<Duration>,
        report: &mut OwnedProbeReport,
    ) -> Result<()> {
        let work = client.apply(command);
        tokio::pin!(work);
        if let Some(timeout) = timeout {
            // Cancellation requests abort, not abandonment of an accepted sync.
            // The future is borrowed by heartbeat; on network failure it is still
            // drained before any next RPC. The worker never issues remote calls.
            let keep_cleanup_running = Cancellation::default();
            match self
                .heartbeat(
                    lease,
                    timeout,
                    self.deadline,
                    &keep_cleanup_running,
                    async { Ok(work.as_mut().await) },
                )
                .await
            {
                Ok(result) => return journal_result(result, report),
                Err(error) => {
                    report.primary_error.get_or_insert(error);
                }
            }
        }
        journal_result(work.await, report)
    }
}
fn journal_result(
    result: std::result::Result<(), OwnershipError>,
    report: &mut OwnedProbeReport,
) -> Result<()> {
    result.map_err(|error| {
        report.journal_error.get_or_insert(error);
        Error::Artifact
    })
}

#[cfg(test)]
mod tests;
