use super::*;
use crate::{artifact_fixture as fixture, ownership::retained::profile::Session};
use std::collections::BTreeMap;
fn usage() -> (f64, i64) {
    let mut r = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: writable output; assume initialized only after successful call.
    assert_eq!(
        unsafe { libc::getrusage(libc::RUSAGE_SELF, r.as_mut_ptr()) },
        0
    );
    // SAFETY: successful getrusage initialized the structure.
    let r = unsafe { r.assume_init() };
    (
        r.ru_utime.tv_sec as f64 * 1000.
            + r.ru_utime.tv_usec as f64 / 1000.
            + r.ru_stime.tv_sec as f64 * 1000.
            + r.ru_stime.tv_usec as f64 / 1000.,
        r.ru_maxrss,
    )
}
#[derive(Default)]
struct MeasuredIo {
    syncs: usize,
    renames: usize,
}
impl Io for MeasuredIo {
    fn sync(&mut self, f: &File, p: &'static str) -> std::io::Result<()> {
        self.syncs += 1;
        SystemIo.sync(f, p)
    }
    fn promote(&mut self, store: &File, from: &str, to: &str) -> Result<()> {
        self.renames += 1;
        SystemIo.promote(store, from, to)
    }
}
#[test]
#[ignore = "opt-in acknowledgment histories and phase-recording controls"]
fn acknowledgment_histories() {
    let pairs: usize = std::env::var("RVVDK_RETAINED_PAIRS")
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=64).contains(&pairs));
    let mode = std::env::var("RVVDK_PROFILE_FIXTURE").unwrap();
    let grains = match mode.as_str() {
        "stored_64" => 1024,
        "deflated_256" => 4096,
        _ => panic!("unknown fixture"),
    };
    let controls = std::env::var("RVVDK_ACK_CONTROLS").is_ok_and(|v| v == "1");
    let counters = std::env::var("RVVDK_ACK_COUNTERS").is_ok_and(|v| v == "1");
    let pattern = fixture::bytes(1);
    let payload = if mode == "stored_64" {
        fixture::stored(&pattern)
    } else {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../rvvdk-vmdk/tests/fixtures/stream/pattern.zlib"
        ))
        .to_vec()
    };
    let present = |i| i < 64 || (grains / 2..grains / 2 + 64).contains(&i);
    let records = (0..grains)
        .filter(|&i| present(i))
        .map(|i| (i, payload.clone()))
        .collect::<Vec<_>>();
    let bytes = fixture::image(true, grains, &records);
    let logical = grains * 65536;
    let mut rows = Vec::new();
    for pair in 0..pairs {
        for second in if pair % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let (synced, recording) = if controls {
                (false, second)
            } else {
                (second, true)
            };
            let t = Temp::new();
            let src = t.populate(&bytes, logical);
            let source_path = t.stage().join(MEMBERS[0]);
            let before_journal = journal(&t);
            let source_allocated = fs::metadata(&source_path).unwrap().blocks() * 512;
            let a =
                RetainedArtifact::open(t.store(), id(), src.clone(), RetainedOptions::default())
                    .unwrap();
            assert!(
                a.convert_owned(
                    oid(),
                    CopyOptions::default(),
                    RetainedOptions::default(),
                    &|_: &rvvdk_datamover::CopyEvent| {}
                )
                .is_success()
            );
            let v = VerifiedOutput::open(
                t.store(),
                oid(),
                id(),
                src.clone(),
                RetainedOptions::default(),
            )
            .unwrap();
            let publication =
                v.publish_hook(destination(&t), RetainedOptions::default(), |state, p| {
                    if state == OutputState::Published
                        && p == if synced {
                            "record_synced"
                        } else {
                            "record_written"
                        }
                    {
                        Err(OwnershipError::Io)
                    } else {
                        Ok(())
                    }
                });
            assert!(!publication.is_success());
            let next = stamp(&candidate(&t));
            let old = stamp(&current_path(&t));
            assert_ne!(next.1, old.1);
            let saved = snapshots(&t);
            let initial = fs::metadata(final_path(&t).join("disk.raw")).unwrap();
            let store = t.store();
            let dest = reopen(&t);
            let mut io = MeasuredIo::default();
            let audit = std::env::var("RVVDK_ACK_AUDIT").is_ok_and(|v| v == "1");
            if audit {
                eprintln!("RVDDK_ACK_BEGIN");
            }
            let session = Session::start(counters);
            let before = usage();
            let start = Instant::now();
            let mut mark = start;
            let mut cpu_mark = before.0;
            let mut phases = BTreeMap::new();
            let r = store.acknowledgment_hook(
                oid(),
                id(),
                src.clone(),
                dest,
                RetainedOptions::default(),
                &mut io,
                |_, p| {
                    let key = match p {
                        "admitted" => "admission",
                        "validated" => "verification",
                        "bundle_synced" => "bundle_sync",
                        "intent_directory_synced" => "intent_sync",
                        "destination_synced" => "parent_sync",
                        "transaction_synced" => "transaction_sync",
                        "acknowledgment_directory_synced" => "promotion_checks_and_acknowledgment",
                        _ => return Ok(()),
                    };
                    if recording {
                        let wall = mark.elapsed().as_secs_f64() * 1000.;
                        let cpu = usage().0;
                        phases.insert(
                            key,
                            serde_json::json!({"wall_ms":wall,"cpu_ms":cpu-cpu_mark}),
                        );
                        mark = Instant::now();
                        cpu_mark = cpu;
                    }
                    Ok(())
                },
            );
            let ended = Instant::now();
            let wall_ms = ended.duration_since(start).as_secs_f64() * 1000.;
            let after = usage();
            if recording {
                phases.insert("assessment_and_release",serde_json::json!({"wall_ms":ended.duration_since(mark).as_secs_f64()*1000.,"cpu_ms":after.0-cpu_mark}));
            }
            let regions = session.finish();
            if audit {
                eprintln!("RVDDK_ACK_END");
            }
            assert!(r.is_success(), "{r:?}");
            assert_eq!(r.logical_bytes_verified, logical);
            assert_eq!(io.syncs, 10);
            assert_eq!(io.renames, 1);
            promoted(&t, &next);
            unchanged(&saved);
            if counters {
                for key in [
                    "logical_source_read",
                    "raw_read",
                    "logical_compare",
                    "logical_hash",
                ] {
                    assert_eq!(regions[key].bytes, logical);
                }
                for key in ["container_read", "container_hash"] {
                    assert_eq!(regions[key].bytes, 2 * bytes.len() as u64);
                }
                assert_eq!(regions["native_map"].calls, 1);
                assert_eq!(regions["present_grain_read"].bytes, 8 << 20);
                assert!(!regions.contains_key("journal_commit_including_sync"));
            }
            let raw = final_path(&t).join("disk.raw");
            let recovered_blocks = fs::metadata(&raw).unwrap().blocks();
            let f = File::open(&raw).unwrap();
            let mut observed = vec![0; 65536];
            for i in 0..grains {
                f.read_exact_at(&mut observed, i * 65536).unwrap();
                if present(i) {
                    assert_eq!(observed, pattern);
                } else {
                    assert!(observed.iter().all(|&b| b == 0));
                }
            }
            drop(f);
            assert_eq!(fs::read(&source_path).unwrap(), bytes);
            assert_eq!(journal(&t), before_journal);
            let final_meta = fs::metadata(&raw).unwrap();

            assert_eq!(initial.blocks(), recovered_blocks);
            assert_eq!(
                (initial.ino(), initial.blocks(), initial.len()),
                (final_meta.ino(), final_meta.blocks(), final_meta.len())
            );
            let mut store = t.store();
            let dest = reopen(&t);
            let cleanup_session = Session::start(counters);
            let cpu = usage();
            let now = Instant::now();
            store
                .cleanup_output(oid(), id(), &src, Some(&dest), RetainedOptions::default())
                .unwrap();
            let cleanup_ms = now.elapsed().as_secs_f64() * 1000.;
            let cleanup_cpu_ms = usage().0 - cpu.0;
            let cleanup_regions = cleanup_session.finish();
            if counters {
                assert_eq!(cleanup_regions["journal_commit_including_sync"].calls, 2);
            }
            assert!(!final_path(&t).exists());
            assert_eq!(
                store.assess_output(oid(), id(), &src).unwrap().state,
                OutputState::Cleaned
            );
            assert_eq!(journal(&t), before_journal);
            rows.push(serde_json::json!({"pair":pair,"synced_candidate":synced,"recording":recording,"controls":controls,"counters":counters,"fixture":mode,"wall_ms":wall_ms,"cpu_ms":after.0-before.0,"process_peak_rss_kib":after.1,"phases":phases,"logical_bytes":logical,"logical_bytes_verified":r.logical_bytes_verified,"source_bytes":bytes.len(),"source_allocated_bytes":source_allocated,"output_allocated_bytes":final_meta.blocks()*512,"initial_allocated_bytes":initial.blocks()*512,"recovered_allocated_bytes":recovered_blocks*512,"regions":regions,"explicit_sync_calls":io.syncs,"rename_calls":io.renames,"output_journal_commits":0,"candidate_preserved":true,"old_journal_replaced":true,"cleanup_journal_commits":2,"cleanup_ms":cleanup_ms,"cleanup_cpu_ms":cleanup_cpu_ms,"oracle_passed":true,"source_unchanged":true,"allocation_unchanged":true,"cleanup_passed":true}));
        }
    }
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(std::env::var_os("RVVDK_RETAINED_REPORT").unwrap())
        .unwrap();
    serde_json::to_writer_pretty(&mut f, &rows).unwrap();
    f.write_all(b"\n").unwrap();
    f.sync_all().unwrap();
}
