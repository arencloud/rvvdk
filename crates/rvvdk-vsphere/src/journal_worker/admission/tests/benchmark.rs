//! Opt-in worker phase timings. Fixture construction and independent oracles are
//! outside each timer; seal still includes sync and fresh full-byte readback.
use super::*;
use std::os::unix::fs::MetadataExt;

fn usage() -> (f64, i64) {
    let mut raw = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: valid writable storage; read only after successful initialization.
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

#[test]
#[ignore = "opt-in transfer hash worker phase benchmark"]
fn worker_hash_phases() {
    let count: usize = std::env::var("RVVDK_WORKER_PAIRS")
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=64).contains(&count));
    let parent = PathBuf::from(std::env::var_os("RVVDK_WORKER_PARENT").unwrap());
    let output = std::env::var_os("RVVDK_WORKER_REPORT").unwrap();
    let case = std::env::var("RVVDK_HASH_CASE").unwrap();
    assert!(["small", "bulk", "zeros"].contains(&case.as_str()));
    let grains = if case == "small" { 1 } else { 128 };
    let records: Vec<_> = (0..grains)
        .map(|i| {
            let bytes = if case == "zeros" {
                vec![0; 65536]
            } else {
                fixture::bytes(i)
            };
            (i, fixture::stored(&bytes))
        })
        .collect();
    let data = fixture::image(true, (30u64 << 30) / 65536, &records);
    // RustCrypto is the independent oracle in both reference and candidate.
    let expected = Expected {
        bytes: data.len() as u64,
        sha256: sha2::Sha256::digest(&data).into(),
        sha1: Sha1::digest(&data).into(),
    };
    let source = source(30 << 30);
    let mut rows = Vec::new();
    for pair in 0..count {
        let path = parent.join(format!("worker-{}-{pair}", std::process::id()));
        fs::DirBuilder::new().mode(0o700).create(&path).unwrap();
        let out = Temp(path);
        let mut store = JobStore::open(&out.0).unwrap();
        let mut job = store.create(id(), &source).unwrap();
        job.prepare_stage().unwrap();
        job.begin_acquire().unwrap();
        job.acknowledge_lease("synthetic").unwrap();
        let mut payload = Payload::default();
        payload.open(&job, data.len() as u64).unwrap();
        for chunk in data.chunks(super::super::super::CHUNK_BYTES) {
            payload.write(chunk).unwrap();
        }
        for phase in ["seal", "admit"] {
            let before = usage();
            let start = Instant::now();
            if phase == "seal" {
                payload.seal(&job, &expected).unwrap();
            } else {
                payload
                    .admit(&job, id(), &source, &Cancellation::default(), deadline())
                    .unwrap();
            }
            let wall_ms = start.elapsed().as_secs_f64() * 1000.;
            let after = usage();
            assert!(payload.progress.verified);
            assert_eq!(payload.progress.durable, data.len() as u64);
            assert_eq!(payload.progress.native_verified, phase == "admit");
            assert_eq!(payload.progress.metadata_durable, phase == "admit");
            let mut reader = job.payload_reader().unwrap();
            let allocated = reader.metadata().unwrap().blocks() * 512;
            let mut observed = Vec::new();
            reader.read_to_end(&mut observed).unwrap();
            assert_eq!(observed, data);
            if phase == "admit" {
                assert_eq!(payload.progress.grains_verified, grains);
                let mut metadata = Vec::new();
                job.metadata_file()
                    .unwrap()
                    .read_to_end(&mut metadata)
                    .unwrap();
                let metadata = ExportArtifact::from_json(&metadata).unwrap();
                metadata.check_source(&source).unwrap();
                metadata
                    .check_container(expected.bytes, &expected.sha256)
                    .unwrap();
            }
            rows.push(serde_json::json!({"case":case,"pair":pair,"phase":phase,
                "wall_ms":wall_ms,"cpu_ms":(after.0-before.0)*1000.,
                "process_peak_rss_kib":after.1,"payload_bytes":data.len(),
                "payload_allocated_bytes":allocated,"byte_comparison_passed":true}));
        }
        // These authored local fixtures never acquire a remote lease.
        drop(job);
        drop(store);
        drop(out);
    }
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .unwrap();
    serde_json::to_writer_pretty(&mut file, &rows).unwrap();
    file.write_all(b"\n").unwrap();
    file.sync_all().unwrap();
}
