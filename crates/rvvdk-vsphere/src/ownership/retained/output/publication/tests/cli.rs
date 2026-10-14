//! Opt-in executable qualification uses real transitions and test-only interruptions.
use super::*;
use crate::artifact_fixture as fixture;
use serde_json::{Value, json};
use std::{
    process::{Command, Stdio},
    time::{Duration, Instant},
};
struct Case {
    temp: Temp,
    source: SourceSelection,
    bytes: Vec<u8>,
    grains: u64,
    mode: String,
}
impl Case {
    fn new(mode: &str, grains: u64, deflated: bool) -> Self {
        let t = Temp::new();
        let pattern = fixture::bytes(1);
        let payload = if deflated {
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../rvvdk-vmdk/tests/fixtures/stream/pattern.zlib"
            ))
            .to_vec()
        } else {
            fixture::stored(&pattern)
        };
        let present = |i| {
            if grains == 32 {
                i == 0 || i == 16
            } else {
                i < 64 || (grains / 2..grains / 2 + 64).contains(&i)
            }
        };
        let records = (0..grains)
            .filter(|&i| present(i))
            .map(|i| (i, payload.clone()))
            .collect::<Vec<_>>();
        let bytes = fixture::image(true, grains, &records);
        let source = t.populate(&bytes, grains * 65536);
        let retained =
            RetainedArtifact::open(t.store(), id(), source.clone(), RetainedOptions::default())
                .unwrap();
        assert!(
            retained
                .convert_owned(
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
            source.clone(),
            RetainedOptions::default(),
        )
        .unwrap();
        let stop = match mode {
            "staged" => "before_rename",
            "renamed" => "bundle_renamed",
            "uncertain" => "record_renamed",
            "written" => "record_written",
            "synced" => "record_synced",
            _ => "never",
        };
        let r = v.publish_hook(destination(&t), RetainedOptions::default(), |s, p| {
            if p == stop && (s == OutputState::Published || matches!(mode, "staged" | "renamed")) {
                Err(OwnershipError::Io)
            } else {
                Ok(())
            }
        });
        assert_eq!(r.is_success(), mode == "published");
        let c = Self {
            temp: t,
            source,
            bytes,
            grains,
            mode: mode.into(),
        };
        c.config();
        c
    }
    fn config(&self) -> PathBuf {
        let p = self.temp.0.join("recover.json");
        let v = json!({"schema_version":1,"endpoint":"https://example.invalid","certificate_sha256":"ab".repeat(32),"pin_provenance":"trust_on_first_use","selection":{"vm_reference":"vm-synthetic","bios_uuid":"01234567-89ab-cdef-0123-456789abcdef","disk_key":2000,"backing":"synthetic","logical_bytes":self.grains*65536},"store":self.temp.jobs(),"destination":self.temp.0.join("published"),"artifact_id":"11".repeat(16),"output_id":"17".repeat(16)});
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&p)
            .unwrap();
        serde_json::to_writer(&mut f, &v).unwrap();
        p
    }
    fn command(&self) -> &'static str {
        match self.mode.as_str() {
            "staged" | "renamed" => "reconcile-intent",
            "published" | "uncertain" => "attest-published",
            _ => "acknowledge-published",
        }
    }
    fn raw(&self) -> PathBuf {
        if self.mode == "staged" {
            raw_stage(&self.temp).join("disk.raw")
        } else {
            final_path(&self.temp).join("disk.raw")
        }
    }
    fn oracle(&self) {
        let f = File::open(final_path(&self.temp).join("disk.raw")).unwrap();
        let mut buf = vec![0; 65536];
        let pattern = fixture::bytes(1);
        for i in 0..self.grains {
            f.read_exact_at(&mut buf, i * 65536).unwrap();
            let present = if self.grains == 32 {
                i == 0 || i == 16
            } else {
                i < 64 || (self.grains / 2..self.grains / 2 + 64).contains(&i)
            };
            if present {
                assert_eq!(buf, pattern)
            } else {
                assert!(buf.iter().all(|&b| b == 0));
            }
        }
        assert_eq!(
            fs::read(self.temp.stage().join(MEMBERS[0])).unwrap(),
            self.bytes
        );
    }
    fn direct(&self) -> Value {
        let s = self.temp.store();
        let d = reopen(&self.temp);
        let o = RetainedOptions::default();
        match self.command() {
            "reconcile-intent" => {
                let r = s.reconcile_publication(oid(), id(), self.source.clone(), d, o);
                assert!(r.is_success(), "{r:?}");
                serde_json::to_value(r).unwrap()
            }
            "attest-published" => {
                let r = s.attest_publication(oid(), id(), self.source.clone(), d, o);
                assert!(r.is_success(), "{r:?}");
                serde_json::to_value(r).unwrap()
            }
            _ => {
                let r =
                    s.recover_publication_acknowledgment(oid(), id(), self.source.clone(), d, o);
                assert!(r.is_success(), "{r:?}");
                serde_json::to_value(r).unwrap()
            }
        }
    }
    fn cli(&self, verb: &str, json_mode: bool) -> (i32, Value, Value) {
        let binary = std::env::var_os("RVDDK_RECOVERY_CLI").expect("explicit built CLI required");
        let stdout = self.temp.0.join("stdout");
        let stderr = self.temp.0.join("stderr");
        let file = |p: &Path| {
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(p)
                .unwrap()
        };
        let mut cmd = Command::new(binary);
        cmd.args(["recover", verb])
            .arg(self.temp.0.join("recover.json"));
        if json_mode {
            cmd.arg("--json");
        }
        let started = Instant::now();
        let mut child = cmd
            .stdout(Stdio::from(file(&stdout)))
            .stderr(Stdio::from(file(&stderr)))
            .spawn()
            .unwrap();
        let (code, usage) = wait(&mut child, Duration::from_secs(60));
        let wall = started.elapsed().as_secs_f64() * 1000.;
        let out = fs::read(stdout).unwrap();
        let err = fs::read(stderr).unwrap();
        let bytes = if code == 0 {
            assert!(err.is_empty());
            out
        } else {
            assert!(out.is_empty());
            err
        };
        for secret in [
            self.temp.0.to_string_lossy().as_ref(),
            "vm-synthetic",
            "example.invalid",
            "01234567-89ab-cdef-0123-456789abcdef",
        ] {
            assert!(!String::from_utf8_lossy(&bytes).contains(secret));
        }
        let start = if json_mode {
            0
        } else {
            bytes.iter().position(|&b| b == b'{').unwrap()
        };
        let report = serde_json::from_slice(&bytes[start..]).unwrap();
        (
            code,
            report,
            json!({"wall_ms":wall,"cpu_ms":usage.0,"peak_rss_kib":usage.1}),
        )
    }
}
fn wait(child: &mut std::process::Child, timeout: Duration) -> (i32, (f64, i64)) {
    let start = Instant::now();
    let mut status = 0;
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    loop {
        // SAFETY: live owned child PID, valid output pointers. This helper alone reaps it.
        let n = unsafe {
            libc::wait4(
                child.id() as i32,
                &mut status,
                libc::WNOHANG,
                usage.as_mut_ptr(),
            )
        };
        if n > 0 {
            break;
        }
        if n < 0 {
            assert_eq!(
                std::io::Error::last_os_error().raw_os_error(),
                Some(libc::EINTR)
            );
        }
        if start.elapsed() > timeout {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("CLI child exceeded test deadline");
        }
        std::thread::sleep(Duration::from_micros(100));
    }
    // SAFETY: successful wait4 initialized usage.
    let u = unsafe { usage.assume_init() };
    assert!(libc::WIFEXITED(status));
    (
        libc::WEXITSTATUS(status),
        (
            u.ru_utime.tv_sec as f64 * 1000.
                + u.ru_utime.tv_usec as f64 / 1000.
                + u.ru_stime.tv_sec as f64 * 1000.
                + u.ru_stime.tv_usec as f64 / 1000.,
            u.ru_maxrss,
        ),
    )
}
fn self_usage() -> (f64, i64) {
    let mut u = std::mem::MaybeUninit::<libc::rusage>::uninit();
    // SAFETY: valid writable output and read only on success.
    assert_eq!(
        unsafe { libc::getrusage(libc::RUSAGE_SELF, u.as_mut_ptr()) },
        0
    );
    // SAFETY: successful getrusage initialized output.
    let u = unsafe { u.assume_init() };
    (
        u.ru_utime.tv_sec as f64 * 1000.
            + u.ru_utime.tv_usec as f64 / 1000.
            + u.ru_stime.tv_sec as f64 * 1000.
            + u.ru_stime.tv_usec as f64 / 1000.,
        u.ru_maxrss,
    )
}
#[test]
#[ignore = "requires explicit built recovery CLI; authored executable qualification"]
fn executable_histories() {
    for mode in [
        "staged",
        "renamed",
        "published",
        "uncertain",
        "written",
        "synced",
    ] {
        for json_mode in [false, true] {
            let c = Case::new(mode, 32, false);
            let source_journal =
                fs::read(c.temp.jobs().join(record_name(id().as_bytes()))).unwrap();
            let raw = fs::metadata(c.raw()).unwrap();
            let (code, a, _) = c.cli("assess", json_mode);
            assert_eq!(code, 0);
            assert_eq!(a["status"], "assessed");
            assert_eq!(
                a["data"]["publication"]["output"]["pending_transaction"],
                matches!(mode, "written" | "synced")
            );
            let (code, r, _) = c.cli(c.command(), json_mode);
            assert_eq!(code, 0, "{r}");
            assert_eq!(r["operation_succeeded"], true);
            c.oracle();
            let after = fs::metadata(final_path(&c.temp).join("disk.raw")).unwrap();
            assert_eq!(
                (raw.ino(), raw.len(), raw.blocks()),
                (after.ino(), after.len(), after.blocks())
            );
            assert_eq!(
                fs::read(c.temp.jobs().join(record_name(id().as_bytes()))).unwrap(),
                source_journal
            );
            let (code, r, _) = c.cli(c.command(), true);
            if matches!(mode, "published" | "uncertain") {
                assert_eq!(code, 0)
            } else {
                assert_ne!(code, 0);
                assert_eq!(r["operation_succeeded"], false);
                assert_eq!(c.cli("attest-published", true).0, 0);
            }
        }
    }
}
#[test]
#[ignore = "requires explicit built recovery CLI; parameterized exclusions"]
fn executable_exclusions() {
    for mode in ["staged", "published", "written"] {
        for fault in [
            "source_txn",
            "missing_source",
            "corrupt_raw",
            "wrong_provenance",
            "unknown_config",
            "wrong_id",
            "foreign_destination",
        ] {
            let c = Case::new(mode, 32, false);
            let config = c.temp.0.join("recover.json");
            let mut v: Value = serde_json::from_slice(&fs::read(&config).unwrap()).unwrap();
            match fault {
                "source_txn" => {
                    fs::write(
                        c.temp.jobs().join(transaction_name(id().as_bytes())),
                        b"partial",
                    )
                    .unwrap();
                }
                "missing_source" => {
                    fs::remove_file(c.temp.jobs().join(record_name(id().as_bytes()))).unwrap();
                }
                "corrupt_raw" => {
                    OpenOptions::new()
                        .write(true)
                        .open(c.raw())
                        .unwrap()
                        .write_all_at(b"bad", 65536)
                        .unwrap();
                }
                "wrong_provenance" => v["pin_provenance"] = json!("externally_verified"),
                "unknown_config" => v["password"] = json!("private-sentinel"),
                "wrong_id" => v["artifact_id"] = json!("22".repeat(16)),
                "foreign_destination" => {
                    let p = c.temp.0.join("other");
                    fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
                    v["destination"] = json!(p);
                }
                _ => unreachable!(),
            }
            fs::write(config, serde_json::to_vec(&v).unwrap()).unwrap();
            let before = fs::read(c.temp.jobs().join(name(&oid().0))).unwrap();
            let (code, r, _) = c.cli(c.command(), true);
            assert_ne!(code, 0, "{mode}/{fault}");
            assert_eq!(r["operation_succeeded"], false);
            assert_eq!(
                fs::read(c.temp.jobs().join(name(&oid().0))).unwrap(),
                before
            );
            if fault == "missing_source" {
                let (code, r, _) = c.cli("assess", true);
                assert_eq!(code, 1);
                assert!(r["data"]["source"].is_null());
                assert!(r["data"]["publication"].is_object());
            }
        }
    }
}
#[test]
#[ignore = "explicit release command/direct comparisons; no overlapping workloads"]
fn command_benchmark() {
    let pairs: usize = std::env::var("RVDDK_CLI_PAIRS").unwrap().parse().unwrap();
    assert!((1..=64).contains(&pairs));
    let fixture = std::env::var("RVDDK_CLI_FIXTURE").unwrap();
    let grains = if fixture == "stored_64" {
        1024
    } else {
        assert_eq!(fixture, "deflated_256");
        4096
    };
    let mode = std::env::var("RVDDK_CLI_HISTORY").unwrap();
    let mut rows = Vec::new();
    for pair in 0..pairs {
        for cli in if pair % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let c = Case::new(&mode, grains, fixture == "deflated_256");
            let raw = fs::metadata(c.raw()).unwrap();
            let before = self_usage();
            let start = Instant::now();
            let (report, metrics) = if cli {
                let (code, r, m) = c.cli(c.command(), true);
                assert_eq!(code, 0);
                (r["data"].clone(), m)
            } else {
                let r = c.direct();
                (
                    r,
                    json!({"wall_ms":start.elapsed().as_secs_f64()*1000.,"cpu_ms":self_usage().0-before.0,"peak_rss_kib":self_usage().1}),
                )
            };
            assert_eq!(report["logical_bytes_verified"], grains * 65536);
            let after = fs::metadata(final_path(&c.temp).join("disk.raw")).unwrap();
            assert_eq!(
                (raw.ino(), raw.len(), raw.blocks()),
                (after.ino(), after.len(), after.blocks())
            );
            c.oracle();
            assert_eq!(
                after.blocks(),
                fs::metadata(final_path(&c.temp).join("disk.raw"))
                    .unwrap()
                    .blocks()
            );
            rows.push(json!({"pair":pair,"cli":cli,"fixture":fixture,"history":mode,"metrics":metrics,"logical_bytes":grains*65536,"allocated_bytes":after.blocks()*512,"oracle_passed":true,"allocation_unchanged":true,"report":report}));
        }
    }
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(std::env::var_os("RVDDK_CLI_REPORT").unwrap())
        .unwrap();
    serde_json::to_writer_pretty(&mut f, &rows).unwrap();
    writeln!(f).unwrap();
}
// Private authored-fixture export for actual-binary strace/signal qualification.
#[test]
#[ignore = "exports only authored fixture into an explicitly selected private parent"]
fn prepare_fixture() {
    let mode = std::env::var("RVDDK_CLI_HISTORY").unwrap();
    let grains = std::env::var("RVDDK_CLI_GRAINS")
        .unwrap_or("1024".into())
        .parse()
        .unwrap();
    let c = Case::new(&mode, grains, false);
    fs::write(
        std::env::var_os("RVDDK_CLI_HANDOFF").unwrap(),
        c.temp.0.as_os_str().as_encoded_bytes(),
    )
    .unwrap();
    std::mem::forget(c);
}
#[test]
#[ignore = "requires explicit built CLI; remaining reconciliation candidate origins"]
fn executable_reconciliation_candidates() {
    for origin in ["staged", "renamed"] {
        for point in ["record_written", "record_synced"] {
            let mut c = Case::new(origin, 32, false);
            let r = c.temp.store().reconcile_hook(
                oid(),
                id(),
                c.source.clone(),
                reopen(&c.temp),
                RetainedOptions::default(),
                &mut super::super::reconciliation::SystemIo,
                |s, p| {
                    if s == OutputState::Published && p == point {
                        Err(OwnershipError::Io)
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(!r.is_success());
            c.mode = "written".into();
            let candidate = c.temp.jobs().join(txn(&oid().0));
            let bytes = fs::read(&candidate).unwrap();
            let inode = fs::metadata(&candidate).unwrap().ino();
            assert_eq!(c.cli(c.command(), true).0, 0);
            assert!(!candidate.exists());
            let current = c.temp.jobs().join(name(&oid().0));
            assert_eq!(fs::read(&current).unwrap(), bytes);
            assert_eq!(fs::metadata(current).unwrap().ino(), inode);
            c.oracle();
        }
    }
}
