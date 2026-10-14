use super::super::tests::{
    admit, cleanup, destination, final_path, publish, ready, reopen, rewrite,
};
use super::*;
use crate::ownership::retained::output::tests::{oid, raw_stage, record, run, setup, verify};
use crate::ownership::retained::tests::{Temp, id, source};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, FileExt, PermissionsExt},
    path::PathBuf,
};

fn pending(renamed: bool) -> Temp {
    let t = ready();
    let r = publish(&t, |_, p| {
        if p == if renamed {
            "bundle_renamed"
        } else {
            "before_rename"
        } {
            Err(OwnershipError::Io)
        } else {
            Ok(())
        }
    });
    assert!(!r.is_success());
    assert_eq!(record(&t).state, OutputState::PublishIntent);
    t
}
fn recover(t: &Temp, hook: impl FnMut(OutputState, &str) -> Result<()>) -> ReconciliationReport {
    t.store().reconcile_hook(
        oid(),
        id(),
        source(32 * 65536),
        reopen(t),
        RetainedOptions::default(),
        &mut SystemIo,
        hook,
    )
}
fn directory(t: &Temp, renamed: bool) -> PathBuf {
    if renamed { final_path(t) } else { raw_stage(t) }
}
fn journal(t: &Temp) -> Vec<u8> {
    fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap()
}
fn preserve_source(t: &Temp, before: &[u8]) {
    assert_eq!(journal(t), before);
}

#[derive(Default)]
struct FaultIo {
    fail: Option<&'static str>,
    calls: Vec<&'static str>,
    collision: bool,
}
impl Io for FaultIo {
    fn sync(&mut self, f: &File, point: &'static str) -> std::io::Result<()> {
        self.calls.push(point);
        if self.fail == Some(point) {
            return Err(std::io::Error::from_raw_os_error(libc::EIO));
        }
        SystemIo.sync(f, point)
    }
    fn rename(&mut self, from: &File, old: &str, to: &File, new: &str) -> Result<()> {
        self.calls.push("rename");
        if self.fail == Some("rename") {
            return Err(OwnershipError::Uncertain);
        }
        if self.collision {
            open_at(to, new, true, true, false)?
                .write_all(b"foreign")
                .unwrap();
        }
        SystemIo.rename(from, old, to, new)
    }
}
#[test]
fn both_branches_verify_preserve_sync_and_acknowledge_without_recopy() {
    for renamed in [false, true] {
        let t = pending(renamed);
        let before = journal(&t);
        let old = directory(&t, renamed);
        let raw = fs::metadata(old.join("disk.raw")).unwrap();
        let r0 = record(&t);
        let mut io = FaultIo::default();
        let report = t.store().reconcile_hook(
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
        assert!(report.is_success(), "{report:?}");
        assert_eq!(report.rename_observed, !renamed);
        assert_eq!(report.logical_bytes_verified, 32 * 65536);
        verify(&final_path(&t).join("disk.raw"), 32, |i| i == 0 || i == 16);
        let after = fs::metadata(final_path(&t).join("disk.raw")).unwrap();
        assert_eq!(
            (raw.dev(), raw.ino(), raw.len(), raw.blocks()),
            (after.dev(), after.ino(), after.len(), after.blocks())
        );
        let now = record(&t);
        assert_eq!(now.operation, r0.operation);
        assert!(now.stage == r0.stage);
        assert_eq!(now.raw_sha256, r0.raw_sha256);
        assert_eq!(now.container_contract, r0.container_contract);
        assert_eq!(
            io.calls.iter().filter(|&&p| p == "rename").count(),
            usize::from(!renamed)
        );
        assert_eq!(
            io.calls.iter().filter(|&&p| p.ends_with("synced")).count(),
            8
        );
        let intent = io
            .calls
            .iter()
            .position(|&p| p == "intent_directory_synced")
            .unwrap();
        let parents = io
            .calls
            .iter()
            .position(|&p| p == "source_directory_synced")
            .unwrap();
        assert!(intent < parents);
        if !renamed {
            assert!(intent < io.calls.iter().position(|&p| p == "rename").unwrap());
        }
        preserve_source(&t, &before);
        let json = serde_json::to_string(&report).unwrap();
        for forbidden in [
            t.0.to_str().unwrap(),
            "1717171717171717",
            "2323232323232323",
            "raw_sha256",
            "operation",
            "container_contract",
        ] {
            assert!(!json.contains(forbidden));
        }
        let retry = recover(&t, |_, _| Ok(()));
        assert_eq!(retry.primary_error, Some(OwnershipError::Transition));
        cleanup(&t, true, |_, _| Ok(())).unwrap();
        assert!(!final_path(&t).exists());
        preserve_source(&t, &before);
    }
}
#[test]
fn namespace_table_and_real_ambiguous_locations_fail_closed() {
    use OutputLocation::*;
    for a in [Absent, Owned, ForeignOrInvalid] {
        for b in [Absent, Owned, ForeignOrInvalid] {
            assert_eq!(
                origin(a, b).is_ok(),
                matches!((a, b), (Owned, Absent) | (Absent, Owned))
            );
        }
    }
    for renamed in [false, true] {
        for kind in ["neither", "both", "invalid", "extra"] {
            let t = pending(renamed);
            let p = directory(&t, renamed);
            let before = journal(&t);
            let other = if renamed {
                t.jobs().join(stage(&record(&t).operation))
            } else {
                final_path(&t)
            };
            match kind {
                "neither" => fs::rename(&p, t.0.join("held")).unwrap(),
                "both" => {
                    fs::DirBuilder::new().mode(0o700).create(&other).unwrap();
                }
                "invalid" => {
                    fs::rename(&p, t.0.join("held")).unwrap();
                    std::os::unix::fs::symlink("missing", &p).unwrap();
                }
                _ => fs::write(p.join("foreign"), b"keep").unwrap(),
            }
            let saved = fs::read(t.jobs().join(name(&oid().0))).unwrap();
            let r = recover(&t, |_, _| Ok(()));
            assert!(!r.is_success());
            assert!(!r.content_verified);
            assert_eq!(fs::read(t.jobs().join(name(&oid().0))).unwrap(), saved);
            preserve_source(&t, &before);
        }
    }
}
#[test]
fn all_non_intent_states_and_invalid_records_are_rejected() {
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
        assert_eq!(record(&t).state, state);
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
                "sequence" => e.record.sequence = 8,
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
                            journal(&t)
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
                assert_eq!(record(&t).state, OutputState::PublishIntent);
            }
        }
    }
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
            assert!(!r.rename_observed && !r.acknowledgment_durable);
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
fn sync_and_rename_syscall_errors_stop_without_acknowledgment() {
    for renamed in [false, true] {
        for point in [
            "raw_synced",
            "metadata_synced",
            "marker_synced",
            "bundle_synced",
            "intent_file_synced",
            "intent_directory_synced",
            "source_directory_synced",
            "destination_directory_synced",
            "rename",
        ] {
            if renamed && point == "rename" {
                continue;
            }
            let t = pending(renamed);
            let before = journal(&t);
            let mut io = FaultIo {
                fail: Some(point),
                ..Default::default()
            };
            let r = t.store().reconcile_hook(
                oid(),
                id(),
                source(32 * 65536),
                reopen(&t),
                RetainedOptions::default(),
                &mut io,
                |_, _| Ok(()),
            );
            assert!(!r.is_success());
            assert!(!r.acknowledgment_durable);
            assert_eq!(io.calls.last(), Some(&point));
            assert_eq!(record(&t).state, OutputState::PublishIntent);
            preserve_source(&t, &before);
            assert!(recover(&t, |_, _| Ok(())).is_success());
        }
    }
    let t = pending(false);
    let mut io = FaultIo {
        collision: true,
        ..Default::default()
    };
    let r = t.store().reconcile_hook(
        oid(),
        id(),
        source(32 * 65536),
        reopen(&t),
        RetainedOptions::default(),
        &mut io,
        |_, _| Ok(()),
    );
    assert_eq!(r.primary_error, Some(OwnershipError::Exists));
    assert!(!r.rename_observed);
    assert_eq!(fs::read(final_path(&t)).unwrap(), b"foreign");
    assert!(raw_stage(&t).exists());
}
#[test]
fn cancellation_boundaries_keep_sync_observations_separate_from_success() {
    for renamed in [false, true] {
        for point in [
            "before_admission",
            "admitted",
            "verified_chunk",
            "validated",
            "bundle_synced",
            "intent_checked",
            "before_rename",
            "rename_checked",
            "before_parent_sync",
            "source_directory_synced",
            "destination_directory_synced",
            "before_acknowledgment",
            "acknowledgment_checked",
            "record_partial",
            "record_durable",
        ] {
            let t = pending(renamed);
            let token = crate::Cancellation::default();
            let cancel = token.clone();
            let r = t.store().reconcile_hook(
                oid(),
                id(),
                source(32 * 65536),
                reopen(&t),
                RetainedOptions {
                    cancellation: token,
                    ..Default::default()
                },
                &mut SystemIo,
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
            let parents = matches!(
                point,
                "before_parent_sync"
                    | "source_directory_synced"
                    | "destination_directory_synced"
                    | "before_acknowledgment"
                    | "acknowledgment_checked"
                    | "record_partial"
                    | "record_durable"
            );
            assert_eq!(r.directories_synced, parents, "{point}");
            assert_eq!(
                r.acknowledgment_durable,
                matches!(point, "record_partial" | "record_durable")
            );
            if point == "verified_chunk" {
                assert_eq!(r.logical_bytes_verified, 1 << 20);
            }
        }
    }
    let t = pending(false);
    let token = crate::Cancellation::default();
    token.cancel();
    let r = t.store().reconcile_publication(
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
    assert!(r.origin.is_none());
}
#[test]
fn deadline_is_shared_with_admission_and_invalid_budgets_do_not_mutate() {
    for timeout in [Duration::ZERO, Duration::from_secs(21601)] {
        let t = pending(false);
        let r = t.store().reconcile_publication(
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
        assert!(r.origin.is_none());
    }
    let t = pending(false);
    let r = t.store().reconcile_hook(
        oid(),
        id(),
        source(32 * 65536),
        reopen(&t),
        RetainedOptions {
            timeout: Duration::from_millis(100),
            ..Default::default()
        },
        &mut SystemIo,
        |_, p| {
            if p == "before_admission" {
                std::thread::sleep(Duration::from_millis(150));
            }
            Ok(())
        },
    );
    assert_eq!(r.primary_error, Some(OwnershipError::Deadline));
    assert!(!r.content_verified);
    assert_eq!(record(&t).state, OutputState::PublishIntent);
}
#[test]
fn acknowledgment_faults_and_assessment_failures_never_become_success() {
    for renamed in [false, true] {
        for point in [
            "record_partial",
            "record_written",
            "record_synced",
            "record_renamed",
            "record_durable",
        ] {
            let t = pending(renamed);
            let r = recover(&t, |s, p| {
                if s == OutputState::Published && p == point {
                    Err(OwnershipError::Io)
                } else {
                    Ok(())
                }
            });
            assert_eq!(r.primary_error, Some(OwnershipError::Uncertain));
            assert!(!r.is_success());
            assert!(!r.acknowledgment_durable);
            let new = matches!(point, "record_renamed" | "record_durable");
            let observed = r.recovery.unwrap();
            assert_eq!(
                observed.output.state,
                if new {
                    OutputState::Published
                } else {
                    OutputState::PublishIntent
                }
            );
            assert_eq!(observed.output.pending_transaction, !new);
            assert!(!recover(&t, |_, _| Ok(())).is_success());
        }
    }
    for primary in [false, true] {
        let t = pending(false);
        let r = recover(&t, |_, p| {
            if p == "before_assessment" || (primary && p == "validated") {
                Err(OwnershipError::Io)
            } else {
                Ok(())
            }
        });
        assert!(!r.is_success());
        assert_eq!(r.assessment_error, Some(OwnershipError::Io));
        assert_eq!(r.primary_error, primary.then_some(OwnershipError::Io));
        assert_eq!(r.acknowledgment_durable, !primary);
    }
}
#[test]
fn late_substitution_and_collisions_preserve_foreign_entries() {
    for renamed in [false, true] {
        for point in [
            "admitted",
            "validated",
            "intent_file_synced",
            "before_rename",
            "before_acknowledgment",
        ] {
            for what in ["raw", "journal", "source_journal", "source_txn"] {
                let t = pending(renamed);
                let mut changed = false;
                let r = recover(&t, |_, p| {
                    if p == point && !changed {
                        changed = true;
                        let path = match what {
                            "raw" => if final_path(&t).exists() {
                                final_path(&t)
                            } else {
                                raw_stage(&t)
                            }
                            .join("disk.raw"),
                            "journal" => t.jobs().join(name(&oid().0)),
                            "source_journal" => t.jobs().join(record_name(id().as_bytes())),
                            _ => t.jobs().join(transaction_name(id().as_bytes())),
                        };
                        if what == "source_txn" {
                            fs::write(path, b"pending").unwrap();
                        } else {
                            let data = fs::read(&path).unwrap();
                            fs::rename(&path, t.0.join("held")).unwrap();
                            let mut f = OpenOptions::new()
                                .write(true)
                                .create_new(true)
                                .mode(0o600)
                                .open(path)
                                .unwrap();
                            f.write_all(&data).unwrap();
                        }
                    }
                    Ok(())
                });
                assert!(changed);
                assert!(!r.is_success(), "{point} {what}");
                assert!(!r.acknowledgment_durable);
            }
        }
    }
    for kind in ["file", "directory", "symlink"] {
        let t = pending(false);
        let r = recover(&t, |_, p| {
            if p == "before_rename" {
                match kind {
                    "file" => fs::write(final_path(&t), b"keep").unwrap(),
                    "directory" => fs::create_dir(final_path(&t)).unwrap(),
                    _ => std::os::unix::fs::symlink("missing", final_path(&t)).unwrap(),
                }
            }
            Ok(())
        });
        assert_eq!(r.primary_error, Some(OwnershipError::Exists));
        assert!(raw_stage(&t).exists());
        assert!(fs::symlink_metadata(final_path(&t)).is_ok());
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
        let r = store.reconcile_publication(
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
        assert!(!r.rename_observed);
        assert!(raw_stage(&t).exists());
        preserve_source(&t, &before);
        drop(t.store());
        drop(reopen(&t));
    }
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
            let r = publish(&t, |_, p| {
                if p == if renamed {
                    "bundle_renamed"
                } else {
                    "before_rename"
                } {
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
            preserve_source(&t, &before);
        }
    }
}

mod benchmark;
mod crash;
