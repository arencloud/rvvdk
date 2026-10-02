//! Journal-owned container transfer and conservative completion; no publication.
use crate::{
    Cancellation, Cleanup, Credentials, Error, InventoryLimits, LeaseCleanup, Result, Session,
    contract::{ArtifactId, SourceSelection},
    export::{ExportFile, verify_manifest},
    inventory::{Kind, Reference},
    journal_worker::{CHUNK_BYTES, Client, Command, Expected, Worker},
    owned_probe::OwnedProbeReport,
    ownership::{JobState, JobStore, OwnershipError},
    session_work,
    transport::{Method, hex_string},
    xml,
};
use serde::Serialize;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::time::{Duration, Instant};
use zeroize::Zeroizing;

#[derive(Clone, Debug)]
pub struct OwnedTransferOptions {
    pub max_encoded_bytes: u64,
    pub transfer_timeout: Duration,
    pub cancellation: Cancellation,
}
impl Default for OwnedTransferOptions {
    fn default() -> Self {
        Self {
            max_encoded_bytes: 40 << 30,
            transfer_timeout: Duration::from_secs(3600),
            cancellation: Cancellation::default(),
        }
    }
}
#[derive(Debug, Default, Serialize)]
pub struct OwnedTransferReport {
    #[serde(flatten)]
    pub lifecycle: OwnedProbeReport,
    /// Accepted HTTP bytes. These are not a claim about local durability.
    pub received_encoded_bytes: u64,
    /// Confirmed completed file writes; may be a lower bound after worker panic.
    pub written_encoded_bytes: u64,
    pub durable_encoded_bytes: u64,
    pub manifest_verified: bool,
    /// Independent digest/length readback, not VMDK structure or logical validation.
    pub container_readback_verified: bool,
    pub payload_error: Option<OwnershipError>,
}
impl OwnedTransferReport {
    pub fn is_success(&self) -> bool {
        let r = &self.lifecycle;
        r.primary_error.is_none()
            && r.journal_error.is_none()
            && self.payload_error.is_none()
            && r.lease_cleanup == LeaseCleanup::Completed
            && r.session_cleanup == Cleanup::LoggedOut
            && r.pagination_cleanup_error.is_none()
            && self.manifest_verified
            && self.container_readback_verified
            && self.received_encoded_bytes == self.written_encoded_bytes
            && self.written_encoded_bytes == self.durable_encoded_bytes
            && r.recovery
                .as_ref()
                .is_some_and(|r| r.state == JobState::CompletedLease && !r.pending_transaction)
    }
}
/// Download, sync and independently compare container bytes in a fresh owned stage,
/// then complete the live lease under durable intent. The private stage is retained;
/// no R6 metadata, conversion, publication or restart capability is produced.
/// Await completion; accepted blocking writes are drained, even on cancellation.
pub async fn transfer_owned_export(
    source: SourceSelection,
    credentials: &Credentials,
    store: JobStore,
    artifact: ArtifactId,
    options: OwnedTransferOptions,
) -> Result<OwnedTransferReport> {
    if options.max_encoded_bytes < 512
        || options.max_encoded_bytes > 1 << 40
        || options.transfer_timeout.is_zero()
        || options.transfer_timeout > Duration::from_secs(21600)
    {
        return Err(Error::InvalidInput);
    }
    options.cancellation.check()?;
    let worker = Worker::start(store, artifact, source.clone());
    transfer_with_worker(source, credentials, options, worker).await
}
async fn transfer_with_worker(
    source: SourceSelection,
    credentials: &Credentials,
    options: OwnedTransferOptions,
    mut worker: Worker,
) -> Result<OwnedTransferReport> {
    let started = Instant::now();
    let mut report = OwnedTransferReport::default();
    match (&mut worker.ready)
        .await
        .unwrap_or(Err(OwnershipError::Uncertain))
    {
        Err(error) => report.lifecycle.journal_error = Some(error),
        Ok(()) if options.cancellation.check().is_err() => {
            report.lifecycle.primary_error = Some(Error::Cancelled)
        }
        Ok(()) => {
            let client = worker.client.clone();
            match session_work(
                source.endpoint().connection_policy(),
                credentials,
                move |session, about| {
                    Box::pin(async move {
                        let mut report = OwnedTransferReport::default();
                        let mut lease = None;
                        let mut completing = false;
                        if let Err(error) = session
                            .owned_transfer(
                                about,
                                &source,
                                &client,
                                &options,
                                &mut report,
                                &mut lease,
                                &mut completing,
                            )
                            .await
                        {
                            report.lifecycle.primary_error.get_or_insert(error);
                        }
                        if let Some((lease, timeout)) = lease
                            && !completing
                            && report.lifecycle.lease_cleanup != LeaseCleanup::Completed
                        {
                            // The worker serializes this behind every accepted payload
                            // write and closes its only writer before persisting intent.
                            let aborted = async {
                                session
                                    .probe_journal(
                                        &client,
                                        Command::Abort,
                                        &lease,
                                        timeout,
                                        &mut report.lifecycle,
                                    )
                                    .await?;
                                session
                                    .transport
                                    .call(
                                        Method::HttpNfcLeaseAbort,
                                        lease.element("_this"),
                                        Instant::now() + session.cleanup_timeout,
                                    )
                                    .await?;
                                report.lifecycle.lease_cleanup = LeaseCleanup::Aborted;
                                client.apply(Command::Aborted).await.map_err(|e| {
                                    report.lifecycle.journal_error.get_or_insert(e);
                                    Error::Artifact
                                })?;
                                Ok::<(), Error>(())
                            }
                            .await;
                            if let Err(error) = aborted {
                                report.lifecycle.primary_error.get_or_insert(error);
                            }
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
                        Err(error) => report.lifecycle.primary_error = Some(error),
                    }
                    report.lifecycle.session_cleanup = outcome.cleanup;
                    report.lifecycle.pagination_cleanup_error = outcome.pagination_cleanup_error;
                    report.lifecycle.requests = outcome.requests;
                }
                Err(error) => report.lifecycle.primary_error = Some(error),
            }
        }
    }
    match worker.finish_payload().await {
        Ok(outcome) => {
            report.written_encoded_bytes = outcome.progress.written;
            report.durable_encoded_bytes = outcome.progress.durable;
            report.container_readback_verified = outcome.progress.verified;
            report.payload_error = outcome.payload_error;
            match outcome.recovery {
                Ok(r) => report.lifecycle.recovery = Some(r),
                Err(e) => {
                    report.lifecycle.journal_error.get_or_insert(e);
                }
            }
        }
        Err(e) => {
            report.lifecycle.journal_error.get_or_insert(e);
        }
    }
    report.lifecycle.elapsed_ms = started.elapsed().as_secs_f64() * 1000.;
    Ok(report)
}
struct Downloaded {
    file: ExportFile,
    expected: Expected,
}
impl Session {
    #[allow(clippy::too_many_arguments)] // Separate report, live capability and irreversible completion guard.
    async fn owned_transfer(
        &mut self,
        about: crate::About,
        source: &SourceSelection,
        client: &Client,
        options: &OwnedTransferOptions,
        report: &mut OwnedTransferReport,
        live: &mut Option<(Reference, Option<Duration>)>,
        completing: &mut bool,
    ) -> Result<()> {
        options.cancellation.check()?;
        let inventory = self.inventory(about, InventoryLimits::default()).await?;
        let reference = &source
            .select_vm(source.endpoint(), &inventory)
            .map_err(crate::export::selection_error)?
            .identity
            .reference;
        self.check_probe_source(source, reference, &options.cancellation)
            .await?;
        for command in [Command::Prepare, Command::Acquire] {
            options.cancellation.check()?;
            client.apply(command).await.map_err(|e| {
                report.lifecycle.journal_error = Some(e);
                Error::Artifact
            })?;
        }
        self.check_probe_source(source, reference, &options.cancellation)
            .await?;
        let acquired = self
            .transport
            .call(Method::ExportVm, reference.element("_this"), self.deadline)
            .await;
        report.lifecycle.lease_cleanup = LeaseCleanup::Unconfirmed;
        let raw = acquired?;
        let doc = xml::parse(&raw)?;
        let node = xml::required(xml::response(&doc, "ExportVm")?, "returnval")?;
        let lease = Reference::parse(node)?;
        if lease.kind != Kind::HttpNfcLease {
            return Err(Error::LeaseUnconfirmed);
        }
        self.deadline = Instant::now() + options.transfer_timeout;
        let info = self
            .probe_ready(
                &lease,
                reference,
                source.logical_bytes(),
                &options.cancellation,
            )
            .await;
        let timeout = info.as_ref().ok().map(|i| i.timeout);
        // The abort path is enabled only after LeaseHeld was durably acknowledged.
        self.probe_journal(
            client,
            Command::Held(Zeroizing::new(xml::text(node)?.to_owned())),
            &lease,
            timeout,
            &mut report.lifecycle,
        )
        .await?;
        *live = Some((lease.clone(), timeout));
        let info = info?;
        check_continue(report, &options.cancellation)?;
        self.probe_journal(
            client,
            Command::PayloadOpen(options.max_encoded_bytes),
            &lease,
            timeout,
            &mut report.lifecycle,
        )
        .await?;
        check_continue(report, &options.cancellation)?;
        self.check_transfer_source(
            source,
            reference,
            &lease,
            info.timeout,
            &options.cancellation,
        )
        .await?;
        let request = self
            .transport
            .data_request(info.device.url, self.deadline)?;
        let downloaded = self
            .heartbeat(
                &lease,
                info.timeout,
                self.deadline,
                &options.cancellation,
                download(
                    request,
                    client,
                    options.max_encoded_bytes,
                    &mut report.received_encoded_bytes,
                ),
            )
            .await?;
        options.cancellation.check()?;
        let manifest = self
            .transport
            .call(
                Method::HttpNfcLeaseGetManifest,
                lease.element("_this"),
                self.lease_read_deadline(info.timeout),
            )
            .await?;
        verify_manifest(
            &manifest,
            &info.device.key,
            &downloaded.file,
            source.logical_bytes(),
        )?;
        report.manifest_verified = true;
        for command in [
            Command::PayloadSeal(downloaded.expected),
            Command::Transferred,
        ] {
            self.probe_journal(client, command, &lease, timeout, &mut report.lifecycle)
                .await?;
            check_continue(report, &options.cancellation)?;
        }
        self.check_transfer_source(
            source,
            reference,
            &lease,
            info.timeout,
            &options.cancellation,
        )
        .await?;
        // Set before submitting intent: even failed persistence can have renamed
        // the new record. Never abort after entering this uncertain boundary.
        *completing = true;
        self.probe_journal(
            client,
            Command::Complete,
            &lease,
            timeout,
            &mut report.lifecycle,
        )
        .await?;
        check_continue(report, &options.cancellation)?;
        self.check_transfer_source(
            source,
            reference,
            &lease,
            info.timeout,
            &options.cancellation,
        )
        .await?;
        self.transport
            .call(
                Method::HttpNfcLeaseComplete,
                lease.element("_this"),
                self.lease_read_deadline(info.timeout),
            )
            .await?;
        report.lifecycle.lease_cleanup = LeaseCleanup::Completed;
        client.apply(Command::Completed).await.map_err(|e| {
            report.lifecycle.journal_error = Some(e);
            Error::Artifact
        })?;
        options.cancellation.check()?;
        Ok(())
    }
    fn lease_read_deadline(&self, timeout: Duration) -> Instant {
        self.deadline
            .min(Instant::now() + (timeout / 3).min(Duration::from_secs(10)))
    }
    async fn check_transfer_source(
        &mut self,
        source: &SourceSelection,
        reference: &Reference,
        lease: &Reference,
        timeout: Duration,
        cancel: &Cancellation,
    ) -> Result<()> {
        cancel.check()?;
        self.transport
            .call(
                Method::HttpNfcLeaseProgress,
                format!("{}<percent>0</percent>", lease.element("_this")),
                self.lease_read_deadline(timeout),
            )
            .await?;
        let previous = self.deadline;
        self.deadline = self.lease_read_deadline(timeout);
        let checked = self.check_probe_source(source, reference, cancel).await;
        self.deadline = previous;
        checked
    }
}
fn check_continue(report: &OwnedTransferReport, cancel: &Cancellation) -> Result<()> {
    if let Some(error) = report.lifecycle.primary_error {
        return Err(error);
    }
    cancel.check()
}
async fn download(
    request: reqwest::RequestBuilder,
    client: &Client,
    limit: u64,
    received: &mut u64,
) -> Result<Downloaded> {
    let started = Instant::now();
    let mut response = request.send().await.map_err(http_error)?;
    if response.status() != reqwest::StatusCode::OK
        || response
            .headers()
            .get(reqwest::header::CONTENT_ENCODING)
            .is_some_and(|v| v != "identity")
    {
        return Err(Error::Http);
    }
    if response.content_length().is_some_and(|n| n > limit) {
        return Err(Error::TransferLimit);
    }
    let mut sha256 = Sha256::new();
    let mut sha1 = Sha1::new();
    let mut prefix = Vec::with_capacity(512);
    let mut batch = Vec::with_capacity(CHUNK_BYTES);
    while let Some(chunk) = response.chunk().await.map_err(http_error)? {
        if chunk.len() > CHUNK_BYTES {
            return Err(Error::TransferLimit);
        }
        let size = received
            .checked_add(chunk.len() as u64)
            .ok_or(Error::TransferLimit)?;
        if size > limit {
            return Err(Error::TransferLimit);
        }
        *received = size;
        if prefix.len() < 512 {
            prefix.extend_from_slice(&chunk[..chunk.len().min(512 - prefix.len())]);
        }
        sha256.update(&chunk);
        sha1.update(&chunk);
        let mut left = chunk.as_ref();
        while !left.is_empty() {
            let n = left.len().min(CHUNK_BYTES - batch.len());
            batch.extend_from_slice(&left[..n]);
            left = &left[n..];
            if batch.len() == CHUNK_BYTES {
                client
                    .apply(Command::PayloadWrite(std::mem::replace(
                        &mut batch,
                        Vec::with_capacity(CHUNK_BYTES),
                    )))
                    .await
                    .map_err(|_| Error::Artifact)?;
            }
        }
    }
    if *received < 512 || !prefix.starts_with(b"KDMV") {
        return Err(Error::ExportScope);
    }
    if !batch.is_empty() {
        client
            .apply(Command::PayloadWrite(batch))
            .await
            .map_err(|_| Error::Artifact)?;
    }
    let sha256: [u8; 32] = sha256.finalize().into();
    let sha1: [u8; 20] = sha1.finalize().into();
    Ok(Downloaded {
        file: ExportFile {
            name: "disk-1.vmdk".into(),
            encoded_bytes: *received,
            sha256: hex_string(&sha256),
            sha1: hex_string(&sha1),
            header_kind: "vmdk_sparse_magic",
            elapsed_ms: started.elapsed().as_secs_f64() * 1000.,
        },
        expected: Expected {
            bytes: *received,
            sha256,
            sha1,
        },
    })
}
fn http_error(error: reqwest::Error) -> Error {
    if error.is_timeout() {
        Error::Deadline
    } else {
        Error::Transport
    }
}

#[cfg(test)]
mod tests;
