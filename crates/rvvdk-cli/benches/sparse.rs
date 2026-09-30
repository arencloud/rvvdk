#[path = "../../rvvdk-vmdk/tests/support/sparse_metadata.rs"]
#[allow(dead_code)]
mod sparse;
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use std::{
    fs,
    time::{Duration, Instant},
};
fn sparse_cli(c: &mut Criterion) {
    let dir = std::env::var_os("RVVDK_BENCH_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let root = dir.join(format!("rvddk-sparse-bench-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let text = sparse::descriptor("monolithicSparse", "RW 2048 SPARSE \"mono.vmdk\"");
    fs::write(root.join("mono.vmdk"), sparse::bytes(Some(&text))).unwrap();
    fs::write(
        root.join("split.vmdk"),
        sparse::descriptor(
            "twoGbMaxExtentSparse",
            "RW 2048 SPARSE \"a\"\nRW 2048 SPARSE \"b\"",
        ),
    )
    .unwrap();
    fs::write(root.join("a"), sparse::bytes(None)).unwrap();
    fs::write(root.join("b"), sparse::bytes(None)).unwrap();
    let mut mono = vec![0; 1048576];
    mono[..65536].fill(0x5a);
    mono[65536..131072].fill(0xa5);
    let split = mono.repeat(2);
    let destination = root.join("output.raw");
    let mut group = c.benchmark_group("cli_sparse");
    group.sampling_mode(SamplingMode::Flat);
    for (name, command, source, expected) in [
        ("inspect_mono", "inspect", "mono.vmdk", &mono),
        ("plan_split", "plan", "split.vmdk", &split),
        ("copy_mono_verify", "copy", "mono.vmdk", &mono),
        ("copy_split_verify", "copy", "split.vmdk", &split),
        ("verify_split", "verify", "split.vmdk", &split),
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
        if command == "copy" {
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
criterion_group! {name=benches; config=Criterion::default().sample_size(30).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2)); targets=sparse_cli}
criterion_main!(benches);
