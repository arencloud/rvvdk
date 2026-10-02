use super::*;
use crate::contract::{EndpointIdentity, PinProvenance};
use std::{
    cell::Cell,
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

thread_local! { pub(super) static FAIL_POINT: Cell<Option<&'static str>> = const { Cell::new(None) }; }
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let parent = std::env::var_os("RVVDK_OWNERSHIP_TEST_PARENT")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let p = parent.join(format!(
            "rvddk-owner-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        Self(p)
    }
    fn store(&self) -> JobStore {
        JobStore::open(&self.0).unwrap()
    }
}
use std::os::unix::fs::DirBuilderExt;
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn id() -> ArtifactId {
    ArtifactId::new([1; 16]).unwrap()
}
fn source() -> SourceSelection {
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
        "[synthetic] disk.vmdk",
        1 << 30,
    )
    .unwrap()
}
fn staged(store: &mut JobStore) -> Job<'_> {
    let mut job = store.create(id(), &source()).unwrap();
    job.prepare_stage().unwrap();
    job
}
fn remote_held(store: &mut JobStore) -> Job<'_> {
    let mut job = staged(store);
    job.begin_acquire().unwrap();
    job.acknowledge_lease("synthetic-lease").unwrap();
    job
}
fn staged_record(store: &JobStore) -> Record {
    store.read(id(), &source()).unwrap()
}

#[test]
fn transitions_require_durable_intents_and_recovery_never_recreates_live_job() {
    let f = Fixture::new();
    let mut store = f.store();
    let operation;
    {
        let mut j = staged(&mut store);
        operation = j.operation();
        assert_eq!(j.acknowledge_complete(), Err(OwnershipError::Transition));
        assert_eq!(j.begin_publish(), Err(OwnershipError::Transition));
        j.payload_file()
            .unwrap()
            .write_all(b"synthetic payload")
            .unwrap();
        j.begin_acquire().unwrap();
        j.acknowledge_lease("synthetic-lease").unwrap();
        j.transfer_complete().unwrap();
        j.begin_complete().unwrap();
        j.acknowledge_complete().unwrap();
        j.begin_publish().unwrap();
        assert_eq!(j.payload_file().unwrap_err(), OwnershipError::Transition);
        j.acknowledge_publish().unwrap();
        assert_eq!(j.state(), JobState::Published);
    }
    drop(store);
    let mut store = f.store();
    let r = store.recover(id(), &source()).unwrap();
    assert_eq!(r.operation, operation);
    assert_eq!(r.sequence, 9);
    assert_eq!(r.action, RecoveryAction::ValidatePublishedArtifact);
    assert_eq!(
        store.cleanup_local(id(), operation, &source()),
        Err(OwnershipError::Transition)
    );
    assert_eq!(
        store.create(id(), &source()).unwrap_err(),
        OwnershipError::Exists
    );
}

#[test]
fn cleanup_requires_no_uncertain_remote_or_publication_outcome() {
    for state in [
        JobState::AcquireIntent,
        JobState::LeaseHeld,
        JobState::TransferComplete,
        JobState::CompleteIntent,
        JobState::AbortIntent,
        JobState::PublishIntent,
    ] {
        let f = Fixture::new();
        let mut store = f.store();
        let operation;
        {
            let mut j = staged(&mut store);
            operation = j.operation();
            j.begin_acquire().unwrap();
            if state != JobState::AcquireIntent {
                j.acknowledge_lease("synthetic-lease").unwrap();
            }
            if state == JobState::AbortIntent {
                j.begin_abort().unwrap();
            }
            if matches!(
                state,
                JobState::TransferComplete | JobState::CompleteIntent | JobState::PublishIntent
            ) {
                j.transfer_complete().unwrap();
            }
            if matches!(state, JobState::CompleteIntent | JobState::PublishIntent) {
                j.begin_complete().unwrap();
            }
            if state == JobState::PublishIntent {
                j.acknowledge_complete().unwrap();
                j.begin_publish().unwrap();
            }
        }
        assert_eq!(
            store.cleanup_local(id(), operation, &source()),
            Err(OwnershipError::Transition)
        );
        assert_eq!(store.recover(id(), &source()).unwrap().state, state);
        assert!(f.0.join(stage_name(&operation.0)).join("owner").exists());
    }
}

#[test]
fn local_cleanup_after_explicit_abort_or_before_acquire_preserves_tombstone() {
    for abort in [false, true] {
        let f = Fixture::new();
        let mut store = f.store();
        let operation;
        {
            let mut job = staged(&mut store);
            operation = job.operation();
            if abort {
                job.begin_acquire().unwrap();
                job.acknowledge_lease("synthetic-lease").unwrap();
                job.begin_abort().unwrap();
                job.acknowledge_abort().unwrap();
            }
        }
        drop(store);
        let mut store = f.store();
        store.cleanup_local(id(), operation, &source()).unwrap();
        store.cleanup_local(id(), operation, &source()).unwrap();
        assert_eq!(
            store.recover(id(), &source()).unwrap().action,
            RecoveryAction::Terminal
        );
        assert!(!f.0.join(stage_name(&operation.0)).exists());
        assert_eq!(
            store.create(id(), &source()).unwrap_err(),
            OwnershipError::Exists
        );
    }
}

#[test]
fn store_lock_is_exclusive_and_symlink_or_public_directories_are_rejected() {
    let f = Fixture::new();
    let store = f.store();
    assert_eq!(JobStore::open(&f.0).unwrap_err(), OwnershipError::Busy);
    let other = Fixture::new();
    symlink(&f.0, other.0.join("link")).unwrap();
    assert!(JobStore::open(&other.0.join("link")).is_err());
    drop(store);
    fs::set_permissions(&f.0, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(JobStore::open(&f.0).unwrap_err(), OwnershipError::Identity);
    fs::set_permissions(&f.0, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(JobStore::open(&f.0).is_ok());
}

#[test]
fn replacement_symlink_hardlink_marker_and_unknown_member_are_preserved() {
    for attack in [
        "replace", "symlink", "hardlink", "marker", "unknown", "stage",
    ] {
        let f = Fixture::new();
        let mut store = f.store();
        let operation = staged(&mut store).operation();
        let stage = f.0.join(stage_name(&operation.0));
        let payload = stage.join("disk-1.vmdk");
        let sentinel = f.0.join("sentinel");
        fs::write(&sentinel, b"keep").unwrap();
        match attack {
            "replace" => {
                fs::rename(&payload, stage.join("old-payload")).unwrap();
                fs::write(&payload, b"keep").unwrap();
            }
            "symlink" => {
                fs::remove_file(&payload).unwrap();
                symlink(&sentinel, &payload).unwrap();
            }
            "hardlink" => {
                fs::hard_link(&payload, stage.join("alias")).unwrap();
            }
            "marker" => fs::write(stage.join("owner"), [2; 16]).unwrap(),
            "unknown" => fs::write(stage.join("unowned"), b"keep").unwrap(),
            "stage" => {
                fs::rename(&stage, f.0.join("old-stage")).unwrap();
                fs::DirBuilder::new().mode(0o700).create(&stage).unwrap();
            }
            _ => unreachable!(),
        }
        assert!(
            store.cleanup_local(id(), operation, &source()).is_err(),
            "{attack}"
        );
        assert_eq!(fs::read(&sentinel).unwrap(), b"keep");
        assert!(stage.exists());
        if attack == "unknown" {
            assert_eq!(fs::read(stage.join("unowned")).unwrap(), b"keep");
        }
    }
}

#[test]
fn stale_operation_source_copied_store_and_corrupt_records_cannot_authorize_cleanup() {
    let f = Fixture::new();
    let mut store = f.store();
    let op = staged(&mut store).operation();
    assert_eq!(
        store.cleanup_local(id(), OperationId([9; 16]), &source()),
        Err(OwnershipError::Identity)
    );
    let different = SourceSelection::new(
        source().endpoint().clone(),
        "different-vm",
        "01234567-89ab-cdef-0123-456789abcdef",
        2000,
        "[synthetic] disk.vmdk",
        1 << 30,
    )
    .unwrap();
    assert!(store.recover(id(), &different).is_err());
    let path = f.0.join(record_name(id().as_bytes()));
    let valid = fs::read(&path).unwrap();
    let other = Fixture::new();
    fs::copy(&path, other.0.join(record_name(id().as_bytes()))).unwrap();
    assert_eq!(
        other.store().recover(id(), &source()).unwrap_err(),
        OwnershipError::Record
    );
    for invalid in [
        b"{".to_vec(),
        vec![b' '; MAX_RECORD as usize + 1],
        valid
            .iter()
            .map(|b| if *b == b'1' { b'2' } else { *b })
            .collect(),
    ] {
        fs::write(&path, invalid).unwrap();
        assert!(store.cleanup_local(id(), op, &source()).is_err());
    }
    fs::write(&path, valid).unwrap();
    fs::hard_link(&path, f.0.join("record-alias")).unwrap();
    assert_eq!(
        store.recover(id(), &source()).unwrap_err(),
        OwnershipError::Identity
    );
}

#[test]
fn failed_commit_poisons_writer_and_preserves_old_or_new_complete_record() {
    for point in [
        "record_partial",
        "record_written",
        "record_synced",
        "record_renamed",
        "record_durable",
    ] {
        let f = Fixture::new();
        let mut store = f.store();
        let op;
        {
            let mut j = staged(&mut store);
            op = j.operation();
            FAIL_POINT.with(|v| v.set(Some(point)));
            assert_eq!(j.begin_acquire(), Err(OwnershipError::Uncertain));
            FAIL_POINT.with(|v| v.set(None));
            assert_eq!(j.begin_acquire(), Err(OwnershipError::Uncertain));
        }
        drop(store);
        let mut store = f.store();
        let r = store.recover(id(), &source()).unwrap();
        assert_eq!(
            r.state,
            if matches!(point, "record_renamed" | "record_durable") {
                JobState::AcquireIntent
            } else {
                JobState::Staged
            }
        );
        assert!(store.cleanup_local(id(), op, &source()).is_err());
    }
}

#[test]
fn empty_or_unknown_prepare_failure_never_creates_cleanup_authority() {
    for point in ["stage_created", "stage_durable"] {
        let f = Fixture::new();
        let mut store = f.store();
        let op;
        {
            let mut j = store.create(id(), &source()).unwrap();
            op = j.operation();
            FAIL_POINT.with(|v| v.set(Some(point)));
            assert!(j.prepare_stage().is_err());
            FAIL_POINT.with(|v| v.set(None));
        }
        assert_eq!(
            store.recover(id(), &source()).unwrap().action,
            RecoveryAction::LocalPreparationUnknown
        );
        assert_eq!(
            store.cleanup_local(id(), op, &source()),
            Err(OwnershipError::Transition)
        );
    }
}

#[test]
fn cleanup_retries_only_durable_partial_cleanup_and_keeps_unknown_files() {
    for point in [
        "cleanup_payload",
        "cleanup_manifest",
        "cleanup_owner",
        "cleanup_directory",
    ] {
        let f = Fixture::new();
        let mut store = f.store();
        let op = staged(&mut store).operation();
        FAIL_POINT.with(|v| v.set(Some(point)));
        assert!(store.cleanup_local(id(), op, &source()).is_err());
        FAIL_POINT.with(|v| v.set(None));
        drop(store);
        let mut store = f.store();
        assert_eq!(
            store.recover(id(), &source()).unwrap().action,
            RecoveryAction::CleanupMayBePartial
        );
        store.cleanup_local(id(), op, &source()).unwrap();
        assert_eq!(
            store.recover(id(), &source()).unwrap().state,
            JobState::Cleaned
        );
    }
}

#[test]
fn records_and_debug_exclude_runtime_lease_reference_and_operational_source() {
    let f = Fixture::new();
    let mut store = f.store();
    let debug = format!("{:?}", remote_held(&mut store));
    let record = fs::read_to_string(f.0.join(record_name(id().as_bytes()))).unwrap();
    for private in [
        "example.invalid",
        "vm-synthetic",
        "synthetic-lease",
        "[synthetic]",
        "01234567",
    ] {
        assert!(!debug.contains(private));
        assert!(!record.contains(private));
    }
    assert_eq!(
        fs::metadata(f.0.join(record_name(id().as_bytes())))
            .unwrap()
            .mode()
            & 0o777,
        0o600
    );
    let stage = staged_record(&store);
    assert!(stage.lease_fingerprint.is_some());
}

fn child(root: &Path, crash: Option<&str>, mode: &str) -> std::process::Output {
    let mut cmd = Command::new(std::env::current_exe().unwrap());
    cmd.args(["--exact", "ownership::tests::crash_child", "--nocapture"])
        .env("RVDDK_TEST_JOB_ROOT", root)
        .env("RVDDK_TEST_JOB_MODE", mode);
    if let Some(crash) = crash {
        cmd.env("RVDDK_TEST_JOB_CRASH", crash);
    }
    cmd.output().unwrap()
}

#[test]
fn actual_process_loss_and_competing_process_are_conservative() {
    for point in [
        "record_partial",
        "record_written",
        "record_synced",
        "record_renamed",
        "record_durable",
        "cleanup_payload",
        "cleanup_manifest",
        "cleanup_owner",
        "cleanup_directory",
    ] {
        let f = Fixture::new();
        let mut store = f.store();
        let op = staged(&mut store).operation();
        drop(store);
        let cleanup = point.starts_with("cleanup");
        let result = child(
            &f.0,
            Some(point),
            if cleanup { "cleanup" } else { "transition" },
        );
        assert_eq!(
            result.status.code(),
            Some(77),
            "{point}: {}",
            String::from_utf8_lossy(&result.stdout)
        );
        let mut store = f.store();
        let r = store.recover(id(), &source()).unwrap();
        if cleanup {
            assert_eq!(r.state, JobState::CleanupIntent);
            store.cleanup_local(id(), op, &source()).unwrap();
        } else {
            assert!(matches!(
                r.state,
                JobState::Staged | JobState::AcquireIntent
            ));
            assert!(store.cleanup_local(id(), op, &source()).is_err());
        }
    }
    let f = Fixture::new();
    let _store = f.store();
    assert!(child(&f.0, None, "busy").status.success());
}

#[test]
fn crash_child() {
    let Ok(root) = std::env::var("RVDDK_TEST_JOB_ROOT") else {
        return;
    };
    let mode = std::env::var("RVDDK_TEST_JOB_MODE").unwrap();
    if mode == "busy" {
        assert_eq!(
            JobStore::open(Path::new(&root)).unwrap_err(),
            OwnershipError::Busy
        );
        return;
    }
    let mut store = JobStore::open(Path::new(&root)).unwrap();
    if mode == "create" {
        store.create(id(), &source()).unwrap();
        panic!("create crash checkpoint did not fire");
    }
    if mode == "prepare" {
        let mut job = store.create(id(), &source()).unwrap();
        job.prepare_stage().unwrap();
        panic!("prepare crash checkpoint did not fire");
    }
    if mode == "cleanup" {
        let r = store.recover(id(), &source()).unwrap();
        store.cleanup_local(id(), r.operation, &source()).unwrap();
    } else {
        // Test-only construction permits exercising journal crash windows; the
        // public API intentionally cannot revive a live Job after process loss.
        let mut r = store.read(id(), &source()).unwrap();
        store.advance(&mut r, JobState::AcquireIntent).unwrap();
    }
    panic!("crash checkpoint did not fire");
}

#[test]
fn process_loss_during_initial_reservation_and_stage_creation_preserves_unknowns() {
    for point in [
        "record_partial",
        "record_written",
        "record_synced",
        "record_renamed",
        "record_durable",
        "stage_created",
        "stage_durable",
    ] {
        let f = Fixture::new();
        let stage = point.starts_with("stage");
        assert_eq!(
            child(&f.0, Some(point), if stage { "prepare" } else { "create" })
                .status
                .code(),
            Some(77)
        );
        let mut store = f.store();
        assert_eq!(
            store.create(id(), &source()).unwrap_err(),
            OwnershipError::Exists
        );
        if stage {
            let r = store.recover(id(), &source()).unwrap();
            assert_eq!(r.action, RecoveryAction::LocalPreparationUnknown);
            assert_eq!(
                store.cleanup_local(id(), r.operation, &source()),
                Err(OwnershipError::Transition)
            );
        } else if matches!(point, "record_renamed" | "record_durable") {
            assert_eq!(
                store.recover(id(), &source()).unwrap().state,
                JobState::Prepared
            );
        } else {
            assert!(store.recover(id(), &source()).is_err());
            assert!(f.0.join(transaction_name(id().as_bytes())).exists());
        }
    }
}

#[test]
fn schema_sequence_and_identity_invariants_are_checked_even_with_valid_checksum() {
    let f = Fixture::new();
    let mut store = f.store();
    let _op = staged(&mut store).operation();
    let original = store.read(id(), &source()).unwrap();
    let mutations: &[fn(&mut Record)] = &[
        |r| r.version = 2,
        |r| r.sequence = 0,
        |r| r.stage = None,
        |r| r.operation = [0; 16],
        |r| r.lease_fingerprint = Some([1; 32]),
        |r| r.stage.as_mut().unwrap().members[0].ino = 0,
    ];
    for mutate in mutations {
        let mut record = original.clone();
        mutate(&mut record);
        let sha256 = Sha256::digest(serde_json::to_vec(&record).unwrap()).into();
        let bytes = serde_json::to_vec(&Envelope { record, sha256 }).unwrap();
        fs::write(f.0.join(record_name(id().as_bytes())), bytes).unwrap();
        assert_eq!(
            store.recover(id(), &source()).unwrap_err(),
            OwnershipError::Record
        );
    }
}
