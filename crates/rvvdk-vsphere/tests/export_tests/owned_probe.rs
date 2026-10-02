use super::*;
use rvvdk_vsphere::{
    Cancellation,
    contract::ArtifactId,
    ownership::{JobState, JobStore, OwnershipError, RecoveryAction},
    probe_owned_export,
};
use std::os::unix::fs::DirBuilderExt;

fn artifact() -> ArtifactId {
    ArtifactId::new([42; 16]).unwrap()
}
fn store(out: &Output) -> JobStore {
    let path = out.0.join("jobs");
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&path)
        .unwrap();
    JobStore::open(&path).unwrap()
}
fn record(path: &std::path::Path) -> serde_json::Value {
    let files: Vec<_> = std::fs::read_dir(path)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.file_name().unwrap().to_str().unwrap().starts_with("job-"))
        .collect();
    assert_eq!(files.len(), 1);
    serde_json::from_slice(&std::fs::read(&files[0]).unwrap()).unwrap()
}
fn replies() -> Vec<Reply> {
    let mut replies = explicit_replies();
    replies.truncate(11); // initial observations, acquisition, ready lease
    replies.extend([abort(), logout()]);
    replies
}
fn state_observer(path: PathBuf, seen: Arc<Mutex<Vec<String>>>) -> RequestObserver {
    Arc::new(move |request| {
        let expected = if request.contains("<ExportVm ") {
            Some("acquire_intent")
        } else if request.contains("<HttpNfcLeaseAbort ") {
            Some("abort_intent")
        } else {
            None
        };
        if let Some(expected) = expected {
            let record = record(&path);
            assert_eq!(record["record"]["state"], expected);
            assert!(record["record"]["stage"].is_object());
            assert!(
                !std::fs::read_dir(&path).unwrap().any(|e| e
                    .unwrap()
                    .file_name()
                    .to_str()
                    .unwrap()
                    .starts_with("txn-"))
            );
            seen.lock().unwrap().push(expected.into());
        }
    })
}

#[tokio::test]
async fn intents_precede_rpc_and_cleanup_requires_the_returned_operation() {
    let out = Output::new();
    let store = store(&out);
    let seen = Arc::new(Mutex::new(Vec::new()));
    let server = Server::start_observed(
        replies(),
        true,
        Some(state_observer(out.0.join("jobs"), seen.clone())),
    );
    let source = explicit_source(&server);
    let report = probe_owned_export(
        source.clone(),
        &credentials(),
        store,
        artifact(),
        Cancellation::default(),
    )
    .await
    .unwrap();
    assert!(report.is_success(), "{report:?}");
    assert_eq!(*seen.lock().unwrap(), ["acquire_intent", "abort_intent"]);
    let recovery = report.recovery.as_ref().unwrap();
    assert_eq!(recovery.state, JobState::AbortedLease);
    assert_eq!(recovery.sequence, 6);
    assert_eq!(recovery.action, RecoveryAction::CheckOwnedLocalResources);
    let wire = serde_json::to_string(&report).unwrap();
    assert!(
        !wire.contains("PRIVATE") && !wire.contains("lease-private") && !wire.contains("operation")
    );
    let mut reopened = JobStore::open(&out.0.join("jobs")).unwrap();
    let stages: Vec<_> = std::fs::read_dir(out.0.join("jobs"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .collect();
    assert_eq!(stages.len(), 1);
    assert_eq!(
        std::fs::metadata(stages[0].join("disk-1.vmdk"))
            .unwrap()
            .len(),
        0
    );
    assert_eq!(
        std::fs::metadata(stages[0].join("manifest.json"))
            .unwrap()
            .len(),
        0
    );
    reopened
        .cleanup_local(artifact(), recovery.operation, &source)
        .unwrap();
    assert_eq!(
        reopened.recover(artifact(), &source).unwrap().state,
        JobState::Cleaned
    );
    assert!(!server.requests().iter().any(|r| r.starts_with("GET ")
        || r.contains("<HttpNfcLeaseComplete ")
        || r.contains("<ShutdownGuest ")));
}

#[tokio::test]
async fn acquisition_and_abort_failures_remain_unknown_without_retry() {
    for phase in [9, 11] {
        let out = Output::new();
        let store = store(&out);
        let mut replies = replies();
        replies[phase] = Reply::fault("RuntimeFault");
        replies.truncate(phase + 1);
        replies.push(logout());
        let server = Server::start(replies);
        let source = explicit_source(&server);
        let report = probe_owned_export(
            source.clone(),
            &credentials(),
            store,
            artifact(),
            Cancellation::default(),
        )
        .await
        .unwrap();
        assert!(!report.is_success());
        assert_eq!(report.primary_error, Some(Error::SoapFault));
        assert_eq!(report.lease_cleanup, LeaseCleanup::Unconfirmed);
        assert_eq!(report.session_cleanup, Cleanup::LoggedOut);
        let recovery = report.recovery.unwrap();
        assert_eq!(
            recovery.state,
            if phase == 9 {
                JobState::AcquireIntent
            } else {
                JobState::AbortIntent
            }
        );
        assert_eq!(recovery.action, RecoveryAction::RemoteOutcomeUnknown);
        let mut store = JobStore::open(&out.0.join("jobs")).unwrap();
        assert_eq!(
            store.cleanup_local(artifact(), recovery.operation, &source),
            Err(OwnershipError::Transition)
        );
        let requests = server.requests();
        assert_eq!(
            requests.iter().filter(|r| r.contains("<ExportVm ")).count(),
            1
        );
        assert_eq!(
            requests
                .iter()
                .filter(|r| r.contains("<HttpNfcLeaseAbort "))
                .count(),
            usize::from(phase == 11)
        );
    }
}

#[tokio::test]
async fn malformed_acquisition_does_not_invent_a_lease() {
    let out = Output::new();
    let store = store(&out);
    let mut replies = replies();
    replies[9] = Reply::soap(
        "ExportVm",
        "<returnval type='VirtualMachine'>wrong-kind</returnval>",
    );
    replies.truncate(10);
    replies.push(logout());
    let server = Server::start(replies);
    let report = probe_owned_export(
        explicit_source(&server),
        &credentials(),
        store,
        artifact(),
        Cancellation::default(),
    )
    .await
    .unwrap();
    assert_eq!(report.primary_error, Some(Error::LeaseUnconfirmed));
    assert_eq!(report.recovery.unwrap().state, JobState::AcquireIntent);
    assert!(
        !server
            .requests()
            .iter()
            .any(|r| r.contains("<HttpNfcLeaseAbort "))
    );
}

#[tokio::test]
async fn failed_lease_record_never_authorizes_abort() {
    let out = Output::new();
    let store = store(&out);
    let path = out.0.join("jobs");
    let observer = Arc::new(move |request: &str| {
        if request.contains("<ExportVm ") {
            let name = format!("txn-{}.json", "2a".repeat(16));
            std::fs::write(path.join(name), b"synthetic competing transaction").unwrap();
        }
    });
    let mut replies = replies();
    replies.truncate(11);
    replies.push(logout());
    let server = Server::start_observed(replies, true, Some(observer));
    let report = probe_owned_export(
        explicit_source(&server),
        &credentials(),
        store,
        artifact(),
        Cancellation::default(),
    )
    .await
    .unwrap();
    assert_eq!(report.journal_error, Some(OwnershipError::Uncertain));
    assert_eq!(report.lease_cleanup, LeaseCleanup::Unconfirmed);
    let recovery = report.recovery.unwrap();
    assert_eq!(recovery.state, JobState::AcquireIntent);
    assert!(recovery.pending_transaction);
    assert!(
        !server
            .requests()
            .iter()
            .any(|r| r.contains("<HttpNfcLeaseAbort "))
    );
}

#[tokio::test]
async fn cancellation_after_acquire_still_persists_abort_intent() {
    let out = Output::new();
    let store = store(&out);
    let cancel = Cancellation::default();
    let trigger = cancel.clone();
    let observed = state_observer(out.0.join("jobs"), Arc::new(Mutex::new(Vec::new())));
    let observer = Arc::new(move |request: &str| {
        observed(request);
        if request.contains("<ExportVm ") {
            trigger.cancel();
        }
    });
    let mut replies = replies();
    replies.remove(10); // cancelled before readiness retrieval
    let server = Server::start_observed(replies, true, Some(observer));
    let report = probe_owned_export(
        explicit_source(&server),
        &credentials(),
        store,
        artifact(),
        cancel,
    )
    .await
    .unwrap();
    assert_eq!(report.primary_error, Some(Error::Cancelled));
    assert_eq!(report.lease_cleanup, LeaseCleanup::Aborted);
    assert_eq!(report.recovery.unwrap().state, JobState::AbortedLease);
}

#[tokio::test]
async fn fresh_identity_change_after_intent_blocks_acquisition() {
    let out = Output::new();
    let store = store(&out);
    let mut replies = replies();
    replies[8].body = replies[8].body.replace("PRIVATE-path", "replacement");
    replies.truncate(9);
    replies.push(logout());
    let server = Server::start(replies);
    let report = probe_owned_export(
        explicit_source(&server),
        &credentials(),
        store,
        artifact(),
        Cancellation::default(),
    )
    .await
    .unwrap();
    assert_eq!(report.primary_error, Some(Error::Identity));
    assert_eq!(report.lease_cleanup, LeaseCleanup::NotAcquired);
    assert_eq!(report.recovery.unwrap().state, JobState::AcquireIntent);
    assert!(!server.requests().iter().any(|r| r.contains("<ExportVm ")));
}

#[tokio::test]
async fn store_id_reuse_fails_before_any_connection() {
    let out = Output::new();
    let mut store = store(&out);
    let server = Server::start(vec![]);
    let source = explicit_source(&server);
    drop(store.create(artifact(), &source).unwrap());
    let report = probe_owned_export(
        source,
        &credentials(),
        store,
        artifact(),
        Cancellation::default(),
    )
    .await
    .unwrap();
    assert_eq!(report.journal_error, Some(OwnershipError::Exists));
    assert!(report.recovery.is_none());
    assert!(server.requests().is_empty());
}

#[tokio::test]
async fn lost_rpc_response_retains_intent_and_never_retries() {
    for phase in [9, 11] {
        let out = Output::new();
        let store = store(&out);
        let mut replies = replies();
        replies[phase].declared = Some(replies[phase].body.len() + 17);
        replies.truncate(phase + 1);
        replies.push(logout());
        let server = Server::start(replies);
        let report = probe_owned_export(
            explicit_source(&server),
            &credentials(),
            store,
            artifact(),
            Cancellation::default(),
        )
        .await
        .unwrap();
        assert!(report.primary_error.is_some());
        assert_eq!(report.lease_cleanup, LeaseCleanup::Unconfirmed);
        assert_eq!(
            report.recovery.unwrap().state,
            if phase == 9 {
                JobState::AcquireIntent
            } else {
                JobState::AbortIntent
            }
        );
        assert_eq!(
            server
                .requests()
                .iter()
                .filter(|r| r.contains("<ExportVm "))
                .count(),
            1
        );
        assert_eq!(
            server
                .requests()
                .iter()
                .filter(|r| r.contains("<HttpNfcLeaseAbort "))
                .count(),
            usize::from(phase == 11)
        );
    }
}

#[tokio::test]
async fn owned_probe_crash_child() {
    let Ok(config) = std::env::var("RVVDK_OWNED_PROBE_CHILD") else {
        return;
    };
    let config: Vec<String> = serde_json::from_str(&config).unwrap();
    use rvvdk_vsphere::contract::{EndpointIdentity, PinProvenance, SourceSelection};
    let source = SourceSelection::new(
        EndpointIdentity::pinned(&config[0], &config[1], PinProvenance::TrustOnFirstUse).unwrap(),
        "vm-private",
        "01234567-89ab-cdef-0123-456789abcdef",
        2000,
        "PRIVATE-path",
        30 << 30,
    )
    .unwrap();
    let store = JobStore::open(std::path::Path::new(&config[2])).unwrap();
    let _ = probe_owned_export(
        source,
        &credentials(),
        store,
        artifact(),
        Cancellation::default(),
    )
    .await;
    panic!("child should be killed at the RPC boundary");
}

#[test]
fn process_loss_at_rpc_boundaries_leaves_durable_intent_without_recovered_capability() {
    use std::os::unix::process::ExitStatusExt;
    for (method, state) in [
        ("<ExportVm ", JobState::AcquireIntent),
        ("<HttpNfcLeaseAbort ", JobState::AbortIntent),
    ] {
        let out = Output::new();
        drop(store(&out));
        let path = out.0.join("jobs");
        let pid = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let child_pid = pid.clone();
        let observer = Arc::new(move |body: &str| {
            if body.contains(method) {
                let pid = child_pid.load(Ordering::Acquire);
                assert_ne!(pid, 0);
                // SAFETY: pid belongs exclusively to this live, unreaped test child.
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
                "export_tests::owned_probe::owned_probe_crash_child",
                "--nocapture",
            ])
            .env("RVVDK_OWNED_PROBE_CHILD", config)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        pid.store(child.id(), Ordering::Release);
        let status = child.wait().unwrap();
        assert_eq!(status.signal(), Some(libc::SIGKILL));
        let mut store = JobStore::open(&path).unwrap();
        let source = explicit_source(&server);
        let recovery = store.recover(artifact(), &source).unwrap();
        assert_eq!(recovery.state, state);
        assert_eq!(recovery.action, RecoveryAction::RemoteOutcomeUnknown);
        assert!(!recovery.pending_transaction);
        assert_eq!(
            store.cleanup_local(artifact(), recovery.operation, &source),
            Err(OwnershipError::Transition)
        );
    }
}

#[tokio::test]
async fn failed_abort_ack_retains_remote_success_but_blocks_local_cleanup() {
    let out = Output::new();
    let store = store(&out);
    let path = out.0.join("jobs");
    let observer = Arc::new(move |request: &str| {
        if request.contains("<HttpNfcLeaseAbort ") {
            std::fs::write(
                path.join(format!("txn-{}.json", "2a".repeat(16))),
                b"synthetic interruption",
            )
            .unwrap();
        }
    });
    let server = Server::start_observed(replies(), true, Some(observer));
    let source = explicit_source(&server);
    let report = probe_owned_export(
        source.clone(),
        &credentials(),
        store,
        artifact(),
        Cancellation::default(),
    )
    .await
    .unwrap();
    assert!(!report.is_success());
    assert_eq!(report.lease_cleanup, LeaseCleanup::Aborted);
    assert_eq!(report.journal_error, Some(OwnershipError::Uncertain));
    let recovery = report.recovery.unwrap();
    assert_eq!(recovery.state, JobState::AbortIntent);
    assert!(recovery.pending_transaction);
    assert!(
        JobStore::open(&out.0.join("jobs"))
            .unwrap()
            .cleanup_local(artifact(), recovery.operation, &source)
            .is_err()
    );
}

#[tokio::test]
async fn cancellation_after_acquire_intent_preserves_conservative_record_without_rpc() {
    let out = Output::new();
    let store = store(&out);
    let cancellation = Cancellation::default();
    let trigger = cancellation.clone();
    let reads = std::sync::atomic::AtomicUsize::new(0);
    let observer = Arc::new(move |request: &str| {
        if request.contains("<RetrievePropertiesEx ")
            && request.contains(">vm-private</obj>")
            && reads.fetch_add(1, Ordering::Relaxed) == 2
        {
            trigger.cancel();
        }
    });
    let mut replies = replies();
    replies.truncate(9);
    replies.push(logout());
    let server = Server::start_observed(replies, true, Some(observer));
    let report = probe_owned_export(
        explicit_source(&server),
        &credentials(),
        store,
        artifact(),
        cancellation,
    )
    .await
    .unwrap();
    assert_eq!(report.primary_error, Some(Error::Cancelled));
    assert_eq!(report.lease_cleanup, LeaseCleanup::NotAcquired);
    assert_eq!(report.recovery.unwrap().state, JobState::AcquireIntent);
    assert!(!server.requests().iter().any(|r| r.contains("<ExportVm ")));
}

mod benchmark;
