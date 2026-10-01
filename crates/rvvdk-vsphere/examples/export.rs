//! Internal standalone-host export proof. No password command arguments or env.
use rvvdk_vsphere::{ConnectionPolicy, Credentials, Error, ExportOptions, export_vm};
use std::{path::PathBuf, time::Duration};
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
            "--allow-shutdown" => options.allow_graceful_shutdown = true,
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
                    "export --endpoint HTTPS_URL --certificate-sha256 SHA256 [--user USER] [--capacity-bytes N] [--output NEW_DIRECTORY --allow-shutdown] [--max-bytes N] [--timeout-seconds N] [--cancel-after-ms N]\nDefault: one 30 GiB disk VM eligibility probe; no power change or download. Supplying output enables export. Shutdown is graceful only and requires its flag. The selected VM is left in its last observed state. Ctrl-C requests cooperative abort/logout."
                );
                return Ok(());
            }
            _ => return Err(Error::InvalidInput),
        }
    }
    let policy = ConnectionPolicy::pinned(
        &endpoint.ok_or(Error::InvalidInput)?,
        &pin.ok_or(Error::InvalidInput)?,
    )?;
    let credentials = Credentials::new(
        user,
        rpassword::prompt_password("ESXi password (memory only): ")
            .map_err(|_| Error::InvalidInput)?,
    )?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| Error::Transport)?;
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
        let report = export_vm(policy, &credentials, options).await;
        handler.abort();
        if let Some(timer) = timer {
            timer.abort();
        }
        report
    })?;
    let success = report.is_success()
        || (report.probe_only
            && report.primary_error.is_none()
            && report.lease_cleanup == rvvdk_vsphere::LeaseCleanup::Aborted
            && report.session_cleanup == rvvdk_vsphere::Cleanup::LoggedOut);
    println!(
        "{}",
        serde_json::to_string_pretty(&report).map_err(|_| Error::Schema)?
    );
    if success {
        Ok(())
    } else {
        Err(report.primary_error.unwrap_or(Error::LeaseState))
    }
}
