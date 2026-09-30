#[path = "../tests/support/parents.rs"]
mod parents;
use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use std::{
    fs,
    time::{Duration, Instant},
};
fn parents_cli(c: &mut Criterion) {
    let mono_fixture = parents::Fixture::new([false; 3]);
    let split_fixture = parents::Fixture::new([true; 3]);
    let mono = &mono_fixture.expected;
    let split = &split_fixture.expected;
    let destination = mono_fixture.destination.clone();
    let mut group = c.benchmark_group("cli_parents");
    group.sampling_mode(SamplingMode::Flat);
    for (name, command, source, expected) in [
        ("inspect_mono", "inspect", &mono_fixture.source, &mono),
        ("plan_split", "plan", &split_fixture.source, &split),
        ("copy_mono_verify", "copy", &mono_fixture.source, &mono),
        ("copy_split_verify", "copy", &split_fixture.source, &split),
        ("verify_split", "verify", &split_fixture.source, &split),
    ] {
        let mut args = vec![
            "rvddk".into(),
            command.into(),
            source.clone().into_os_string(),
        ];
        if command != "inspect" {
            args.push(destination.clone().into_os_string());
        }
        args.extend([
            "--format".into(),
            "vmdk".into(),
            "--json".into(),
            "--allow-parents".into(),
        ]);
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
            assert_eq!(&fs::read(&destination).unwrap(), *expected);
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
                        assert_eq!(&fs::read(&destination).unwrap(), *expected);
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
}
criterion_group! {name=benches; config=Criterion::default().sample_size(30).warm_up_time(Duration::from_millis(300)).measurement_time(Duration::from_secs(2)); targets=parents_cli}
criterion_main!(benches);
