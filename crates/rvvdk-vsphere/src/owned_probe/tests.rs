use super::*;
#[allow(dead_code, unused_imports)]
mod mock {
    use crate as rvvdk_vsphere;
    include!("../../tests/support/mock_tls.rs");
    pub(super) async fn delayed_journal(command: &str, cancel: bool, fail_progress: bool) {
        use crate::{
            Cancellation,
            contract::{ArtifactId, EndpointIdentity, PinProvenance, SourceSelection},
            journal_worker::{Command, Worker},
            ownership::{JobState, JobStore},
        };
        use std::os::unix::fs::DirBuilderExt;
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rvddk-owned-slow-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .unwrap();
        let mut replies = full();
        replies.pop();
        replies[6].body = replies[6]
            .body
            .replace("poweredOn", "poweredOff")
            .replace("<string>ExportVm</string>", "");
        replies.push(replies[6].clone());
        replies.push(replies[6].clone());
        replies.push(Reply::soap(
            "ExportVm",
            "<returnval type='HttpNfcLease'>lease-private</returnval>",
        ));
        replies.push(properties("HttpNfcLease","lease-private",&(property("state","ready")+&property("info","<lease type='HttpNfcLease'>lease-private</lease><entity type='VirtualMachine'>vm-private</entity><deviceUrl><key>disk</key><disk>true</disk><url>ENDPOINT/nfc/private</url><sslThumbprint>THUMBPRINT</sslThumbprint></deviceUrl><totalDiskCapacityInKB>31457280</totalDiskCapacityInKB><leaseTimeout>3</leaseTimeout>"))));
        replies.extend([Reply::soap("HttpNfcLeaseAbort", ""), logout()]);
        let cancellation = Cancellation::default();
        let trigger = cancellation.clone();
        let progress = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counted = progress.clone();
        let observer = Arc::new(move |body: &str| {
            if body.contains("<HttpNfcLeaseProgress ") {
                counted.fetch_add(1, Ordering::Relaxed);
                if cancel {
                    trigger.cancel();
                }
            }
        });
        let server = Server::start_observed(replies, true, Some(observer));
        let source = SourceSelection::new(
            EndpointIdentity::pinned(
                &server.endpoint,
                &server.pin,
                PinProvenance::TrustOnFirstUse,
            )
            .unwrap(),
            "vm-private",
            "01234567-89ab-cdef-0123-456789abcdef",
            2000,
            "PRIVATE-path",
            30 << 30,
        )
        .unwrap();
        let delayed = command.to_owned();
        let worker = Worker::start_hook(
            JobStore::open(&path).unwrap(),
            ArtifactId::new([1; 16]).unwrap(),
            source.clone(),
            move |command| {
                if matches!(
                    (delayed.as_str(), command),
                    ("held", Command::Held(_)) | ("abort", Command::Abort)
                ) {
                    std::thread::sleep(Duration::from_millis(1300));
                }
                Ok(())
            },
        );
        // A failed progress response is injected by the shared mock's explicit
        // switch; production journal hooks and transport are unchanged.
        if fail_progress {
            server.progress_failure.store(true, Ordering::Relaxed);
        }
        let report = super::probe_with_worker(source, &credentials(), cancellation, worker)
            .await
            .unwrap();
        assert_eq!(
            report.recovery.as_ref().unwrap().state,
            JobState::AbortedLease,
            "{report:?}"
        );
        assert_eq!(report.lease_cleanup, crate::LeaseCleanup::Aborted);
        assert_eq!(report.journal_error, None);
        assert!(
            JobStore::open(&path).is_ok(),
            "worker must release lock before returning"
        );
        if fail_progress {
            assert_eq!(report.primary_error, Some(Error::SoapFault));
        } else if cancel {
            assert_eq!(report.primary_error, Some(Error::Cancelled));
        } else {
            assert!(report.is_success(), "{report:?}");
        }
        assert!(progress.load(Ordering::Relaxed) >= if fail_progress { 1 } else { 2 });
        let reqs = server.requests();
        assert_eq!(
            reqs.iter()
                .filter(|r| r.contains("<HttpNfcLeaseAbort "))
                .count(),
            1
        );
        std::fs::remove_dir_all(path).unwrap();
    }
}
#[tokio::test]
async fn heartbeat_runs_while_held_and_abort_intents_sync() {
    for command in ["held", "abort"] {
        mock::delayed_journal(command, false, false).await;
    }
}
#[tokio::test]
async fn cancellation_keeps_heartbeat_and_drains_accepted_intent_before_abort() {
    mock::delayed_journal("abort", true, false).await;
}
#[tokio::test]
async fn failed_progress_still_drains_intent_before_exactly_one_abort() {
    mock::delayed_journal("abort", false, true).await;
}
