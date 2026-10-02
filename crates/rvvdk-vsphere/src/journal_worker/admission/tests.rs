use super::super::payload::Expected;
use super::*;
use crate::artifact_fixture as fixture;
use crate::contract::{EndpointIdentity, PinProvenance};
use crate::ownership::{JobStore, RecoveryAction};
use sha1::Sha1;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, FileExt},
    path::PathBuf,
    time::Duration,
};
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rvddk-admission-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn id() -> ArtifactId {
    ArtifactId::new([13; 16]).unwrap()
}
fn source(capacity: u64) -> SourceSelection {
    SourceSelection::new(
        EndpointIdentity::pinned(
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
fn valid() -> Vec<u8> {
    fixture::image(
        true,
        8,
        &[
            (0, fixture::stored(&fixture::bytes(0))),
            (7, fixture::stored(&fixture::bytes(7))),
        ],
    )
}
fn held<'a>(store: &'a mut JobStore, source: &SourceSelection, data: &[u8]) -> (Job<'a>, Payload) {
    let mut job = store.create(id(), source).unwrap();
    job.prepare_stage().unwrap();
    job.begin_acquire().unwrap();
    job.acknowledge_lease("synthetic").unwrap();
    let mut p = Payload::default();
    p.open(&job, data.len() as u64).unwrap();
    for chunk in data.chunks(super::super::CHUNK_BYTES) {
        p.write(chunk).unwrap();
    }
    p.seal(
        &job,
        &Expected {
            bytes: data.len() as u64,
            sha256: Sha256::digest(data).into(),
            sha1: Sha1::digest(data).into(),
        },
    )
    .unwrap();
    (job, p)
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(30)
}
#[test]
fn native_admission_checks_both_layouts_and_keeps_digest_claim_private() {
    for footer in [false, true] {
        for populated in [false, true] {
            let out = Temp::new();
            let mut store = JobStore::open(&out.0).unwrap();
            let source = source(8 * 65536);
            let data = fixture::image(
                footer,
                8,
                &if populated {
                    vec![
                        (0, fixture::stored(&fixture::bytes(0))),
                        (7, fixture::stored(&fixture::bytes(7))),
                    ]
                } else {
                    vec![]
                },
            );
            let (job, mut p) = held(&mut store, &source, &data);
            p.admit(&job, id(), &source, &Cancellation::default(), deadline())
                .unwrap();
            assert!(p.progress.native_verified && p.progress.metadata_durable);
            assert_eq!(p.progress.grains_verified, if populated { 2 } else { 0 });
            let mut bytes = Vec::new();
            job.metadata_file()
                .unwrap()
                .read_to_end(&mut bytes)
                .unwrap();
            let metadata = ExportArtifact::from_json(&bytes).unwrap();
            assert_eq!(metadata.id(), id());
            metadata.check_source(&source).unwrap();
            metadata
                .check_container(data.len() as u64, &Sha256::digest(&data).into())
                .unwrap();
            assert_eq!(
                metadata.validation_claim(),
                ValidationClaim::ContainerDigestVerified
            );
            assert_eq!(format!("{metadata:?}"), "ExportArtifact([redacted])");
            let file = job.payload_reader().unwrap();
            use std::os::fd::AsRawFd;
            // SAFETY: inspect flags on the retained live descriptor.
            assert_eq!(
                unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) } & libc::O_ACCMODE,
                libc::O_RDONLY
            );
        }
    }
}
#[test]
fn native_rejects_matching_digest_with_bad_structure_payload_or_capacity() {
    for kind in ["header", "payload", "truncated", "capacity", "limit"] {
        let out = Temp::new();
        let mut store = JobStore::open(&out.0).unwrap();
        let mut data = valid();
        match kind {
            "header" => fixture::put32(&mut data, 4, 99),
            "payload" => data[65536 + 12 + 8] ^= 1,
            "truncated" => {
                data.truncate(data.len() - 512);
            }
            _ => {}
        }
        let source = source(if kind == "capacity" {
            9 * 65536
        } else if kind == "limit" {
            2 << 40
        } else {
            8 * 65536
        });
        if kind == "limit" {
            fixture::put64(&mut data, 12, (2u64 << 40) / 512);
        }
        let (job, mut p) = held(&mut store, &source, &data);
        assert_eq!(
            p.admit(&job, id(), &source, &Cancellation::default(), deadline()),
            Err(OwnershipError::Content),
            "{kind}"
        );
        assert!(!p.progress.native_verified && !p.progress.metadata_durable);
        assert_eq!(job.metadata_file().unwrap().metadata().unwrap().len(), 0);
    }
}
#[test]
fn admission_rechecks_sealed_bytes_and_preserves_existing_metadata() {
    for kind in ["payload", "marker", "metadata", "member"] {
        let out = Temp::new();
        let mut store = JobStore::open(&out.0).unwrap();
        let source = source(8 * 65536);
        let (job, mut p) = held(&mut store, &source, &valid());
        let stage = fs::read_dir(&out.0)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.is_dir())
            .unwrap();
        match kind {
            "payload" => job
                .payload_file()
                .unwrap()
                .write_all_at(b"changed", 12000)
                .unwrap(),
            "marker" => fs::write(stage.join("owner"), [0; 16]).unwrap(),
            "metadata" => job.metadata_file().unwrap().write_all(b"sentinel").unwrap(),
            _ => {
                fs::rename(stage.join("disk-1.vmdk"), stage.join("original")).unwrap();
                fs::write(stage.join("disk-1.vmdk"), b"foreign").unwrap();
            }
        }
        assert!(
            p.admit(&job, id(), &source, &Cancellation::default(), deadline())
                .is_err(),
            "{kind}"
        );
        assert!(!p.progress.metadata_durable);
        if kind == "metadata" {
            assert_eq!(fs::read(stage.join("manifest.json")).unwrap(), b"sentinel");
        }
        if kind == "member" {
            assert_eq!(fs::read(stage.join("disk-1.vmdk")).unwrap(), b"foreign");
        }
    }
}
#[test]
fn metadata_faults_and_readback_tampering_never_acknowledge_durability() {
    for point in [
        "metadata_partial",
        "metadata_written",
        "metadata_synced",
        "tamper",
    ] {
        let out = Temp::new();
        let mut store = JobStore::open(&out.0).unwrap();
        let source = source(8 * 65536);
        let (job, mut p) = held(&mut store, &source, &valid());
        let result = p.admit_with(
            &job,
            id(),
            &source,
            &Cancellation::default(),
            deadline(),
            |at| {
                if at == point {
                    return Err(OwnershipError::Io);
                }
                if at == "metadata_synced" && point == "tamper" {
                    job.metadata_file()?.write_all_at(b"x", 0).unwrap();
                }
                Ok(())
            },
        );
        assert!(result.is_err());
        assert!(!p.progress.metadata_durable);
        assert_eq!(job.state(), JobState::LeaseHeld);
        drop(job);
        let r = store.recover(id(), &source).unwrap();
        assert_eq!(r.state, JobState::LeaseHeld);
        assert!(store.cleanup_local(id(), r.operation, &source).is_err());
    }
}
#[test]
fn native_cancel_and_deadline_stop_before_metadata_and_preserve_partial_counts() {
    for mode in ["before", "grain", "deadline", "metadata_synced"] {
        let out = Temp::new();
        let mut store = JobStore::open(&out.0).unwrap();
        let source = source(8 * 65536);
        let (job, mut p) = held(&mut store, &source, &valid());
        let cancel = Cancellation::default();
        if mode == "before" {
            cancel.cancel();
        }
        let result = p.admit_with(
            &job,
            id(),
            &source,
            &cancel,
            if mode == "deadline" {
                Instant::now()
            } else {
                deadline()
            },
            |at| {
                if at == mode {
                    cancel.cancel();
                }
                Ok(())
            },
        );
        assert_eq!(
            result,
            Err(if mode == "deadline" {
                OwnershipError::Deadline
            } else {
                OwnershipError::Cancelled
            })
        );
        assert!(!p.progress.metadata_durable);
        if mode == "grain" {
            assert_eq!(p.progress.grains_verified, 1);
            assert_eq!(job.metadata_file().unwrap().metadata().unwrap().len(), 0);
        }
    }
}
#[test]
fn metadata_crash_child() {
    let Some(path) = std::env::var_os("RVVDK_ADMISSION_CRASH_PATH") else {
        return;
    };
    let point = std::env::var("RVVDK_ADMISSION_CRASH_POINT").unwrap();
    let mut store = JobStore::open(std::path::Path::new(&path)).unwrap();
    let source = source(8 * 65536);
    let (job, mut p) = held(&mut store, &source, &valid());
    let _ = p.admit_with(
        &job,
        id(),
        &source,
        &Cancellation::default(),
        deadline(),
        |at| {
            if at == point {
                // SAFETY: terminate only this dedicated synthetic test child.
                unsafe { libc::kill(libc::getpid(), libc::SIGKILL) };
            }
            Ok(())
        },
    );
    panic!("crash point not reached");
}
#[test]
fn process_loss_during_metadata_never_creates_recovered_authority() {
    use std::os::unix::process::ExitStatusExt;
    for point in ["metadata_partial", "metadata_synced"] {
        let out = Temp::new();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "journal_worker::admission::tests::metadata_crash_child",
                "--nocapture",
            ])
            .env("RVVDK_ADMISSION_CRASH_PATH", &out.0)
            .env("RVVDK_ADMISSION_CRASH_POINT", point)
            .stdout(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let end = deadline();
        let status = loop {
            if let Some(s) = child.try_wait().unwrap() {
                break s;
            }
            if Instant::now() > end {
                child.kill().unwrap();
                panic!("crash child timeout");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(status.signal(), Some(libc::SIGKILL));
        let mut store = JobStore::open(&out.0).unwrap();
        let r = store.recover(id(), &source(8 * 65536)).unwrap();
        assert_eq!(r.state, JobState::LeaseHeld);
        assert_ne!(r.action, RecoveryAction::CheckOwnedLocalResources);
        assert!(
            store
                .cleanup_local(id(), r.operation, &source(8 * 65536))
                .is_err()
        );
        let stage = fs::read_dir(&out.0)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.is_dir())
            .unwrap();
        let parsed = ExportArtifact::from_json(&fs::read(stage.join("manifest.json")).unwrap());
        assert_eq!(parsed.is_ok(), point == "metadata_synced");
    }
}
