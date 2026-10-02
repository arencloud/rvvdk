use super::*;
use std::os::unix::fs::MetadataExt;
fn usage() -> (f64, i64) {
    let mut raw = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: writable storage, initialized before reading on success.
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
#[ignore = "opt-in matched journal/probe performance matrix"]
async fn matched_owned_probe() {
    let count: usize = std::env::var("RVVDK_PROBE_PAIRS").unwrap().parse().unwrap();
    assert!((1..=128).contains(&count));
    let parent = PathBuf::from(std::env::var_os("RVVDK_PROBE_PARENT").unwrap());
    let output = std::env::var_os("RVVDK_PROBE_REPORT").unwrap();
    let mut rows = Vec::new();
    for pair in 0..count {
        for owned in if pair % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let mut replies = replies();
            if !owned {
                replies.remove(10);
            }
            let server = Server::start_with_nodelay(replies, true);
            let source = explicit_source(&server);
            let credentials = credentials();
            let path = parent.join(format!("fixture-{}-{pair}", std::process::id()));
            if owned {
                std::fs::DirBuilder::new()
                    .mode(0o700)
                    .create(&path)
                    .unwrap();
            }
            let store = if owned {
                Some(JobStore::open(&path).unwrap())
            } else {
                None
            };
            let before = usage();
            let start = std::time::Instant::now();
            let recovery = if let Some(store) = store {
                let report = probe_owned_export(
                    source.clone(),
                    &credentials,
                    store,
                    artifact(),
                    Cancellation::default(),
                )
                .await
                .unwrap();
                assert!(report.is_success(), "{report:?}");
                Some(report.recovery.unwrap())
            } else {
                let report = rvvdk_vsphere::export_selected_vm(
                    source.clone(),
                    &credentials,
                    ExportOptions::probe(30 << 30),
                )
                .await
                .unwrap();
                assert_eq!(report.primary_error, None);
                assert_eq!(report.lease_cleanup, LeaseCleanup::Aborted);
                assert_eq!(report.session_cleanup, Cleanup::LoggedOut);
                None
            };
            let wall_ms = start.elapsed().as_secs_f64() * 1000.;
            let after = usage();
            let mut file_bytes = 0;
            let mut allocated_bytes = 0;
            let mut cleanup_ms = 0.;
            if let Some(recovery) = recovery {
                assert_eq!(recovery.state, JobState::AbortedLease);
                assert_eq!(recovery.sequence, 6);
                for entry in std::fs::read_dir(&path).unwrap() {
                    let p = entry.unwrap().path();
                    let files = if p.is_dir() {
                        std::fs::read_dir(p)
                            .unwrap()
                            .map(|e| e.unwrap().path())
                            .collect()
                    } else {
                        vec![p]
                    };
                    for p in files {
                        let meta = std::fs::metadata(&p).unwrap();
                        file_bytes += meta.len();
                        allocated_bytes += meta.blocks() * 512;
                        if ["disk-1.vmdk", "manifest.json"]
                            .iter()
                            .any(|n| p.file_name().unwrap() == *n)
                        {
                            assert_eq!(meta.len(), 0);
                        }
                    }
                }
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
                drop(store);
                std::fs::remove_dir_all(&path).unwrap();
            }
            assert!(
                !server
                    .requests()
                    .iter()
                    .any(|r| r.starts_with("GET ") || r.contains("<HttpNfcLeaseComplete "))
            );
            rows.push(serde_json::json!({"pair":pair,"owned":owned,"wall_ms":wall_ms,"cpu_ms":(after.0-before.0)*1000.,"process_peak_rss_kib":after.1,"cleanup_ms":cleanup_ms,"file_bytes":file_bytes,"allocated_bytes":allocated_bytes,"journal_commits":if owned {7} else {0},"requests":server.requests().len(),"empty_payload_verified":true}));
        }
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    serde_json::to_writer_pretty(&mut file, &rows).unwrap();
    file.write_all(b"\n").unwrap();
    file.sync_all().unwrap();
}
