use crate::{
    error::Failure,
    preview::PathReport,
    target::{self, Target},
};
use rvvdk_core::{RawDisk, VirtualDisk};
use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionBackend, ExecutionSelectionReason, ExecutionStrategy,
    IoUringExecutionOptions, NativeRuntimeFallback, Verifier,
};
use rvvdk_local::LocalFileBlockDevice;
use serde_json::{Value, json};
use std::{fs::File, io::Write, os::unix::fs::MetadataExt, path::PathBuf, time::Instant};

type Result<T> = std::result::Result<T, Failure>;
fn disk(file: &File) -> Result<RawDisk<LocalFileBlockDevice>> {
    Ok(RawDisk::new(LocalFileBlockDevice::from_buffered_file(
        file.try_clone()
            .map_err(|e| Failure::io("duplicate owned file", e))?,
    )?))
}
fn stamp(file: &File) -> Result<(u64, u64, u64, i64, i64, i64, i64)> {
    let m = file
        .metadata()
        .map_err(|e| Failure::io("inspect live file", e))?;
    Ok((
        m.dev(),
        m.ino(),
        m.len(),
        m.mtime(),
        m.mtime_nsec(),
        m.ctime(),
        m.ctime_nsec(),
    ))
}
fn backend(value: ExecutionBackend) -> &'static str {
    match value {
        ExecutionBackend::Threaded => "threaded",
        ExecutionBackend::IoUring => "io-uring",
    }
}
fn reason(value: ExecutionSelectionReason) -> &'static str {
    match value {
        ExecutionSelectionReason::RequestedThreaded => "requested_threaded",
        ExecutionSelectionReason::RequestedNative => "requested_native",
        ExecutionSelectionReason::RawDescriptorsCompatible => "raw_descriptors_compatible",
        ExecutionSelectionReason::RawDescriptorsIncompatible => "raw_descriptors_incompatible",
        ExecutionSelectionReason::RawRequestsIncompatible(_) => "raw_requests_incompatible",
        ExecutionSelectionReason::PortableApi => "portable_api",
        _ => "unknown",
    }
}
pub(crate) fn run(
    name: &str,
    args: &clap::ArgMatches,
    json_output: bool,
    out: &mut impl Write,
) -> Result<()> {
    let report = if name == "copy" {
        copy(args, |_, _| Ok(()))?
    } else {
        verify(args)?
    };
    let result = if json_output {
        serde_json::to_writer_pretty(&mut *out, &report)
            .map_err(|e| Failure::new("output", e.to_string()))
            .and_then(|_| writeln!(out).map_err(|e| Failure::io("write output", e)))
    } else {
        writeln!(
            out,
            "RAW {name}: {} ({} logical bytes)",
            report["status"].as_str().unwrap(),
            report["logical_bytes"]
        )
        .and_then(|_| {
            if name == "copy" {
                writeln!(
                    out,
                    "Backend: {}; verification: {}; durability: {}",
                    report["backend"].as_str().unwrap(),
                    !report["verification"].is_null(),
                    report["durability"].as_str().unwrap()
                )
            } else {
                writeln!(
                    out,
                    "Verified source-length prefix; destination tail: {} bytes",
                    report["destination_tail_bytes"]
                )
            }
        })
        .map_err(|e| Failure::io("write output", e))
    }
    .and_then(|_| out.flush().map_err(|e| Failure::io("flush output", e)));
    result.map_err(|mut e| {
        e.details =
            Some(json!({"phase":"report_output", "operation_completed":true, "result": report}));
        e
    })
}
fn verify(args: &clap::ArgMatches) -> Result<Value> {
    let started = Instant::now();
    let source_path = args.get_one::<PathBuf>("source").unwrap();
    let destination_path = args.get_one::<PathBuf>("destination").unwrap();
    let source_file = target::source(source_path)?;
    let destination_file = target::verify_destination(destination_path)?;
    let source_stamp = stamp(&source_file)?;
    let destination_stamp = stamp(&destination_file)?;
    let source = disk(&source_file)?;
    let destination = disk(&destination_file)?;
    let mut verifier = Verifier::new(
        source.size(),
        *args.get_one("block-size").unwrap(),
        *args.get_one("memory-budget").unwrap(),
    )?;
    let report = verifier.verify(&source, &destination)?;
    if stamp(&source_file)? != source_stamp || stamp(&destination_file)? != destination_stamp {
        return Err(Failure::new(
            "source_changed",
            "verification endpoint changed during comparison",
        ));
    }
    Ok(
        json!({"schema_version":1,"command":"verify","format":"raw","status":"verified", "source":PathReport::from(source_path.as_path()),"destination":PathReport::from(destination_path.as_path()),"logical_bytes":report.bytes_verified,"destination_tail_bytes":destination.size()-source.size(),"verification_payload_bytes":verifier.storage_bytes(),"elapsed_seconds":started.elapsed().as_secs_f64()}),
    )
}
fn copy(
    args: &clap::ArgMatches,
    mut hook: impl FnMut(&str, &Target) -> Result<()>,
) -> Result<Value> {
    let started = Instant::now();
    let source_path = args.get_one::<PathBuf>("source").unwrap();
    let destination_path = args.get_one::<PathBuf>("destination").unwrap();
    let source_file = target::source(source_path)?;
    let initial = stamp(&source_file)?;
    let source = disk(&source_file)?;
    let budget = *args.get_one::<usize>("memory-budget").unwrap();
    let block = *args.get_one::<usize>("block-size").unwrap();
    let mut verifier = if args.get_flag("verify") {
        Some(Verifier::new(source.size(), block, budget)?)
    } else {
        None
    };
    let reserved = verifier.as_ref().map_or(0, Verifier::storage_bytes);
    let options = CopyOptions::with_concurrency(block, 4096, *args.get_one("workers").unwrap())?
        .with_memory_budget(budget - reserved);
    let native = IoUringExecutionOptions::new(*args.get_one("queue-depth").unwrap()).unwrap();
    let requested = args.get_one::<String>("backend").unwrap();
    let strategy = match requested.as_str() {
        "threaded" => ExecutionStrategy::Threaded,
        "auto" => ExecutionStrategy::Auto(native),
        _ => ExecutionStrategy::IoUring(native),
    };
    let mover = DataMover::with_execution_strategy(options, strategy);
    let mut target = Target::open(destination_path, args.get_flag("overwrite"), &source_file)?;
    let mut phase = "planning";
    let mut counters = Value::Null;
    let outcome = (|| {
        let destination = disk(&target.file)?;
        let plan = mover.plan_raw_with_destination(&source, &destination)?;
        if target.existing {
            target.check_name()?;
        }
        if stamp(&source_file)? != initial {
            return Err(Failure::new("source_changed", "source changed before copy"));
        }
        phase = "copy";
        target.mutation_started = true;
        let report = mover.execute_raw_plan(&plan, &source, &destination)?;
        let stats = report.stats();
        counters = json!({"bytes_read":stats.bytes_read(),"bytes_written":stats.bytes_written(),"bytes_zeroed":stats.bytes_zeroed(),"bytes_discarded":stats.bytes_discarded(),"blocks_copied":stats.blocks_copied(),"extents_processed":stats.extents_processed()});
        phase = "verification";
        hook(phase, &target)?;
        let verification = match &mut verifier {
            Some(v) => {
                let r = v.verify(&source, &destination)?;
                json!({"bytes_verified":r.bytes_verified,"elapsed_seconds":r.elapsed.as_secs_f64()})
            }
            None => Value::Null,
        };
        if stamp(&source_file)? != initial {
            return Err(Failure::new(
                "source_changed",
                "source changed during copy or verification",
            ));
        }
        phase = "file_sync";
        hook(phase, &target)?;
        target
            .file
            .sync_all()
            .map_err(|e| Failure::io("sync completed file", e))?;
        if target.existing {
            target.check_name()?;
        } else {
            phase = "publication";
            hook(phase, &target)?;
            target.publish()?;
            phase = "directory_sync";
            hook(phase, &target)?;
            target.sync_parent()?;
            target.check_name()?;
        }
        let runtime_fallback = match report.runtime_fallback() {
            Some(NativeRuntimeFallback::RingUnavailable { os_error }) => {
                json!({"reason":"ring_unavailable","os_error":os_error})
            }
            Some(_) => json!({"reason":"unknown"}),
            None => Value::Null,
        };
        Ok(
            json!({"schema_version":1,"command":"copy","format":"raw","status":"completed", "source":PathReport::from(source_path.as_path()),"destination":PathReport::from(destination_path.as_path()),"destination_policy":if target.existing { "overwrite_in_place" } else { "create_new_no_clobber" },"logical_bytes":source.size(),"destination_tail_bytes":destination.size()-source.size(),"requested_backend":requested,"planned_backend":backend(plan.backend()),"selection_reason":reason(plan.execution_selection().reason()),"backend":backend(report.backend()),"runtime_fallback":runtime_fallback,"stats":counters,"verification":verification,"verification_payload_bytes":reserved,"copy_memory_budget_bytes":budget-reserved,"durability":if target.existing { "file_synced" } else { "file_and_directory_synced" },"elapsed_seconds":started.elapsed().as_secs_f64()}),
        )
    })();
    outcome.map_err(|mut error: Failure| {
        error.details = Some(json!({"phase":phase,"destination_state":target.failure_state(),"confirmed_copy_stats":counters,"cause":error.details})); error
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        os::unix::fs::FileExt,
        sync::atomic::{AtomicUsize, Ordering},
    };
    struct Fixture {
        root: PathBuf,
        source: PathBuf,
        destination: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicUsize = AtomicUsize::new(0);
            let root = std::env::temp_dir().join(format!(
                "rvddk-r32-fault-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            let source = root.join("source");
            let destination = root.join("destination");
            fs::write(&source, vec![0x5a; 8199]).unwrap();
            Self {
                root,
                source,
                destination,
            }
        }
        fn args(&self, existing: bool) -> clap::ArgMatches {
            let mut args = vec![
                "rvddk".into(),
                "copy".into(),
                self.source.clone().into_os_string(),
                self.destination.clone().into_os_string(),
                "--format".into(),
                "raw".into(),
                "--verify".into(),
            ];
            if existing {
                args.push("--overwrite".into());
            }
            crate::args::command()
                .try_get_matches_from(args)
                .unwrap()
                .remove_subcommand()
                .unwrap()
                .1
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }
    #[test]
    fn corruption_fails_before_publication_and_reports_offset() {
        let f = Fixture::new();
        let error = copy(&f.args(false), |phase, target| {
            if phase == "verification" {
                target.file.write_all_at(&[0], 8198).unwrap();
            }
            Ok(())
        })
        .unwrap_err();
        assert_eq!(error.code, "verification_mismatch");
        assert_eq!(
            error.details.as_ref().unwrap()["cause"]["mismatch_offset"],
            8198
        );
        assert!(!f.destination.exists());
        assert_eq!(fs::read_dir(&f.root).unwrap().count(), 1);
    }
    #[test]
    fn racing_name_is_never_replaced_even_when_overwrite_was_requested() {
        let f = Fixture::new();
        let error = copy(&f.args(true), |phase, _| {
            if phase == "publication" {
                fs::write(&f.destination, b"winner").unwrap();
            }
            Ok(())
        })
        .unwrap_err();
        assert_eq!(error.os_error, Some(libc::EEXIST));
        assert_eq!(
            error.details.unwrap()["destination_state"],
            "private_unpublished"
        );
        assert_eq!(fs::read(&f.destination).unwrap(), b"winner");
    }
    #[test]
    fn file_sync_and_directory_sync_failures_have_distinct_output_states() {
        for (phase, exists, state) in [
            ("file_sync", false, "private_unpublished"),
            ("directory_sync", true, "published"),
        ] {
            let f = Fixture::new();
            let error = copy(&f.args(false), |step, _| {
                if step == phase {
                    Err(Failure::io(
                        "injected sync failure",
                        std::io::Error::from_raw_os_error(libc::EIO),
                    ))
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
            assert_eq!(error.details.unwrap()["destination_state"], state);
            assert_eq!(f.destination.exists(), exists);
            if exists {
                assert_eq!(
                    fs::read(&f.destination).unwrap(),
                    fs::read(&f.source).unwrap()
                );
            }
        }
    }
    #[test]
    fn failed_overwrite_keeps_partial_output_and_tail() {
        let f = Fixture::new();
        fs::write(&f.destination, vec![0xa5; 9000]).unwrap();
        let error = copy(&f.args(true), |phase, target| {
            if phase == "verification" {
                target.file.write_all_at(&[0], 10).unwrap();
            }
            Ok(())
        })
        .unwrap_err();
        assert_eq!(
            error.details.unwrap()["destination_state"],
            "existing_may_be_modified"
        );
        let bytes = fs::read(&f.destination).unwrap();
        assert_eq!(bytes[10], 0);
        assert!(bytes[8199..].iter().all(|b| *b == 0xa5));
    }
    #[test]
    fn observed_source_mutation_rejects_publication() {
        let f = Fixture::new();
        let error = copy(&f.args(false), |phase, _| {
            if phase == "file_sync" {
                return Ok(());
            }
            if phase == "verification" {
                File::options()
                    .write(true)
                    .open(&f.source)
                    .unwrap()
                    .set_len(8198)
                    .unwrap();
            }
            Ok(())
        })
        .unwrap_err();
        assert!(!f.destination.exists());
        assert!(error.details.is_some());
    }
}
