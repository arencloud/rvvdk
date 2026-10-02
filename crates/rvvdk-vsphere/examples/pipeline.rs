//! Linux qualification runner. Config/inventory are private; passwords terminal-only.
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("pipeline requires Linux");
    std::process::exit(1);
}
#[cfg(target_os = "linux")]
fn main() {
    if app::run().is_err() {
        eprintln!("pipeline command failed; inspect the retained report/resources");
        std::process::exit(1);
    }
}
#[cfg(target_os = "linux")]
mod app {
    use rvvdk_vsphere::{contract::*, ownership::*, *};
    use serde::Deserialize;
    use std::{
        fs::OpenOptions,
        io::Read,
        os::unix::fs::{MetadataExt, OpenOptionsExt},
        path::PathBuf,
        time::{Duration, Instant},
    };
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Selection {
        vm_reference: String,
        bios_uuid: String,
        disk_key: u64,
        backing: String,
        logical_bytes: u64,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Config {
        endpoint: String,
        certificate_sha256: String,
        user: String,
        selection: Option<Selection>,
        store: Option<PathBuf>,
        destination: Option<PathBuf>,
        artifact_id: Option<String>,
        output_id: Option<String>,
        max_encoded_bytes: Option<u64>,
        timeout_seconds: Option<u64>,
    }
    fn input<T>(v: Option<T>) -> Result<T> {
        v.ok_or(Error::InvalidInput)
    }
    fn id(s: &str) -> Result<[u8; 16]> {
        if s.len() != 32 || !s.is_ascii() {
            return Err(Error::InvalidInput);
        }
        let mut v = [0; 16];
        for (i, b) in v.iter_mut().enumerate() {
            *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|_| Error::InvalidInput)?;
        }
        Ok(v)
    }
    fn source(c: &Config) -> Result<SourceSelection> {
        let s = input(c.selection.as_ref())?;
        SourceSelection::new(
            EndpointIdentity::pinned(
                &c.endpoint,
                &c.certificate_sha256,
                PinProvenance::TrustOnFirstUse,
            )
            .map_err(|_| Error::InvalidInput)?,
            &s.vm_reference,
            &s.bios_uuid,
            s.disk_key,
            &s.backing,
            s.logical_bytes,
        )
        .map_err(|_| Error::InvalidInput)
    }
    fn local<T>(r: std::result::Result<T, OwnershipError>) -> Result<T> {
        r.map_err(|_| Error::Artifact)
    }
    fn usage() -> (f64, i64) {
        let mut r = std::mem::MaybeUninit::<libc::rusage>::uninit();
        // SAFETY: writable output, only read after successful getrusage.
        if unsafe { libc::getrusage(libc::RUSAGE_SELF, r.as_mut_ptr()) } != 0 {
            return (0., 0);
        }
        // SAFETY: successful call initialized the value.
        let r = unsafe { r.assume_init() };
        (
            r.ru_utime.tv_sec as f64
                + r.ru_utime.tv_usec as f64 / 1e6
                + r.ru_stime.tv_sec as f64
                + r.ru_stime.tv_usec as f64 / 1e6,
            r.ru_maxrss,
        )
    }
    pub fn run() -> Result<()> {
        let args: Vec<_> = std::env::args_os().skip(1).collect();
        if args.len() == 1 && args[0] == "--help" {
            println!(
                "pipeline inventory|inspect|run|cleanup|cleanup-aborted PRIVATE_CONFIG.json\nLinux internal qualification runner. Config must be a private regular file <=16 KiB, with endpoint, certificate_sha256, user; run/inspect require explicit selection. Run/cleanup also require existing private store/destination directories and nonzero 32-hex artifact_id/output_id. Optional max_encoded_bytes and timeout_seconds bound export/local stages. Password is prompted without echo for network commands. Inventory output includes private VM/disk identities: redirect only to private evidence. Run retains source and published RAW on success; failures retain resources. Cleanup is a separate explicit deletion request for the published output and eligible source, preserving journals. cleanup-aborted explicitly removes only an acknowledged aborted source stage; it does not touch output. No power changes, inferred selection, retry or uncertain-state repair. Ctrl-C cooperatively cancels run; await process completion."
            );
            return Ok(());
        }
        if args.len() != 2 {
            return Err(Error::InvalidInput);
        }
        let command = args[0].to_str().ok_or(Error::InvalidInput)?;
        if !matches!(
            command,
            "inventory" | "inspect" | "run" | "cleanup" | "cleanup-aborted"
        ) {
            return Err(Error::InvalidInput);
        }
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
            .open(&args[1])
            .map_err(|_| Error::InvalidInput)?;
        let m = file.metadata().map_err(|_| Error::InvalidInput)?;
        // SAFETY: geteuid takes no arguments.
        if !m.is_file()
            || m.nlink() != 1
            || m.mode() & 0o077 != 0
            || m.uid() != unsafe { libc::geteuid() }
            || m.len() > 16384
        {
            return Err(Error::InvalidInput);
        }
        let mut bytes = Vec::new();
        file.take(16385)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::InvalidInput)?;
        if bytes.len() > 16384 {
            return Err(Error::InvalidInput);
        }
        let c: Config = serde_json::from_slice(&bytes).map_err(|_| Error::InvalidInput)?;
        let now = Instant::now();
        let before = usage();
        let (success, data) = if command == "cleanup" || command == "cleanup-aborted" {
            let s = source(&c)?;
            let a = ArtifactId::new(id(input(c.artifact_id.as_deref())?)?)
                .map_err(|_| Error::InvalidInput)?;
            let o = local(OutputId::new(id(input(c.output_id.as_deref())?)?))?;
            let mut store = local(JobStore::open(input(c.store.as_deref())?))?;
            let aborted = command == "cleanup-aborted";
            let dest = if aborted {
                None
            } else {
                Some(local(PublicationDirectory::open(input(
                    c.destination.as_deref(),
                )?))?)
            };
            let prior = local(store.recover(a, &s))?;
            if prior.pending_transaction
                || !(if aborted {
                    matches!(prior.state, JobState::AbortedLease | JobState::Cleaned)
                } else {
                    matches!(
                        prior.state,
                        JobState::CompletedLease | JobState::CleanupIntent | JobState::Cleaned
                    )
                })
            {
                return Err(Error::Artifact);
            }
            let output_error = if aborted {
                None
            } else {
                store
                    .cleanup_output(o, a, &s, dest.as_ref(), RetainedOptions::default())
                    .err()
            };
            let source_error = if output_error.is_none() {
                store.cleanup_local(a, prior.operation, &s).err()
            } else {
                None
            };
            let success = output_error.is_none() && source_error.is_none();
            (
                success,
                serde_json::json!({"output_error":output_error,"source_error":source_error,"output_cleanup_requested":!aborted,"output_cleaned":!aborted&&output_error.is_none(),"source_cleaned":success}),
            )
        } else {
            let credentials = Credentials::new(
                c.user.clone(),
                rpassword::prompt_password("ESXi password (memory only): ")
                    .map_err(|_| Error::InvalidInput)?,
            )?;
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|_| Error::Transport)?;
            rt.block_on(async {
                if command == "inventory" {
                    let report = discover(
                        ConnectionPolicy::pinned(&c.endpoint, &c.certificate_sha256)?,
                        &credentials,
                        InventoryLimits::default(),
                    )
                    .await?;
                    let selections: Vec<_> = report
                        .inventory
                        .as_ref()
                        .into_iter()
                        .flat_map(|i| i.vms.iter())
                        .flat_map(|vm| {
                            vm.disks.iter().map(move |disk| {
                                serde_json::json!({
                                    "vm_reference": vm.identity.managed_reference(),
                                    "bios_uuid": vm.identity.bios_uuid(),
                                    "disk_key": disk.device_key(),
                                    "backing": disk.backing_identity(),
                                    "logical_bytes": disk.capacity_bytes,
                                    "power_state": vm.power_state,
                                })
                            })
                        })
                        .collect();
                    return Ok((
                        report.is_success(),
                        serde_json::json!({"report": report, "selections": selections}),
                    ));
                }
                let s = source(&c)?;
                if command == "inspect" {
                    let mut options = ExportOptions::probe(s.logical_bytes());
                    options.inspect_only = true;
                    let report = export_selected_vm(s, &credentials, options).await?;
                    let success = report.primary_error.is_none()
                        && report.session_cleanup == Cleanup::LoggedOut
                        && report.pagination_cleanup_error.is_none();
                    return Ok((
                        success,
                        serde_json::to_value(report).map_err(|_| Error::Schema)?,
                    ));
                }
                let store = local(JobStore::open(input(c.store.as_deref())?))?;
                let dest = local(PublicationDirectory::open(input(c.destination.as_deref())?))?;
                let artifact = ArtifactId::new(id(input(c.artifact_id.as_deref())?)?)
                    .map_err(|_| Error::InvalidInput)?;
                let output = local(OutputId::new(id(input(c.output_id.as_deref())?)?))?;
                let mut options = PipelineOptions::default();
                if let Some(v) = c.max_encoded_bytes {
                    options.transfer.max_encoded_bytes = v;
                }
                if let Some(v) = c.timeout_seconds {
                    options.transfer.transfer_timeout = Duration::from_secs(v);
                    options.local_timeout = Duration::from_secs(v);
                }
                let cancel = options.transfer.cancellation.clone();
                let handler = tokio::spawn(async move {
                    if tokio::signal::ctrl_c().await.is_ok() {
                        cancel.cancel();
                    }
                });
                let report =
                    run_export_pipeline(s, &credentials, store, dest, artifact, output, options)
                        .await;
                handler.abort();
                Ok((
                    report.is_success(),
                    serde_json::to_value(report).map_err(|_| Error::Schema)?,
                ))
            })?
        };
        let after = usage();
        println!("{}",serde_json::to_string_pretty(&serde_json::json!({"success":success,"data":data,"cpu_seconds":after.0-before.0,"process_peak_rss_kib":after.1,"elapsed_ms":now.elapsed().as_secs_f64()*1000.})).map_err(|_|Error::Schema)?);
        if success {
            Ok(())
        } else {
            Err(Error::Artifact)
        }
    }
}
