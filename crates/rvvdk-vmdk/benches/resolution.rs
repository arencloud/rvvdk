#[cfg(target_os = "linux")]
fn main() {
    use criterion::{Criterion, SamplingMode};
    use rvvdk_vmdk::{Descriptor, Limits, LocalResolver, ResolutionLimits, ResolvedDescriptor};
    use std::{
        fs::{self, File},
        hint::black_box,
        path::PathBuf,
    };
    let root = std::env::var_os("RVVDK_BENCH_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let path = root.join(format!("rvvdk-resolution-{}", std::process::id()));
    fs::create_dir(&path).unwrap();
    let header = "version=1\nCID=12345678\nparentCID=ffffffff\ncreateType=\"custom\"\n";
    let small = format!("{header}RW 8 FLAT \"data0\" 0\n");
    let many = format!(
        "{header}{}",
        (0..32)
            .map(|i| format!("RW 8 FLAT \"data{i}\" 0\n"))
            .collect::<String>()
    );
    let repeated = format!("{header}{}", "RW 8 FLAT \"data0\" 0\n".repeat(1024));
    for i in 0..32 {
        fs::write(path.join(format!("data{i}")), [7; 4096]).unwrap();
    }
    fs::write(path.join("disk.vmdk"), &small).unwrap();
    let resolver = LocalResolver::from_directory(File::open(&path).unwrap()).unwrap();
    let mut c = Criterion::default().configure_from_args();
    let mut group = c.benchmark_group("resolution");
    group.sampling_mode(SamplingMode::Flat);
    for (name, input, count) in [
        ("one_file", small.as_bytes(), 1),
        ("32_files", many.as_bytes(), 32),
        ("1024_shared", repeated.as_bytes(), 1),
    ] {
        let d = Descriptor::parse(input).unwrap();
        let check =
            ResolvedDescriptor::resolve(&d, &resolver, ResolutionLimits::default()).unwrap();
        assert_eq!(check.backings().len(), count);
        check.revalidate().unwrap();
        drop(check);
        group.bench_function(name, |b| {
            b.iter(|| {
                black_box(
                    ResolvedDescriptor::resolve(
                        black_box(&d),
                        black_box(&resolver),
                        ResolutionLimits::default(),
                    )
                    .unwrap(),
                )
            })
        });
    }
    group.bench_function("load_descriptor", |b| {
        b.iter(|| {
            black_box(
                LocalResolver::open_descriptor(
                    black_box(path.join("disk.vmdk")),
                    Limits::default(),
                )
                .unwrap(),
            )
        })
    });
    group.finish();
    c.final_summary();
    drop(resolver);
    fs::remove_dir_all(path).unwrap();
}
#[cfg(not(target_os = "linux"))]
fn main() {}
