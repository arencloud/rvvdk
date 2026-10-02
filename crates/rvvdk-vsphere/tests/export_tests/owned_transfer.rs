use super::owned_probe::{artifact, record, store};
use super::*;
use rvvdk_vsphere::{
    OwnedTransferOptions,
    ownership::{JobState, JobStore, OwnershipError, RecoveryAction},
    transfer_owned_export,
};
pub(super) fn replies() -> Vec<Reply> {
    let mut r = explicit_replies();
    r.insert(15, r[14].clone());
    r
}
pub(super) fn opts() -> OwnedTransferOptions {
    OwnedTransferOptions {
        max_encoded_bytes: 8 << 20,
        ..Default::default()
    }
}
pub(super) fn stage(path: &std::path::Path) -> PathBuf {
    let stages: Vec<_> = std::fs::read_dir(path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    assert_eq!(stages.len(), 1);
    stages[0].clone()
}
#[tokio::test]
async fn completes_only_after_durable_bytes_readback_and_completion_intent() {
    let out = Output::new();
    let store = store(&out);
    let path = out.0.join("jobs");
    let seen = Arc::new(Mutex::new(Vec::new()));
    let observed = seen.clone();
    let observer = Arc::new(move |request: &str| {
        let expected = if request.contains("<ExportVm ") {
            Some("acquire_intent")
        } else if request.contains("<HttpNfcLeaseComplete ") {
            Some("complete_intent")
        } else {
            None
        };
        if let Some(expected) = expected {
            assert_eq!(record(&path)["record"]["state"], expected);
            observed.lock().unwrap().push(expected);
        }
    });
    let server = Server::start_observed(replies(), true, Some(observer));
    let source = explicit_source(&server);
    let report = transfer_owned_export(source.clone(), &credentials(), store, artifact(), opts())
        .await
        .unwrap();
    assert!(report.is_success(), "{report:?}");
    assert_eq!(*seen.lock().unwrap(), ["acquire_intent", "complete_intent"]);
    assert_eq!(
        (
            report.received_encoded_bytes,
            report.written_encoded_bytes,
            report.durable_encoded_bytes
        ),
        (65536, 65536, 65536)
    );
    let recovery = report.lifecycle.recovery.as_ref().unwrap();
    assert_eq!(recovery.state, JobState::CompletedLease);
    assert_eq!(recovery.sequence, 7);
    let path = stage(&out.0.join("jobs"));
    assert_eq!(
        std::fs::read(path.join("disk-1.vmdk")).unwrap(),
        payload().as_bytes()
    );
    assert_eq!(
        std::fs::metadata(path.join("manifest.json")).unwrap().len(),
        0
    );
    assert!(
        !server
            .requests()
            .iter()
            .any(|r| r.contains("<HttpNfcLeaseAbort ") || r.contains("<ShutdownGuest "))
    );
    let wire = serde_json::to_string(&report).unwrap();
    assert!(
        !wire.contains("PRIVATE") && !wire.contains("sha256") && !wire.contains("lease-private")
    );
    JobStore::open(&out.0.join("jobs"))
        .unwrap()
        .cleanup_local(artifact(), recovery.operation, &source)
        .unwrap();
}
#[tokio::test]
async fn completion_fault_or_lost_reply_never_triggers_abort_or_retry() {
    for lost in [false, true] {
        let out = Output::new();
        let store = store(&out);
        let mut r = replies();
        if lost {
            r[16].declared = Some(r[16].body.len() + 17);
        } else {
            r[16] = Reply::fault("RuntimeFault");
        }
        let server = Server::start(r);
        let source = explicit_source(&server);
        let report =
            transfer_owned_export(source.clone(), &credentials(), store, artifact(), opts())
                .await
                .unwrap();
        assert!(!report.is_success());
        assert!(report.lifecycle.primary_error.is_some());
        assert_eq!(report.lifecycle.lease_cleanup, LeaseCleanup::Unconfirmed);
        let recovery = report.lifecycle.recovery.unwrap();
        assert_eq!(recovery.state, JobState::CompleteIntent);
        assert_eq!(recovery.action, RecoveryAction::RemoteOutcomeUnknown);
        assert_eq!(
            server
                .requests()
                .iter()
                .filter(|r| r.contains("<HttpNfcLeaseComplete "))
                .count(),
            1
        );
        assert!(
            !server
                .requests()
                .iter()
                .any(|r| r.contains("<HttpNfcLeaseAbort "))
        );
        assert_eq!(
            JobStore::open(&out.0.join("jobs")).unwrap().cleanup_local(
                artifact(),
                recovery.operation,
                &source
            ),
            Err(OwnershipError::Transition)
        );
    }
}
#[tokio::test]
async fn independent_readback_rejects_changed_file_and_preserves_foreign_members() {
    for mutation in ["bytes", "replace", "owner"] {
        let out = Output::new();
        let store = store(&out);
        let path = out.0.join("jobs");
        let observer = Arc::new(move |request: &str| {
            if request.contains("<HttpNfcLeaseGetManifest ") {
                let s = stage(&path);
                match mutation {
                    "bytes" => {
                        use std::os::unix::fs::FileExt;
                        std::fs::OpenOptions::new()
                            .write(true)
                            .open(s.join("disk-1.vmdk"))
                            .unwrap()
                            .write_at(b"changed", 100)
                            .unwrap();
                    }
                    "replace" => {
                        std::fs::rename(s.join("disk-1.vmdk"), s.join("original")).unwrap();
                        std::fs::write(s.join("disk-1.vmdk"), b"foreign sentinel").unwrap();
                    }
                    _ => std::fs::write(s.join("owner"), b"invalid-owner-xx").unwrap(),
                }
            }
        });
        let mut r = replies();
        r.truncate(14);
        r.extend([abort(), logout()]);
        let server = Server::start_observed(r, true, Some(observer));
        let report = transfer_owned_export(
            explicit_source(&server),
            &credentials(),
            store,
            artifact(),
            opts(),
        )
        .await
        .unwrap();
        assert!(report.manifest_verified);
        assert!(!report.container_readback_verified);
        assert!(report.payload_error.is_some());
        assert_eq!(report.lifecycle.lease_cleanup, LeaseCleanup::Aborted);
        assert_eq!(
            report.lifecycle.recovery.unwrap().state,
            JobState::AbortedLease
        );
        assert!(
            !server
                .requests()
                .iter()
                .any(|r| r.contains("<HttpNfcLeaseComplete "))
        );
        if mutation == "replace" {
            assert_eq!(
                std::fs::read(stage(&out.0.join("jobs")).join("disk-1.vmdk")).unwrap(),
                b"foreign sentinel"
            );
        }
    }
}
#[tokio::test]
async fn cancellation_before_and_after_completion_intent_has_distinct_disposition() {
    for after in [false, true] {
        let out = Output::new();
        let store = store(&out);
        let options = opts();
        let cancel = options.cancellation.clone();
        let reads = std::sync::atomic::AtomicUsize::new(0);
        let observer = Arc::new(move |request: &str| {
            if request.contains("<RetrievePropertiesEx ")
                && request.contains(">vm-private</obj>")
                && reads.fetch_add(1, Ordering::Relaxed) == if after { 5 } else { 4 }
            {
                cancel.cancel();
            }
        });
        let mut r = replies();
        r.truncate(if after { 16 } else { 15 });
        if !after {
            r.push(abort());
        }
        r.push(logout());
        let server = Server::start_observed(r, true, Some(observer));
        let report = transfer_owned_export(
            explicit_source(&server),
            &credentials(),
            store,
            artifact(),
            options,
        )
        .await
        .unwrap();
        assert_eq!(report.lifecycle.primary_error, Some(Error::Cancelled));
        assert!(report.container_readback_verified);
        assert_eq!(
            report.lifecycle.recovery.unwrap().state,
            if after {
                JobState::CompleteIntent
            } else {
                JobState::AbortedLease
            }
        );
        assert!(
            !server
                .requests()
                .iter()
                .any(|r| r.contains("<HttpNfcLeaseComplete "))
        );
        assert_eq!(
            server
                .requests()
                .iter()
                .filter(|r| r.contains("<HttpNfcLeaseAbort "))
                .count(),
            usize::from(!after)
        );
    }
}
#[tokio::test]
async fn source_drift_before_download_or_completion_blocks_next_rpc() {
    for phase in [11, 14, 15] {
        let out = Output::new();
        let store = store(&out);
        let mut r = replies();
        r[phase].body = r[phase].body.replace("PRIVATE-path", "changed-backing");
        r.truncate(phase + 1);
        if phase != 15 {
            r.push(abort());
        }
        r.push(logout());
        let server = Server::start(r);
        let report = transfer_owned_export(
            explicit_source(&server),
            &credentials(),
            store,
            artifact(),
            opts(),
        )
        .await
        .unwrap();
        assert_eq!(report.lifecycle.primary_error, Some(Error::Identity));
        assert_eq!(
            report.lifecycle.recovery.unwrap().state,
            if phase == 15 {
                JobState::CompleteIntent
            } else {
                JobState::AbortedLease
            }
        );
        assert!(
            !server
                .requests()
                .iter()
                .any(|r| r.contains("<HttpNfcLeaseComplete "))
        );
        if phase == 11 {
            assert!(!server.requests().iter().any(|r| r.starts_with("GET ")));
        }
    }
}
#[tokio::test]
async fn completion_persistence_failures_never_authorize_abort() {
    for acknowledgment in [false, true] {
        let out = Output::new();
        let store = store(&out);
        let path = out.0.join("jobs");
        let reads = std::sync::atomic::AtomicUsize::new(0);
        let observer = Arc::new(move |request: &str| {
            let inject = if acknowledgment {
                request.contains("<HttpNfcLeaseComplete ")
            } else {
                request.contains("<RetrievePropertiesEx ")
                    && request.contains(">vm-private</obj>")
                    && reads.fetch_add(1, Ordering::Relaxed) == 4
            };
            if inject {
                std::fs::write(
                    path.join(format!("txn-{}.json", "2a".repeat(16))),
                    b"synthetic interruption",
                )
                .unwrap();
            }
        });
        let mut r = replies();
        if !acknowledgment {
            r.truncate(15);
            r.push(logout());
        }
        let server = Server::start_observed(r, true, Some(observer));
        let report = transfer_owned_export(
            explicit_source(&server),
            &credentials(),
            store,
            artifact(),
            opts(),
        )
        .await
        .unwrap();
        assert_eq!(
            report.lifecycle.journal_error,
            Some(OwnershipError::Uncertain)
        );
        let recovery = report.lifecycle.recovery.unwrap();
        assert!(recovery.pending_transaction);
        assert_eq!(
            recovery.state,
            if acknowledgment {
                JobState::CompleteIntent
            } else {
                JobState::TransferComplete
            }
        );
        assert_eq!(
            report.lifecycle.lease_cleanup,
            if acknowledgment {
                LeaseCleanup::Completed
            } else {
                LeaseCleanup::Unconfirmed
            }
        );
        assert!(
            !server
                .requests()
                .iter()
                .any(|r| r.contains("<HttpNfcLeaseAbort "))
        );
    }
}
#[tokio::test]
async fn truncated_over_budget_and_bad_manifest_transfers_abort_without_validation() {
    for case in ["truncated", "limit", "manifest"] {
        let out = Output::new();
        let store = store(&out);
        let mut r = replies();
        let mut options = opts();
        match case {
            "truncated" => {
                r[12].declared = Some(65537);
                r.truncate(13);
            }
            "limit" => {
                options.max_encoded_bytes = 512;
                r.truncate(13);
            }
            _ => {
                r[13].body = r[13]
                    .body
                    .replace("<size>65536</size>", "<size>65535</size>");
                r.truncate(14);
            }
        }
        r.extend([abort(), logout()]);
        let server = Server::start(r);
        let report = transfer_owned_export(
            explicit_source(&server),
            &credentials(),
            store,
            artifact(),
            options,
        )
        .await
        .unwrap();
        assert!(report.lifecycle.primary_error.is_some());
        assert_eq!(report.lifecycle.lease_cleanup, LeaseCleanup::Aborted);
        assert!(!report.container_readback_verified);
        assert_eq!(report.durable_encoded_bytes, 0);
        assert!(
            !server
                .requests()
                .iter()
                .any(|r| r.contains("<HttpNfcLeaseComplete "))
        );
    }
}

#[tokio::test]
async fn transfer_crash_child() {
    let Ok(config) = std::env::var("RVVDK_TRANSFER_CRASH_CHILD") else {
        return;
    };
    let c: Vec<String> = serde_json::from_str(&config).unwrap();
    use rvvdk_vsphere::contract::{EndpointIdentity, PinProvenance, SourceSelection};
    let source = SourceSelection::new(
        EndpointIdentity::pinned(&c[0], &c[1], PinProvenance::TrustOnFirstUse).unwrap(),
        "vm-private",
        "01234567-89ab-cdef-0123-456789abcdef",
        2000,
        "PRIVATE-path",
        30 << 30,
    )
    .unwrap();
    let _ = transfer_owned_export(
        source,
        &credentials(),
        JobStore::open(std::path::Path::new(&c[2])).unwrap(),
        artifact(),
        opts(),
    )
    .await;
    panic!("child should be killed at completion boundary");
}
#[test]
fn process_loss_at_completion_keeps_private_bytes_and_blocks_cleanup() {
    use std::os::unix::process::ExitStatusExt;
    let out = Output::new();
    drop(store(&out));
    let path = out.0.join("jobs");
    let pid = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let child_pid = pid.clone();
    let observer = Arc::new(move |request: &str| {
        if request.contains("<HttpNfcLeaseComplete ") {
            let pid = child_pid.load(Ordering::Acquire);
            assert_ne!(pid, 0);
            // SAFETY: PID belongs to this live, unreaped test child.
            assert_eq!(unsafe { libc::kill(pid as i32, libc::SIGKILL) }, 0);
        }
    });
    let server = Server::start_observed(replies(), true, Some(observer));
    let config = serde_json::to_string(&[
        server.endpoint.as_str(),
        server.pin.as_str(),
        path.to_str().unwrap(),
    ])
    .unwrap();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "export_tests::owned_transfer::transfer_crash_child",
        ])
        .env("RVVDK_TRANSFER_CRASH_CHILD", config)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    pid.store(child.id(), Ordering::Release);
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    let status = loop {
        if let Some(s) = child.try_wait().unwrap() {
            break s;
        }
        if std::time::Instant::now() > deadline {
            child.kill().unwrap();
            let _ = child.wait();
            panic!("child deadline");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(status.signal(), Some(libc::SIGKILL));
    let source = explicit_source(&server);
    let mut store = JobStore::open(&path).unwrap();
    let recovery = store.recover(artifact(), &source).unwrap();
    assert_eq!(recovery.state, JobState::CompleteIntent);
    assert!(!recovery.pending_transaction);
    assert_eq!(
        std::fs::read(stage(&path).join("disk-1.vmdk")).unwrap(),
        payload().as_bytes()
    );
    assert_eq!(
        store.cleanup_local(artifact(), recovery.operation, &source),
        Err(OwnershipError::Transition)
    );
    assert!(
        !server
            .requests()
            .iter()
            .any(|r| r.contains("<HttpNfcLeaseAbort "))
    );
}

mod benchmark;
