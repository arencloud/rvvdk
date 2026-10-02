use super::*;
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
#[ignore = "opt-in matched owned RAW conversion and verification benchmark"]
fn matched_owned_output() {
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
        for owned in if pair % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let t = Temp::new();
            let source = t.populate(&bytes, 64 << 20);
            let dest = if owned {
                None
            } else {
                Some(t.output(64 << 20))
            };
            let path = t.stage().join(MEMBERS[0]);
            let source_allocated = fs::metadata(&path).unwrap().blocks() * 512;
            let journal = fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap();
            let store = t.store();
            let progress = std::cell::Cell::new(rvvdk_core::CopyProgress::default());
            let observer = |e: &CopyEvent| {
                if e.phase == CopyPhase::Completed {
                    progress.set(e.progress);
                }
            };
            let before = usage();
            let started = Instant::now();
            let mut a =
                RetainedArtifact::open(store, id(), source.clone(), RetainedOptions::default())
                    .unwrap();
            let admission_ms = started.elapsed().as_secs_f64() * 1000.;
            let verified = if owned {
                let r = a.convert_owned(
                    oid(),
                    CopyOptions::default(),
                    RetainedOptions::default(),
                    &observer,
                );
                assert!(r.is_success(), "{r:?}");
                assert_eq!(r.logical_bytes_verified, 64 << 20);
                r.logical_bytes_verified
            } else {
                a.convert_to(
                    dest.as_ref().unwrap(),
                    CopyOptions::default(),
                    RetainedOptions::default(),
                    &observer,
                )
                .unwrap();
                drop(a);
                0
            };
            drop(dest);
            let wall_ms = started.elapsed().as_secs_f64() * 1000.;
            let after = usage();
            let output_path = if owned {
                raw_stage(&t).join("disk.raw")
            } else {
                t.0.join("output.raw")
            };
            verify(&output_path, 1024, |i| i < 64 || (512..576).contains(&i));
            assert_eq!(fs::read(&path).unwrap(), bytes);
            assert_eq!(
                fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap(),
                journal
            );
            assert_eq!(
                t.store().recover(id(), &source).unwrap().state,
                JobState::CompletedLease
            );
            let p = progress.get();
            assert_eq!(p.bytes_read, 8 << 20);
            assert_eq!(p.bytes_written + p.bytes_zeroed, 64 << 20);
            let allocated = fs::metadata(&output_path).unwrap().blocks() * 512;
            let metadata_bytes = if owned {
                fs::metadata(raw_stage(&t).join("output.json"))
                    .unwrap()
                    .len()
            } else {
                0
            };
            rows.push(serde_json::json!({"pair":pair,"owned":owned,"wall_ms":wall_ms,"cpu_ms":(after.0-before.0)*1000.,"admission_ms":admission_ms,"conversion_and_checks_ms":wall_ms-admission_ms,"process_peak_rss_kib":after.1,"logical_bytes":64<<20,"source_bytes":bytes.len(),"source_allocated_bytes":source_allocated,"output_allocated_bytes":allocated,"read_bytes":p.bytes_read,"written_bytes":p.bytes_written,"zeroed_bytes":p.bytes_zeroed,"byte_comparison_passed":true,"journal_unchanged":true,"logical_bytes_verified_in_timing":verified,"output_journal_commits":if owned {7}else{0},"output_metadata_bytes":metadata_bytes}));
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
