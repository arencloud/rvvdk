use super::*;
use std::os::unix::fs::MetadataExt;
fn usage() -> (f64, i64) {
    let mut raw = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: writable storage; successful getrusage initializes it.
    assert_eq!(
        unsafe { libc::getrusage(libc::RUSAGE_SELF, raw.as_mut_ptr()) },
        0
    );
    // SAFETY: getrusage succeeded.
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
#[ignore = "opt-in matched retained artifact conversion benchmark"]
fn matched_retained_conversion() {
    let count: usize = std::env::var("RVVDK_RETAINED_PAIRS")
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=64).contains(&count));
    let output = std::env::var_os("RVVDK_RETAINED_REPORT").unwrap();
    let records: Vec<_> = (0..64)
        .chain(512..576)
        .map(|i| (i, fixture::stored(&fixture::bytes(i))))
        .collect();
    let bytes = fixture::image(true, 1024, &records);
    let mut rows = Vec::new();
    for pair in 0..count {
        for retained in if pair % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let t = Temp::new();
            let source = t.populate(&bytes, 64 << 20);
            let dest = t.output(64 << 20);
            let path = t.stage().join(MEMBERS[0]);
            let source_allocated = fs::metadata(&path).unwrap().blocks() * 512;
            let store = if retained { Some(t.store()) } else { None };
            let before = usage();
            let started = Instant::now();
            let (admit_ms, report) = if let Some(store) = store {
                let mut artifact =
                    RetainedArtifact::open(store, id(), source.clone(), RetainedOptions::default())
                        .unwrap();
                let admit_ms = started.elapsed().as_secs_f64() * 1000.;
                let report = artifact
                    .convert_to(
                        &dest,
                        CopyOptions::default(),
                        RetainedOptions::default(),
                        &|_: &CopyEvent| {},
                    )
                    .unwrap();
                drop(artifact);
                (admit_ms, report)
            } else {
                let disk = StreamDisk::load(
                    Arc::new(LocalFileBlockDevice::open_read_only(&path).unwrap()),
                    StreamDiskLimits::default(),
                )
                .unwrap();
                let admit_ms = started.elapsed().as_secs_f64() * 1000.;
                let mover = DataMover::new(CopyOptions::default());
                let plan = mover.plan_with_destination(&disk, &dest).unwrap();
                let report = mover
                    .execute_plan_controlled(
                        &plan,
                        &disk,
                        &dest,
                        &rvvdk_datamover::NoCancellation,
                        &|_: &CopyEvent| {},
                    )
                    .unwrap();
                drop(disk);
                (admit_ms, report)
            };
            let wall_ms = started.elapsed().as_secs_f64() * 1000.;
            let after = usage();
            assert_eq!(report.stats().bytes_read(), 8 << 20);
            assert_eq!(
                report.stats().bytes_written() + report.stats().bytes_zeroed(),
                64 << 20
            );
            verify_output(&t, 1024, |i| i < 64 || (512..576).contains(&i));
            assert_eq!(fs::read(&path).unwrap(), bytes);
            let allocated = fs::metadata(t.0.join("output.raw")).unwrap().blocks() * 512;
            let mut store = t.store();
            let r = store.recover(id(), &source).unwrap();
            assert_eq!(r.state, JobState::CompletedLease);
            assert_eq!(r.sequence, 7);
            store.cleanup_local(id(), r.operation, &source).unwrap();
            assert_eq!(
                store.recover(id(), &source).unwrap().state,
                JobState::Cleaned
            );
            rows.push(serde_json::json!({"pair":pair,"retained":retained,"wall_ms":wall_ms,"cpu_ms":(after.0-before.0)*1000.,"admission_ms":admit_ms,"conversion_ms":wall_ms-admit_ms,"process_peak_rss_kib":after.1,"logical_bytes":64<<20,"source_bytes":bytes.len(),"source_allocated_bytes":source_allocated,"output_allocated_bytes":allocated,"read_bytes":report.stats().bytes_read(),"written_bytes":report.stats().bytes_written(),"zeroed_bytes":report.stats().bytes_zeroed(),"byte_comparison_passed":true,"journal_unchanged":true}));
        }
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    serde_json::to_writer_pretty(&mut file, &rows).unwrap();
    file.write_all(b"\n").unwrap();
    file.sync_all().unwrap();
}
