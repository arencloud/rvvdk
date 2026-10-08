use super::*;
use crate::ownership::retained::profile::Session;

#[test]
#[ignore = "opt-in paired local profiling and disabled-instrumentation controls"]
fn local_profile() {
    assert_eq!(
        CopyOptions::default().concurrency(),
        1,
        "thread CPU regions require serial copy"
    );
    let count: usize = std::env::var("RVVDK_RETAINED_PAIRS")
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=64).contains(&count));
    let mode = std::env::var("RVVDK_PROFILE_FIXTURE").unwrap();
    let grains = match mode.as_str() {
        "stored_64" | "deflated_64" => 1024,
        "deflated_256" => 4096,
        _ => panic!("unknown profile fixture"),
    };
    let logical = grains * 65536;
    let expected = fixture::bytes(1);
    let payload = if mode == "stored_64" {
        fixture::stored(&expected)
    } else {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../rvvdk-vmdk/tests/fixtures/stream/pattern.zlib"
        ))
        .to_vec()
    };
    let present = |i: u64| i < 64 || (grains / 2..grains / 2 + 64).contains(&i);
    let records: Vec<_> = (0..grains)
        .filter(|&i| present(i))
        .map(|i| (i, payload.clone()))
        .collect();
    let bytes = fixture::image(true, grains, &records);
    let mut rows = Vec::new();
    for pair in 0..count {
        for enabled in if pair % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let t = Temp::new();
            let source = t.populate(&bytes, logical);
            let source_path = t.stage().join(MEMBERS[0]);
            let journal = fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap();
            let source_allocated = fs::metadata(&source_path).unwrap().blocks() * 512;
            let store = t.store();
            let dest = destination(&t);
            let progress = std::cell::Cell::new(rvvdk_core::CopyProgress::default());
            let observer = |e: &CopyEvent| {
                if e.phase == CopyPhase::Completed {
                    progress.set(e.progress);
                }
            };
            let profile = Session::start(enabled);
            let before = usage();
            let start = Instant::now();
            let a = RetainedArtifact::open(store, id(), source.clone(), RetainedOptions::default())
                .unwrap();
            let admission_ms = start.elapsed().as_secs_f64() * 1000.;
            let r = a.convert_owned(
                oid(),
                CopyOptions::default(),
                RetainedOptions::default(),
                &observer,
            );
            assert!(r.is_success(), "{r:?}");
            let converted_ms = start.elapsed().as_secs_f64() * 1000.;
            let now = Instant::now();
            let v = VerifiedOutput::open(
                t.store(),
                oid(),
                id(),
                source.clone(),
                RetainedOptions::default(),
            )
            .unwrap();
            let publication_admission_ms = now.elapsed().as_secs_f64() * 1000.;
            let now = Instant::now();
            let r = v.publish(dest, RetainedOptions::default());
            assert!(r.is_success(), "{r:?}");
            let publication_ms = now.elapsed().as_secs_f64() * 1000.;
            let wall_ms = start.elapsed().as_secs_f64() * 1000.;
            let after = usage();
            let regions = profile.finish();
            if enabled {
                for key in [
                    "logical_source_read",
                    "raw_read",
                    "logical_compare",
                    "logical_hash",
                ] {
                    assert_eq!(regions[key].bytes, 3 * logical, "{key}");
                }
                for key in ["container_read", "container_hash"] {
                    assert_eq!(regions[key].bytes, 7 * bytes.len() as u64, "{key}");
                }
                assert_eq!(regions["native_map"].calls, 3);
                assert_eq!(regions["present_grain_read"].bytes, 3 * (8 << 20));
                assert_eq!(regions["journal_commit_including_sync"].calls, 9);
                assert_eq!(regions["copy_engine_including_flush"].bytes, logical);
                assert!(regions.values().map(|r| r.wall_ms).sum::<f64>() <= wall_ms);
            } else {
                assert!(regions.is_empty());
            }
            let raw = final_path(&t).join("disk.raw");
            let f = File::open(&raw).unwrap();
            let mut observed = vec![0; 65536];
            for i in 0..grains {
                f.read_exact_at(&mut observed, i * 65536).unwrap();
                if present(i) {
                    assert_eq!(observed, expected);
                } else {
                    assert!(observed.iter().all(|&b| b == 0));
                }
            }
            drop(f);
            assert_eq!(fs::read(&source_path).unwrap(), bytes);
            assert_eq!(
                fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap(),
                journal
            );
            let allocated = fs::metadata(&raw).unwrap().blocks() * 512;
            let p = progress.get();
            assert_eq!(p.bytes_read, 8 << 20);
            assert_eq!(p.bytes_written + p.bytes_zeroed, logical);
            let dest = reopen(&t);
            let mut store = t.store();
            let cpu = usage();
            let now = Instant::now();
            store
                .cleanup_output(
                    oid(),
                    id(),
                    &source,
                    Some(&dest),
                    RetainedOptions::default(),
                )
                .unwrap();
            let cleanup_ms = now.elapsed().as_secs_f64() * 1000.;
            let cleanup_cpu_ms = (usage().0 - cpu.0) * 1000.;
            assert!(!final_path(&t).exists());
            assert_eq!(
                store.assess_output(oid(), id(), &source).unwrap().state,
                OutputState::Cleaned
            );
            assert_eq!(
                fs::read(t.jobs().join(record_name(id().as_bytes()))).unwrap(),
                journal
            );
            rows.push(serde_json::json!({"pair":pair,"instrumented":enabled,"fixture":mode,"wall_ms":wall_ms,"cpu_ms":(after.0-before.0)*1000.,"admission_ms":admission_ms,"conversion_ms":converted_ms-admission_ms,"publication_admission_ms":publication_admission_ms,"publication_ms":publication_ms,"cleanup_ms":cleanup_ms,"cleanup_cpu_ms":cleanup_cpu_ms,"process_peak_rss_kib":after.1,"logical_bytes":logical,"source_bytes":bytes.len(),"source_allocated_bytes":source_allocated,"output_allocated_bytes":allocated,"regions":regions,"oracle_passed":true,"source_unchanged":true,"cleanup_passed":true}));
        }
    }
    let output = std::env::var_os("RVVDK_RETAINED_REPORT").unwrap();
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    serde_json::to_writer_pretty(&mut f, &rows).unwrap();
    f.write_all(b"\n").unwrap();
    f.sync_all().unwrap();
}
