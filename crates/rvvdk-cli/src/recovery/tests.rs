use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::PathBuf,
    sync::atomic::AtomicU64,
};
pub(super) struct Temp(pub PathBuf);
impl Temp {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "rvddk-cli-recovery-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        Self(p)
    }
    pub fn file(&self, bytes: &[u8]) -> PathBuf {
        let p = self.0.join("config");
        let mut f = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&p)
            .unwrap();
        f.write_all(bytes).unwrap();
        p
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn control_precancel_and_deadline_prevent_dispatch() {
    let cancel = rvvdk_datamover::CancellationToken::default();
    cancel.cancel();
    let mut e = Envelope::new(Some(Operation::Assess));
    let s = controlled(
        &cancel,
        |c, e| dispatch(Operation::Assess, Path::new("/not-read"), c, e),
        &mut e,
    );
    assert_eq!(s, Status::Cancelled);
    assert!(!e.operation_started);
    let c = Control {
        token: Cancellation::default(),
        requested: Arc::new(AtomicBool::new(false)),
        started: Instant::now() - Duration::from_secs(2),
    };
    assert_eq!(
        c.check(Duration::from_secs(1)),
        Err(OwnershipError::Deadline)
    );
}
#[test]
fn relay_observes_cancellation_and_joins_after_dispatch_unwind() {
    let token = rvvdk_datamover::CancellationToken::default();
    let mut e = Envelope::new(Some(Operation::Attest));
    let s = controlled(
        &token,
        |c, e| {
            e.operation_started = true;
            token.cancel();
            let until = Instant::now() + Duration::from_secs(2);
            while c.check(Duration::from_secs(10)).is_ok() {
                assert!(Instant::now() < until);
                std::thread::yield_now();
            }
            Err(Status::Cancelled)
        },
        &mut e,
    );
    assert_eq!(s, Status::Cancelled);
    assert!(e.operation_started);
    let s = controlled(
        &rvvdk_datamover::NoCancellation,
        |_, e| {
            e.operation_started = true;
            panic!("synthetic dispatch unwind")
        },
        &mut e,
    );
    assert_eq!(s, Status::Uncertain);
}
#[test]
fn relay_panic_requests_cancellation_and_is_not_success() {
    struct Panics(AtomicU64);
    impl rvvdk_datamover::Cancellation for Panics {
        fn is_cancelled(&self) -> bool {
            if self.0.fetch_add(1, Ordering::Relaxed) > 0 {
                panic!("synthetic relay unwind");
            }
            false
        }
    }
    let mut e = Envelope::new(Some(Operation::Attest));
    let s = controlled(
        &Panics(AtomicU64::new(0)),
        |c, e| {
            let until = Instant::now() + Duration::from_secs(2);
            while c.check(Duration::from_secs(10)).is_ok() {
                assert!(Instant::now() < until);
                std::thread::yield_now();
            }
            e.operation_succeeded = true;
            Ok(Status::Completed)
        },
        &mut e,
    );
    assert_eq!(s, Status::Uncertain);
    assert!(e.operation_succeeded);
}
#[test]
fn request_routing_redacts_usage_and_preserves_delimiter() {
    for args in [
        vec!["rvddk", "--json", "recover", "private-sentinel"],
        vec![
            "rvddk",
            "recover",
            "assess",
            "/private-sentinel",
            "--password",
            "private-sentinel",
            "--json",
        ],
        vec!["rvddk", "recover", "assess", "--json"],
    ] {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(crate::run(args, &mut out, &mut err), 2);
        assert!(out.is_empty());
        assert!(
            !String::from_utf8(err.clone())
                .unwrap()
                .contains("private-sentinel")
        );
        let v: serde_json::Value = serde_json::from_slice(&err).unwrap();
        assert_eq!(v["status"], "invalid_request");
    }
    for args in [
        vec!["rvddk", "--json", "recover", "--help"],
        vec!["rvddk", "recover", "assess", "--help"],
    ] {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(crate::run(args, &mut out, &mut err), 0);
        assert!(err.is_empty());
        assert!(String::from_utf8(out).unwrap().contains("Usage:"));
    }
    let t = Temp::new();
    let p = t.file(b"{}");
    let args = vec![
        "rvddk".into(),
        "recover".into(),
        "assess".into(),
        "--json".into(),
        "--".into(),
        p.into_os_string(),
    ];
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(crate::run(args, &mut out, &mut err), 2);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&err).unwrap()["status"],
        "invalid_request"
    );
}
#[test]
fn second_lock_failure_releases_first_and_reports_no_dispatch() {
    let t = Temp::new();
    let store = t.0.join("store");
    let dest = t.0.join("dest");
    for p in [&store, &dest] {
        fs::DirBuilder::new().mode(0o700).create(p).unwrap();
    }
    let mut v = super::config::tests::valid();
    v["store"] = serde_json::json!(store);
    v["destination"] = serde_json::json!(dest);
    let p = t.file(&serde_json::to_vec(&v).unwrap());
    let lock = JobStore::open(&dest).unwrap();
    let mut e = Envelope::new(Some(Operation::Assess));
    let s = controlled(
        &rvvdk_datamover::NoCancellation,
        |c, e| dispatch(Operation::Assess, &p, c, e),
        &mut e,
    );
    assert_eq!(s, Status::Failed);
    assert!(!e.operation_started);
    assert!(JobStore::open(&store).is_ok());
    drop(lock);
}
thread_local! {pub(super) static FAIL_SPAWN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };}
#[test]
fn injected_spawn_failure_does_not_enter_work() {
    FAIL_SPAWN.with(|v| v.set(true));
    let mut e = Envelope::new(Some(Operation::Assess));
    assert_eq!(
        controlled(
            &rvvdk_datamover::NoCancellation,
            |_, _| panic!("must not dispatch"),
            &mut e
        ),
        Status::Failed
    );
    assert!(!e.operation_started);
}
#[test]
#[ignore = "explicit authored fixture; release adapter phases and recording controls"]
fn adapter_profile() {
    let path = std::env::var_os("RVDDK_CLI_CONFIG").unwrap();
    let pairs: usize = std::env::var("RVDDK_CLI_PAIRS").unwrap().parse().unwrap();
    assert!((1..=64).contains(&pairs));
    let mut rows = Vec::new();
    for pair in 0..pairs {
        for recording in if pair % 2 == 0 {
            [false, true]
        } else {
            [true, false]
        } {
            let (mut out, mut err) = (Vec::new(), Vec::new());
            profile::start(recording);
            let before = profile::usage();
            let started = Instant::now();
            let code = crate::run(
                vec![
                    "rvddk".into(),
                    "recover".into(),
                    "attest-published".into(),
                    path.clone(),
                    "--json".into(),
                ],
                &mut out,
                &mut err,
            );
            let wall = started.elapsed().as_secs_f64() * 1000.;
            let after = profile::usage();
            let phases = profile::finish();
            assert_eq!(code, 0);
            assert!(err.is_empty());
            let r: serde_json::Value = serde_json::from_slice(&out).unwrap();
            assert_eq!(r["operation_succeeded"], true);
            rows.push(serde_json::json!({"pair":pair,"recording":recording,"wall_ms":wall,"cpu_ms":after.0-before.0,"peak_rss_kib":after.1,"phases":phases}));
        }
    }
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(std::env::var_os("RVDDK_CLI_REPORT").unwrap())
        .unwrap();
    serde_json::to_writer_pretty(file, &rows).unwrap();
}
