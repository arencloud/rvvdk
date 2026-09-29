use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use std::{
    fs::{self, File},
    io::Write,
    os::unix::fs::FileExt,
    time::Duration,
};

fn preview(c: &mut Criterion) {
    let dir = std::env::var_os("RVVDK_BENCH_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    fs::create_dir_all(&dir).unwrap();
    let root = dir.join(format!("rvddk-cli-bench-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let dense = root.join("dense.raw");
    let fragmented = root.join("fragmented.raw");
    let destination = root.join("absent.raw");
    let mut file = File::create(&dense).unwrap();
    file.write_all(&vec![0x5a; 1024 * 1024]).unwrap();
    file.sync_all().unwrap();
    let file = File::create(&fragmented).unwrap();
    file.set_len(8 * 1024 * 1024).unwrap();
    for offset in (0..8 * 1024 * 1024).step_by(8192) {
        file.write_all_at(&[0xa5; 4096], offset).unwrap();
    }
    file.sync_all().unwrap();
    let mut group = c.benchmark_group("cli_preview");
    group.sampling_mode(SamplingMode::Flat);
    for (name, command, path, extents) in [
        ("inspect_dense_json", "inspect", &dense, false),
        ("plan_new_json", "plan", &dense, false),
        (
            "inspect_fragmented_extents_json",
            "inspect",
            &fragmented,
            true,
        ),
    ] {
        let mut args = vec!["rvddk".into(), command.into(), path.as_os_str().to_owned()];
        if command == "plan" {
            args.push(destination.as_os_str().to_owned());
        }
        args.extend(["--format".into(), "raw".into(), "--json".into()]);
        if extents {
            args.push("--extents".into());
        }
        let mut output = Vec::new();
        assert_eq!(
            rvvdk_cli::run(args.clone(), &mut output, &mut std::io::sink()),
            0
        );
        let report: serde_json::Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(
            report["summary"]["logical_bytes"],
            fs::metadata(path).unwrap().len()
        );
        if extents {
            assert!(
                report["summary"]["extent_count"].as_u64().unwrap() >= 1024,
                "fragmented fixture was not exposed by this filesystem"
            );
        }
        group.bench_function(name, |b| {
            b.iter(|| {
                assert_eq!(
                    rvvdk_cli::run(args.clone(), &mut std::io::sink(), &mut std::io::sink()),
                    0
                );
            })
        });
        assert!(!destination.exists());
    }
    group.finish();
    fs::remove_dir_all(root).unwrap();
}
criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(30).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2));
    targets = preview
}
criterion_main!(benches);
