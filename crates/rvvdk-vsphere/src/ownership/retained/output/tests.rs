use super::super::tests::{Temp, data, id, open, source};
use super::*;
use crate::artifact_fixture as fixture;
use rvvdk_datamover::{CopyEvent, CopyPhase};
use std::{
    fs,
    os::unix::fs::{FileExt, MetadataExt},
    path::PathBuf,
};
pub(super) fn oid() -> OutputId {
    OutputId::new([23; 16]).unwrap()
}
pub(super) fn raw_stage(t: &Temp) -> PathBuf {
    fs::read_dir(t.jobs())
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("raw-stage-")
        })
        .unwrap()
}
pub(super) fn record(t: &Temp) -> OutputRecord {
    serde_json::from_slice::<OutputEnvelope>(&fs::read(t.jobs().join(name(&oid().0))).unwrap())
        .unwrap()
        .record
}
pub(super) fn run(t: &Temp, hook: impl FnMut(OutputState, &str) -> Result<()>) -> OutputReport {
    open(t, source(32 * 65536)).unwrap().convert_owned_hook(
        oid(),
        CopyOptions::default(),
        RetainedOptions::default(),
        &|_: &CopyEvent| {},
        hook,
    )
}
pub(super) fn setup() -> Temp {
    let t = Temp::new();
    t.populate(&data(), 32 * 65536);
    t
}
pub(super) fn verify(path: &std::path::Path, grains: u64, present: impl Fn(u64) -> bool) {
    let f = File::open(path).unwrap();
    let mut b = vec![0; 65536];
    for i in 0..grains {
        f.read_exact_at(&mut b, i * 65536).unwrap();
        if present(i) {
            assert_eq!(b, fixture::bytes(i));
        } else {
            assert!(b.iter().all(|&v| v == 0));
        }
    }
}
#[test]
fn owned_conversion_verifies_bytes_metadata_lock_and_source_preservation() {
    for concurrency in [1, 2] {
        let t = setup();
        let before = fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap();
        let a = open(&t, source(32 * 65536)).unwrap();
        let r = a.convert_owned(
            oid(),
            CopyOptions::with_concurrency(65536, 4096, concurrency).unwrap(),
            RetainedOptions::default(),
            &|_: &CopyEvent| {
                assert!(matches!(
                    JobStore::open(&t.jobs()),
                    Err(OwnershipError::Busy)
                ));
            },
        );
        assert!(r.is_success(), "{r:?}");
        assert_eq!(r.logical_bytes_verified, 32 * 65536);
        let stage = raw_stage(&t);
        verify(&stage.join("disk.raw"), 32, |i| i == 0 || i == 16);
        let m: OutputMetadata =
            serde_json::from_slice(&fs::read(stage.join("output.json")).unwrap()).unwrap();
        assert!(m.format == RawFormat::Raw);
        assert!(m.verification == OutputVerification::LogicalSourceReadback);
        assert_eq!(
            m.raw_sha256,
            <[u8; 32]>::from(Sha256::digest(fs::read(stage.join("disk.raw")).unwrap()))
        );
        assert_eq!(record(&t).raw_sha256, Some(m.raw_sha256));
        for n in OUTPUT_MEMBERS {
            assert_eq!(fs::metadata(stage.join(n)).unwrap().mode() & 0o777, 0o600);
        }
        assert_eq!(
            fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap(),
            before
        );
        assert_eq!(
            t.store()
                .assess_output(oid(), id(), &source(32 * 65536))
                .unwrap()
                .sequence,
            6
        );
        let again = run(&t, |_, _| Ok(()));
        assert_eq!(again.primary_error, Some(OwnershipError::Exists));
        assert!(!again.is_success());
    }
}
#[test]
fn journal_faults_preserve_conservative_prefix_and_pending_transactions() {
    for state in [
        OutputState::Prepared,
        OutputState::StageIntent,
        OutputState::Staged,
        OutputState::ConvertIntent,
        OutputState::Converted,
        OutputState::VerifyIntent,
        OutputState::Verified,
    ] {
        for phase in [
            "record_partial",
            "record_written",
            "record_synced",
            "record_renamed",
            "record_durable",
        ] {
            let t = setup();
            let r = run(&t, |s, p| {
                if s == state && p == phase {
                    Err(OwnershipError::Io)
                } else {
                    Ok(())
                }
            });
            assert_eq!(r.primary_error, Some(OwnershipError::Uncertain));
            assert!(!r.is_success());
            let renamed = matches!(phase, "record_renamed" | "record_durable");
            if state == OutputState::Prepared && !renamed {
                assert!(r.recovery.is_none());
            } else {
                let recovered = r.recovery.unwrap();
                assert_eq!(recovered.sequence, state.sequence() - u64::from(!renamed));
                assert_eq!(recovered.pending_transaction, !renamed);
            }
            assert!(open(&t, source(32 * 65536)).is_ok());
        }
    }
}
#[test]
fn cancellation_deadline_and_engine_completion_are_not_owned_success() {
    for phase in [
        "stage_created",
        "conversion_drained",
        "verified_chunk",
        "metadata_synced",
    ] {
        let t = setup();
        let cancel = crate::Cancellation::default();
        let trigger = cancel.clone();
        let r = open(&t, source(32 * 65536)).unwrap().convert_owned_hook(
            oid(),
            CopyOptions::default(),
            RetainedOptions {
                cancellation: cancel,
                timeout: Duration::from_secs(30),
            },
            &|_: &CopyEvent| {},
            |_, p| {
                if p == phase {
                    trigger.cancel();
                }
                Ok(())
            },
        );
        assert_eq!(r.primary_error, Some(OwnershipError::Cancelled));
        assert!(!r.is_success());
        assert_ne!(r.recovery.unwrap().state, OutputState::Verified);
    }
    let t = setup();
    let cancel = crate::Cancellation::default();
    cancel.cancel();
    let r = open(&t, source(32 * 65536)).unwrap().convert_owned(
        oid(),
        CopyOptions::default(),
        RetainedOptions {
            cancellation: cancel,
            timeout: Duration::from_secs(30),
        },
        &|_: &CopyEvent| {},
    );
    assert_eq!(r.primary_error, Some(OwnershipError::Cancelled));
    assert!(!t.jobs().join(name(&oid().0)).exists());
    let t = setup();
    let r = open(&t, source(32 * 65536)).unwrap().convert_owned_hook(
        oid(),
        CopyOptions::default(),
        RetainedOptions {
            timeout: Duration::from_millis(100),
            ..RetainedOptions::default()
        },
        &|_: &CopyEvent| {},
        |_, p| {
            if p == "stage_created" {
                std::thread::sleep(Duration::from_millis(110));
            }
            Ok(())
        },
    );
    assert_eq!(r.primary_error, Some(OwnershipError::Deadline));
    assert!(!r.is_success());
    let t = setup();
    let cancel = crate::Cancellation::default();
    let trigger = cancel.clone();
    let r = open(&t, source(32 * 65536)).unwrap().convert_owned(
        oid(),
        CopyOptions::default(),
        RetainedOptions {
            cancellation: cancel,
            ..RetainedOptions::default()
        },
        &|e: &CopyEvent| {
            if e.phase == CopyPhase::Completed {
                trigger.cancel();
            }
        },
    );
    assert!(!r.is_success());
    assert_eq!(r.recovery.unwrap().state, OutputState::ConvertIntent);
}
#[test]
fn changed_raw_namespace_marker_and_metadata_never_verify() {
    for kind in [
        "bytes",
        "size",
        "replacement",
        "marker",
        "metadata_occupied",
        "metadata_readback",
    ] {
        let t = setup();
        let phase = if kind == "metadata_readback" {
            "metadata_synced"
        } else {
            "conversion_drained"
        };
        let r = run(&t, |_, p| {
            if p == phase {
                let dir = raw_stage(&t);
                let path = dir.join("disk.raw");
                match kind {
                    "bytes" => {
                        OpenOptions::new()
                            .write(true)
                            .open(path)
                            .unwrap()
                            .write_all_at(b"wrong", 65536)
                            .unwrap();
                    }
                    "size" => {
                        OpenOptions::new()
                            .write(true)
                            .open(path)
                            .unwrap()
                            .set_len(1)
                            .unwrap();
                    }
                    "replacement" => {
                        fs::rename(&path, dir.join("held.raw")).unwrap();
                        let mut f = OpenOptions::new()
                            .create_new(true)
                            .write(true)
                            .mode(0o600)
                            .open(path)
                            .unwrap();
                        f.write_all(b"foreign").unwrap();
                    }
                    "marker" => {
                        fs::write(dir.join("owner"), [0; 16]).unwrap();
                    }
                    _ => {
                        fs::write(dir.join("output.json"), b"foreign").unwrap();
                    }
                }
            }
            Ok(())
        });
        assert!(!r.is_success(), "{kind}");
        assert!(r.primary_error.is_some());
        assert_ne!(record(&t).state, OutputState::Verified);
        if kind == "replacement" {
            assert_eq!(
                fs::read(raw_stage(&t).join("disk.raw")).unwrap(),
                b"foreign"
            );
        }
    }
}
#[test]
fn interrupted_stage_and_metadata_remain_owned_without_cleanup_or_resume() {
    for phase in [
        "stage_created",
        "stage_durable",
        "conversion_drained",
        "verification_started",
        "verified_chunk",
        "metadata_partial",
        "metadata_written",
        "metadata_synced",
    ] {
        let t = setup();
        let r = run(&t, |_, p| {
            if p == phase {
                Err(OwnershipError::Io)
            } else {
                Ok(())
            }
        });
        assert_eq!(r.primary_error, Some(OwnershipError::Io));
        assert!(!r.is_success());
        assert!(raw_stage(&t).exists());
        assert_ne!(record(&t).state, OutputState::Verified);
    }
}
#[test]
fn assessment_rejects_bad_binding_record_permissions_and_pending_collision() {
    assert!(OutputId::new([0; 16]).is_err());
    assert_eq!(format!("{:?}", oid()), "OutputId([redacted])");
    let t = setup();
    let path = t.jobs().join(txn(&oid().0));
    fs::write(&path, b"foreign").unwrap();
    assert_eq!(
        run(&t, |_, _| Ok(())).primary_error,
        Some(OwnershipError::Exists)
    );
    assert_eq!(fs::read(path).unwrap(), b"foreign");
    for kind in ["checksum", "version", "state", "source", "permissions"] {
        let t = setup();
        assert!(run(&t, |_, _| Ok(())).is_success());
        let path = t.jobs().join(name(&oid().0));
        if kind == "permissions" {
            fs::set_permissions(&path, std::os::unix::fs::PermissionsExt::from_mode(0o644))
                .unwrap();
        } else {
            let mut e: OutputEnvelope = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            match kind {
                "checksum" => e.sha256 = [0; 32],
                "version" => e.record.version = 2,
                "state" => e.record.sequence = 5,
                _ => e.record.source = [0; 32],
            }
            if kind != "checksum" {
                e.sha256 = Sha256::digest(serde_json::to_vec(&e.record).unwrap()).into();
            }
            fs::write(path, serde_json::to_vec(&e).unwrap()).unwrap();
        }
        assert!(
            t.store()
                .assess_output(oid(), id(), &source(32 * 65536))
                .is_err()
        );
    }
}
#[test]
fn output_crash_child() {
    let Some(path) = std::env::var_os("RVVDK_OUTPUT_CRASH") else {
        return;
    };
    let root = PathBuf::from(path);
    let phase = std::env::var("RVVDK_OUTPUT_PHASE").unwrap();
    let mut a = RetainedArtifact::open(
        JobStore::open(&root.join("jobs")).unwrap(),
        id(),
        source(32 * 65536),
        RetainedOptions::default(),
    )
    .unwrap();
    a.convert_owned_hook(
        oid(),
        CopyOptions::new(65536).unwrap(),
        RetainedOptions::default(),
        &|e: &CopyEvent| {
            if phase == "copy" && e.phase == CopyPhase::Transferring {
                kill_self();
            }
        },
        |s, p| {
            if p == phase
                || (phase == "verified_rename"
                    && s == OutputState::Verified
                    && p == "record_renamed")
            {
                kill_self();
            }
            Ok(())
        },
    );
    panic!("crash point not reached");
}
fn kill_self() {
    // SAFETY: called only in the dedicated synthetic subprocess.
    unsafe { libc::kill(libc::getpid(), libc::SIGKILL) };
}
#[test]
fn process_loss_preserves_prefix_source_and_partial_output() {
    use std::os::unix::process::ExitStatusExt;
    for (phase, state) in [
        ("stage_created", OutputState::StageIntent),
        ("copy", OutputState::ConvertIntent),
        ("metadata_partial", OutputState::VerifyIntent),
        ("verified_rename", OutputState::Verified),
    ] {
        let t = setup();
        let before = fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "ownership::retained::output::tests::output_crash_child",
                "--nocapture",
            ])
            .env("RVVDK_OUTPUT_CRASH", &t.0)
            .env("RVVDK_OUTPUT_PHASE", phase)
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
                child.wait().unwrap();
                panic!("child timeout");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert_eq!(status.signal(), Some(libc::SIGKILL));
        assert_eq!(
            t.store()
                .assess_output(oid(), id(), &source(32 * 65536))
                .unwrap()
                .state,
            state
        );
        assert!(raw_stage(&t).exists());
        assert_eq!(
            fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap(),
            before
        );
        assert!(open(&t, source(32 * 65536)).is_ok());
    }
}
mod benchmark;
