use super::super::super::tests::{Temp, id, source};
use super::super::tests::{oid, raw_stage, record, run, setup, verify};
use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, FileExt, PermissionsExt},
    path::PathBuf,
};
fn ready() -> Temp {
    let t = setup();
    assert!(run(&t, |_, _| Ok(())).is_success());
    t
}
fn admit(t: &Temp) -> Result<VerifiedOutput> {
    VerifiedOutput::open(
        t.store(),
        oid(),
        id(),
        source(32 * 65536),
        RetainedOptions::default(),
    )
}
fn destination(t: &Temp) -> PublicationDirectory {
    let p = t.0.join("published");
    fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
    PublicationDirectory::open(&p).unwrap()
}
fn reopen(t: &Temp) -> PublicationDirectory {
    PublicationDirectory::open(&t.0.join("published")).unwrap()
}
fn final_path(t: &Temp) -> PathBuf {
    t.0.join("published").join(oid().bundle_name())
}
fn publish(t: &Temp, hook: impl FnMut(OutputState, &str) -> Result<()>) -> PublicationReport {
    admit(t)
        .unwrap()
        .publish_hook(destination(t), RetainedOptions::default(), hook)
}
fn cleanup(
    t: &Temp,
    published: bool,
    hook: impl FnMut(OutputState, &str) -> Result<()>,
) -> Result<()> {
    let dest = if published { Some(reopen(t)) } else { None };
    t.store().cleanup_output_hook(
        oid(),
        id(),
        &source(32 * 65536),
        dest.as_ref(),
        RetainedOptions::default(),
        hook,
    )
}
fn rewrite(t: &Temp, change: impl FnOnce(&mut OutputEnvelope)) {
    let path = t.jobs().join(name(&oid().0));
    let mut e: OutputEnvelope = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    change(&mut e);
    e.sha256 = Sha256::digest(serde_json::to_vec(&e.record).unwrap()).into();
    fs::write(path, serde_json::to_vec(&e).unwrap()).unwrap();
}
#[test]
fn v1_admission_atomic_bundle_publication_and_explicit_cleanup() {
    let t = ready();
    let before = fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap();
    assert_eq!(record(&t).version, 1);
    let v: serde_json::Value =
        serde_json::from_slice(&fs::read(t.jobs().join(name(&oid().0))).unwrap()).unwrap();
    assert!(v["record"].get("publication_parent").is_none());
    assert!(v["record"].get("cleanup_from").is_none());
    let old = raw_stage(&t);
    let inode = fs::metadata(old.join("disk.raw")).unwrap().ino();
    let r = publish(&t, |_, _| {
        assert!(matches!(
            JobStore::open(&t.jobs()),
            Err(OwnershipError::Busy)
        ));
        assert!(matches!(
            PublicationDirectory::open(&t.0.join("published")),
            Err(OwnershipError::Busy)
        ));
        Ok(())
    });
    assert!(r.is_success(), "{r:?}");
    assert!(!old.exists());
    assert_eq!(record(&t).version, 2);
    assert_eq!(
        fs::metadata(final_path(&t).join("disk.raw")).unwrap().ino(),
        inode
    );
    verify(&final_path(&t).join("disk.raw"), 32, |i| i == 0 || i == 16);
    assert_eq!(
        fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap(),
        before
    );
    let dest = reopen(&t);
    let mut store = t.store();
    let r = store
        .assess_publication(oid(), id(), &source(32 * 65536), &dest)
        .unwrap();
    assert_eq!(r.output.state, OutputState::Published);
    assert_eq!(r.published, OutputLocation::Owned);
    assert_eq!(r.staged, OutputLocation::Absent);
    // Source cleanup is separate and publication cleanup does not require source bytes.
    let s = source(32 * 65536);
    let recovery = store.recover(id(), &s).unwrap();
    store.cleanup_local(id(), recovery.operation, &s).unwrap();
    store
        .cleanup_output(oid(), id(), &s, Some(&dest), RetainedOptions::default())
        .unwrap();
    assert!(!final_path(&t).exists());
    assert_eq!(record(&t).state, OutputState::Cleaned);
    assert_eq!(record(&t).sequence, 10);
    store
        .cleanup_output(oid(), id(), &s, Some(&dest), RetainedOptions::default())
        .unwrap();
}
#[test]
fn fresh_admission_rejects_forged_claims_metadata_changes_and_unknown_members() {
    for kind in [
        "bytes",
        "forged_digest",
        "metadata",
        "contract",
        "extra",
        "symlink",
        "hardlink",
        "pending",
        "missing_source",
        "version",
        "v1_extended",
    ] {
        let t = ready();
        let p = raw_stage(&t);
        let raw = p.join("disk.raw");
        match kind {
            "bytes" | "forged_digest" => {
                OpenOptions::new()
                    .write(true)
                    .open(&raw)
                    .unwrap()
                    .write_all_at(b"drift", 65536)
                    .unwrap();
                if kind == "forged_digest" {
                    let hash: [u8; 32] = Sha256::digest(fs::read(&raw).unwrap()).into();
                    rewrite(&t, |e| e.record.raw_sha256 = Some(hash));
                    let path = p.join("output.json");
                    let mut m: OutputMetadata =
                        serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                    m.raw_sha256 = hash;
                    fs::write(path, serde_json::to_vec(&m).unwrap()).unwrap();
                }
            }
            "metadata" => {
                fs::write(p.join("output.json"), b"{}").unwrap();
            }
            "contract" => rewrite(&t, |e| e.record.container_contract = [0; 32]),
            "extra" => {
                fs::write(p.join("foreign"), b"keep").unwrap();
            }
            "symlink" => {
                fs::rename(&raw, p.join("original")).unwrap();
                std::os::unix::fs::symlink("original", raw).unwrap();
            }
            "hardlink" => {
                fs::hard_link(raw, t.0.join("other-link")).unwrap();
            }
            "pending" => {
                fs::write(t.jobs().join(txn(&oid().0)), b"pending").unwrap();
            }
            "missing_source" => {
                let source_stage = fs::read_dir(t.jobs())
                    .unwrap()
                    .map(|e| e.unwrap().path())
                    .find(|p| {
                        p.file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .starts_with("stage-")
                    })
                    .unwrap();
                fs::remove_file(source_stage.join("disk-1.vmdk")).unwrap();
            }
            "version" => rewrite(&t, |e| e.record.version = 3),
            _ => rewrite(&t, |e| {
                e.record.state = OutputState::PublishIntent;
                e.record.sequence = 7;
                e.record.publication_parent = Some(Identity {
                    dev: e.record.store.dev,
                    ino: 1,
                });
            }),
        }
        assert!(admit(&t).is_err(), "{kind}");
        assert!(p.exists());
    }
}
#[test]
fn publication_revalidates_after_admission_and_never_overwrites_collisions() {
    let t = ready();
    let a = admit(&t).unwrap();
    OpenOptions::new()
        .write(true)
        .open(raw_stage(&t).join("disk.raw"))
        .unwrap()
        .write_all_at(b"drift", 65536)
        .unwrap();
    let r = a.publish(destination(&t), RetainedOptions::default());
    assert_eq!(r.primary_error, Some(OwnershipError::Content));
    assert_eq!(record(&t).state, OutputState::Verified);
    assert!(!final_path(&t).exists());
    for kind in ["file", "directory", "symlink", "race"] {
        let t = ready();
        let a = admit(&t).unwrap();
        let dest = destination(&t);
        let final_name = final_path(&t);
        match kind {
            "file" => fs::write(&final_name, b"foreign").unwrap(),
            "directory" => fs::create_dir(&final_name).unwrap(),
            "symlink" => std::os::unix::fs::symlink("missing", &final_name).unwrap(),
            _ => {}
        }
        let r = a.publish_hook(dest, RetainedOptions::default(), |_, p| {
            if kind == "race" && p == "before_rename" {
                fs::write(&final_name, b"foreign").unwrap();
            }
            Ok(())
        });
        assert_eq!(r.primary_error, Some(OwnershipError::Exists));
        assert!(!r.rename_observed);
        assert!(!r.is_success());
        assert!(raw_stage(&t).exists());
        assert_eq!(
            record(&t).state,
            if kind == "race" {
                OutputState::PublishIntent
            } else {
                OutputState::Verified
            }
        );
        if kind == "file" || kind == "race" {
            assert_eq!(fs::read(final_name).unwrap(), b"foreign");
        }
    }
}
#[test]
fn publication_journal_faults_never_turn_uncertainty_into_success() {
    for state in [OutputState::PublishIntent, OutputState::Published] {
        for phase in [
            "record_partial",
            "record_written",
            "record_synced",
            "record_renamed",
            "record_durable",
        ] {
            let t = ready();
            let r = publish(&t, |s, p| {
                if s == state && p == phase {
                    Err(OwnershipError::Io)
                } else {
                    Ok(())
                }
            });
            assert_eq!(r.primary_error, Some(OwnershipError::Uncertain));
            assert!(!r.is_success());
            let renamed = matches!(phase, "record_renamed" | "record_durable");
            let observed = r.recovery.unwrap();
            assert_eq!(
                observed.output.sequence,
                state.sequence() - u64::from(!renamed)
            );
            assert_eq!(observed.output.pending_transaction, !renamed);
            assert_eq!(final_path(&t).exists(), state == OutputState::Published);
            if !renamed || observed.output.state == OutputState::PublishIntent {
                assert!(cleanup(&t, state == OutputState::Published, |_, _| Ok(())).is_err());
            }
        }
    }
}
#[test]
fn publication_boundaries_cancellation_and_namespace_substitution_are_conservative() {
    for phase in [
        "validated",
        "member_synced",
        "bundle_synced",
        "before_rename",
        "bundle_renamed",
        "source_directory_synced",
        "destination_directory_synced",
    ] {
        let t = ready();
        let r = publish(&t, |_, p| {
            if p == phase {
                Err(OwnershipError::Io)
            } else {
                Ok(())
            }
        });
        assert!(!r.is_success());
        assert_eq!(r.primary_error, Some(OwnershipError::Io));
        let renamed = matches!(
            phase,
            "bundle_renamed" | "source_directory_synced" | "destination_directory_synced"
        );
        assert_eq!(r.rename_observed, renamed);
        assert_eq!(final_path(&t).exists(), renamed);
    }
    for phase in [
        "validated",
        "before_rename",
        "bundle_renamed",
        "destination_directory_synced",
    ] {
        let t = ready();
        let token = crate::Cancellation::default();
        let cancel = token.clone();
        let a = admit(&t).unwrap();
        let r = a.publish_hook(
            destination(&t),
            RetainedOptions {
                cancellation: token,
                ..RetainedOptions::default()
            },
            |_, p| {
                if p == phase {
                    cancel.cancel();
                }
                Ok(())
            },
        );
        assert_eq!(r.primary_error, Some(OwnershipError::Cancelled));
        assert!(!r.is_success());
        let renamed = matches!(phase, "bundle_renamed" | "destination_directory_synced");
        assert_eq!(r.rename_observed, renamed);
        assert_eq!(r.directories_synced, renamed);
    }
    let t = ready();
    let r = publish(&t, |_, p| {
        if p == "before_rename" {
            let path = raw_stage(&t).join("disk.raw");
            fs::rename(&path, t.0.join("held.raw")).unwrap();
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(path)
                .unwrap();
            f.write_all(b"foreign").unwrap();
        }
        Ok(())
    });
    assert_eq!(r.primary_error, Some(OwnershipError::Identity));
    assert_eq!(
        fs::read(raw_stage(&t).join("disk.raw")).unwrap(),
        b"foreign"
    );
    assert!(!final_path(&t).exists());
}
#[test]
fn destination_constraints_and_binding_are_explicit() {
    let t = ready();
    let a = admit(&t).unwrap();
    let nested = t.jobs().join("destination");
    fs::DirBuilder::new().mode(0o700).create(&nested).unwrap();
    let r = a.publish(
        PublicationDirectory::open(&nested).unwrap(),
        RetainedOptions::default(),
    );
    assert_eq!(r.primary_error, Some(OwnershipError::Identity));
    let t = ready();
    let dest = destination(&t);
    drop(dest);
    fs::set_permissions(t.0.join("published"), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(PublicationDirectory::open(&t.0.join("published")).is_err());
    let t = ready();
    assert!(publish(&t, |_, _| Ok(())).is_success());
    let other = t.0.join("other");
    fs::DirBuilder::new().mode(0o700).create(&other).unwrap();
    let wrong = PublicationDirectory::open(&other).unwrap();
    assert!(
        t.store()
            .assess_publication(oid(), id(), &source(32 * 65536), &wrong)
            .is_err()
    );
    assert!(
        t.store()
            .cleanup_output(
                oid(),
                id(),
                &source(32 * 65536),
                Some(&wrong),
                RetainedOptions::default()
            )
            .is_err()
    );
    assert!(final_path(&t).exists());
    let t = ready();
    let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let parent = if fs::metadata(&t.0).unwrap().dev() == fs::metadata(&cwd).unwrap().dev() {
        std::env::temp_dir()
    } else {
        cwd.join("target")
    };
    if fs::metadata(&parent).unwrap().dev() != fs::metadata(&t.0).unwrap().dev() {
        let mut op = [0; 16];
        random(&mut op).unwrap();
        let p = parent.join(format!(
            "rvddk-publish-cross-{}",
            crate::transport::hex_string(&op)
        ));
        fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        let other = Temp(p);
        let r = admit(&t).unwrap().publish(
            PublicationDirectory::open(&other.0).unwrap(),
            RetainedOptions::default(),
        );
        assert_eq!(r.primary_error, Some(OwnershipError::Identity));
        assert_eq!(record(&t).state, OutputState::Verified);
    }
}
#[test]
fn cleanup_accepts_stamped_partial_states_but_refuses_unstamped_and_uncertain_ones() {
    for state in [
        OutputState::Staged,
        OutputState::ConvertIntent,
        OutputState::Converted,
        OutputState::VerifyIntent,
        OutputState::Verified,
    ] {
        let t = setup();
        let r = run(&t, |s, p| {
            if s == state && p == "record_durable" {
                Err(OwnershipError::Io)
            } else {
                Ok(())
            }
        });
        assert!(!r.is_success());
        let path = raw_stage(&t);
        cleanup(&t, false, |_, _| Ok(())).unwrap();
        assert!(!path.exists());
        assert_eq!(record(&t).state, OutputState::Cleaned);
        assert_eq!(record(&t).sequence, state.sequence() + 2);
    }
    let t = setup();
    run(&t, |_, p| {
        if p == "stage_created" {
            Err(OwnershipError::Io)
        } else {
            Ok(())
        }
    });
    assert_eq!(
        cleanup(&t, false, |_, _| Ok(())),
        Err(OwnershipError::Transition)
    );
    assert!(raw_stage(&t).exists());
    let t = ready();
    publish(&t, |_, p| {
        if p == "bundle_renamed" {
            Err(OwnershipError::Io)
        } else {
            Ok(())
        }
    });
    assert_eq!(
        cleanup(&t, true, |_, _| Ok(())),
        Err(OwnershipError::Transition)
    );
    assert!(final_path(&t).exists());
}
#[test]
fn cleanup_faults_resume_only_checked_missing_members_and_sync_before_ack() {
    for published in [false, true] {
        for phase in [
            "cleanup_raw",
            "cleanup_metadata",
            "cleanup_marker",
            "cleanup_members_synced",
            "cleanup_directory_removed",
            "cleanup_parent_synced",
        ] {
            let t = ready();
            if published {
                assert!(publish(&t, |_, _| Ok(())).is_success());
            }
            let path = if published {
                final_path(&t)
            } else {
                raw_stage(&t)
            };
            assert_eq!(
                cleanup(&t, published, |_, p| if p == phase {
                    Err(OwnershipError::Io)
                } else {
                    Ok(())
                }),
                Err(OwnershipError::Io)
            );
            assert_eq!(record(&t).state, OutputState::CleanupIntent);
            let mut parent_synced = false;
            cleanup(&t, published, |_, p| {
                if p == "cleanup_parent_synced" {
                    parent_synced = true;
                }
                Ok(())
            })
            .unwrap();
            assert!(parent_synced);
            assert!(!path.exists());
            assert_eq!(record(&t).state, OutputState::Cleaned);
        }
    }
    for state in [OutputState::CleanupIntent, OutputState::Cleaned] {
        for phase in [
            "record_partial",
            "record_synced",
            "record_renamed",
            "record_durable",
        ] {
            let t = ready();
            let path = raw_stage(&t);
            assert_eq!(
                cleanup(&t, false, |s, p| if s == state && p == phase {
                    Err(OwnershipError::Io)
                } else {
                    Ok(())
                }),
                Err(OwnershipError::Uncertain)
            );
            assert_eq!(path.exists(), state == OutputState::CleanupIntent);
            let result = cleanup(&t, false, |_, _| Ok(()));
            if matches!(phase, "record_partial" | "record_synced") {
                assert_eq!(result, Err(OwnershipError::Uncertain));
            } else {
                result.unwrap();
            }
        }
    }
}
#[test]
fn cleanup_preserves_unknown_or_replaced_files_and_cancellation_keeps_intent() {
    for kind in ["extra", "raw", "marker"] {
        let t = ready();
        let dir = raw_stage(&t);
        match kind {
            "extra" => fs::write(dir.join("foreign"), b"keep").unwrap(),
            "raw" => {
                fs::rename(dir.join("disk.raw"), t.0.join("old.raw")).unwrap();
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(dir.join("disk.raw"))
                    .unwrap()
                    .write_all(b"keep")
                    .unwrap();
            }
            _ => fs::write(dir.join("owner"), b"keep").unwrap(),
        }
        assert_eq!(
            cleanup(&t, false, |_, _| Ok(())),
            Err(OwnershipError::Identity)
        );
        assert_eq!(record(&t).state, OutputState::Verified);
        assert!(dir.join("output.json").exists());
    }
    let t = ready();
    let dir = raw_stage(&t);
    let token = crate::Cancellation::default();
    let cancel = token.clone();
    let result = t.store().cleanup_output_hook(
        oid(),
        id(),
        &source(32 * 65536),
        None,
        RetainedOptions {
            cancellation: token,
            ..RetainedOptions::default()
        },
        |_, p| {
            if p == "cleanup_raw" {
                cancel.cancel();
            }
            Ok(())
        },
    );
    assert_eq!(result, Err(OwnershipError::Cancelled));
    assert!(!dir.join("disk.raw").exists());
    assert!(dir.join("output.json").exists());
    assert_eq!(record(&t).state, OutputState::CleanupIntent);
    cleanup(&t, false, |_, _| Ok(())).unwrap();
    let t = ready();
    let dir = raw_stage(&t);
    let result = cleanup(&t, false, |_, p| {
        if p == "cleanup_raw" {
            fs::rename(dir.join("output.json"), t.0.join("held.json")).unwrap();
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(dir.join("output.json"))
                .unwrap()
                .write_all(b"foreign")
                .unwrap();
        }
        Ok(())
    });
    assert_eq!(result, Err(OwnershipError::Identity));
    assert_eq!(fs::read(dir.join("output.json")).unwrap(), b"foreign");
}
#[test]
fn publication_crash_child() {
    let Some(root) = std::env::var_os("RVVDK_PUBLICATION_CRASH") else {
        return;
    };
    let root = PathBuf::from(root);
    let phase = std::env::var("RVVDK_PUBLICATION_PHASE").unwrap();
    let kill = |_: OutputState, p: &str| -> Result<()> {
        if p == phase {
            // SAFETY: terminate only this dedicated synthetic subprocess.
            unsafe { libc::kill(libc::getpid(), libc::SIGKILL) };
        }
        Ok(())
    };
    let store = JobStore::open(&root.join("jobs")).unwrap();
    let dest = PublicationDirectory::open(&root.join("published")).unwrap();
    if phase.starts_with("cleanup_") {
        let mut store = store;
        store
            .cleanup_output_hook(
                oid(),
                id(),
                &source(32 * 65536),
                Some(&dest),
                RetainedOptions::default(),
                kill,
            )
            .unwrap();
    } else {
        VerifiedOutput::open(
            store,
            oid(),
            id(),
            source(32 * 65536),
            RetainedOptions::default(),
        )
        .unwrap()
        .publish_hook(dest, RetainedOptions::default(), kill);
    }
    panic!("crash point not reached");
}
#[test]
fn process_loss_observes_rename_without_replay_and_cleanup_can_finish_absent_directory() {
    use std::os::unix::process::ExitStatusExt;
    for phase in [
        "before_rename",
        "bundle_renamed",
        "source_directory_synced",
        "destination_directory_synced",
        "cleanup_raw",
        "cleanup_directory_removed",
    ] {
        let t = ready();
        let cleaning = phase.starts_with("cleanup_");
        if cleaning {
            assert!(publish(&t, |_, _| Ok(())).is_success());
        } else {
            drop(destination(&t));
        }
        let before = fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap();
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "ownership::retained::output::publication::tests::publication_crash_child",
                "--nocapture",
            ])
            .env("RVVDK_PUBLICATION_CRASH", &t.0)
            .env("RVVDK_PUBLICATION_PHASE", phase)
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
            fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap(),
            before
        );
        let r = t
            .store()
            .assess_publication(oid(), id(), &source(32 * 65536), &reopen(&t))
            .unwrap();
        if cleaning {
            assert_eq!(r.output.state, OutputState::CleanupIntent);
            cleanup(&t, true, |_, _| Ok(())).unwrap();
            assert!(!final_path(&t).exists());
        } else {
            assert_eq!(r.output.state, OutputState::PublishIntent);
            assert_eq!(
                r.published,
                if phase == "before_rename" {
                    OutputLocation::Absent
                } else {
                    OutputLocation::Owned
                }
            );
            assert!(cleanup(&t, phase != "before_rename", |_, _| Ok(())).is_err());
        }
    }
}

mod benchmark;
