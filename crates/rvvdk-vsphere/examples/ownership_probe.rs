//! Synthetic local journal timing. Never connects to VMware or writes disk payload.
#[cfg(target_os = "linux")]
fn main() {
    if run().is_err() {
        eprintln!("ownership probe failed");
        std::process::exit(1);
    }
}
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("ownership probe requires Linux");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn run() -> Result<(), Box<dyn std::error::Error>> {
    use rvvdk_vsphere::{contract::*, ownership::*};
    use std::{
        fs,
        os::unix::fs::{DirBuilderExt, MetadataExt},
        path::PathBuf,
        time::Instant,
    };
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: ownership_probe NEW_PRIVATE_DIRECTORY COUNT".into());
    }
    let root = PathBuf::from(&args[0]);
    let count: usize = args[1].to_str().ok_or("count")?.parse()?;
    if !(1..=128).contains(&count) {
        return Err("count must be 1..128".into());
    }
    // Never reuse or remove an existing directory, even after a failed probe.
    fs::DirBuilder::new().mode(0o700).create(&root)?;
    let source = SourceSelection::new(
        EndpointIdentity::pinned(
            "https://example.invalid",
            &"ab".repeat(32),
            PinProvenance::TrustOnFirstUse,
        )?,
        "vm-synthetic",
        "01234567-89ab-cdef-0123-456789abcdef",
        2000,
        "[synthetic] disk.vmdk",
        1 << 30,
    )?;
    let mut rows = Vec::with_capacity(count);
    for n in 1..=count {
        let id = ArtifactId::new((n as u128).to_le_bytes())?;
        let mut store = JobStore::open(&root)?;
        let now = Instant::now();
        let mut job = store.create(id, &source)?;
        let create_ns = now.elapsed().as_nanos() as u64;
        let now = Instant::now();
        job.prepare_stage()?;
        let stage_ns = now.elapsed().as_nanos() as u64;
        let now = Instant::now();
        job.begin_acquire()?;
        job.acknowledge_lease("synthetic-lease")?;
        job.transfer_complete()?;
        job.begin_complete()?;
        job.acknowledge_complete()?;
        let lease_journal_ns = now.elapsed().as_nanos() as u64;
        let operation = job.operation();
        drop(job);
        drop(store);
        let now = Instant::now();
        let mut store = JobStore::open(&root)?;
        let report = store.recover(id, &source)?;
        let recover_ns = now.elapsed().as_nanos() as u64;
        assert_eq!(report.action, RecoveryAction::CheckOwnedLocalResources);
        let now = Instant::now();
        store.cleanup_local(id, operation, &source)?;
        let cleanup_ns = now.elapsed().as_nanos() as u64;
        assert_eq!(store.recover(id, &source)?.state, JobState::Cleaned);
        rows.push(
            serde_json::json!({"index":n,"create_ns":create_ns,"stage_ns":stage_ns,
            "lease_journal_ns":lease_journal_ns,"recover_ns":recover_ns,"cleanup_ns":cleanup_ns}),
        );
    }
    // Terminal tombstones remain throughout the measurement. Remove only this
    // example's newly-created synthetic fixture, outside every timed phase.
    assert_eq!(fs::read_dir(&root)?.count(), count);
    let mut logical = 0;
    let mut allocated = 0;
    let mut largest = 0;
    for entry in fs::read_dir(&root)? {
        let m = entry?.metadata()?;
        logical += m.len();
        allocated += m.blocks() * 512;
        largest = largest.max(m.len());
    }
    fs::remove_dir_all(&root)?;
    println!(
        "{}",
        serde_json::json!({"schema_version":1,"scope":"synthetic_local_ownership", "jobs":count,"journal_commits_per_job":10,
            "journal_logical_bytes":logical,"journal_allocated_bytes":allocated,"largest_record_bytes":largest,"rows":rows})
    );
    Ok(())
}
