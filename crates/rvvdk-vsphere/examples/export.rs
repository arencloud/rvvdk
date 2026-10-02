//! Internal standalone-host export proof. No password command arguments or env.
use rvvdk_vsphere::contract::{EndpointIdentity, PinProvenance, SourceSelection};
use rvvdk_vsphere::{
    ConnectionPolicy, Credentials, Error, ExportOptions, export_selected_vm, export_vm,
};
use std::{path::PathBuf, time::Duration};
#[cfg(target_os = "linux")]
fn usage() -> (Option<f64>, Option<i64>) {
    let mut value = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: valid writable storage; only read after getrusage reports success.
    if unsafe { libc::getrusage(libc::RUSAGE_SELF, value.as_mut_ptr()) } != 0 {
        return (None, None);
    }
    let value = unsafe { value.assume_init() };
    let cpu = value.ru_utime.tv_sec as f64
        + value.ru_utime.tv_usec as f64 / 1e6
        + value.ru_stime.tv_sec as f64
        + value.ru_stime.tv_usec as f64 / 1e6;
    (Some(cpu), Some(value.ru_maxrss))
}
#[cfg(not(target_os = "linux"))]
fn usage() -> (Option<f64>, Option<i64>) {
    (None, None)
}

#[derive(serde::Serialize)]
struct Measurement {
    #[serde(flatten)]
    report: rvvdk_vsphere::ExportReport,
    cpu_seconds: Option<f64>,
    process_peak_rss_kib: Option<i64>,
}
fn main() {
    if let Err(error) = run() {
        eprintln!("export proof failed: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Error> {
    let mut args = std::env::args().skip(1);
    let mut endpoint = None;
    let mut pin = None;
    let mut user = "root".to_owned();
    let mut options = ExportOptions::probe(30 * 1024 * 1024 * 1024);
    let mut cancel_after = None;
    let (mut vm_reference, mut vm_uuid, mut disk_key, mut disk_backing, mut provenance) =
        (None, None, None, None, None);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--endpoint" => endpoint = args.next(),
            "--certificate-sha256" => pin = args.next(),
            "--user" => user = args.next().ok_or(Error::InvalidInput)?,
            "--capacity-bytes" => {
                options.single_disk_capacity_bytes = args
                    .next()
                    .ok_or(Error::InvalidInput)?
                    .parse()
                    .map_err(|_| Error::InvalidInput)?
            }
            "--output" => {
                options.output = PathBuf::from(args.next().ok_or(Error::InvalidInput)?);
                options.probe_only = false;
            }
            "--vm-reference" => vm_reference = Some(args.next().ok_or(Error::InvalidInput)?),
            "--vm-uuid" => vm_uuid = Some(args.next().ok_or(Error::InvalidInput)?),
            "--disk-key" => {
                disk_key = Some(
                    args.next()
                        .ok_or(Error::InvalidInput)?
                        .parse::<u64>()
                        .map_err(|_| Error::InvalidInput)?,
                )
            }
            "--disk-backing" => disk_backing = Some(args.next().ok_or(Error::InvalidInput)?),
            "--pin-provenance" => {
                provenance = Some(match args.next().as_deref() {
                    Some("tofu") => PinProvenance::TrustOnFirstUse,
                    Some("externally-verified") => PinProvenance::ExternallyVerified,
                    _ => return Err(Error::InvalidInput),
                })
            }
            "--allow-shutdown" => options.allow_graceful_shutdown = true,
            "--inspect" => options.inspect_only = true,
            "--max-bytes" => {
                options.max_encoded_bytes = args
                    .next()
                    .ok_or(Error::InvalidInput)?
                    .parse()
                    .map_err(|_| Error::InvalidInput)?
            }
            "--timeout-seconds" => {
                options.transfer_timeout = Duration::from_secs(
                    args.next()
                        .ok_or(Error::InvalidInput)?
                        .parse()
                        .map_err(|_| Error::InvalidInput)?,
                )
            }
            "--cancel-after-ms" => {
                cancel_after = Some(Duration::from_millis(
                    args.next()
                        .ok_or(Error::InvalidInput)?
                        .parse()
                        .map_err(|_| Error::InvalidInput)?,
                ))
            }
            "--help" => {
                println!(
                    "export --endpoint HTTPS_URL --certificate-sha256 SHA256 [--user USER] [--capacity-bytes N] [--inspect] [--output NEW_DIRECTORY --allow-shutdown] [--max-bytes N] [--timeout-seconds N] [--cancel-after-ms N]\nDefault: one 30 GiB disk VM eligibility probe; requires poweredOff, no power change or download. --inspect reads recent tasks without acquiring a lease and permits poweredOn. Supplying output enables export. Shutdown is graceful only and requires its flag. The selected VM is left in its last observed state. Ctrl-C requests cooperative abort/logout."
                );
                println!(
                    "Explicit selection: supply all of --vm-reference ID --vm-uuid UUID --disk-key N --disk-backing ID --pin-provenance tofu|externally-verified. Capacity remains an expected identity field. This mode requires poweredOff and rejects --allow-shutdown. It adds fresh identity checks; durable workflow recovery and conversion remain pending. Identities and export reports are private operational data."
                );
                return Ok(());
            }
            _ => return Err(Error::InvalidInput),
        }
    }
    let endpoint = endpoint.ok_or(Error::InvalidInput)?;
    let pin = pin.ok_or(Error::InvalidInput)?;
    let policy = ConnectionPolicy::pinned(&endpoint, &pin)?;
    let source = match (vm_reference, vm_uuid, disk_key, disk_backing, provenance) {
        (None, None, None, None, None) => None,
        (Some(reference), Some(uuid), Some(key), Some(backing), Some(provenance))
            if !options.allow_graceful_shutdown =>
        {
            Some(
                SourceSelection::new(
                    EndpointIdentity::pinned(&endpoint, &pin, provenance)
                        .map_err(|_| Error::InvalidInput)?,
                    &reference,
                    &uuid,
                    key,
                    &backing,
                    options.single_disk_capacity_bytes,
                )
                .map_err(|_| Error::InvalidInput)?,
            )
        }
        _ => return Err(Error::InvalidInput),
    };
    let credentials = Credentials::new(
        user,
        rpassword::prompt_password("ESXi password (memory only): ")
            .map_err(|_| Error::InvalidInput)?,
    )?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| Error::Transport)?;
    let before = usage();
    let report = runtime.block_on(async {
        let cancellation = options.cancellation.clone();
        let handler = tokio::spawn(async move {
            if tokio::signal::ctrl_c().await.is_ok() {
                cancellation.cancel();
            }
        });
        let timer = cancel_after.map(|delay| {
            let cancellation = options.cancellation.clone();
            tokio::spawn(async move {
                tokio::time::sleep(delay).await;
                cancellation.cancel();
            })
        });
        let report = match source {
            Some(source) => export_selected_vm(source, &credentials, options).await,
            None => export_vm(policy, &credentials, options).await,
        };
        handler.abort();
        if let Some(timer) = timer {
            timer.abort();
        }
        report
    })?;
    let after = usage();
    let success = report.is_success()
        || (report.inspection_only
            && report.primary_error.is_none()
            && report.session_cleanup == rvvdk_vsphere::Cleanup::LoggedOut
            && report.pagination_cleanup_error.is_none())
        || (report.probe_only
            && report.primary_error.is_none()
            && report.lease_cleanup == rvvdk_vsphere::LeaseCleanup::Aborted
            && report.session_cleanup == rvvdk_vsphere::Cleanup::LoggedOut);
    let primary_error = report.primary_error;
    let measurement = Measurement {
        report,
        cpu_seconds: before.0.zip(after.0).map(|(b, a)| a - b),
        process_peak_rss_kib: after.1,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&measurement).map_err(|_| Error::Schema)?
    );
    if success {
        Ok(())
    } else {
        Err(primary_error.unwrap_or(Error::LeaseState))
    }
}
