use super::*;
use crate::artifact_fixture as fixture;
use crate::contract::{ArtifactObservation, Completeness, PinProvenance, ValidationClaim};
use rvvdk_datamover::{CopyEvent, CopyPhase};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, FileExt},
    path::PathBuf,
};
pub(super) struct Temp(pub(super) PathBuf);
impl Temp {
    pub(super) fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let root = std::env::var_os("RVVDK_RETAINED_PARENT")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let path = root.join(format!(
            "rvddk-retained-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }
    pub(super) fn jobs(&self) -> PathBuf {
        self.0.join("jobs")
    }
    pub(super) fn store(&self) -> JobStore {
        JobStore::open(&self.jobs()).unwrap()
    }
    pub(super) fn stage(&self) -> PathBuf {
        fs::read_dir(self.jobs())
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.is_dir())
            .unwrap()
    }
    pub(super) fn output(&self, size: u64) -> RawDisk<LocalFileBlockDevice> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(self.0.join("output.raw"))
            .unwrap();
        file.set_len(size).unwrap();
        RawDisk::new(LocalFileBlockDevice::from_buffered_file(file).unwrap())
    }
    pub(super) fn populate(&self, data: &[u8], capacity: u64) -> SourceSelection {
        fs::DirBuilder::new()
            .mode(0o700)
            .create(self.jobs())
            .unwrap();
        let source = source(capacity);
        let mut store = self.store();
        let mut job = store.create(id(), &source).unwrap();
        job.prepare_stage().unwrap();
        job.begin_acquire().unwrap();
        job.acknowledge_lease("synthetic").unwrap();
        let mut file = job.payload_file().unwrap();
        file.write_all(data).unwrap();
        file.sync_all().unwrap();
        drop(file);
        let metadata = ExportArtifact::new(
            id(),
            &source,
            ArtifactObservation {
                container_bytes: data.len() as u64,
                completeness: Completeness::Complete,
                validation: ValidationClaim::ContainerDigestVerified,
                container_sha256: Some(Sha256::digest(data).into()),
            },
        )
        .unwrap();
        let mut file = job.metadata_file().unwrap();
        file.write_all(&metadata.to_json().unwrap()).unwrap();
        file.sync_all().unwrap();
        drop(file);
        job.transfer_complete().unwrap();
        job.begin_complete().unwrap();
        job.acknowledge_complete().unwrap();
        source
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
pub(super) fn id() -> ArtifactId {
    ArtifactId::new([17; 16]).unwrap()
}
pub(super) fn source(capacity: u64) -> SourceSelection {
    SourceSelection::new(
        crate::contract::EndpointIdentity::pinned(
            "https://example.invalid",
            &"ab".repeat(32),
            PinProvenance::TrustOnFirstUse,
        )
        .unwrap(),
        "vm-synthetic",
        "01234567-89ab-cdef-0123-456789abcdef",
        2000,
        "synthetic",
        capacity,
    )
    .unwrap()
}
pub(super) fn data() -> Vec<u8> {
    fixture::image(
        true,
        32,
        &[
            (0, fixture::stored(&fixture::bytes(0))),
            (16, fixture::stored(&fixture::bytes(16))),
        ],
    )
}
pub(super) fn open(t: &Temp, s: SourceSelection) -> Result<RetainedArtifact> {
    RetainedArtifact::open(t.store(), id(), s, RetainedOptions::default())
}
fn verify_output(t: &Temp, grains: u64, present: impl Fn(u64) -> bool) {
    let file = File::open(t.0.join("output.raw")).unwrap();
    let mut buf = vec![0; 65536];
    for i in 0..grains {
        file.read_exact_at(&mut buf, i * 65536).unwrap();
        if present(i) {
            assert_eq!(buf, fixture::bytes(i));
        } else {
            assert!(buf.iter().all(|&v| v == 0), "grain {i}");
        }
    }
}
#[test]
fn retained_lock_conversion_and_cleanup_have_separate_lifetimes() {
    for concurrency in [1, 2] {
        let t = Temp::new();
        let source = t.populate(&data(), 32 * 65536);
        let journal = fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap();
        let mut a = open(&t, source.clone()).unwrap();
        assert_eq!(a.logical_bytes(), 32 * 65536);
        assert!(matches!(
            JobStore::open(&t.jobs()),
            Err(OwnershipError::Busy)
        ));
        assert_eq!(format!("{a:?}"), "RetainedArtifact([private])");
        let dest = t.output(a.logical_bytes());
        dest.write_all_at(65536, &vec![0x55; 65536]).unwrap();
        let phases = std::sync::Mutex::new(Vec::new());
        let copy = CopyOptions::with_concurrency(65536, 4096, concurrency).unwrap();
        let report = a
            .convert_to(&dest, copy, RetainedOptions::default(), &|e: &CopyEvent| {
                assert!(matches!(
                    JobStore::open(&t.jobs()),
                    Err(OwnershipError::Busy)
                ));
                phases.lock().unwrap().push(e.phase);
            })
            .unwrap();
        assert_eq!(report.stats().bytes_read(), 2 * 65536);
        assert!(phases.lock().unwrap().contains(&CopyPhase::Completed));
        verify_output(&t, 32, |i| i == 0 || i == 16);
        assert_eq!(
            fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap(),
            journal
        );
        drop(a);
        let mut store = t.store();
        let r = store.recover(id(), &source).unwrap();
        assert_eq!(r.state, JobState::CompletedLease);
        store.cleanup_local(id(), r.operation, &source).unwrap();
        assert!(t.0.join("output.raw").exists());
    }
}
#[test]
fn open_rejects_pending_nonterminal_wrong_source_and_stale_store_records() {
    for kind in ["transaction", "state", "source", "record", "store"] {
        let t = Temp::new();
        let mut source = t.populate(&data(), 32 * 65536);
        match kind {
            "transaction" => {
                fs::write(t.jobs().join(transaction_name(id().as_bytes())), b"pending").unwrap();
            }
            "state" => {
                let mut store = t.store();
                let mut record = store.read(id(), &source).unwrap();
                record.state = JobState::CompleteIntent;
                record.sequence = 6;
                store.commit(&record, false).unwrap();
            }
            "source" => {
                source = super::tests::source(64 * 65536);
            }
            "record" => {
                fs::write(t.jobs().join(record_name(id().as_bytes())), b"{}").unwrap();
            }
            _ => {
                let mut store = t.store();
                let mut record = store.read(id(), &source).unwrap();
                record.store.ino += 1;
                store.commit(&record, false).unwrap();
            }
        }
        assert!(open(&t, source).is_err(), "{kind}");
        assert!(JobStore::open(&t.jobs()).is_ok());
    }
}
#[test]
fn open_rejects_forged_claims_bad_bytes_and_foreign_resources() {
    for kind in [
        "digest",
        "native",
        "capacity",
        "metadata",
        "id",
        "oversized",
        "marker",
        "replacement",
        "symlink",
        "hardlink",
    ] {
        let t = Temp::new();
        let source = t.populate(&data(), 32 * 65536);
        let stage = t.stage();
        match kind {
            "digest" => {
                let f = OpenOptions::new()
                    .write(true)
                    .open(stage.join(MEMBERS[0]))
                    .unwrap();
                f.write_all_at(b"x", 12000).unwrap();
            }
            "native" | "capacity" => {
                let mut bytes = data();
                if kind == "native" {
                    bytes[65560] ^= 1;
                } else {
                    fixture::put64(&mut bytes, 12, 1);
                }
                fs::write(stage.join(MEMBERS[0]), &bytes).unwrap();
                let mut m: serde_json::Value =
                    serde_json::from_slice(&fs::read(stage.join(MEMBERS[1])).unwrap()).unwrap();
                m["validation"] = "logical_readback_verified".into();
                m["container_sha256"] =
                    crate::transport::hex_string(&Sha256::digest(&bytes)).into();
                fs::write(stage.join(MEMBERS[1]), serde_json::to_vec(&m).unwrap()).unwrap();
            }
            "metadata" | "id" => {
                let mut m: serde_json::Value =
                    serde_json::from_slice(&fs::read(stage.join(MEMBERS[1])).unwrap()).unwrap();
                if kind == "id" {
                    m["artifact_id"] = "aa".repeat(16).into();
                } else {
                    m["path"] = "/PRIVATE/no-authority".into();
                }
                fs::write(stage.join(MEMBERS[1]), serde_json::to_vec(&m).unwrap()).unwrap();
            }
            "oversized" => fs::write(stage.join(MEMBERS[1]), vec![b' '; 4097]).unwrap(),
            "marker" => fs::write(stage.join(MEMBERS[2]), [0; 16]).unwrap(),
            "replacement" => {
                fs::rename(stage.join(MEMBERS[0]), stage.join("original")).unwrap();
                fs::write(stage.join(MEMBERS[0]), data()).unwrap();
            }
            "symlink" => {
                fs::rename(stage.join(MEMBERS[0]), stage.join("original")).unwrap();
                std::os::unix::fs::symlink("original", stage.join(MEMBERS[0])).unwrap();
            }
            _ => fs::hard_link(stage.join(MEMBERS[0]), stage.join("alias")).unwrap(),
        }
        assert!(open(&t, source).is_err(), "{kind}");
        assert!(stage.exists());
    }
}
#[test]
fn every_serialized_validation_claim_still_requires_fresh_admission() {
    for claim in [
        "unchecked",
        "container_digest_verified",
        "logical_readback_verified",
    ] {
        let t = Temp::new();
        let source = t.populate(&data(), 32 * 65536);
        let path = t.stage().join(MEMBERS[1]);
        let mut m: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        m["validation"] = claim.into();
        fs::write(path, serde_json::to_vec(&m).unwrap()).unwrap();
        let mut a = open(&t, source).unwrap();
        let dest = t.output(a.logical_bytes());
        a.convert_to(
            &dest,
            CopyOptions::default(),
            RetainedOptions::default(),
            &|_: &CopyEvent| {},
        )
        .unwrap();
        verify_output(&t, 32, |i| i == 0 || i == 16);
    }
}
#[test]
fn conversion_rechecks_changes_before_writes_and_rejects_aliases() {
    for kind in [
        "payload", "metadata", "record", "marker", "alias", "budget", "short",
    ] {
        let t = Temp::new();
        let source = t.populate(&data(), 32 * 65536);
        let mut a = open(&t, source).unwrap();
        let stage = t.stage();
        let dest = t.output(if kind == "short" {
            65536
        } else {
            a.logical_bytes()
        });
        dest.write_all_at(0, &[0x55; 512]).unwrap();
        match kind {
            "payload" => OpenOptions::new()
                .write(true)
                .open(stage.join(MEMBERS[0]))
                .unwrap()
                .write_all_at(b"x", 12000)
                .unwrap(),
            "metadata" => fs::write(stage.join(MEMBERS[1]), b"{}").unwrap(),
            "record" => fs::write(t.jobs().join(record_name(id().as_bytes())), b"{}").unwrap(),
            "marker" => fs::write(stage.join(MEMBERS[2]), [0; 16]).unwrap(),
            _ => {}
        }
        let copy = if kind == "budget" {
            CopyOptions::default().with_memory_budget(0)
        } else {
            CopyOptions::default()
        };
        let alias = if kind == "alias" {
            Some(RawDisk::new(
                LocalFileBlockDevice::open_read_write(stage.join(MEMBERS[0])).unwrap(),
            ))
        } else {
            None
        };
        assert!(
            a.convert_to(
                alias.as_ref().unwrap_or(&dest),
                copy,
                RetainedOptions::default(),
                &|_: &CopyEvent| {}
            )
            .is_err(),
            "{kind}"
        );
        let mut buf = [0; 512];
        dest.read_exact_at(0, &mut buf).unwrap();
        assert_eq!(buf, [0x55; 512]);
        assert!(matches!(
            JobStore::open(&t.jobs()),
            Err(OwnershipError::Busy)
        ));
    }
}
#[test]
fn cancellation_and_deadline_preserve_source_and_report_partial_destination() {
    for phase in [
        CopyPhase::Preparing,
        CopyPhase::Transferring,
        CopyPhase::Flushing,
    ] {
        let t = Temp::new();
        let source = t.populate(&data(), 32 * 65536);
        let mut a = open(&t, source.clone()).unwrap();
        let dest = t.output(a.logical_bytes());
        let options = RetainedOptions::default();
        let cancel = options.cancellation.clone();
        let result = a.convert_to(
            &dest,
            CopyOptions::new(65536).unwrap(),
            options,
            &|e: &CopyEvent| {
                if e.phase == phase {
                    cancel.cancel();
                }
            },
        );
        assert_eq!(result.unwrap_err(), OwnershipError::Cancelled);
        drop(a);
        assert_eq!(
            t.store().recover(id(), &source).unwrap().state,
            JobState::CompletedLease
        );
    }
    let t = Temp::new();
    let source = t.populate(&data(), 32 * 65536);
    let options = RetainedOptions {
        timeout: Duration::from_nanos(1),
        ..Default::default()
    };
    assert_eq!(
        RetainedArtifact::open(t.store(), id(), source, options).unwrap_err(),
        OwnershipError::Deadline
    );
}
#[test]
fn post_flush_source_drift_prevents_conversion_success() {
    let t = Temp::new();
    let source = t.populate(&data(), 32 * 65536);
    let mut a = open(&t, source).unwrap();
    let dest = t.output(a.logical_bytes());
    let path = t.stage().join(MEMBERS[0]);
    let result = a.convert_to(
        &dest,
        CopyOptions::default(),
        RetainedOptions::default(),
        &|e: &CopyEvent| {
            if e.phase == CopyPhase::Completed {
                OpenOptions::new()
                    .write(true)
                    .open(&path)
                    .unwrap()
                    .write_all_at(b"x", 12000)
                    .unwrap();
            }
        },
    );
    assert_eq!(result.unwrap_err(), OwnershipError::Content);
}
#[test]
fn conversion_crash_child() {
    let Some(path) = std::env::var_os("RVVDK_RETAINED_CRASH") else {
        return;
    };
    let root = PathBuf::from(path);
    let source = source(32 * 65536);
    let mut a = RetainedArtifact::open(
        JobStore::open(&root.join("jobs")).unwrap(),
        id(),
        source,
        RetainedOptions::default(),
    )
    .unwrap();
    let dest =
        RawDisk::new(LocalFileBlockDevice::open_read_write(root.join("output.raw")).unwrap());
    let _ = a.convert_to(
        &dest,
        CopyOptions::new(65536).unwrap(),
        RetainedOptions::default(),
        &|e: &CopyEvent| {
            if e.phase == CopyPhase::Transferring {
                // SAFETY: terminate only this dedicated synthetic child.
                unsafe { libc::kill(libc::getpid(), libc::SIGKILL) };
            }
        },
    );
    panic!("crash point not reached");
}
#[test]
fn killed_conversion_leaves_completed_source_and_caller_owned_partial_output() {
    use std::os::unix::process::ExitStatusExt;
    let t = Temp::new();
    let source = t.populate(&data(), 32 * 65536);
    let _dest = t.output(32 * 65536);
    let before = fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "ownership::retained::tests::conversion_crash_child",
            "--nocapture",
        ])
        .env("RVVDK_RETAINED_CRASH", &t.0)
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let end = Instant::now() + Duration::from_secs(30);
    let status = loop {
        if let Some(s) = child.try_wait().unwrap() {
            break s;
        }
        if Instant::now() > end {
            child.kill().unwrap();
            panic!("child timeout");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(status.signal(), Some(libc::SIGKILL));
    assert_eq!(
        fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap(),
        before
    );
    assert!(t.0.join("output.raw").exists());
    assert!(open(&t, source).is_ok());
}
mod benchmark;
