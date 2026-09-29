use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use std::{
    fs,
    io::Write,
    time::{Duration, Instant},
};
fn transfer(c: &mut Criterion) {
    let dir = std::env::var_os("RVVDK_BENCH_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let root = dir.join(format!("rvddk-transfer-bench-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let source = root.join("source");
    let destination = root.join("destination");
    let mut state = 0x52563332_u64;
    let expected: Vec<u8> = (0..16 * 1024 * 1024)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8
        })
        .collect();
    let mut file = fs::File::create(&source).unwrap();
    file.write_all(&expected).unwrap();
    file.sync_all().unwrap();
    let mut group = c.benchmark_group("cli_transfer");
    group.sampling_mode(SamplingMode::Flat);
    for (name, command, backend, verify) in [
        ("threaded_new", "copy", "threaded", false),
        ("threaded_new_verify", "copy", "threaded", true),
        ("native_new_verify", "copy", "io-uring", true),
        ("verify_only", "verify", "threaded", false),
    ] {
        let mut args = vec![
            "rvddk".into(),
            command.into(),
            source.as_os_str().to_owned(),
            destination.as_os_str().to_owned(),
            "--format".into(),
            "raw".into(),
            "--json".into(),
            "--block-size".into(),
            "65536".into(),
        ];
        if command == "copy" {
            args.extend([
                "--backend".into(),
                backend.into(),
                "--workers".into(),
                "4".into(),
            ]);
        } else {
            fs::copy(&source, &destination).unwrap();
        }
        if verify {
            args.push("--verify".into());
        }
        let mut output = Vec::new();
        assert_eq!(
            rvvdk_cli::run(args.clone(), &mut output, &mut std::io::sink()),
            0
        );
        let report: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(report["logical_bytes"], expected.len());
        if command == "copy" {
            assert_eq!(report["backend"], backend);
            fs::remove_file(&destination).unwrap();
        }
        group.bench_function(name, |b| {
            b.iter_custom(|n| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..n {
                    let started = Instant::now();
                    let status =
                        rvvdk_cli::run(args.clone(), &mut std::io::sink(), &mut std::io::sink());
                    elapsed += started.elapsed();
                    assert_eq!(status, 0);
                    assert_eq!(fs::read(&destination).unwrap(), expected);
                    if command == "copy" {
                        fs::remove_file(&destination).unwrap();
                    }
                }
                elapsed
            })
        });
        if command == "verify" {
            fs::remove_file(&destination).unwrap();
        }
    }
    group.finish();
    fs::remove_dir_all(root).unwrap();
}
criterion_group! { name=benches; config=Criterion::default().sample_size(30).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2)); targets=transfer }
criterion_main!(benches);
