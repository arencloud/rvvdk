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
    recording: bool,
    syncs: usize,
    renames: usize,
    rename_ms: f64,
    rename_cpu_ms: f64,
}
impl Io for MeasuredIo {
    fn sync(&mut self, f: &File, p: &'static str) -> std::io::Result<()> {
        self.syncs += 1;
        SystemIo.sync(f, p)
    }
    fn rename(&mut self, from: &File, old: &str, to: &File, new: &str) -> Result<()> {
        self.renames += 1;
        let before = self.recording.then(|| (Instant::now(), usage().0));
        let r = SystemIo.rename(from, old, to, new);
        if let Some((start, cpu)) = before {
            self.rename_ms = start.elapsed().as_secs_f64() * 1000.;
            self.rename_cpu_ms = usage().0 - cpu;
        }
        r
    }
}
#[test]
#[ignore = "opt-in recovery branches and phase-recording controls"]
fn recovery_branches() {
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
    let controls = std::env::var("RVVDK_RECONCILE_CONTROLS").is_ok_and(|v| v == "1");
    let counters = std::env::var("RVVDK_RECONCILE_COUNTERS").is_ok_and(|v| v == "1");
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
            let (renamed, recording) = if controls {
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
            let interrupted =
                v.publish_hook(destination(&t), RetainedOptions::default(), |_, p| {
                    if p == if renamed {
                        "bundle_renamed"
                    } else {
                        "before_rename"
                    } {
                        Err(OwnershipError::Io)
                    } else {
                        Ok(())
                    }
                });
            assert!(!interrupted.is_success());
            let initial = fs::metadata(directory(&t, renamed).join("disk.raw")).unwrap();
            let store = t.store();
            let dest = reopen(&t);
            let mut io = MeasuredIo {
                recording,
                ..Default::default()
            };
            let session = Session::start(counters);
            let before = usage();
            let start = Instant::now();
            let mut mark = start;
            let mut cpu_mark = before.0;
            let mut phases = BTreeMap::new();
            let r = store.reconcile_hook(
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
                        "intent_checked" => "intent_sync",
                        "before_parent_sync" => "rename_and_checks",
                        "destination_directory_synced" => "parent_sync",
                        "before_assessment" => "acknowledgment",
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
            assert!(r.is_success(), "{r:?}");
            assert_eq!(r.logical_bytes_verified, logical);
            assert_eq!(io.syncs, 8);
            assert_eq!(io.renames, usize::from(!renamed));
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
                assert_eq!(regions["journal_commit_including_sync"].calls, 1);
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
            preserve_source(&t, &before_journal);
            let final_meta = fs::metadata(&raw).unwrap();

            assert_eq!(initial.blocks(), recovered_blocks);
            assert_eq!(
                (initial.ino(), initial.blocks(), initial.len()),
                (final_meta.ino(), final_meta.blocks(), final_meta.len())
            );
            let mut store = t.store();
            let dest = reopen(&t);
            let cpu = usage();
            let now = Instant::now();
            store
                .cleanup_output(oid(), id(), &src, Some(&dest), RetainedOptions::default())
                .unwrap();
            let cleanup_ms = now.elapsed().as_secs_f64() * 1000.;
            let cleanup_cpu_ms = usage().0 - cpu.0;
            assert!(!final_path(&t).exists());
            assert_eq!(
                store.assess_output(oid(), id(), &src).unwrap().state,
                OutputState::Cleaned
            );
            preserve_source(&t, &before_journal);
            rows.push(serde_json::json!({"pair":pair,"already_renamed":renamed,"recording":recording,"controls":controls,"counters":counters,"fixture":mode,"wall_ms":wall_ms,"cpu_ms":after.0-before.0,"process_peak_rss_kib":after.1,"phases":phases,"rename_ms":io.rename_ms,"rename_cpu_ms":io.rename_cpu_ms,"logical_bytes":logical,"logical_bytes_verified":r.logical_bytes_verified,"source_bytes":bytes.len(),"source_allocated_bytes":source_allocated,"output_allocated_bytes":final_meta.blocks()*512,"initial_allocated_bytes":initial.blocks()*512,"recovered_allocated_bytes":recovered_blocks*512,"regions":regions,"explicit_sync_calls":io.syncs,"rename_calls":io.renames,"output_journal_commits":1,"cleanup_journal_commits":2,"cleanup_ms":cleanup_ms,"cleanup_cpu_ms":cleanup_cpu_ms,"oracle_passed":true,"source_unchanged":true,"allocation_unchanged":true,"cleanup_passed":true}));
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
