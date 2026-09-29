use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use std::{
    fs,
    time::{Duration, Instant},
};
fn vmdk(c: &mut Criterion) {
    let dir = std::env::var_os("RVVDK_BENCH_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let root = dir.join(format!("rvddk-vmdk-bench-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let bytes: Vec<_> = (0..1024 * 1024).map(|n| (n * 29 + n / 251) as u8).collect();
    fs::write(root.join("backing"), &bytes).unwrap();
    let header = "version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"custom\"\n";
    fs::write(
        root.join("flat.vmdk"),
        format!("{header}RW 2048 FLAT \"backing\" 0\n"),
    )
    .unwrap();
    let mut text = header.to_owned();
    let mut mixed = Vec::new();
    for i in 0..64 {
        if i % 2 == 0 {
            text.push_str(&format!("RW 32 FLAT \"backing\" {}\n", i * 32));
            mixed.extend_from_slice(&bytes[i * 16384..(i + 1) * 16384]);
        } else {
            text.push_str("RW 32 ZERO\n");
            mixed.extend([0; 16384]);
        }
    }
    fs::write(root.join("mixed.vmdk"), text).unwrap();
    let destination = root.join("output.raw");
    let mut group = c.benchmark_group("cli_vmdk");
    group.sampling_mode(SamplingMode::Flat);
    for (name, command, source, expected) in [
        ("inspect_mixed", "inspect", "mixed.vmdk", &mixed),
        ("plan_mixed", "plan", "mixed.vmdk", &mixed),
        ("copy_flat_verify", "copy", "flat.vmdk", &bytes),
        ("copy_mixed_verify", "copy", "mixed.vmdk", &mixed),
        ("verify_mixed", "verify", "mixed.vmdk", &mixed),
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
criterion_group! {name=benches; config=Criterion::default().sample_size(30).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2)); targets=vmdk}
criterion_main!(benches);
