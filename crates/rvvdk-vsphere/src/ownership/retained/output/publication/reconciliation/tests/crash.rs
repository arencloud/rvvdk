use super::*;
#[test]
fn reconciliation_crash_child() {
    let Some(root) = std::env::var_os("RVVDK_RECONCILE_CRASH_ROOT") else {
        return;
    };
    let t = Temp(PathBuf::from(root));
    let point = std::env::var("RVVDK_RECONCILE_CRASH_POINT").unwrap();
    let initial = std::env::var("RVVDK_RECONCILE_CRASH_INITIAL").unwrap() == "yes";
    let kill = |state: OutputState, p: &str| -> Result<()> {
        if p == point && (!initial || state == OutputState::PublishIntent) {
            // SAFETY: dedicated synthetic child only; bypass destructors on purpose.
            unsafe { libc::kill(libc::getpid(), libc::SIGKILL) };
        }
        Ok(())
    };
    if initial {
        admit(&t)
            .unwrap()
            .publish_hook(reopen(&t), RetainedOptions::default(), kill);
    } else {
        recover(&t, kill);
    }
    // Never tear down evidence if a hook was accidentally not reached.
    std::mem::forget(t);
    panic!("crash point not reached");
}
fn kill_at(t: &Temp, initial: bool, point: &str) {
    use std::os::unix::process::ExitStatusExt;
    let mut child=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","ownership::retained::output::publication::reconciliation::tests::crash::reconciliation_crash_child"])
        .env("RVVDK_RECONCILE_CRASH_ROOT",&t.0).env("RVVDK_RECONCILE_CRASH_POINT",point).env("RVVDK_RECONCILE_CRASH_INITIAL",if initial{"yes"}else{"no"})
        .stdout(std::process::Stdio::null()).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.signal(), Some(libc::SIGKILL), "{point}");
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("child timeout: {point}");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn initial_publication_crashes_reconcile_even_visible_unsynced_intent() {
    for point in [
        "record_renamed",
        "record_durable",
        "before_rename",
        "bundle_renamed",
        "source_directory_synced",
        "destination_directory_synced",
    ] {
        let t = ready();
        drop(destination(&t));
        let before = journal(&t);
        kill_at(&t, true, point);
        assert_eq!(record(&t).state, OutputState::PublishIntent);
        preserve_source(&t, &before);
        let mut io = FaultIo::default();
        let r = t.store().reconcile_hook(
            oid(),
            id(),
            source(32 * 65536),
            reopen(&t),
            RetainedOptions::default(),
            &mut io,
            |_, _| Ok(()),
        );
        assert!(r.is_success(), "{point}: {r:?}");
        let sync = io
            .calls
            .iter()
            .position(|&v| v == "intent_directory_synced")
            .unwrap();
        if r.rename_observed {
            assert!(sync < io.calls.iter().position(|&v| v == "rename").unwrap());
        }
        verify(&final_path(&t).join("disk.raw"), 32, |i| i == 0 || i == 16);
        preserve_source(&t, &before);
    }
}
#[test]
fn reconciliation_process_loss_preserves_retry_or_blocked_acknowledgment() {
    for renamed in [false, true] {
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
            "intent_checked",
            "before_rename",
            "bundle_renamed",
            "before_parent_sync",
            "source_directory_synced",
            "destination_directory_synced",
            "before_acknowledgment",
            "record_partial",
            "record_written",
            "record_synced",
            "record_renamed",
            "record_durable",
            "before_assessment",
        ] {
            if renamed && point == "bundle_renamed" {
                continue;
            }
            let t = pending(renamed);
            let before = journal(&t);
            kill_at(&t, false, point);
            preserve_source(&t, &before);
            let assessment = t
                .store()
                .assess_publication(oid(), id(), &source(32 * 65536), &reopen(&t))
                .unwrap();
            let blocked = point.starts_with("record_") || point == "before_assessment";
            let r = recover(&t, |_, _| Ok(()));
            assert_eq!(r.is_success(), !blocked, "{renamed} {point}: {r:?}");
            if blocked {
                assert!(
                    assessment.output.pending_transaction
                        || assessment.output.state == OutputState::Published
                );
            }
            verify(&final_path(&t).join("disk.raw"), 32, |i| i == 0 || i == 16);
            preserve_source(&t, &before);
        }
    }
}
