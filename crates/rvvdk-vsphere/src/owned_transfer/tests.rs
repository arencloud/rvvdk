use super::*;
use crate::artifact_fixture as fixture;
#[allow(dead_code, unused_imports)]
mod mock {
    use crate as rvvdk_vsphere;
    include!("../../tests/support/mock_tls.rs");
    pub(super) async fn delayed(command: &str, cancel: bool, fail: bool) {
        use crate::{
            OwnedTransferOptions,
            contract::{ArtifactId, EndpointIdentity, PinProvenance, SourceSelection},
            journal_worker::{Command, Worker},
            ownership::{JobState, JobStore, OwnershipError},
        };
        use std::os::unix::fs::DirBuilderExt;
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rvddk-transfer-slow-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .unwrap();
        let payload = if command == "admit" {
            super::fixture::image(
                true,
                (30u64 << 30) / 65536,
                &[(0, super::fixture::stored(&super::fixture::bytes(0)))],
            )
        } else {
            ("KDMV".to_owned() + &"x".repeat((1 << 20) - 4)).into_bytes()
        };
        let size = payload.len();
        let digest = Sha256::digest(&payload)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let mut r = full();
        r.pop();
        r[6].body = r[6]
            .body
            .replace("poweredOn", "poweredOff")
            .replace("<string>ExportVm</string>", "");
        let vm = r[6].clone();
        r.extend([
            vm.clone(),
            vm.clone(),
            Reply::soap(
                "ExportVm",
                "<returnval type='HttpNfcLease'>lease-private</returnval>",
            ),
        ]);
        r.push(properties("HttpNfcLease","lease-private",&(property("state","ready")+&property("info","<lease type='HttpNfcLease'>lease-private</lease><entity type='VirtualMachine'>vm-private</entity><deviceUrl><key>disk</key><disk>true</disk><url>ENDPOINT/nfc/private</url><sslThumbprint>THUMBPRINT</sslThumbprint></deviceUrl><totalDiskCapacityInKB>31457280</totalDiskCapacityInKB><leaseTimeout>3</leaseTimeout>"))));
        r.push(vm.clone());
        let mut data = Reply::soap("unused", "");
        data.binary = Some(payload.clone());
        r.push(data);
        if command != "write" || (!cancel && !fail) {
            r.push(Reply::soap("HttpNfcLeaseGetManifest",&format!("<returnval><key>disk</key><disk>true</disk><size>{size}</size><capacity>32212254720</capacity><checksumType>sha256</checksumType><checksum>{digest}</checksum></returnval>")));
        }
        if command == "complete" || (!cancel && !fail) {
            r.push(vm.clone());
        }
        if !cancel && !fail {
            r.extend([vm, Reply::soap("HttpNfcLeaseComplete", "")]);
        } else if command != "complete" {
            r.push(Reply::soap("HttpNfcLeaseAbort", ""));
        }
        r.push(logout());
        let progress = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let counted = progress.clone();
        let observer = Arc::new(move |body: &str| {
            if body.contains("<HttpNfcLeaseProgress ") {
                counted.fetch_add(1, Ordering::Relaxed);
            }
        });
        let server = Server::start_with_idle(r, true, Some(observer), Duration::from_secs(3));
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
        let options = OwnedTransferOptions {
            max_encoded_bytes: 2 << 20,
            ..Default::default()
        };
        let trigger = options.cancellation.clone();
        let which = command.to_owned();
        let worker = Worker::start_hook(
            JobStore::open(&path).unwrap(),
            ArtifactId::new([1; 16]).unwrap(),
            source.clone(),
            move |cmd| {
                if matches!(
                    (which.as_str(), cmd),
                    ("write", Command::PayloadWrite(_))
                        | ("seal", Command::PayloadSeal(_))
                        | ("complete", Command::Complete)
                        | ("admit", Command::PayloadAdmit(..))
                ) {
                    if cancel {
                        trigger.cancel();
                    }
                    std::thread::sleep(Duration::from_millis(1300));
                    if fail {
                        return Err(OwnershipError::Io);
                    }
                }
                Ok(())
            },
        );
        let report = super::transfer_with_worker(
            source,
            &credentials(),
            options,
            worker,
            command == "admit",
        )
        .await
        .unwrap();
        assert!(JobStore::open(&path).is_ok());
        assert!(
            progress.load(Ordering::Relaxed) >= 2,
            "heartbeat must progress during worker delay"
        );
        let state = report.lifecycle.recovery.as_ref().unwrap().state;
        if cancel {
            assert_eq!(
                report.lifecycle.primary_error,
                Some(Error::Cancelled),
                "{report:?}"
            );
            assert_eq!(
                state,
                if command == "complete" {
                    JobState::CompleteIntent
                } else {
                    JobState::AbortedLease
                }
            );
        } else if fail {
            assert_eq!(report.payload_error, Some(OwnershipError::Io));
            assert_eq!(state, JobState::AbortedLease);
        } else {
            assert!(report.is_success(), "{report:?}");
            if command == "admit" {
                assert!(report.is_artifact_success());
            }
        }
        let stages: Vec<_> = std::fs::read_dir(&path)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_dir())
            .collect();
        if !fail {
            assert_eq!(
                std::fs::read(stages[0].join("disk-1.vmdk")).unwrap(),
                payload.as_slice()
            );
            assert_eq!(report.written_encoded_bytes, size as u64);
        }
        if command == "write" && (cancel || fail) {
            assert_eq!(report.durable_encoded_bytes, 0);
        }
        if command == "complete" && cancel {
            assert!(
                !server
                    .requests()
                    .iter()
                    .any(|r| r.contains("<HttpNfcLeaseAbort ")
                        || r.contains("<HttpNfcLeaseComplete "))
            );
        }
        std::fs::remove_dir_all(path).unwrap();
    }
}
#[tokio::test]
async fn cancellation_drains_pending_write_before_abort_and_return() {
    mock::delayed("write", true, false).await;
}
#[tokio::test]
async fn slow_readback_and_completion_intent_keep_heartbeats_alive() {
    for command in ["seal", "complete"] {
        mock::delayed(command, false, false).await;
    }
}
#[tokio::test]
async fn cancellation_during_seal_or_complete_intent_obeys_durable_boundary() {
    for command in ["seal", "complete"] {
        mock::delayed(command, true, false).await;
    }
}
#[tokio::test]
async fn payload_write_failure_still_allows_durable_abort_after_writer_closes() {
    mock::delayed("write", false, true).await;
}

#[tokio::test]
async fn slow_native_admission_keeps_heartbeats_and_drains_on_cancel_or_error() {
    mock::delayed("admit", false, false).await;
    mock::delayed("admit", true, false).await;
    mock::delayed("admit", false, true).await;
}
