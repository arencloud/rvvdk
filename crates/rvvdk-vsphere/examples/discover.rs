//! Internal qualification entry point. Never accepts a password argument or env var.
use rvvdk_vsphere::{
    Cleanup, ConnectionPolicy, ConnectionReuse, Credentials, DiscoveryReport, Error,
    InventoryLimits, discover,
};
use serde::Serialize;

#[derive(Serialize)]
struct Run {
    index: usize,
    report: DiscoveryReport,
    cpu_seconds: Option<f64>,
    process_peak_rss_kib: Option<i64>,
}
#[derive(Serialize)]
struct Evidence {
    schema: u32,
    scope: &'static str,
    expected_inventory_limit: bool,
    runs: Vec<Run>,
}

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

fn main() {
    if let Err(error) = run() {
        eprintln!("qualification failed: {error}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Error> {
    let mut args = std::env::args().skip(1);
    let mut endpoint = None;
    let mut pin = None;
    let mut user = "root".to_owned();
    let mut paired = false;
    let mut fail_inventory = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--endpoint" => endpoint = args.next(),
            "--certificate-sha256" => pin = args.next(),
            "--user" => user = args.next().ok_or(Error::InvalidInput)?,
            "--paired" => paired = true,
            "--fail-inventory" => fail_inventory = true,
            "--help" => {
                println!(
                    "discover --endpoint HTTPS_URL --certificate-sha256 SHA256 [--user USER] [--paired | --fail-inventory]\nPassword is read from the terminal without echo. Default: three reused-connection sessions; paired: three alternating fresh/reuse pairs; fail-inventory: one bounded failure and logout."
                );
                return Ok(());
            }
            _ => return Err(Error::InvalidInput),
        }
    }
    if paired && fail_inventory {
        return Err(Error::InvalidInput);
    }
    let policy = ConnectionPolicy::pinned(
        &endpoint.ok_or(Error::InvalidInput)?,
        &pin.ok_or(Error::InvalidInput)?,
    )?;
    let password = rpassword::prompt_password("ESXi password (memory only): ")
        .map_err(|_| Error::InvalidInput)?;
    let credentials = Credentials::new(user, password)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| Error::Transport)?;
    let sequence = if paired {
        vec![
            ConnectionReuse::Fresh,
            ConnectionReuse::Reuse,
            ConnectionReuse::Reuse,
            ConnectionReuse::Fresh,
            ConnectionReuse::Fresh,
            ConnectionReuse::Reuse,
        ]
    } else if fail_inventory {
        vec![ConnectionReuse::Reuse]
    } else {
        vec![ConnectionReuse::Reuse; 3]
    };
    let limits = InventoryLimits {
        max_objects: if fail_inventory { 1 } else { 128 },
        ..InventoryLimits::default()
    };
    let mut runs = Vec::new();
    let mut passed = true;
    for (index, reuse) in sequence.into_iter().enumerate() {
        let before = usage();
        let report = runtime.block_on(discover(
            policy.clone().with_connection_reuse(reuse),
            &credentials,
            limits,
        ))?;
        let after = usage();
        passed &= if fail_inventory {
            report.primary_error == Some(Error::InventoryLimit)
                && report.cleanup == Cleanup::LoggedOut
                && report.pagination_cleanup_error.is_none()
        } else {
            report.is_success()
        };
        runs.push(Run {
            index: index + 1,
            report,
            cpu_seconds: before.0.zip(after.0).map(|(b, a)| a - b),
            process_peak_rss_kib: after.1,
        });
        // Stop after any unexpected failure to avoid repeated authentication attempts.
        if !passed {
            break;
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&Evidence {
            schema: 1,
            scope: "rust_read_only_inventory",
            expected_inventory_limit: fail_inventory,
            runs
        })
        .map_err(|_| Error::Schema)?
    );
    if passed { Ok(()) } else { Err(Error::Schema) }
}
