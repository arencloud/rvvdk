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
const SYNCS: [&str; 8] = [
    "raw_synced",
    "metadata_synced",
    "marker_synced",
    "bundle_synced",
    "store_namespace_synced",
    "destination_synced",
    "published_file_synced",
    "published_directory_synced",
];
fn pending(uncertain: bool) -> Temp {
    let t = ready();
    let r = publish(&t, |s, p| {
        if uncertain && s == OutputState::Published && p == "record_renamed" {
            Err(OwnershipError::Io)
        } else {
            Ok(())
        }
    });
    assert_eq!(r.is_success(), !uncertain);
    assert_eq!(record(&t).state, OutputState::Published);
    t
}
fn journal(t: &Temp) -> Vec<u8> {
    fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap()
}
fn preserve_source(t: &Temp, before: &[u8]) {
    assert_eq!(journal(t), before);
}
fn directory(t: &Temp, _: bool) -> PathBuf {
    final_path(t)
}
fn recover(
    t: &Temp,
    hook: impl FnMut(OutputState, &str) -> Result<()>,
) -> PublicationAttestationReport {
    t.store().attest_hook(
        oid(),
        id(),
        source(32 * 65536),
        reopen(t),
        RetainedOptions::default(),
        &mut SystemIo,
        hook,
    )
}
#[derive(Default)]
struct FaultIo {
    fail: Option<usize>,
    errno: i32,
    calls: Vec<&'static str>,
    identities: Vec<Identity>,
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
}
fn snapshots(t: &Temp) -> Vec<(PathBuf, Vec<u8>, u64)> {
    let source_record = record_source(t);
    [
        t.jobs().join(record_name(id().as_bytes())),
        t.jobs().join(name(&oid().0)),
        t.jobs()
            .join(stage_name(&source_record.operation))
            .join(MEMBERS[0]),
    ]
    .into_iter()
    .map(|p| {
        let bytes = fs::read(&p).unwrap();
        let ino = fs::metadata(&p).unwrap().ino();
        (p, bytes, ino)
    })
    .collect()
}
fn unchanged(saved: &[(PathBuf, Vec<u8>, u64)]) {
    for (p, b, i) in saved {
        assert_eq!(&fs::read(p).unwrap(), b);
        assert_eq!(fs::metadata(p).unwrap().ino(), *i);
    }
}
#[test]
fn repeated_attestation_preserves_original_journals_and_syncs_without_mutation() {
    for uncertain in [false, true] {
        let t = pending(uncertain);
        let saved = snapshots(&t);
        let r0 = record(&t);
        let original = fs::metadata(final_path(&t).join("disk.raw")).unwrap();
        for _ in 0..2 {
            let mut io = FaultIo::default();
            let session = crate::ownership::retained::profile::Session::start(true);
            let r = t.store().attest_hook(
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
                    assert!(!t.jobs().join(txn(&oid().0)).exists());
                    Ok(())
                },
            );
            let regions = session.finish();
            assert!(r.is_success(), "{r:?}");
            assert_eq!(io.calls, SYNCS);
            assert!(!regions.contains_key("journal_commit_including_sync"));
            assert_eq!(io.identities[6].ino, saved[1].2);
            assert_eq!(r.logical_bytes_verified, 32 * 65536);
            assert!(record(&t) == r0);
            unchanged(&saved);
            let raw = final_path(&t).join("disk.raw");
            let after = fs::metadata(&raw).unwrap();
            assert_eq!(
                (original.ino(), original.len(), original.blocks()),
                (after.ino(), after.len(), after.blocks())
            );
            verify(&raw, 32, |i| i == 0 || i == 16);
            assert_eq!(fs::metadata(raw).unwrap().blocks(), original.blocks());
            assert!(!t.jobs().join(stage(&r0.operation)).exists());
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
        }
        cleanup(&t, true, |_, _| Ok(())).unwrap();
        assert!(!recover(&t, |_, _| Ok(())).is_success());
    }
}
#[test]
fn report_success_needs_every_gate_and_valid_final_assessment() {
    let t = pending(false);
    for case in 0..13 {
        let mut r = recover(&t, |_, _| Ok(()));
        assert!(r.is_success());
        match case {
            0 => r.content_verified = false,
            1 => r.bundle_synced = false,
            2 => r.directories_synced = false,
            3 => r.published_record_synced = false,
            4 => r.bindings_rechecked = false,
            5 => r.primary_error = Some(OwnershipError::Io),
            6 => r.assessment_error = Some(OwnershipError::Io),
            7 => r.recovery = None,
            8 => r.recovery.as_mut().unwrap().output.state = OutputState::PublishIntent,
            9 => r.recovery.as_mut().unwrap().output.sequence = 7,
            10 => r.recovery.as_mut().unwrap().output.pending_transaction = true,
            11 => r.recovery.as_mut().unwrap().staged = OutputLocation::Owned,
            _ => r.recovery.as_mut().unwrap().published = OutputLocation::Absent,
        }
        assert!(!r.is_success(), "{case}");
    }
}
#[test]
fn namespace_table_and_misplaced_or_foreign_entries_are_rejected() {
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
    for kind in [
        "missing",
        "staged",
        "extra_stage",
        "foreign_final",
        "unknown",
    ] {
        let t = pending(false);
        let staged = t.jobs().join(stage(&record(&t).operation));
        match kind {
            "missing" => fs::rename(final_path(&t), t.0.join("held")).unwrap(),
            "staged" => fs::rename(final_path(&t), staged).unwrap(),
            "extra_stage" => fs::create_dir(staged).unwrap(),
            "foreign_final" => {
                fs::rename(final_path(&t), t.0.join("held")).unwrap();
                std::os::unix::fs::symlink("missing", final_path(&t)).unwrap();
            }
            _ => fs::write(final_path(&t).join("unknown"), b"keep").unwrap(),
        }
        let saved = snapshots(&t);
        let mut io = FaultIo::default();
        let r = t.store().attest_hook(
            oid(),
            id(),
            source(32 * 65536),
            reopen(&t),
            RetainedOptions::default(),
            &mut io,
            |_, _| Ok(()),
        );
        assert!(!r.is_success(), "{kind}");
        assert!(io.calls.is_empty());
        unchanged(&saved);
    }
}
#[test]
fn every_sync_failure_stops_and_retry_repeats_all_work() {
    for errno in [libc::EIO, libc::EINTR, libc::EINVAL] {
        for fail in 0..8 {
            let t = pending(true);
            let saved = snapshots(&t);
            let mut io = FaultIo {
                fail: Some(fail),
                errno,
                ..Default::default()
            };
            let r = t.store().attest_hook(
                oid(),
                id(),
                source(32 * 65536),
                reopen(&t),
                RetainedOptions::default(),
                &mut io,
                |_, _| Ok(()),
            );
            assert_eq!(io.calls, &SYNCS[..=fail]);
            assert!(!r.is_success());
            assert_eq!(
                r.primary_error,
                Some(if fail < 4 {
                    OwnershipError::Io
                } else {
                    OwnershipError::Uncertain
                })
            );
            assert!(!r.published_record_synced);
            unchanged(&saved);
            assert!(recover(&t, |_, _| Ok(())).is_success());
            unchanged(&saved);
        }
    }
}
#[test]
fn cancellation_is_checked_after_pairs_and_after_assessment() {
    for (point, count) in [
        ("before_admission", 0),
        ("admitted", 0),
        ("verified_chunk", 0),
        ("validated", 0),
        ("raw_synced", 1),
        ("metadata_synced", 2),
        ("marker_synced", 3),
        ("bundle_synced", 4),
        ("parents_checked", 4),
        ("store_namespace_synced", 6),
        ("destination_synced", 6),
        ("record_checked", 6),
        ("published_file_synced", 8),
        ("published_directory_synced", 8),
        ("before_assessment", 8),
        ("after_assessment", 8),
        ("bindings_rechecked", 8),
    ] {
        let t = pending(false);
        let token = crate::Cancellation::default();
        let cancel = token.clone();
        let mut io = FaultIo::default();
        let r = t.store().attest_hook(
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
        assert!(!r.is_success());
        assert_eq!(io.calls.len(), count, "{point}");
        assert_eq!(r.directories_synced, count >= 6);
        assert_eq!(r.published_record_synced, count == 8);
        if point == "verified_chunk" {
            assert_eq!(r.logical_bytes_verified, 1 << 20);
        }
    }
    let t = pending(false);
    let token = crate::Cancellation::default();
    token.cancel();
    let r = t.store().attest_publication(
        oid(),
        id(),
        source(32 * 65536),
        reopen(&t),
        RetainedOptions {
            cancellation: token,
            ..Default::default()
        },
    );
    assert_eq!(r.primary_error, Some(OwnershipError::Cancelled));
    assert!(!r.content_verified);
}
#[test]
fn invalid_budgets_and_shared_deadline_include_final_assessment() {
    for timeout in [Duration::ZERO, Duration::from_secs(21601)] {
        let t = pending(false);
        let r = t.store().attest_publication(
            oid(),
            id(),
            source(32 * 65536),
            reopen(&t),
            RetainedOptions {
                timeout,
                ..Default::default()
            },
        );
        assert_eq!(r.primary_error, Some(OwnershipError::Content));
        assert!(!r.content_verified);
    }
    for point in [
        "before_admission",
        "verified_chunk",
        "before_assessment",
        "after_assessment",
    ] {
        let t = pending(false);
        let start = Instant::now();
        let mut reached = false;
        let r = t.store().attest_hook(
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
fn late_bindings_and_transactions_are_checked_even_after_assessment() {
    for point in [
        "admitted",
        "validated",
        "bundle_synced",
        "destination_synced",
        "published_directory_synced",
        "before_assessment",
        "after_assessment",
    ] {
        for change in [
            "output_journal",
            "source_journal",
            "source_txn",
            "output_txn",
            "raw",
            "stage",
        ] {
            let t = pending(false);
            let mut hit = false;
            let r = recover(&t, |_, p| {
                if p == point && !hit {
                    hit = true;
                    match change {
                        "source_txn" => {
                            fs::write(t.jobs().join(transaction_name(id().as_bytes())), b"keep")
                                .unwrap()
                        }
                        "output_txn" => fs::write(t.jobs().join(txn(&oid().0)), b"keep").unwrap(),
                        "stage" => {
                            fs::create_dir(t.jobs().join(stage(&record(&t).operation))).unwrap()
                        }
                        _ => {
                            let p = match change {
                                "output_journal" => t.jobs().join(name(&oid().0)),
                                "source_journal" => t.jobs().join(record_name(id().as_bytes())),
                                _ => final_path(&t).join("disk.raw"),
                            };
                            let b = fs::read(&p).unwrap();
                            fs::rename(&p, t.0.join("held")).unwrap();
                            fs::write(&p, b).unwrap();
                            fs::set_permissions(p, fs::Permissions::from_mode(0o600)).unwrap();
                        }
                    }
                }
                Ok(())
            });
            assert!(hit);
            assert!(!r.is_success(), "{point}/{change}: {r:?}");
            assert!(r.primary_error.is_some(), "{point}/{change}");
        }
    }
}
#[test]
fn assessment_error_preserves_primary_error_and_completed_flags() {
    for primary in [false, true] {
        let t = pending(false);
        let r = recover(&t, |_, p| {
            if primary && p == "validated" {
                Err(OwnershipError::Content)
            } else if p == "before_assessment" {
                Err(OwnershipError::Io)
            } else {
                Ok(())
            }
        });
        assert_eq!(r.primary_error, primary.then_some(OwnershipError::Content));
        assert_eq!(r.assessment_error, Some(OwnershipError::Io));
        assert!(!r.is_success());
        assert_eq!(r.published_record_synced, !primary);
    }
}
#[test]
fn source_cleanup_blocks_attestation_without_blocking_explicit_output_cleanup() {
    let t = pending(false);
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
fn fresh_content_and_member_checks_reject_corruption_and_forged_claims() {
    for renamed in [false, true] {
        for case in [
            "data",
            "zero",
            "length",
            "forged",
            "metadata",
            "extra",
            "marker",
            "missing",
            "symlink",
            "hardlink",
            "mode",
            "source_bytes",
            "source_missing",
            "source_cleaned",
        ] {
            let t = pending(renamed);
            let p = directory(&t, renamed);
            let raw = p.join("disk.raw");
            let source_path = t
                .jobs()
                .join(stage_name(&record_source(&t).operation))
                .join(MEMBERS[0]);
            match case {
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
                        let m = p.join("output.json");
                        let mut v: OutputMetadata =
                            serde_json::from_slice(&fs::read(&m).unwrap()).unwrap();
                        v.raw_sha256 = hash;
                        fs::write(m, serde_json::to_vec(&v).unwrap()).unwrap();
                    }
                }
                "length" => OpenOptions::new()
                    .write(true)
                    .open(raw)
                    .unwrap()
                    .set_len(17)
                    .unwrap(),
                "metadata" => fs::write(p.join("output.json"), b"{}").unwrap(),
                "extra" => fs::write(p.join("unknown"), b"keep").unwrap(),
                "marker" => fs::write(p.join("owner"), [0; 16]).unwrap(),
                "missing" => fs::remove_file(raw).unwrap(),
                "symlink" => {
                    fs::rename(&raw, t.0.join("held")).unwrap();
                    std::os::unix::fs::symlink(t.0.join("held"), raw).unwrap();
                }
                "hardlink" => fs::hard_link(raw, t.0.join("held")).unwrap(),
                "mode" => fs::set_permissions(raw, fs::Permissions::from_mode(0o644)).unwrap(),
                "source_bytes" => {
                    OpenOptions::new()
                        .write(true)
                        .open(source_path)
                        .unwrap()
                        .write_all_at(b"drift", 4096)
                        .unwrap();
                }
                "source_missing" => fs::remove_file(source_path).unwrap(),
                _ => {
                    let mut s = t.store();
                    let src = source(32 * 65536);
                    let r = s.recover(id(), &src).unwrap();
                    s.cleanup_local(id(), r.operation, &src).unwrap();
                }
            }
            let saved = fs::read(t.jobs().join(name(&oid().0))).unwrap();
            let r = recover(&t, |_, _| Ok(()));
            assert!(!r.is_success(), "{case}");
            assert!(!r.content_verified && !r.bundle_synced);
            assert_eq!(fs::read(t.jobs().join(name(&oid().0))).unwrap(), saved);
        }
    }
}
fn record_source(t: &Temp) -> Record {
    serde_json::from_slice::<Envelope>(&journal(t))
        .unwrap()
        .record
}

#[test]
fn dense_and_deflated_fixtures_have_independent_full_byte_oracles() {
    use crate::artifact_fixture as fixture;
    for dense in [false, true] {
        for renamed in [false, true] {
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
                if renamed && state == OutputState::Published && point == "record_renamed" {
                    Err(OwnershipError::Io)
                } else {
                    Ok(())
                }
            });
            assert_eq!(r.is_success(), !renamed);
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
            preserve_source(&t, &before);
        }
    }
}

mod benchmark;
mod crash;

#[test]
fn all_non_published_states_and_invalid_records_are_rejected() {
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
        let before = fs::read(t.jobs().join(name(&oid().0))).unwrap();
        assert_eq!(
            recover(&t, |_, _| Ok(())).primary_error,
            Some(OwnershipError::Transition)
        );
        assert_eq!(fs::read(t.jobs().join(name(&oid().0))).unwrap(), before);
    }
    for state in [OutputState::CleanupIntent, OutputState::Cleaned] {
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
        assert_eq!(record(&t).state, state);
        assert_eq!(
            recover(&t, |_, _| Ok(())).primary_error,
            Some(OwnershipError::Transition)
        );
    }
    {
        let t = ready();
        publish(&t, |_, p| {
            if p == "before_rename" {
                Err(OwnershipError::Io)
            } else {
                Ok(())
            }
        });
        assert_eq!(
            recover(&t, |_, _| Ok(())).primary_error,
            Some(OwnershipError::Transition)
        );
    }
    for case in [
        "version", "sequence", "stamp", "checksum", "field", "store", "source", "artifact",
        "output", "parent",
    ] {
        let t = pending(false);
        match case {
            "checksum" => {
                let p = t.jobs().join(name(&oid().0));
                let mut v: serde_json::Value =
                    serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
                v["sha256"][0] = serde_json::json!(255);
                fs::write(p, serde_json::to_vec(&v).unwrap()).unwrap();
            }
            "field" => {
                let p = t.jobs().join(name(&oid().0));
                let mut v: serde_json::Value =
                    serde_json::from_slice(&fs::read(&p).unwrap()).unwrap();
                v["extra"] = true.into();
                fs::write(p, serde_json::to_vec(&v).unwrap()).unwrap();
            }
            _ => rewrite(&t, |e| match case {
                "version" => e.record.version = 9,
                "sequence" => e.record.sequence = 7,
                "stamp" => e.record.stage = None,
                "store" => e.record.store.ino += 1,
                "source" => e.record.source = [0; 32],
                "artifact" => e.record.artifact = [1; 16],
                "output" => e.record.output = [1; 16],
                _ => e.record.publication_parent.as_mut().unwrap().ino += 1,
            }),
        }
        assert!(!recover(&t, |_, _| Ok(())).is_success(), "{case}");
    }
}
#[test]
fn pending_transactions_are_retained_for_both_journals_and_branches() {
    for renamed in [false, true] {
        for source_txn in [false, true] {
            for kind in ["partial", "full", "symlink", "directory"] {
                let t = pending(renamed);
                let p = t.jobs().join(if source_txn {
                    transaction_name(id().as_bytes())
                } else {
                    txn(&oid().0)
                });
                match kind {
                    "symlink" => std::os::unix::fs::symlink("missing", &p).unwrap(),
                    "directory" => fs::create_dir(&p).unwrap(),
                    _ => fs::write(
                        &p,
                        if kind == "full" {
                            if source_txn {
                                journal(&t)
                            } else {
                                fs::read(t.jobs().join(name(&oid().0))).unwrap()
                            }
                        } else {
                            b"partial".to_vec()
                        },
                    )
                    .unwrap(),
                }
                let r = recover(&t, |_, _| Ok(()));
                assert_eq!(r.primary_error, Some(OwnershipError::Uncertain));
                assert!(fs::symlink_metadata(p).is_ok());
                assert!(directory(&t, renamed).exists());
                assert_eq!(record(&t).state, OutputState::Published);
            }
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
        let r = store.attest_publication(
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
        preserve_source(&t, &before);
        drop(t.store());
        drop(reopen(&t));
    }
}
