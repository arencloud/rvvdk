use super::super::super::reconciliation;
use super::*;
fn reconcile(t: &Temp, hook: impl FnMut(OutputState, &str) -> Result<()>) -> ReconciliationReport {
    t.store().reconcile_hook(
        oid(),
        id(),
        source(32 * 65536),
        reopen(t),
        RetainedOptions::default(),
        &mut reconciliation::SystemIo,
        hook,
    )
}
fn seed(origin: &str) -> Temp {
    let t = ready();
    drop(destination(&t));
    if origin != "normal" {
        let r = admit(&t)
            .unwrap()
            .publish_hook(reopen(&t), RetainedOptions::default(), |_, p| {
                if p == if origin == "staged" {
                    "before_rename"
                } else {
                    "bundle_renamed"
                } {
                    Err(OwnershipError::Io)
                } else {
                    Ok(())
                }
            });
        assert!(!r.is_success());
        assert_eq!(record(&t).state, OutputState::PublishIntent);
    }
    t
}
#[test]
fn attestation_crash_child() {
    let Some(root) = std::env::var_os("RVVDK_ATTEST_CRASH_ROOT") else {
        return;
    };
    let t = Temp(PathBuf::from(root));
    let mode = std::env::var("RVVDK_ATTEST_CRASH_MODE").unwrap();
    let point = std::env::var("RVVDK_ATTEST_CRASH_POINT").unwrap();
    let kill = || {
        // SAFETY: this dedicated synthetic child intentionally bypasses destructors.
        unsafe { libc::kill(libc::getpid(), libc::SIGKILL) };
    };
    let hook = |s, p: &str| {
        if p == point && (mode == "attest" || s == OutputState::Published) {
            kill();
        }
        Ok(())
    };
    let success = match mode.as_str() {
        "attest" => recover(&t, hook).is_success(),
        "normal" => admit(&t)
            .unwrap()
            .publish_hook(reopen(&t), RetainedOptions::default(), hook)
            .is_success(),
        _ => reconcile(&t, hook).is_success(),
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
    let mut child=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","ownership::retained::output::publication::attestation::tests::crash::attestation_crash_child"])
        .env("RVVDK_ATTEST_CRASH_ROOT",&t.0).env("RVVDK_ATTEST_CRASH_MODE",mode).env("RVVDK_ATTEST_CRASH_POINT",point).stdout(std::process::Stdio::null()).spawn().unwrap();
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
fn acknowledgment_process_loss_keeps_transactions_blocked_and_attests_only_published() {
    for origin in ["normal", "staged", "renamed"] {
        for point in [
            "record_partial",
            "record_written",
            "record_synced",
            "record_renamed",
            "record_durable",
            "after_return",
        ] {
            let t = seed(origin);
            let before = journal(&t);
            kill_at(&t, origin, point);
            preserve_source(&t, &before);
            let saved = snapshots(&t);
            let eligible = matches!(point, "record_renamed" | "record_durable" | "after_return");
            let r = recover(&t, |_, _| Ok(()));
            assert_eq!(r.is_success(), eligible, "{origin}/{point}: {r:?}");
            unchanged(&saved);
            verify(&final_path(&t).join("disk.raw"), 32, |i| i == 0 || i == 16);
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
        }
    }
}
#[test]
fn interrupted_attestation_repeats_all_work_without_journal_replacement() {
    for point in [
        "before_admission",
        "admitted",
        "verified_chunk",
        "validated",
        "raw_synced",
        "metadata_synced",
        "marker_synced",
        "bundle_synced",
        "parents_checked",
        "store_namespace_synced",
        "destination_synced",
        "record_checked",
        "published_file_synced",
        "published_directory_synced",
        "before_assessment",
        "after_assessment",
        "bindings_rechecked",
        "after_return",
    ] {
        let t = pending(true);
        let saved = snapshots(&t);
        kill_at(&t, "attest", point);
        unchanged(&saved);
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
        assert!(r.is_success(), "{point}: {r:?}");
        assert_eq!(io.calls, SYNCS);
        unchanged(&saved);
        verify(&final_path(&t).join("disk.raw"), 32, |i| i == 0 || i == 16);
    }
}
#[test]
fn acknowledgment_hook_failures_remain_failures_after_fresh_attestation() {
    for origin in ["normal", "staged", "renamed"] {
        for point in ["record_renamed", "record_durable"] {
            let t = seed(origin);
            let hook = |s, p: &str| {
                if s == OutputState::Published && p == point {
                    Err(OwnershipError::Io)
                } else {
                    Ok(())
                }
            };
            let (error, success) = if origin == "normal" {
                let r =
                    admit(&t)
                        .unwrap()
                        .publish_hook(reopen(&t), RetainedOptions::default(), hook);
                (r.primary_error, r.is_success())
            } else {
                let r = reconcile(&t, hook);
                (r.primary_error, r.is_success())
            };
            assert_eq!(error, Some(OwnershipError::Uncertain));
            assert!(!success);
            let saved = snapshots(&t);
            assert!(recover(&t, |_, _| Ok(())).is_success());
            unchanged(&saved);
            assert!(!success);
        }
    }
}
