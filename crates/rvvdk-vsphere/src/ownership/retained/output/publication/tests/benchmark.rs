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
use crate::artifact_fixture as fixture;
use rvvdk_datamover::{CopyEvent, CopyPhase};
#[test]
#[ignore = "opt-in matched owned output and durable publication benchmark"]
fn matched_publication() {
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
        for published in if pair % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let t = Temp::new();
            let source = t.populate(&bytes, 64 << 20);
            let path = t.stage().join(MEMBERS[0]);
            let source_allocated = fs::metadata(&path).unwrap().blocks() * 512;
            let journal = fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap();
            let store = t.store();
            let dest = if published {
                Some(destination(&t))
            } else {
                None
            };
            let progress = std::cell::Cell::new(rvvdk_core::CopyProgress::default());
            let observer = |e: &CopyEvent| {
                if e.phase == CopyPhase::Completed {
                    progress.set(e.progress);
                }
            };
            let before = usage();
            let started = Instant::now();
            let a = RetainedArtifact::open(store, id(), source.clone(), RetainedOptions::default())
                .unwrap();
            let admission_ms = started.elapsed().as_secs_f64() * 1000.;
            let r = a.convert_owned(
                oid(),
                CopyOptions::default(),
                RetainedOptions::default(),
                &observer,
            );
            assert!(r.is_success(), "{r:?}");
            assert_eq!(r.logical_bytes_verified, 64 << 20);
            let conversion_end = started.elapsed().as_secs_f64() * 1000.;
            let (publication_admission_ms, publication_ms) = if let Some(dest) = dest {
                let now = Instant::now();
                let v = VerifiedOutput::open(
                    t.store(),
                    oid(),
                    id(),
                    source.clone(),
                    RetainedOptions::default(),
                )
                .unwrap();
                let admitted = now.elapsed().as_secs_f64() * 1000.;
                let now = Instant::now();
                let r = v.publish(dest, RetainedOptions::default());
                assert!(r.is_success(), "{r:?}");
                (admitted, now.elapsed().as_secs_f64() * 1000.)
            } else {
                (0., 0.)
            };
            let wall_ms = started.elapsed().as_secs_f64() * 1000.;
            let after = usage();
            let dir = if published {
                final_path(&t)
            } else {
                raw_stage(&t)
            };
            let out = dir.join("disk.raw");
            verify(&out, 1024, |i| i < 64 || (512..576).contains(&i));
            assert_eq!(fs::read(&path).unwrap(), bytes);
            assert_eq!(
                fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap(),
                journal
            );
            let allocated = fs::metadata(&out).unwrap().blocks() * 512;
            let p = progress.get();
            assert_eq!(p.bytes_read, 8 << 20);
            assert_eq!(p.bytes_written + p.bytes_zeroed, 64 << 20);
            let metadata_bytes = fs::metadata(dir.join("output.json")).unwrap().len();
            let dest = if published { Some(reopen(&t)) } else { None };
            let mut store = t.store();
            let cpu = usage();
            let now = Instant::now();
            store
                .cleanup_output(
                    oid(),
                    id(),
                    &source,
                    dest.as_ref(),
                    RetainedOptions::default(),
                )
                .unwrap();
            let cleanup_ms = now.elapsed().as_secs_f64() * 1000.;
            let cleanup_cpu_ms = (usage().0 - cpu.0) * 1000.;
            assert!(!dir.exists());
            assert_eq!(
                store.assess_output(oid(), id(), &source).unwrap().state,
                OutputState::Cleaned
            );
            assert_eq!(
                fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap(),
                journal
            );
            rows.push(serde_json::json!({"pair":pair,"published":published,"wall_ms":wall_ms,"cpu_ms":(after.0-before.0)*1000.,"admission_ms":admission_ms,"conversion_and_checks_ms":conversion_end-admission_ms,"publication_admission_ms":publication_admission_ms,"publication_ms":publication_ms,"cleanup_ms":cleanup_ms,"cleanup_cpu_ms":cleanup_cpu_ms,"cleanup_passed":true,"process_peak_rss_kib":after.1,"logical_bytes":64<<20,"source_bytes":bytes.len(),"source_allocated_bytes":source_allocated,"output_allocated_bytes":allocated,"read_bytes":p.bytes_read,"written_bytes":p.bytes_written,"zeroed_bytes":p.bytes_zeroed,"byte_comparison_passed":true,"journal_unchanged":true,"logical_bytes_verified_in_timing":if published {3*(64<<20)}else{64<<20},"output_journal_commits":if published {9}else{7},"cleanup_journal_commits":2,"output_metadata_bytes":metadata_bytes}));
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
