use super::super::tests::{
    admit, cleanup, destination, final_path, publish, ready, reopen, rewrite,
};
use super::*;
use crate::ownership::retained::output::tests::{oid, record, run, setup, verify};
use crate::ownership::retained::tests::{Temp, id, source};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, FileExt, PermissionsExt},
    path::PathBuf,
};
const SYNCS: [&str; 10] = [
    "raw_synced",
    "metadata_synced",
    "marker_synced",
    "bundle_synced",
    "intent_file_synced",
    "intent_directory_synced",
    "store_namespace_synced",
    "destination_synced",
    "transaction_synced",
    "acknowledgment_directory_synced",
];
fn origin(t: &Temp, which: &str, hook: impl FnMut(OutputState, &str) -> Result<()>) -> bool {
    if which == "normal" {
        return publish(t, hook).is_success();
    }
    assert!(
        !publish(t, |_, p| {
            if p == if which == "staged" {
                "before_rename"
            } else {
                "bundle_renamed"
            } {
                Err(OwnershipError::Io)
            } else {
                Ok(())
            }
        })
        .is_success()
    );
    t.store()
        .reconcile_hook(
            oid(),
            id(),
            source(32 * 65536),
            reopen(t),
            RetainedOptions::default(),
            &mut super::super::reconciliation::SystemIo,
            hook,
        )
        .is_success()
}
fn pending_origin(which: &str, point: &str) -> Temp {
    let t = ready();
    assert!(!origin(&t, which, |s, p| {
        if s == OutputState::Published && p == point {
            Err(OwnershipError::Io)
        } else {
            Ok(())
        }
    }));
    assert_eq!(record(&t).state, OutputState::PublishIntent);
    t
}
fn pending(synced: bool) -> Temp {
    pending_origin(
        "normal",
        if synced {
            "record_synced"
        } else {
            "record_written"
        },
    )
}
fn recover(
    t: &Temp,
    hook: impl FnMut(OutputState, &str) -> Result<()>,
) -> PublicationAcknowledgmentRecoveryReport {
    t.store().acknowledgment_hook(
        oid(),
        id(),
        source(32 * 65536),
        reopen(t),
        RetainedOptions::default(),
        &mut SystemIo,
        hook,
    )
}
fn journal(t: &Temp) -> Vec<u8> {
    fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap()
}
fn record_source(t: &Temp) -> Record {
    serde_json::from_slice::<Envelope>(&journal(t))
        .unwrap()
        .record
}
fn candidate(t: &Temp) -> PathBuf {
    t.jobs().join(txn(&oid().0))
}
fn current_path(t: &Temp) -> PathBuf {
    t.jobs().join(name(&oid().0))
}
fn edit(path: &Path, change: impl FnOnce(&mut OutputRecord)) {
    let mut e: OutputEnvelope = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    change(&mut e.record);
    e.sha256 = Sha256::digest(serde_json::to_vec(&e.record).unwrap()).into();
    fs::write(path, serde_json::to_vec(&e).unwrap()).unwrap();
}
fn snapshots(t: &Temp) -> Vec<(PathBuf, Vec<u8>, u64)> {
    let source_dir = t.jobs().join(stage_name(&record_source(t).operation));
    [
        t.jobs().join(record_name(id().as_bytes())),
        source_dir.join(MEMBERS[0]),
        source_dir.join(MEMBERS[1]),
        source_dir.join(MEMBERS[2]),
    ]
    .into_iter()
    .map(|p| {
        let b = fs::read(&p).unwrap();
        let i = fs::metadata(&p).unwrap().ino();
        (p, b, i)
    })
    .collect()
}
fn unchanged(saved: &[(PathBuf, Vec<u8>, u64)]) {
    for (p, b, i) in saved {
        assert_eq!(&fs::read(p).unwrap(), b);
        assert_eq!(fs::metadata(p).unwrap().ino(), *i);
    }
}
fn stamp(p: &Path) -> (Vec<u8>, u64) {
    (fs::read(p).unwrap(), fs::metadata(p).unwrap().ino())
}
fn promoted(t: &Temp, before: &(Vec<u8>, u64)) {
    assert_eq!(stamp(&current_path(t)), *before);
    assert!(!candidate(t).exists());
    assert_eq!(record(t).state, OutputState::Published);
}
#[derive(Default)]
struct FaultIo {
    fail: Option<usize>,
    errno: i32,
    calls: Vec<&'static str>,
    identities: Vec<Identity>,
    renames: usize,
    rename_error: Option<i32>,
    ambiguous: bool,
}
impl Io for FaultIo {
    fn sync(&mut self, f: &File, p: &'static str) -> std::io::Result<()> {
        self.calls.push(p);
        self.identities.push(identity(f).unwrap());
        if self.fail == Some(self.calls.len() - 1) {
            return Err(std::io::Error::from_raw_os_error(self.errno));
        }
        SystemIo.sync(f, p)
    }
    fn promote(&mut self, s: &File, from: &str, to: &str) -> Result<()> {
        self.renames += 1;
        if self.rename_error.is_some() && !self.ambiguous {
            return Err(OwnershipError::Uncertain);
        }
        SystemIo.promote(s, from, to)?;
        if self.ambiguous {
            Err(OwnershipError::Uncertain)
        } else {
            Ok(())
        }
    }
}
#[test]
fn six_histories_preserve_exact_candidate_and_full_work_under_both_locks() {
    for which in ["normal", "staged", "renamed"] {
        for point in ["record_written", "record_synced"] {
            let t = pending_origin(which, point);
            let old = stamp(&current_path(&t));
            let next = stamp(&candidate(&t));
            let saved = snapshots(&t);
            let raw = final_path(&t).join("disk.raw");
            let before = fs::metadata(&raw).unwrap();
            let mut io = FaultIo::default();
            let session = crate::ownership::retained::profile::Session::start(true);
            let r = t.store().acknowledgment_hook(
                oid(),
                id(),
                source(32 * 65536),
                reopen(&t),
                RetainedOptions::default(),
                &mut io,
                |_, _| {
                    assert!(matches!(
                        JobStore::open(&t.jobs()),
                        Err(OwnershipError::Busy)
                    ));
                    assert!(matches!(
                        PublicationDirectory::open(&t.0.join("published")),
                        Err(OwnershipError::Busy)
                    ));
                    Ok(())
                },
            );
            let regions = session.finish();
            assert!(r.is_success(), "{r:?}");
            assert_eq!(io.calls, SYNCS);
            assert_eq!(io.renames, 1);
            assert_eq!(io.identities[4].ino, old.1);
            assert_eq!(io.identities[8].ino, next.1);
            assert_ne!(old.1, next.1);
            assert!(!regions.contains_key("journal_commit_including_sync"));
            for key in [
                "raw_read",
                "logical_source_read",
                "logical_compare",
                "logical_hash",
            ] {
                assert_eq!(regions[key].bytes, 32 * 65536);
            }
            for key in ["container_read", "container_hash"] {
                assert_eq!(regions[key].bytes, 2 * saved[1].1.len() as u64);
            }
            assert_eq!(regions["native_map"].calls, 1);
            assert_eq!(regions["present_grain_read"].bytes, 2 * 65536);
            promoted(&t, &next);
            unchanged(&saved);
            verify(&raw, 32, |i| i == 0 || i == 16);
            let after = fs::metadata(raw).unwrap();
            assert_eq!(
                (before.ino(), before.len(), before.blocks()),
                (after.ino(), after.len(), after.blocks())
            );
            for text in [format!("{r:?}"), serde_json::to_string(&r).unwrap()] {
                for secret in [
                    t.0.to_str().unwrap(),
                    "1717171717171717",
                    "2323232323232323",
                    "raw_sha256",
                    "container_contract",
                ] {
                    assert!(!text.contains(secret));
                }
            }
            drop(t.store());
            drop(reopen(&t));
            assert_eq!(
                recover(&t, |_, _| Ok(())).primary_error,
                Some(OwnershipError::Transition)
            );
            assert!(
                t.store()
                    .attest_publication(
                        oid(),
                        id(),
                        source(32 * 65536),
                        reopen(&t),
                        RetainedOptions::default()
                    )
                    .is_success()
            );
            promoted(&t, &next);
        }
    }
}
#[test]
fn report_requires_every_flag_and_final_observation() {
    for case in 0..16 {
        let t = pending(false);
        let mut r = recover(&t, |_, _| Ok(()));
        assert!(r.is_success());
        match case {
            0 => r.content_verified = false,
            1 => r.bundle_synced = false,
            2 => r.intent_synced = false,
            3 => r.directories_synced = false,
            4 => r.transaction_synced = false,
            5 => r.transaction_promoted = false,
            6 => r.acknowledgment_durable = false,
            7 => r.bindings_rechecked = false,
            8 => r.primary_error = Some(OwnershipError::Io),
            9 => r.assessment_error = Some(OwnershipError::Io),
            10 => r.recovery = None,
            11 => r.recovery.as_mut().unwrap().output.state = OutputState::PublishIntent,
            12 => r.recovery.as_mut().unwrap().output.sequence = 7,
            13 => r.recovery.as_mut().unwrap().output.pending_transaction = true,
            14 => r.recovery.as_mut().unwrap().staged = OutputLocation::Owned,
            _ => r.recovery.as_mut().unwrap().published = OutputLocation::Absent,
        }
        assert!(!r.is_success(), "{case}");
    }
}
#[test]
fn thirty_sync_faults_stop_and_choose_explicit_retry() {
    for errno in [libc::EIO, libc::EINTR, libc::EINVAL] {
        for fail in 0..10 {
            let t = pending(true);
            let old = stamp(&current_path(&t));
            let next = stamp(&candidate(&t));
            let mut io = FaultIo {
                fail: Some(fail),
                errno,
                ..Default::default()
            };
            let r = t.store().acknowledgment_hook(
                oid(),
                id(),
                source(32 * 65536),
                reopen(&t),
                RetainedOptions::default(),
                &mut io,
                |_, _| Ok(()),
            );
            assert_eq!(io.calls, &SYNCS[..=fail]);
            assert_eq!(io.renames, usize::from(fail == 9));
            assert!(!r.is_success());
            assert!(!r.acknowledgment_durable);
            assert_eq!(
                r.primary_error,
                Some(if [5, 6, 7, 9].contains(&fail) {
                    OwnershipError::Uncertain
                } else {
                    OwnershipError::Io
                })
            );
            if fail < 9 {
                assert_eq!(stamp(&current_path(&t)), old);
                assert_eq!(stamp(&candidate(&t)), next);
                assert!(recover(&t, |_, _| Ok(())).is_success());
            } else {
                assert!(r.transaction_promoted);
                assert!(!recover(&t, |_, _| Ok(())).is_success());
                assert!(
                    t.store()
                        .attest_publication(
                            oid(),
                            id(),
                            source(32 * 65536),
                            reopen(&t),
                            RetainedOptions::default()
                        )
                        .is_success()
                );
            }
            promoted(&t, &next);
        }
    }
}
#[test]
fn rename_failures_and_ambiguous_returns_never_claim_success_or_rollback() {
    for errno in [libc::EIO, libc::EINTR, libc::ENOSPC, libc::EACCES] {
        for ambiguous in [false, true] {
            let t = pending(false);
            let old = stamp(&current_path(&t));
            let next = stamp(&candidate(&t));
            let mut io = FaultIo {
                rename_error: Some(errno),
                ambiguous,
                ..Default::default()
            };
            let r = t.store().acknowledgment_hook(
                oid(),
                id(),
                source(32 * 65536),
                reopen(&t),
                RetainedOptions::default(),
                &mut io,
                |_, _| Ok(()),
            );
            assert_eq!(io.calls, &SYNCS[..9]);
            assert_eq!(io.renames, 1);
            assert_eq!(r.primary_error, Some(OwnershipError::Uncertain));
            assert!(!r.transaction_promoted && !r.acknowledgment_durable && !r.is_success());
            if ambiguous {
                promoted(&t, &next);
                assert_eq!(r.recovery.unwrap().output.state, OutputState::Published);
            } else {
                assert_eq!(stamp(&current_path(&t)), old);
                assert_eq!(stamp(&candidate(&t)), next);
            }
        }
    }
}
#[test]
fn cancellation_respects_pairs_and_the_last_gate() {
    for (point, count) in [
        ("before_admission", 0),
        ("admitted", 0),
        ("verified_chunk", 0),
        ("validated", 0),
        ("raw_synced", 1),
        ("metadata_synced", 2),
        ("marker_synced", 3),
        ("bundle_synced", 4),
        ("intent_file_synced", 6),
        ("intent_directory_synced", 6),
        ("store_namespace_synced", 8),
        ("destination_synced", 8),
        ("transaction_checked", 8),
        ("transaction_synced", 9),
        ("before_promotion", 9),
        ("transaction_promoted", 10),
        ("acknowledgment_directory_synced", 10),
        ("before_assessment", 10),
        ("after_assessment", 10),
        ("bindings_rechecked", 10),
    ] {
        let t = pending(false);
        let token = crate::Cancellation::default();
        let cancel = token.clone();
        let mut io = FaultIo::default();
        let r = t.store().acknowledgment_hook(
            oid(),
            id(),
            source(32 * 65536),
            reopen(&t),
            RetainedOptions {
                cancellation: token,
                ..Default::default()
            },
            &mut io,
            |_, p| {
                if p == point {
                    cancel.cancel();
                }
                Ok(())
            },
        );
        assert_eq!(
            r.primary_error,
            Some(OwnershipError::Cancelled),
            "{point}: {r:?}"
        );
        assert_eq!(io.calls.len(), count, "{point}");
        assert_eq!(io.renames, usize::from(count == 10));
        assert!(!r.is_success());
        assert_eq!(candidate(&t).exists(), count < 10);
    }
}
#[test]
fn invalid_options_precancellation_and_shared_deadline_block_success() {
    for timeout in [
        Duration::ZERO,
        Duration::from_secs(21601),
        Duration::from_secs(60),
    ] {
        let t = pending(false);
        let cancellation = crate::Cancellation::default();
        if timeout.as_secs() == 60 {
            cancellation.cancel();
        }
        let mut io = FaultIo::default();
        let r = t.store().acknowledgment_hook(
            oid(),
            id(),
            source(32 * 65536),
            reopen(&t),
            RetainedOptions {
                timeout,
                cancellation,
            },
            &mut io,
            |_, _| Ok(()),
        );
        assert!(!r.is_success());
        assert!(io.calls.is_empty());
        assert_eq!(io.renames, 0);
    }
    for point in [
        "before_admission",
        "verified_chunk",
        "before_promotion",
        "before_assessment",
        "after_assessment",
    ] {
        let t = pending(true);
        let start = Instant::now();
        let mut reached = false;
        let r = t.store().acknowledgment_hook(
            oid(),
            id(),
            source(32 * 65536),
            reopen(&t),
            RetainedOptions {
                timeout: Duration::from_millis(500),
                ..Default::default()
            },
            &mut SystemIo,
            |_, p| {
                if p == point && !reached {
                    reached = true;
                    std::thread::sleep(Duration::from_millis(550).saturating_sub(start.elapsed()));
                }
                Ok(())
            },
        );
        assert!(reached, "{point}");
        assert_eq!(
            r.primary_error,
            Some(OwnershipError::Deadline),
            "{point}: {r:?}"
        );
        assert!(!r.is_success());
    }
}
#[test]
fn every_successor_field_is_bound_to_the_old_record() {
    for case in 0..14 {
        let t = pending(false);
        edit(&candidate(&t), |r| match case {
            0 => r.version = 1,
            1 => r.store.ino += 1,
            2 => r.output = [1; 16],
            3 => r.artifact = [1; 16],
            4 => r.source = [1; 32],
            5 => r.container_contract = [1; 32],
            6 => r.operation = [1; 16],
            7 => r.logical_bytes += 512,
            8 => r.state = OutputState::PublishIntent,
            9 => r.sequence += 1,
            10 => r.stage.as_mut().unwrap().members[0].ino += 1,
            11 => r.raw_sha256 = Some([1; 32]),
            12 => r.publication_parent.as_mut().unwrap().ino += 1,
            _ => r.cleanup_from = Some(OutputState::Published),
        });
        let next = stamp(&candidate(&t));
        let old = stamp(&current_path(&t));
        let r = recover(&t, |_, _| Ok(()));
        assert!(!r.is_success() && !r.content_verified, "{case}");
        assert_eq!(stamp(&candidate(&t)), next);
        assert_eq!(stamp(&current_path(&t)), old);
    }
}
#[test]
fn partial_foreign_and_unsafe_candidate_entries_are_retained() {
    for case in [
        "empty",
        "partial",
        "oversized",
        "checksum",
        "unknown",
        "duplicate",
        "trailing",
        "equal",
        "cleanup",
        "symlink",
        "directory",
        "fifo",
        "hardlink",
        "mode",
        "missing_old",
        "bad_old",
    ] {
        let t = pending(false);
        let p = candidate(&t);
        match case {
            "empty" => fs::write(&p, b"").unwrap(),
            "partial" => fs::write(&p, b"{\"record\":").unwrap(),
            "oversized" => fs::write(&p, vec![b' '; MAX_RECORD as usize + 1]).unwrap(),
            "checksum" | "unknown" => {
                let mut v: serde_json::Value =
                    serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
                if case == "checksum" {
                    v["sha256"][0] = serde_json::json!(999);
                } else {
                    v["extra"] = true.into();
                }
                fs::write(&p, serde_json::to_vec(&v).unwrap()).unwrap();
            }
            "duplicate" => {
                let b = String::from_utf8(fs::read(&p).unwrap()).unwrap().replacen(
                    "\"version\":2",
                    "\"version\":2,\"version\":2",
                    1,
                );
                fs::write(&p, b).unwrap();
            }
            "trailing" => {
                let mut b = fs::read(&p).unwrap();
                b.extend_from_slice(b" {}");
                fs::write(&p, b).unwrap();
            }
            "equal" => fs::write(&p, fs::read(current_path(&t)).unwrap()).unwrap(),
            "cleanup" => edit(&p, |r| {
                r.state = OutputState::CleanupIntent;
                r.sequence = 9;
                r.cleanup_from = Some(OutputState::Published);
            }),
            "symlink" | "directory" | "fifo" => {
                fs::remove_file(&p).unwrap();
                if case == "symlink" {
                    std::os::unix::fs::symlink("missing", &p).unwrap();
                } else if case == "directory" {
                    fs::create_dir(&p).unwrap();
                } else {
                    let name = std::ffi::CString::new(p.as_os_str().as_encoded_bytes()).unwrap(); /* SAFETY: valid private fixture pathname. */
                    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
                }
            }
            "hardlink" => fs::hard_link(&p, t.0.join("held")).unwrap(),
            "mode" => fs::set_permissions(&p, fs::Permissions::from_mode(0o644)).unwrap(),
            "missing_old" => fs::remove_file(current_path(&t)).unwrap(),
            _ => fs::write(current_path(&t), b"{}").unwrap(),
        }
        let before = fs::symlink_metadata(&p).unwrap();
        let bytes = if before.is_file() {
            Some(fs::read(&p).unwrap())
        } else {
            None
        };
        let r = recover(&t, |_, _| Ok(()));
        assert!(!r.is_success() && !r.content_verified, "{case}");
        let after = fs::symlink_metadata(&p).unwrap();
        assert_eq!((before.ino(), before.mode()), (after.ino(), after.mode()));
        if let Some(b) = bytes {
            assert_eq!(fs::read(&p).unwrap(), b);
        }
    }
}
#[test]
fn all_other_states_and_absent_candidates_retain_existing_gates() {
    let template = pending(false);
    let complete = fs::read(candidate(&template)).unwrap();
    for state in [
        OutputState::Prepared,
        OutputState::StageIntent,
        OutputState::Staged,
        OutputState::ConvertIntent,
        OutputState::Converted,
        OutputState::VerifyIntent,
        OutputState::Verified,
    ] {
        let t = setup();
        run(&t, |s, p| {
            if s == state && p == "record_durable" {
                Err(OwnershipError::Io)
            } else {
                Ok(())
            }
        });
        drop(destination(&t));
        let before = stamp(&current_path(&t));
        assert_eq!(
            recover(&t, |_, _| Ok(())).primary_error,
            Some(OwnershipError::Transition)
        );
        assert_eq!(stamp(&current_path(&t)), before);
        fs::write(candidate(&t), &complete).unwrap();
        assert_eq!(
            recover(&t, |_, _| Ok(())).primary_error,
            Some(OwnershipError::Transition)
        );
        assert_eq!(fs::read(candidate(&t)).unwrap(), complete);
    }
    for state in [
        OutputState::Published,
        OutputState::CleanupIntent,
        OutputState::Cleaned,
    ] {
        let t = ready();
        assert!(publish(&t, |_, _| Ok(())).is_success());
        if state != OutputState::Published {
            let _ = cleanup(&t, true, |s, p| {
                if state == OutputState::CleanupIntent && s == state && p == "record_durable" {
                    Err(OwnershipError::Io)
                } else {
                    Ok(())
                }
            });
        }
        let old = stamp(&current_path(&t));
        assert_eq!(
            recover(&t, |_, _| Ok(())).primary_error,
            Some(OwnershipError::Transition)
        );
        assert_eq!(stamp(&current_path(&t)), old);
        fs::write(candidate(&t), &complete).unwrap();
        assert_eq!(
            recover(&t, |_, _| Ok(())).primary_error,
            Some(OwnershipError::Transition)
        );
        assert_eq!(fs::read(candidate(&t)).unwrap(), complete);
    }
    let t = pending(false);
    fs::remove_file(candidate(&t)).unwrap();
    assert_eq!(
        recover(&t, |_, _| Ok(())).primary_error,
        Some(OwnershipError::Transition)
    );
    assert!(
        t.store()
            .reconcile_publication(
                oid(),
                id(),
                source(32 * 65536),
                reopen(&t),
                RetainedOptions::default()
            )
            .is_success()
    );
    let t = pending(false);
    assert!(
        !t.store()
            .reconcile_publication(
                oid(),
                id(),
                source(32 * 65536),
                reopen(&t),
                RetainedOptions::default()
            )
            .is_success()
    );
    assert!(
        !t.store()
            .attest_publication(
                oid(),
                id(),
                source(32 * 65536),
                reopen(&t),
                RetainedOptions::default()
            )
            .is_success()
    );
    assert!(admit(&t).is_err());
    assert!(cleanup(&t, true, |_, _| Ok(())).is_err());
    assert!(candidate(&t).exists());
}
#[test]
fn namespace_and_content_exclusions_preserve_journals() {
    for a in [
        OutputLocation::Absent,
        OutputLocation::Owned,
        OutputLocation::ForeignOrInvalid,
    ] {
        for b in [
            OutputLocation::Absent,
            OutputLocation::Owned,
            OutputLocation::ForeignOrInvalid,
        ] {
            assert_eq!(
                eligible(a, b),
                a == OutputLocation::Absent && b == OutputLocation::Owned
            );
        }
    }
    for case in [
        "missing_final",
        "staged",
        "extra_stage",
        "foreign_final",
        "unknown",
        "data",
        "zero",
        "forged",
        "length",
        "metadata",
        "marker",
        "missing",
        "symlink",
        "hardlink",
        "mode",
        "source_bytes",
        "source_missing",
        "source_cleaned",
    ] {
        let t = pending(false);
        let dir = final_path(&t);
        let raw = dir.join("disk.raw");
        let src = t.stage().join(MEMBERS[0]);
        match case {
            "missing_final" => fs::rename(&dir, t.0.join("held")).unwrap(),
            "staged" => fs::rename(&dir, t.jobs().join(stage(&record(&t).operation))).unwrap(),
            "extra_stage" => fs::create_dir(t.jobs().join(stage(&record(&t).operation))).unwrap(),
            "foreign_final" => {
                fs::rename(&dir, t.0.join("held")).unwrap();
                std::os::unix::fs::symlink("missing", &dir).unwrap();
            }
            "unknown" => fs::write(dir.join("extra"), b"keep").unwrap(),
            "data" | "zero" | "forged" => {
                OpenOptions::new()
                    .write(true)
                    .open(&raw)
                    .unwrap()
                    .write_all_at(b"drift", if case == "data" { 0 } else { 65536 })
                    .unwrap();
                if case == "forged" {
                    let hash = Sha256::digest(fs::read(&raw).unwrap()).into();
                    rewrite(&t, |e| e.record.raw_sha256 = Some(hash));
                    edit(&candidate(&t), |r| r.raw_sha256 = Some(hash));
                    let p = dir.join("output.json");
                    let mut m: OutputMetadata =
                        serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
                    m.raw_sha256 = hash;
                    fs::write(p, serde_json::to_vec(&m).unwrap()).unwrap();
                }
            }
            "length" => OpenOptions::new()
                .write(true)
                .open(raw)
                .unwrap()
                .set_len(17)
                .unwrap(),
            "metadata" => fs::write(dir.join("output.json"), b"{}").unwrap(),
            "marker" => fs::write(dir.join("owner"), [0; 16]).unwrap(),
            "missing" => fs::remove_file(raw).unwrap(),
            "symlink" => {
                fs::rename(&raw, t.0.join("held")).unwrap();
                std::os::unix::fs::symlink("missing", raw).unwrap();
            }
            "hardlink" => fs::hard_link(raw, t.0.join("held")).unwrap(),
            "mode" => fs::set_permissions(raw, fs::Permissions::from_mode(0o644)).unwrap(),
            "source_bytes" => OpenOptions::new()
                .write(true)
                .open(src)
                .unwrap()
                .write_all_at(b"drift", 4096)
                .unwrap(),
            "source_missing" => fs::remove_file(src).unwrap(),
            _ => {
                let mut s = t.store();
                let src = source(32 * 65536);
                let r = s.recover(id(), &src).unwrap();
                s.cleanup_local(id(), r.operation, &src).unwrap();
            }
        }
        let old = stamp(&current_path(&t));
        let next = stamp(&candidate(&t));
        let mut io = FaultIo::default();
        let r = t.store().acknowledgment_hook(
            oid(),
            id(),
            source(32 * 65536),
            reopen(&t),
            RetainedOptions::default(),
            &mut io,
            |_, _| Ok(()),
        );
        assert!(!r.is_success() && !r.content_verified, "{case}");
        assert!(io.calls.is_empty());
        assert_eq!(stamp(&current_path(&t)), old);
        assert_eq!(stamp(&candidate(&t)), next);
    }
}
#[test]
fn source_transactions_are_never_adopted() {
    for kind in ["partial", "full", "symlink", "directory"] {
        let t = pending(false);
        let p = t.jobs().join(transaction_name(id().as_bytes()));
        match kind {
            "symlink" => std::os::unix::fs::symlink("missing", &p).unwrap(),
            "directory" => fs::create_dir(&p).unwrap(),
            _ => fs::write(
                &p,
                if kind == "full" {
                    journal(&t)
                } else {
                    b"partial".to_vec()
                },
            )
            .unwrap(),
        };
        let next = stamp(&candidate(&t));
        let r = recover(&t, |_, _| Ok(()));
        assert_eq!(r.primary_error, Some(OwnershipError::Uncertain));
        assert_eq!(stamp(&candidate(&t)), next);
        assert!(fs::symlink_metadata(p).is_ok());
    }
}
#[test]
fn late_pair_source_and_namespace_changes_are_checked_through_assessment() {
    for point in [
        "admitted",
        "validated",
        "bundle_synced",
        "intent_directory_synced",
        "destination_synced",
        "transaction_synced",
        "before_promotion",
        "before_assessment",
        "after_assessment",
    ] {
        for kind in [
            "old_inode",
            "old_bytes",
            "candidate_inode",
            "candidate_bytes",
            "source_inode",
            "source_txn",
            "new_output_txn",
            "stage",
            "raw_inode",
        ] {
            let late = matches!(point, "before_assessment" | "after_assessment");
            if (late && kind.starts_with("candidate")) || (!late && kind == "new_output_txn") {
                continue;
            }
            let t = pending(false);
            let mut done = false;
            let r = recover(&t, |_, p| {
                if p == point && !done {
                    done = true;
                    let old = current_path(&t);
                    let next = candidate(&t);
                    match kind {
                        "old_inode" | "candidate_inode" | "source_inode" | "raw_inode" => {
                            let path = match kind {
                                "old_inode" => old,
                                "candidate_inode" => next,
                                "source_inode" => t.jobs().join(record_name(id().as_bytes())),
                                _ => final_path(&t).join("disk.raw"),
                            };
                            let bytes = fs::read(&path).unwrap();
                            fs::rename(&path, t.0.join("held")).unwrap();
                            let f = OpenOptions::new()
                                .create_new(true)
                                .write(true)
                                .mode(0o600)
                                .open(&path)
                                .unwrap();
                            drop(f);
                            fs::write(path, bytes).unwrap();
                        }
                        "old_bytes" | "candidate_bytes" => {
                            let path = if kind == "old_bytes" { old } else { next };
                            let mut bytes = fs::read(&path).unwrap();
                            bytes.push(b' ');
                            fs::write(path, bytes).unwrap();
                        }
                        "new_output_txn" => fs::write(candidate(&t), b"keep").unwrap(),
                        "source_txn" => {
                            fs::write(t.jobs().join(transaction_name(id().as_bytes())), b"keep")
                                .unwrap()
                        }
                        _ => fs::create_dir(t.jobs().join(stage(&record(&t).operation))).unwrap(),
                    }
                }
                Ok(())
            });
            assert!(done);
            assert!(!r.is_success(), "{point}/{kind}: {r:?}");
            assert_eq!(r.transaction_promoted, late);
            assert_eq!(candidate(&t).exists(), !late || kind == "new_output_txn");
        }
    }
}
#[test]
fn assessment_failure_preserves_primary_and_durable_flags() {
    for early in [false, true] {
        let t = pending(false);
        let r = recover(&t, |_, p| {
            if (early && p == "validated") || p == "before_assessment" {
                Err(OwnershipError::Io)
            } else {
                Ok(())
            }
        });
        assert!(!r.is_success());
        assert_eq!(r.assessment_error, Some(OwnershipError::Io));
        assert_eq!(r.primary_error, early.then_some(OwnershipError::Io));
        assert_eq!(r.acknowledgment_durable, !early);
        assert_eq!(candidate(&t).exists(), early);
    }
}
#[test]
fn explicit_cleanup_remains_separate() {
    let t = pending(false);
    assert!(cleanup(&t, true, |_, _| Ok(())).is_err());
    assert!(recover(&t, |_, _| Ok(())).is_success());
    let mut s = t.store();
    let src = source(32 * 65536);
    let r = s.recover(id(), &src).unwrap();
    s.cleanup_local(id(), r.operation, &src).unwrap();
    drop(s);
    assert!(!recover(&t, |_, _| Ok(())).is_success());
    cleanup(&t, true, |_, _| Ok(())).unwrap();
    assert!(!final_path(&t).exists());
}

#[test]
fn dense_and_deflated_fixtures_have_independent_full_byte_oracles() {
    use crate::artifact_fixture as fixture;
    for dense in [false, true] {
        for synced in [false, true] {
            let t = Temp::new();
            let grains = 32;
            let pattern = fixture::bytes(1);
            let present = |i| dense || i == 0 || i == 15;
            let compressed = include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../rvvdk-vmdk/tests/fixtures/stream/pattern.zlib"
            ))
            .to_vec();
            let records = (0..grains)
                .filter(|&i| present(i))
                .map(|i| (i, compressed.clone()))
                .collect::<Vec<_>>();
            let bytes = fixture::image(true, grains, &records);
            let src = t.populate(&bytes, grains * 65536);
            let a =
                RetainedArtifact::open(t.store(), id(), src.clone(), RetainedOptions::default())
                    .unwrap();
            assert!(
                a.convert_owned(
                    oid(),
                    CopyOptions::default(),
                    RetainedOptions::default(),
                    &|_: &rvvdk_datamover::CopyEvent| {}
                )
                .is_success()
            );
            let source_path = t
                .jobs()
                .join(stage_name(&record_source(&t).operation))
                .join(MEMBERS[0]);
            let r = publish(&t, |state, point| {
                if state == OutputState::Published
                    && point
                        == if synced {
                            "record_synced"
                        } else {
                            "record_written"
                        }
                {
                    Err(OwnershipError::Io)
                } else {
                    Ok(())
                }
            });
            assert!(!r.is_success());
            let before = journal(&t);
            assert!(recover(&t, |_, _| Ok(())).is_success());
            let f = File::open(final_path(&t).join("disk.raw")).unwrap();
            let mut observed = vec![0; 65536];
            for i in 0..grains {
                f.read_exact_at(&mut observed, i * 65536).unwrap();
                if present(i) {
                    assert_eq!(observed, pattern);
                } else {
                    assert!(observed.iter().all(|&b| b == 0));
                }
            }
            assert_eq!(fs::read(source_path).unwrap(), bytes);
            assert_eq!(journal(&t), before);
        }
    }
}

#[test]
fn pinned_parents_do_not_follow_replaced_pathnames() {
    for replace_store in [true, false] {
        let t = pending(false);
        let old = if replace_store {
            t.jobs()
        } else {
            t.0.join("published")
        };
        let held = t.0.join("held-parent");
        let r = recover(&t, |_, p| {
            if p == "admitted" {
                fs::rename(&old, &held).unwrap();
                fs::DirBuilder::new().mode(0o700).create(&old).unwrap();
            }
            Ok(())
        });
        assert!(r.is_success(), "{r:?}");
        assert_eq!(fs::read_dir(&old).unwrap().count(), 0);
        fs::remove_dir(&old).unwrap();
        fs::rename(&held, &old).unwrap();
        verify(&final_path(&t).join("disk.raw"), 32, |i| i == 0 || i == 16);
    }
}

#[test]
fn original_destination_and_explicit_selection_are_required() {
    for case in [
        "artifact",
        "output",
        "source",
        "destination",
        "nested",
        "store",
    ] {
        let t = pending(false);
        let before = journal(&t);
        let other = t.0.join("other");
        fs::DirBuilder::new().mode(0o700).create(&other).unwrap();
        let nested = t.jobs().join("nested");
        fs::DirBuilder::new().mode(0o700).create(&nested).unwrap();
        let store = if case == "store" {
            JobStore::open(&other).unwrap()
        } else {
            t.store()
        };
        let dest = if case == "destination" {
            PublicationDirectory::open(&other).unwrap()
        } else if case == "nested" {
            PublicationDirectory::open(&nested).unwrap()
        } else {
            reopen(&t)
        };
        let r = store.recover_publication_acknowledgment(
            if case == "output" {
                OutputId::new([29; 16]).unwrap()
            } else {
                oid()
            },
            if case == "artifact" {
                ArtifactId::new([29; 16]).unwrap()
            } else {
                id()
            },
            source(if case == "source" { 65536 } else { 32 * 65536 }),
            dest,
            RetainedOptions::default(),
        );
        assert!(!r.is_success(), "{case}");
        assert!(!r.content_verified);
        assert!(final_path(&t).exists());
        assert_eq!(journal(&t), before);
        drop(t.store());
        drop(reopen(&t));
    }
}

mod crash;

mod benchmark;
