use super::*;
use rvvdk_core::VirtualDisk;
use rvvdk_vsphere::{OwnedTransferOptions, ownership::JobStore, transfer_owned_export};
use std::os::unix::fs::{DirBuilderExt, MetadataExt};
fn usage() -> (f64, i64) {
    let mut raw = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: valid storage, initialized before reading on success.
    assert_eq!(
        unsafe { libc::getrusage(libc::RUSAGE_SELF, raw.as_mut_ptr()) },
        0
    );
    // SAFETY: getrusage succeeded above.
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
#[ignore = "opt-in matched native artifact admission benchmark"]
async fn matched_owned_artifact() {
    let count: usize = std::env::var("RVVDK_ARTIFACT_PAIRS")
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=64).contains(&count));
    let parent = PathBuf::from(std::env::var_os("RVVDK_ARTIFACT_PARENT").unwrap());
    let output = std::env::var_os("RVVDK_ARTIFACT_REPORT").unwrap();
    let records: Vec<_> = (0..128)
        .map(|i| (i, fixture::stored(&fixture::bytes(i))))
        .collect();
    let payload = fixture::image(true, (30u64 << 30) / 65536, &records);
    let mut rows = Vec::new();
    for pair in 0..count {
        for admitted in if pair % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let server = Server::start_with_nodelay(replies(&payload), true);
            let source = explicit_source(&server);
            let credentials = credentials();
            let path = parent.join(format!(
                "artifact-{}-{pair}-{}",
                std::process::id(),
                usize::from(admitted)
            ));
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&path)
                .unwrap();
            let store = JobStore::open(&path).unwrap();
            let options = OwnedTransferOptions {
                max_encoded_bytes: payload.len() as u64,
                ..Default::default()
            };
            let before = usage();
            let start = std::time::Instant::now();
            let report = if admitted {
                transfer_owned_artifact(source.clone(), &credentials, store, artifact(), options)
                    .await
                    .unwrap()
            } else {
                transfer_owned_export(source.clone(), &credentials, store, artifact(), options)
                    .await
                    .unwrap()
            };
            let wall_ms = start.elapsed().as_secs_f64() * 1000.;
            let after = usage();
            assert!(report.is_success(), "{report:?}");
            assert_eq!(report.is_artifact_success(), admitted);
            assert_eq!(
                report.present_grains_verified,
                if admitted { 128 } else { 0 }
            );
            assert_eq!(
                (
                    report.received_encoded_bytes,
                    report.written_encoded_bytes,
                    report.durable_encoded_bytes
                ),
                (
                    payload.len() as u64,
                    payload.len() as u64,
                    payload.len() as u64
                )
            );
            let dir = stage(&path);
            let file = dir.join("disk-1.vmdk");
            assert_eq!(std::fs::read(&file).unwrap(), payload);
            let allocated = std::fs::metadata(&file).unwrap().blocks() * 512;
            let bytes = std::fs::read(dir.join("manifest.json")).unwrap();
            if admitted {
                let m = ExportArtifact::from_json(&bytes).unwrap();
                m.check_source(&source).unwrap();
                m.check_container(payload.len() as u64, &Sha256::digest(&payload).into())
                    .unwrap();
                assert_eq!(
                    m.validation_claim(),
                    ValidationClaim::ContainerDigestVerified
                );
            } else {
                assert!(bytes.is_empty());
            }
            // Outside the timer: both outputs must decode to the independent authored
            // fixture bytes. Also sample the structurally absent trailing zero region.
            let disk = rvvdk_vmdk::StreamDisk::load(
                Arc::new(rvvdk_local::LocalFileBlockDevice::open_read_only(&file).unwrap()),
                Default::default(),
            )
            .unwrap();
            let mut buffer = vec![0; 65536];
            for i in 0..128 {
                disk.read_exact_at(i * 65536, &mut buffer).unwrap();
                assert_eq!(buffer, fixture::bytes(i));
            }
            for at in [8 << 20, (30u64 << 30) - 65536] {
                disk.read_exact_at(at, &mut buffer).unwrap();
                assert!(buffer.iter().all(|&v| v == 0));
            }
            drop(disk);
            let recovery = report.lifecycle.recovery.unwrap();
            assert_eq!(recovery.state, JobState::CompletedLease);
            assert_eq!(recovery.sequence, 7);
            let mut store = JobStore::open(&path).unwrap();
            let start = std::time::Instant::now();
            store
                .cleanup_local(artifact(), recovery.operation, &source)
                .unwrap();
            let cleanup_ms = start.elapsed().as_secs_f64() * 1000.;
            assert_eq!(
                store.recover(artifact(), &source).unwrap().state,
                JobState::Cleaned
            );
            drop(store);
            std::fs::remove_dir_all(&path).unwrap();
            rows.push(serde_json::json!({"pair":pair,"admitted":admitted,"wall_ms":wall_ms,"cpu_ms":(after.0-before.0)*1000.,"process_peak_rss_kib":after.1,"cleanup_ms":cleanup_ms,"payload_bytes":payload.len(),"payload_allocated_bytes":allocated,"metadata_bytes":bytes.len(),"journal_commits":8,"requests":server.requests().len(),"byte_comparison_passed":true,"decoded_oracle_passed":true,"present_grains_verified":report.present_grains_verified}));
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
