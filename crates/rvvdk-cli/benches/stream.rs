#[path = "../../rvvdk-vmdk/tests/support/stream_disk.rs"]
mod stream;
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use std::{
    fs,
    time::{Duration, Instant},
};
fn stream_cli(c: &mut Criterion) {
    let dir = std::env::var_os("RVVDK_BENCH_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let root = dir.join(format!("rvddk-stream-bench-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let payload = include_bytes!("../../rvvdk-vmdk/tests/fixtures/stream/pattern.zlib").to_vec();
    let mut mono = vec![0; 4 * 1048576];
    // pattern.zlib independently encodes stream::bytes(1).
    let records: Vec<_> = (0..64).map(|i| (i, payload.clone())).collect();
    for chunk in mono.chunks_mut(65536) {
        chunk.copy_from_slice(&stream::bytes(1));
    }
    fs::write(root.join("dense.vmdk"), stream::image(true, 64, &records)).unwrap();
    let mut split = vec![0; 4 * 1048576];
    split[65536..131072].copy_from_slice(&stream::bytes(1));
    fs::write(
        root.join("sparse.vmdk"),
        stream::image(true, 64, &[(1, payload)]),
    )
    .unwrap();
    let destination = root.join("output.raw");
    let mut group = c.benchmark_group("cli_stream");
    group.sampling_mode(SamplingMode::Flat);
    for (name, command, source, expected) in [
        ("inspect_dense", "inspect", "dense.vmdk", &mono),
        ("plan_sparse", "plan", "sparse.vmdk", &split),
        ("copy_dense", "copy", "dense.vmdk", &mono),
        ("copy_dense_verify", "copy", "dense.vmdk", &mono),
        ("copy_sparse_verify", "copy", "sparse.vmdk", &split),
        ("verify_dense", "verify", "dense.vmdk", &mono),
    ] {
        let mut args = vec![
            "rvddk".into(),
            command.into(),
            root.join(source).into_os_string(),
        ];
        if command != "inspect" {
            args.push(destination.clone().into_os_string());
        }
        args.extend(["--format".into(), "vmdk".into(), "--json".into()]);
        if command != "inspect" {
            args.extend(["--block-size".into(), "65536".into()]);
        }
        if command == "plan" || command == "copy" {
            args.extend([
                "--backend".into(),
                "auto".into(),
                "--workers".into(),
                "4".into(),
            ]);
        }
        if command == "copy" && name.ends_with("verify") {
            args.push("--verify".into());
        }
        if command == "verify" {
            fs::write(&destination, expected).unwrap();
        }
        let mut output = Vec::new();
        let mut errors = Vec::new();
        assert_eq!(
            rvvdk_cli::run(args.clone(), &mut output, &mut errors),
            0,
            "{}",
            String::from_utf8_lossy(&errors)
        );
        let report: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(report["format"], "vmdk");
        if command == "copy" {
            assert_eq!(report["backend"], "threaded");
            assert_eq!(fs::read(&destination).unwrap(), *expected);
            fs::remove_file(&destination).unwrap();
        }
        group.bench_function(name, |b| {
            b.iter_custom(|n| {
                let mut elapsed = Duration::ZERO;
                for _ in 0..n {
                    let started = Instant::now();
                    let code =
                        rvvdk_cli::run(args.clone(), &mut std::io::sink(), &mut std::io::sink());
                    elapsed += started.elapsed();
                    assert_eq!(code, 0);
                    if command == "copy" {
                        assert_eq!(fs::read(&destination).unwrap(), *expected);
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
criterion_group! {name=benches; config=Criterion::default().sample_size(30).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2)); targets=stream_cli}
criterion_main!(benches);
