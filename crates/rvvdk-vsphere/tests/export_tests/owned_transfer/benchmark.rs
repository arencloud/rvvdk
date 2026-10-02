use super::*;
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
fn usage() -> (f64, i64) {
    let mut raw = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: valid storage, initialized before reading on success.
    assert_eq!(
        unsafe { libc::getrusage(libc::RUSAGE_SELF, raw.as_mut_ptr()) },
        0
    );
    let r = unsafe { raw.assume_init() };
    (
        r.ru_utime.tv_sec as f64
            + r.ru_utime.tv_usec as f64 / 1e6
            + r.ru_stime.tv_sec as f64
            + r.ru_stime.tv_usec as f64 / 1e6,
        r.ru_maxrss,
    )
}
#[tokio::test]
#[ignore = "opt-in matched owned transfer benchmark"]
async fn matched_owned_transfer() {
    let count: usize = std::env::var("RVVDK_TRANSFER_PAIRS")
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=64).contains(&count));
    let parent = PathBuf::from(std::env::var_os("RVVDK_TRANSFER_PARENT").unwrap());
    let output = std::env::var_os("RVVDK_TRANSFER_REPORT").unwrap();
    let bytes = 8 << 20;
    let payload = "KDMV".to_owned() + &"x".repeat(bytes - 4);
    let digest = Sha256::digest(payload.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let mut rows = Vec::new();
    for pair in 0..count {
        for owned in if pair % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let mut r = if owned { replies() } else { explicit_replies() };
            r[12].body = payload.clone();
            r[13] = Reply::soap(
                "HttpNfcLeaseGetManifest",
                &format!(
                    "<returnval><key>private-disk</key><disk>true</disk><size>{bytes}</size><capacity>32212254720</capacity><checksumType>sha256</checksumType><checksum>{digest}</checksum></returnval>"
                ),
            );
            let server = Server::start_with_nodelay(r, true);
            let source = explicit_source(&server);
            let credentials = credentials();
            let path = parent.join(format!(
                "transfer-{}-{pair}-{}",
                std::process::id(),
                usize::from(owned)
            ));
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&path)
                .unwrap();
            let store = if owned {
                Some(JobStore::open(&path).unwrap())
            } else {
                None
            };
            let options = OwnedTransferOptions {
                max_encoded_bytes: bytes as u64,
                ..Default::default()
            };
            let mut legacy = ExportOptions::probe(30 << 30);
            legacy.probe_only = false;
            legacy.max_encoded_bytes = bytes as u64;
            legacy.output = path.join("artifact");
            let before = usage();
            let start = std::time::Instant::now();
            let recovery = if let Some(store) = store {
                let report =
                    transfer_owned_export(source.clone(), &credentials, store, artifact(), options)
                        .await
                        .unwrap();
                assert!(report.is_success(), "{report:?}");
                assert_eq!(
                    (
                        report.received_encoded_bytes,
                        report.written_encoded_bytes,
                        report.durable_encoded_bytes
                    ),
                    (bytes as u64, bytes as u64, bytes as u64)
                );
                Some(report.lifecycle.recovery.unwrap())
            } else {
                let report =
                    rvvdk_vsphere::export_selected_vm(source.clone(), &credentials, legacy)
                        .await
                        .unwrap();
                assert!(report.is_success(), "{report:?}");
                None
            };
            let wall_ms = start.elapsed().as_secs_f64() * 1000.;
            let after = usage();
            let file = if owned {
                stage(&path).join("disk-1.vmdk")
            } else {
                path.join("artifact/disk-1.vmdk")
            };
            assert_eq!(std::fs::read(&file).unwrap(), payload.as_bytes());
            let allocated = std::fs::metadata(&file).unwrap().blocks() * 512;
            let mut cleanup_ms = 0.;
            if let Some(recovery) = recovery {
                assert_eq!(recovery.state, JobState::CompletedLease);
                assert_eq!(recovery.sequence, 7);
                let mut store = JobStore::open(&path).unwrap();
                let start = std::time::Instant::now();
                store
                    .cleanup_local(artifact(), recovery.operation, &source)
                    .unwrap();
                cleanup_ms = start.elapsed().as_secs_f64() * 1000.;
                assert_eq!(
                    store.recover(artifact(), &source).unwrap().state,
                    JobState::Cleaned
                );
            }
            std::fs::remove_dir_all(&path).unwrap();
            rows.push(serde_json::json!({"pair":pair,"owned":owned,"wall_ms":wall_ms,"cpu_ms":(after.0-before.0)*1000.,"process_peak_rss_kib":after.1,"cleanup_ms":cleanup_ms,"payload_bytes":bytes,"payload_allocated_bytes":allocated,"journal_commits":if owned{8}else{0},"requests":server.requests().len(),"byte_comparison_passed":true}));
        }
    }
    let mut file = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .unwrap();
    serde_json::to_writer_pretty(&mut file, &rows).unwrap();
    file.write_all(b"\n").unwrap();
    file.sync_all().unwrap();
}
