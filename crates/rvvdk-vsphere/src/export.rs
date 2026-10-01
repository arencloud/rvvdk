use crate::{
    About, Cleanup, ConnectionPolicy, Credentials, Error, InventoryLimits, Result, Session,
    inventory::{Kind, Properties, Reference, Vm},
    session_work,
    transport::{Method, hex_string},
    xml,
};
use serde::Serialize;
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::{
    fmt,
    future::Future,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::io::AsyncWriteExt;

#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
    fn check(&self) -> Result<()> {
        if self.0.load(Ordering::Relaxed) {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
}
impl fmt::Debug for Cancellation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Cancellation")
    }
}
/// Qualification subset: exactly one persistent unencrypted disk without snapshots.
/// Capacity selects an unambiguous VM; its private reference+UUID are rechecked
/// immediately before shutdown/acquisition. No implicit hard power-off exists.
#[derive(Clone)]
pub struct ExportOptions {
    pub single_disk_capacity_bytes: u64,
    pub probe_only: bool,
    /// Read-only task inspection: no ExportVm or shutdown call.
    pub inspect_only: bool,
    pub allow_graceful_shutdown: bool,
    pub output: PathBuf,
    pub max_encoded_bytes: u64,
    pub transfer_timeout: Duration,
    pub cancellation: Cancellation,
}
impl fmt::Debug for ExportOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ExportOptions([private output/selection])")
    }
}
impl ExportOptions {
    pub fn probe(single_disk_capacity_bytes: u64) -> Self {
        Self {
            single_disk_capacity_bytes,
            probe_only: true,
            inspect_only: false,
            allow_graceful_shutdown: false,
            output: PathBuf::new(),
            max_encoded_bytes: 40 * 1024 * 1024 * 1024,
            transfer_timeout: Duration::from_secs(3600),
            cancellation: Cancellation::default(),
        }
    }
    fn validate(&self) -> Result<()> {
        if self.single_disk_capacity_bytes == 0
            || self.max_encoded_bytes == 0
            || self.max_encoded_bytes > 1024 * 1024 * 1024 * 1024
            || self.transfer_timeout.is_zero()
            || self.transfer_timeout > Duration::from_secs(21600)
            || (!self.probe_only && self.output.as_os_str().is_empty())
            || (self.inspect_only && (!self.probe_only || self.allow_graceful_shutdown))
        {
            Err(Error::InvalidInput)
        } else {
            Ok(())
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LeaseCleanup {
    NotAcquired,
    Rejected,
    Aborted,
    Completed,
    Unconfirmed,
}
#[derive(Debug, Serialize)]
pub struct ExportFile {
    pub name: String,
    pub encoded_bytes: u64,
    pub sha256: String,
    pub sha1: String,
    pub header_kind: &'static str,
    pub elapsed_ms: f64,
}
#[derive(Debug, Serialize)]
pub struct ExportReport {
    pub probe_only: bool,
    pub inspection_only: bool,
    pub recent_tasks: Vec<TaskSummary>,
    pub selected_capacity_bytes: u64,
    pub initial_power_state: Option<String>,
    pub last_observed_power_state: Option<String>,
    pub shutdown_requested: bool,
    pub primary_error: Option<Error>,
    pub lease_cleanup: LeaseCleanup,
    pub lease_cleanup_error: Option<Error>,
    pub session_cleanup: Cleanup,
    pub pagination_cleanup_error: Option<Error>,
    /// Accepted HTTP body bytes, retained on failure; not durable/published bytes.
    pub received_encoded_bytes: u64,
    /// Explicit non-disk lease entries omitted by this disk-only proof.
    pub ignored_non_disk_devices: usize,
    pub files: Vec<ExportFile>,
    pub manifest_verified: bool,
    pub artifact_published: bool,
    pub artifact_cleanup_error: Option<Error>,
    pub requests: Vec<crate::RequestMeasurement>,
    pub elapsed_ms: f64,
}
impl ExportReport {
    pub fn is_success(&self) -> bool {
        self.primary_error.is_none()
            && self.lease_cleanup == LeaseCleanup::Completed
            && self.session_cleanup == Cleanup::LoggedOut
            && self.manifest_verified
            && self.artifact_published
            && self.artifact_cleanup_error.is_none()
            && self.lease_cleanup_error.is_none()
            && self.pagination_cleanup_error.is_none()
    }
}
impl ExportReport {
    fn new(options: &ExportOptions) -> Self {
        Self {
            probe_only: options.probe_only,
            inspection_only: options.inspect_only,
            recent_tasks: Vec::new(),
            selected_capacity_bytes: options.single_disk_capacity_bytes,
            initial_power_state: None,
            last_observed_power_state: None,
            shutdown_requested: false,
            primary_error: None,
            lease_cleanup: LeaseCleanup::NotAcquired,
            lease_cleanup_error: None,
            session_cleanup: Cleanup::NotNeeded,
            pagination_cleanup_error: None,
            received_encoded_bytes: 0,
            ignored_non_disk_devices: 0,
            files: Vec::new(),
            manifest_verified: false,
            artifact_published: false,
            artifact_cleanup_error: None,
            requests: Vec::new(),
            elapsed_ms: 0.,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct TaskSummary {
    pub state: &'static str,
    pub operation: &'static str,
    pub cancelable: bool,
    pub cancelled: bool,
    pub queued_at: Option<String>,
    pub started_at: Option<String>,
}
#[cfg(target_os = "linux")]
struct Attempt {
    report: ExportReport,
    artifact: Option<crate::artifact::Artifact>,
}

/// Execute a scoped export proof, or acquire-and-abort eligibility probe.
/// Await to completion; cancel via Cancellation, not by dropping this future.
#[cfg(target_os = "linux")]
pub async fn export_vm(
    policy: ConnectionPolicy,
    credentials: &Credentials,
    options: ExportOptions,
) -> Result<ExportReport> {
    let started = Instant::now();
    options.validate()?;
    let mut report = ExportReport::new(&options);
    let outcome = session_work(policy, credentials, move |session, about| {
        Box::pin(session.export_attempt(about, options))
    })
    .await?;
    let mut artifact = None;
    match outcome.value {
        Ok(attempt) => {
            report = attempt.report;
            artifact = attempt.artifact;
        }
        Err(error) => report.primary_error = Some(error),
    }
    report.session_cleanup = outcome.cleanup;
    report.pagination_cleanup_error = outcome.pagination_cleanup_error;
    report.requests = outcome.requests;
    if let Some(mut writer) = artifact {
        if report.primary_error.is_none()
            && report.session_cleanup == Cleanup::LoggedOut
            && report.pagination_cleanup_error.is_none()
            && report.lease_cleanup == LeaseCleanup::Completed
            && report.manifest_verified
        {
            if let Err(error) = writer.publish() {
                report.primary_error = Some(error);
            }
            report.artifact_published = writer.published;
        }
        if !writer.published {
            report.artifact_cleanup_error = writer.discard().err();
        }
    }
    report.elapsed_ms = started.elapsed().as_secs_f64() * 1000.;
    Ok(report)
}
#[cfg(not(target_os = "linux"))]
pub async fn export_vm(
    _: ConnectionPolicy,
    _: &Credentials,
    _: ExportOptions,
) -> Result<ExportReport> {
    Err(Error::ExportScope)
}

struct Device {
    key: String,
    url: reqwest::Url,
}
struct LeaseInfo {
    device: Device,
    ignored_non_disk_devices: usize,
    timeout: Duration,
}
#[cfg(target_os = "linux")]
impl Session {
    async fn vm_tasks(&mut self, vm: &Reference) -> Result<Vec<TaskSummary>> {
        let raw = self.properties_fields(vm, &["recentTask"]).await?;
        let doc = xml::parse(&raw)?;
        let props = Properties::parse_fields(
            xml::response(&doc, "RetrievePropertiesEx")?,
            vm,
            &["recentTask"],
        )?;
        let tasks = props.references("recentTask")?;
        if tasks.len() > 32 {
            return Err(Error::InventoryLimit);
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut result = Vec::new();
        for task in &tasks {
            if task.kind != Kind::Task || !seen.insert(task.clone()) {
                return Err(Error::Schema);
            }
        }
        for task in tasks {
            let raw = self.properties(&task).await?;
            let doc = xml::parse(&raw)?;
            let props = Properties::parse(xml::response(&doc, "RetrievePropertiesEx")?, &task)?;
            let info = props.get("info")?;
            if Reference::parse(xml::required(info, "task")?)? != task {
                return Err(Error::Identity);
            }
            if let Some(entity) = xml::child(info, "entity")?
                && Reference::parse(entity)? != *vm
            {
                return Err(Error::Identity);
            }
            let state = match xml::text(xml::required(info, "state")?)? {
                "queued" => "queued",
                "running" => "running",
                "success" => "success",
                "error" => "error",
                _ => return Err(Error::Schema),
            };
            let operation = match xml::text(xml::required(info, "descriptionId")?)? {
                "VirtualMachine.exportVm" | "vim.VirtualMachine.exportVm" => "export_vm",
                "VirtualMachine.powerOn" | "vim.VirtualMachine.powerOn" => "power_on",
                "VirtualMachine.powerOff" | "vim.VirtualMachine.powerOff" => "power_off",
                "VirtualMachine.reconfigure" | "vim.VirtualMachine.reconfigure" => "reconfigure",
                _ => "other",
            };
            let summary = TaskSummary {
                state,
                operation,
                cancelable: xml::boolean(xml::required(info, "cancelable")?)?,
                cancelled: xml::boolean(xml::required(info, "cancelled")?)?,
                queued_at: xml::child(info, "queueTime")?.map(task_time).transpose()?,
                started_at: xml::child(info, "startTime")?.map(task_time).transpose()?,
            };
            result.push(summary);
        }
        Ok(result)
    }

    async fn selected_vm(&mut self, reference: &Reference) -> Result<Vm> {
        let raw = self.properties(reference).await?;
        let doc = xml::parse(&raw)?;
        Properties::parse(xml::response(&doc, "RetrievePropertiesEx")?, reference)?.vm(
            reference.clone(),
            1,
            16,
        )
    }
    async fn export_attempt(&mut self, about: About, options: ExportOptions) -> Result<Attempt> {
        let mut attempt = Attempt {
            report: ExportReport::new(&options),
            artifact: None,
        };
        let mut lease = None;
        let result = self
            .export_inner(about, &options, &mut attempt, &mut lease)
            .await;
        attempt.report.primary_error = result.err();
        if let Some(lease) = lease
            && attempt.report.lease_cleanup != LeaseCleanup::Completed
        {
            match self
                .transport
                .call(
                    Method::HttpNfcLeaseAbort,
                    lease.element("_this"),
                    Instant::now() + self.cleanup_timeout,
                )
                .await
            {
                Ok(_) => attempt.report.lease_cleanup = LeaseCleanup::Aborted,
                Err(error) => {
                    attempt.report.lease_cleanup = LeaseCleanup::Unconfirmed;
                    attempt.report.lease_cleanup_error = Some(error);
                }
            }
        }
        Ok(attempt)
    }
    async fn export_inner(
        &mut self,
        about: About,
        options: &ExportOptions,
        attempt: &mut Attempt,
        owned: &mut Option<Reference>,
    ) -> Result<()> {
        options.cancellation.check()?;
        let inventory = self.inventory(about, InventoryLimits::default()).await?;
        let mut candidates = inventory.vms.into_iter().filter(|vm| {
            vm.disks.len() == 1 && vm.disks[0].capacity_bytes == options.single_disk_capacity_bytes
        });
        let selected = candidates.next().ok_or(Error::Identity)?;
        if candidates.next().is_some() {
            return Err(Error::Identity);
        }
        let reference = selected.identity.reference.clone();
        let mut vm = self.selected_vm(&reference).await?;
        if vm.identity.bios_uuid() != selected.identity.bios_uuid()
            || vm.disks.len() != 1
            || vm.disks[0].identity != selected.disks[0].identity
        {
            return Err(Error::Identity);
        }
        admit_vm(&vm, options.single_disk_capacity_bytes)?;
        attempt.report.initial_power_state = Some(vm.power_state.clone());
        attempt.report.last_observed_power_state = Some(vm.power_state.clone());
        if options.inspect_only {
            attempt.report.recent_tasks = self.vm_tasks(&reference).await?;
            return Ok(());
        }
        // Some hosts start an export task before rejecting a powered-on request.
        // Never use ExportVm as a powered-on license probe.
        if options.probe_only && vm.power_state != "poweredOff" {
            return Err(Error::InvalidPowerState);
        }
        if !options.probe_only {
            // Output admission happens before any guest power operation or lease.
            attempt.artifact = Some(crate::artifact::Artifact::create(
                &options.output,
                options.max_encoded_bytes,
            )?);
            if vm.power_state != "poweredOff" {
                if !options.allow_graceful_shutdown
                    || vm.power_state != "poweredOn"
                    || vm.tools != "guestToolsRunning"
                {
                    return Err(Error::InvalidPowerState);
                }
                options.cancellation.check()?;
                attempt.report.shutdown_requested = true;
                self.transport
                    .call(
                        Method::ShutdownGuest,
                        reference.element("_this"),
                        self.deadline,
                    )
                    .await?;
                loop {
                    options.cancellation.check()?;
                    vm = self.selected_vm(&reference).await?;
                    attempt.report.last_observed_power_state = Some(vm.power_state.clone());
                    if vm.identity.bios_uuid() != selected.identity.bios_uuid() {
                        return Err(Error::Identity);
                    }
                    if vm.power_state == "poweredOff" {
                        break;
                    }
                    if Instant::now() >= self.deadline {
                        return Err(Error::Deadline);
                    }
                    tokio::time::sleep(Duration::from_millis(500)).await;
                }
            }
            admit_vm(&vm, options.single_disk_capacity_bytes)?;
            if vm.disks[0].identity != selected.disks[0].identity {
                return Err(Error::Identity);
            }
        }
        options.cancellation.check()?;
        let acquired = self
            .transport
            .call(Method::ExportVm, reference.element("_this"), self.deadline)
            .await;
        let raw = match acquired {
            Ok(raw) => raw,
            Err(error) => {
                attempt.report.lease_cleanup = if matches!(
                    error,
                    Error::LicenseRestricted
                        | Error::InvalidPowerState
                        | Error::MethodDisabled
                        | Error::InvalidState
                        | Error::NoPermission
                        | Error::Restricted
                ) {
                    LeaseCleanup::Rejected
                } else {
                    LeaseCleanup::Unconfirmed
                };
                return Err(error);
            }
        };
        // Any successful-looking but unparseable acquisition is ambiguous; never retry.
        attempt.report.lease_cleanup = LeaseCleanup::Unconfirmed;
        let doc = xml::parse(&raw)?;
        let node = xml::required(xml::response(&doc, "ExportVm")?, "returnval")?;
        let lease = Reference::parse(node)?;
        if lease.kind != Kind::HttpNfcLease {
            return Err(Error::LeaseUnconfirmed);
        }
        *owned = Some(lease.clone());
        if options.probe_only {
            return Ok(());
        }
        let deadline = Instant::now() + options.transfer_timeout;
        self.deadline = deadline;
        let info = loop {
            options.cancellation.check()?;
            let raw = self.properties(&lease).await?;
            let doc = xml::parse(&raw)?;
            let props = Properties::parse(xml::response(&doc, "RetrievePropertiesEx")?, &lease)?;
            match xml::text(props.get("state")?)? {
                "initializing" => {
                    if Instant::now() >= deadline {
                        return Err(Error::Deadline);
                    }
                    tokio::time::sleep(Duration::from_millis(250)).await;
                }
                "ready" => {
                    break self.lease_info(
                        props.get("info")?,
                        &lease,
                        &reference,
                        options.single_disk_capacity_bytes,
                    )?;
                }
                _ => return Err(Error::LeaseState),
            }
        };
        attempt.report.ignored_non_disk_devices = info.ignored_non_disk_devices;
        self.transport
            .call(
                Method::HttpNfcLeaseProgress,
                format!("{}<percent>0</percent>", lease.element("_this")),
                deadline,
            )
            .await?;
        let request = self
            .transport
            .data_request(info.device.url.clone(), deadline)?;
        let writer = attempt
            .artifact
            .as_mut()
            .ok_or(Error::Artifact)?
            .file("disk-1.vmdk")?;
        let downloaded = self
            .heartbeat(
                &lease,
                info.timeout,
                deadline,
                &options.cancellation,
                download(
                    request,
                    writer,
                    options.max_encoded_bytes,
                    &mut attempt.report.received_encoded_bytes,
                ),
            )
            .await?;
        attempt.report.files.push(downloaded);
        let raw = self
            .transport
            .call(
                Method::HttpNfcLeaseGetManifest,
                lease.element("_this"),
                deadline,
            )
            .await?;
        verify_manifest(
            &raw,
            &info.device.key,
            &attempt.report.files[0],
            options.single_disk_capacity_bytes,
        )?;
        attempt.report.manifest_verified = true;
        // Final progress and durable artifact metadata are covered by the heartbeat.
        let metadata = serde_json::to_vec(&attempt.report.files).map_err(|_| Error::Artifact)?;
        let mut file = attempt
            .artifact
            .as_mut()
            .ok_or(Error::Artifact)?
            .file("manifest.json")?;
        self.heartbeat(
            &lease,
            info.timeout,
            deadline,
            &options.cancellation,
            async {
                file.write_all(&metadata)
                    .await
                    .map_err(|_| Error::Artifact)?;
                file.sync_all().await.map_err(|_| Error::Artifact)
            },
        )
        .await?;
        self.transport
            .call(
                Method::HttpNfcLeaseComplete,
                lease.element("_this"),
                deadline,
            )
            .await?;
        attempt.report.lease_cleanup = LeaseCleanup::Completed;
        Ok(())
    }
    fn lease_info(
        &self,
        node: roxmltree::Node<'_, '_>,
        lease: &Reference,
        vm: &Reference,
        capacity: u64,
    ) -> Result<LeaseInfo> {
        if Reference::parse(xml::required(node, "lease")?)? != *lease
            || Reference::parse(xml::required(node, "entity")?)? != *vm
            || xml::number(xml::required(node, "totalDiskCapacityInKB")?)?.checked_mul(1024)
                != Some(capacity)
        {
            return Err(Error::Identity);
        }
        let timeout = xml::number(xml::required(node, "leaseTimeout")?)?;
        if !(3..=3600).contains(&timeout) {
            return Err(Error::LeaseState);
        }
        let (device, ignored_non_disk_devices) = one_disk(
            node.children()
                .filter(|n| n.has_tag_name((xml::VIM, "deviceUrl"))),
        )?;
        let key = xml::text(xml::required(device, "key")?)?;
        if key.is_empty() || key.len() > 256 {
            return Err(Error::Schema);
        }
        let url = self.transport.admit_data_url(
            xml::text(xml::required(device, "url")?)?,
            xml::text(xml::required(device, "sslThumbprint")?)?,
        )?;
        Ok(LeaseInfo {
            ignored_non_disk_devices,
            device: Device {
                key: key.to_owned(),
                url,
            },
            timeout: Duration::from_secs(timeout),
        })
    }
    async fn heartbeat<T>(
        &mut self,
        lease: &Reference,
        timeout: Duration,
        deadline: Instant,
        cancel: &Cancellation,
        work: impl Future<Output = Result<T>>,
    ) -> Result<T> {
        let mut interval = tokio::time::interval((timeout / 3).min(Duration::from_secs(10)));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut cancellation = tokio::time::interval(Duration::from_millis(100));
        tokio::pin!(work);
        loop {
            tokio::select! {
                result=&mut work=>return result,
                _=tokio::time::sleep_until(deadline.into())=>return Err(Error::Deadline),
                _=cancellation.tick()=>{cancel.check()?;},
                _=interval.tick()=>{cancel.check()?;self.transport.call(Method::HttpNfcLeaseProgress,format!("{}<percent>0</percent>",lease.element("_this")),deadline.min(Instant::now()+timeout/3)).await?;},
            }
        }
    }
}
// UTC timestamps only; never propagate arbitrary server strings into diagnostics.
fn task_time(node: roxmltree::Node<'_, '_>) -> Result<String> {
    checked_task_time(xml::text(node)?)
}
fn checked_task_time(s: &str) -> Result<String> {
    let b = s.as_bytes();
    if !(20..=30).contains(&b.len()) || b.last() != Some(&b'Z') {
        return Err(Error::Schema);
    }
    for (i, &c) in b[..19].iter().enumerate() {
        let valid = match i {
            4 | 7 => c == b'-',
            10 => c == b'T',
            13 | 16 => c == b':',
            _ => c.is_ascii_digit(),
        };
        if !valid {
            return Err(Error::Schema);
        }
    }
    if b.len() > 20
        && (b[19] != b'.' || b.len() == 21 || !b[20..b.len() - 1].iter().all(u8::is_ascii_digit))
    {
        return Err(Error::Schema);
    }
    let n = |a, b| s[a..b].parse::<u32>().map_err(|_| Error::Schema);
    let (year, month, day) = (n(0, 4)?, n(5, 7)?, n(8, 10)?);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if !(1..=12).contains(&month)
        || day == 0
        || day > days[(month - 1) as usize]
        || n(11, 13)? > 23
        || n(14, 16)? > 59
        || n(17, 19)? > 60
    {
        return Err(Error::Schema);
    }
    Ok(s.to_owned())
}
fn admit_vm(vm: &Vm, capacity: u64) -> Result<()> {
    if vm.template || vm.snapshot_present || vm.disks.len() != 1 {
        return Err(Error::ExportScope);
    }
    let disk = &vm.disks[0];
    if disk.identity.key.is_none() || disk.identity.backing.is_none() {
        return Err(Error::ExportScope);
    }
    if disk.capacity_bytes != capacity
        || disk.encrypted
        || disk.parent_present
        || disk.disk_mode != "persistent"
        || disk.backing_type != "VirtualDiskFlatVer2BackingInfo"
    {
        return Err(Error::ExportScope);
    }
    Ok(())
}
async fn download(
    request: reqwest::RequestBuilder,
    file: tokio::fs::File,
    limit: u64,
    received: &mut u64,
) -> Result<ExportFile> {
    let started = Instant::now();
    let mut response = request.send().await.map_err(|e| {
        if e.is_timeout() {
            Error::Deadline
        } else {
            Error::Transport
        }
    })?;
    if response.status() != reqwest::StatusCode::OK
        || response
            .headers()
            .get(reqwest::header::CONTENT_ENCODING)
            .is_some_and(|v| v != "identity")
    {
        return Err(Error::Http);
    }
    if response.content_length().is_some_and(|v| v > limit) {
        return Err(Error::TransferLimit);
    }
    let mut writer = tokio::io::BufWriter::with_capacity(1024 * 1024, file);
    let mut bytes = 0u64;
    let mut sha256 = Sha256::new();
    let mut sha1 = Sha1::new();
    let mut prefix = Vec::with_capacity(512);
    while let Some(chunk) = response.chunk().await.map_err(|e| {
        if e.is_timeout() {
            Error::Deadline
        } else {
            Error::Transport
        }
    })? {
        bytes = bytes
            .checked_add(chunk.len() as u64)
            .ok_or(Error::TransferLimit)?;
        if bytes > limit || chunk.len() > 1024 * 1024 {
            return Err(Error::TransferLimit);
        }
        *received = bytes;
        if prefix.len() < 512 {
            prefix.extend_from_slice(&chunk[..chunk.len().min(512 - prefix.len())]);
        }
        sha256.update(&chunk);
        sha1.update(&chunk);
        writer
            .write_all(&chunk)
            .await
            .map_err(|_| Error::Artifact)?;
    }
    if bytes < 512 {
        return Err(Error::ExportScope);
    }
    let header_kind = if prefix.starts_with(b"KDMV") {
        "vmdk_sparse_magic"
    } else {
        return Err(Error::ExportScope);
    };
    writer.flush().await.map_err(|_| Error::Artifact)?;
    writer
        .into_inner()
        .sync_all()
        .await
        .map_err(|_| Error::Artifact)?;
    Ok(ExportFile {
        name: "disk-1.vmdk".to_owned(),
        encoded_bytes: bytes,
        sha256: hex_string(&sha256.finalize()),
        sha1: hex_string(&sha1.finalize()),
        header_kind,
        elapsed_ms: started.elapsed().as_secs_f64() * 1000.,
    })
}
// Lease metadata may include auxiliary files. This proof transfers one disk only.
// Explicit flags, unique bounded keys and cardinality prevent guessing from URLs.
fn one_disk<'a, 'i>(
    nodes: impl Iterator<Item = roxmltree::Node<'a, 'i>>,
) -> Result<(roxmltree::Node<'a, 'i>, usize)> {
    let mut disk = None;
    let mut ignored = 0;
    let mut keys = std::collections::BTreeSet::new();
    for (index, node) in nodes.enumerate() {
        if index >= 32 {
            return Err(Error::InventoryLimit);
        }
        let key = xml::text(xml::required(node, "key")?)?;
        if key.is_empty() || key.len() > 256 || !keys.insert(key) {
            return Err(Error::Schema);
        }
        if xml::boolean(xml::required(node, "disk")?)? {
            if disk.replace(node).is_some() {
                return Err(Error::ExportScope);
            }
        } else {
            ignored += 1;
        }
    }
    Ok((disk.ok_or(Error::ExportScope)?, ignored))
}

fn verify_manifest(raw: &str, key: &str, file: &ExportFile, capacity: u64) -> Result<()> {
    let doc = xml::parse(raw)?;
    let response = xml::response(&doc, "HttpNfcLeaseGetManifest")?;
    let (entry, _) = one_disk(
        response
            .children()
            .filter(|n| n.has_tag_name((xml::VIM, "returnval"))),
    )?;
    if xml::text(xml::required(entry, "key")?)? != key
        || !xml::boolean(xml::required(entry, "disk")?)?
        || xml::number(xml::required(entry, "size")?)? != file.encoded_bytes
    {
        return Err(Error::Manifest);
    }
    if let Some(value) = xml::child(entry, "capacity")?
        && xml::number(value)? != capacity
    {
        return Err(Error::Manifest);
    }
    if let Some(checksum) = xml::child(entry, "checksum")? {
        let expected = match xml::text(xml::required(entry, "checksumType")?)? {
            "sha256" => &file.sha256,
            "sha1" => &file.sha1,
            _ => return Err(Error::Manifest),
        };
        if xml::text(checksum)?.to_ascii_lowercase() != *expected {
            return Err(Error::Manifest);
        }
    } else if xml::text(xml::required(entry, "sha1")?)?.to_ascii_lowercase() != file.sha1 {
        return Err(Error::Manifest);
    }
    Ok(())
}

#[cfg(test)]
mod qualification_tests {
    use super::*;
    #[test]
    fn disk_selection_rejects_ambiguous_untyped_and_unbounded_entries() {
        let select = |entries: &str| {
            let raw = format!("<root xmlns='urn:vim25'>{entries}</root>");
            let doc = xml::parse(&raw)?;
            one_disk(doc.root_element().children().filter(|n| n.is_element()))
                .map(|(_, ignored)| ignored)
        };
        let disk = "<entry><key>disk</key><disk>true</disk></entry>";
        assert_eq!(select(disk), Ok(0));
        assert_eq!(select(&(disk.to_owned() + disk)), Err(Error::Schema));
        assert_eq!(
            select(&(disk.to_owned() + "<entry><key>other</key><disk>true</disk></entry>")),
            Err(Error::ExportScope)
        );
        assert_eq!(
            select("<entry><key>other</key></entry>"),
            Err(Error::Schema)
        );
        assert_eq!(
            select("<entry><key>other</key><disk>false</disk></entry>"),
            Err(Error::ExportScope)
        );
        let mut entries = disk.to_owned();
        for index in 0..31 {
            entries.push_str(&format!(
                "<entry><key>aux-{index}</key><disk>false</disk></entry>"
            ));
        }
        assert_eq!(select(&entries), Ok(31));
        entries.push_str("<entry><key>overflow</key><disk>false</disk></entry>");
        assert_eq!(select(&entries), Err(Error::InventoryLimit));
        assert_eq!(
            select(&format!(
                "<entry><key>{}</key><disk>true</disk></entry>",
                "x".repeat(257)
            )),
            Err(Error::Schema)
        );
    }

    #[test]
    fn timestamps_are_bounded_calendar_values_not_server_messages() {
        for value in [
            "2024-02-29T23:59:59Z",
            "2026-01-02T03:04:05.123456Z",
            "2000-02-29T00:00:00.123456789Z",
        ] {
            assert_eq!(checked_task_time(value).unwrap(), value);
        }
        for value in [
            "PRIVATE",
            "2025-02-29T00:00:00Z",
            "1900-02-29T00:00:00Z",
            "2026-00-01T00:00:00Z",
            "2026-04-31T00:00:00Z",
            "2026-10-01T24:00:00Z",
            "2026-10-01T00:00:00.Z",
            "2026-10-01T00:00:00.1234567890Z",
            "2026-10-01T00:00:00+00:00",
            "2026-10-01T00:00:00.PRIVATEZ",
            "ééééééééééZ",
        ] {
            assert_eq!(checked_task_time(value), Err(Error::Schema), "{value}");
        }
    }
}
