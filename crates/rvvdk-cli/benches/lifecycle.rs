use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use rvvdk_datamover::CancellationToken;
use std::{
    fs,
    io::Write,
    time::{Duration, Instant},
};
struct Capture {
    bytes: Vec<u8>,
    token: CancellationToken,
    cancel: bool,
}
impl Write for Capture {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.bytes.extend_from_slice(bytes);
        if self.cancel
            && self
                .bytes
                .windows(b"\"phase\":\"copying\"".len())
                .any(|w| w == b"\"phase\":\"copying\"")
        {
            self.token.cancel();
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn lifecycle(c: &mut Criterion) {
    let dir = std::env::var_os("RVVDK_BENCH_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(format!("rvddk-lifecycle-{}", std::process::id()));
    fs::create_dir(&dir).unwrap();
    let source = dir.join("source");
    let dest = dir.join("destination");
    let mut seed = 0x52563333u64;
    let expected: Vec<_> = (0..16 * 1024 * 1024)
        .map(|_| {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed as u8
        })
        .collect();
    let mut file = fs::File::create(&source).unwrap();
    file.write_all(&expected).unwrap();
    file.sync_all().unwrap();
    let mut group = c.benchmark_group("cli_lifecycle");
    group.sampling_mode(SamplingMode::Flat);
    for (name, backend, cancel) in [
        ("threaded_progress", "threaded", false),
        ("native_progress", "io-uring", false),
        ("threaded_cancel", "threaded", true),
        ("native_cancel", "io-uring", true),
    ] {
        let args: Vec<std::ffi::OsString> = vec![
            "rvddk".into(),
            "copy".into(),
            source.as_os_str().into(),
            dest.as_os_str().into(),
            "--format".into(),
            "raw".into(),
            "--json".into(),
            "--verify".into(),
            "--progress".into(),
            "--backend".into(),
            backend.into(),
            "--workers".into(),
            "4".into(),
            "--block-size".into(),
            "65536".into(),
        ];
        group.bench_function(name, |b| {
            b.iter_custom(|n| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..n {
                    let token = CancellationToken::new();
                    let mut capture = Capture {
                        bytes: Vec::new(),
                        token: token.clone(),
                        cancel,
                    };
                    let mut out = Vec::new();
                    let started = Instant::now();
                    let code = rvvdk_cli::run_with_cancellation(
                        args.clone(),
                        &mut out,
                        &mut capture,
                        &token,
                    );
                    elapsed += started.elapsed();
                    let events: Vec<serde_json::Value> = std::str::from_utf8(&capture.bytes)
                        .unwrap()
                        .lines()
                        .map(|l| serde_json::from_str(l).unwrap())
                        .collect();
                    if cancel {
                        assert_eq!(code, 130);
                        assert!(out.is_empty());
                        assert!(!dest.exists());
                        assert_eq!(events.last().unwrap()["error"]["code"], "cancelled");
                        assert!(events.iter().any(|e| e["phase"] == "copying"));
                    } else {
                        assert_eq!(code, 0);
                        let report: serde_json::Value = serde_json::from_slice(&out).unwrap();
                        assert_eq!(report["backend"], backend);
                        assert_eq!(events.last().unwrap()["phase"], "completed");
                        assert_eq!(fs::read(&dest).unwrap(), expected);
                        fs::remove_file(&dest).unwrap();
                    }
                }
                elapsed
            })
        });
    }
    group.finish();
    fs::remove_dir_all(dir).unwrap();
}
criterion_group! {name=benches;config=Criterion::default().sample_size(30).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2));targets=lifecycle}
criterion_main!(benches);
