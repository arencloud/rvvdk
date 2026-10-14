use super::*;
#[test]
fn acknowledgment_crash_child() {
    let Some(root) = std::env::var_os("RVVDK_ACK_CRASH_ROOT") else {
        return;
    };
    let t = Temp(PathBuf::from(root));
    let mode = std::env::var("RVVDK_ACK_CRASH_MODE").unwrap();
    let point = std::env::var("RVVDK_ACK_CRASH_POINT").unwrap();
    let kill = || {
        /* SAFETY: dedicated synthetic child intentionally bypasses destructors. */
        unsafe { libc::kill(libc::getpid(), libc::SIGKILL) };
    };
    let hook = |s, p: &str| {
        if p == point && (mode == "recover" || s == OutputState::Published) {
            kill();
        }
        Ok(())
    };
    let success = if mode == "recover" {
        recover(&t, hook).is_success()
    } else {
        origin(&t, &mode, hook)
    };
    if point == "after_return" {
        assert!(success);
        kill();
    }
    std::mem::forget(t);
    panic!("crash point not reached");
}
fn kill_at(t: &Temp, mode: &str, point: &str) {
    use std::os::unix::process::ExitStatusExt;
    let mut child=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","ownership::retained::output::publication::acknowledgment::tests::crash::acknowledgment_crash_child"])
    .env("RVVDK_ACK_CRASH_ROOT",&t.0).env("RVVDK_ACK_CRASH_MODE",mode).env("RVVDK_ACK_CRASH_POINT",point).stdout(std::process::Stdio::null()).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.signal(), Some(libc::SIGKILL), "{mode}/{point}");
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("crash child timeout");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn originating_process_loss_classifies_all_three_paths_without_editing_records() {
    for which in ["normal", "staged", "renamed"] {
        for point in [
            "record_partial",
            "record_written",
            "record_synced",
            "record_renamed",
            "record_durable",
            "after_return",
        ] {
            let t = ready();
            let saved = snapshots(&t);
            kill_at(&t, which, point);
            unchanged(&saved);
            let eligible = matches!(point, "record_written" | "record_synced");
            let has_candidate = candidate(&t).exists();
            let before = if has_candidate {
                stamp(&candidate(&t))
            } else {
                stamp(&current_path(&t))
            };
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
            let r = recover(&t, |_, _| Ok(()));
            assert_eq!(r.is_success(), eligible, "{which}/{point}: {r:?}");
            if point == "record_partial" {
                assert_eq!(stamp(&candidate(&t)), before);
            } else {
                promoted(&t, &before);
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
            unchanged(&saved);
            verify(&final_path(&t).join("disk.raw"), 32, |i| i == 0 || i == 16);
        }
    }
}
#[test]
fn recovery_process_loss_preserves_pair_or_requires_separate_attestation() {
    for synced in [false, true] {
        for point in [
            "before_admission",
            "admitted",
            "verified_chunk",
            "validated",
            "raw_synced",
            "metadata_synced",
            "marker_synced",
            "bundle_synced",
            "intent_file_synced",
            "intent_directory_synced",
            "store_namespace_synced",
            "destination_synced",
            "transaction_checked",
            "transaction_synced",
            "before_promotion",
            "transaction_promoted",
            "acknowledgment_directory_synced",
            "before_assessment",
            "after_assessment",
            "bindings_rechecked",
            "after_return",
        ] {
            let t = pending(synced);
            let old = stamp(&current_path(&t));
            let next = stamp(&candidate(&t));
            let saved = snapshots(&t);
            kill_at(&t, "recover", point);
            unchanged(&saved);
            if candidate(&t).exists() {
                assert_eq!(stamp(&candidate(&t)), next);
                assert_eq!(stamp(&current_path(&t)), old);
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
                assert!(r.is_success());
                assert_eq!(io.calls, SYNCS);
                assert_eq!(io.renames, 1);
            } else {
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
            unchanged(&saved);
            verify(&final_path(&t).join("disk.raw"), 32, |i| i == 0 || i == 16);
        }
    }
}
