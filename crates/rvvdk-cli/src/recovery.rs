//! Explicit local adapters; no network, state dispatch, retry or cleanup.
mod config;
#[cfg(test)]
pub(crate) mod profile;
mod report;
use report::{Envelope, Status};
use rvvdk_vsphere::{
    Cancellation,
    ownership::{JobStore, OutputId, OwnershipError, PublicationDirectory, RetainedOptions},
};
use std::{
    io::Write,
    panic::{AssertUnwindSafe, catch_unwind},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operation {
    Assess,
    Reconcile,
    Attest,
    Acknowledge,
}
impl Operation {
    fn name(self) -> &'static str {
        match self {
            Self::Assess => "assess",
            Self::Reconcile => "reconcile-intent",
            Self::Attest => "attest-published",
            Self::Acknowledge => "acknowledge-published",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        match s {
            "assess" => Some(Self::Assess),
            "reconcile-intent" => Some(Self::Reconcile),
            "attest-published" => Some(Self::Attest),
            "acknowledge-published" => Some(Self::Acknowledge),
            _ => None,
        }
    }
}
struct Control {
    token: Cancellation,
    requested: Arc<AtomicBool>,
    started: Instant,
}
impl Control {
    fn check(&self, timeout: Duration) -> Result<Duration, OwnershipError> {
        if self.requested.load(Ordering::Relaxed) {
            return Err(OwnershipError::Cancelled);
        }
        timeout
            .checked_sub(self.started.elapsed())
            .filter(|d| !d.is_zero())
            .ok_or(OwnershipError::Deadline)
    }
    fn cancel(&self) {
        self.requested.store(true, Ordering::Relaxed);
        self.token.cancel();
    }
}
fn dispatch(op: Operation, path: &Path, c: &Control, e: &mut Envelope) -> Result<Status, Status> {
    c.check(Duration::from_secs(3600))
        .map_err(report::error_status)?;
    let r = config::read(path)?;
    #[cfg(test)]
    profile::mark("config");
    c.check(r.timeout).map_err(report::error_status)?;
    let store = JobStore::open(&r.store).map_err(report::error_status)?;
    c.check(r.timeout).map_err(report::error_status)?;
    let dest = PublicationDirectory::open(&r.destination).map_err(report::error_status)?;
    let timeout = c.check(r.timeout).map_err(report::error_status)?;
    #[cfg(test)]
    profile::mark("locks");
    e.operation_started = true;
    if op == Operation::Assess {
        let source = store.recover(r.artifact, &r.source);
        let publication = c
            .check(r.timeout)
            .and_then(|_| store.assess_publication(r.output, r.artifact, &r.source, &dest));
        let control = c.check(r.timeout);
        e.data = serde_json::json!({"source":source.as_ref().ok().map(report::source),"source_error":source.as_ref().err(),"publication":report::observation(publication.as_ref().ok()),"publication_error":publication.as_ref().err(),"control_error":control.as_ref().err()});
        let status = report::errors(
            source
                .err()
                .into_iter()
                .chain(publication.err())
                .chain(control.err()),
        )
        .unwrap_or(Status::Assessed);
        e.operation_succeeded = status == Status::Assessed;
        Ok(status)
    } else {
        let options = RetainedOptions {
            cancellation: c.token.clone(),
            timeout,
        };
        Ok(match op {
            Operation::Reconcile => report::complete(
                store.reconcile_publication(r.output, r.artifact, r.source, dest, options),
                e,
            ),
            Operation::Attest => report::complete(
                store.attest_publication(r.output, r.artifact, r.source, dest, options),
                e,
            ),
            Operation::Acknowledge => report::complete(
                store.recover_publication_acknowledgment(
                    r.output, r.artifact, r.source, dest, options,
                ),
                e,
            ),
            Operation::Assess => unreachable!(),
        })
    }
}
// Cancellation on relay unwind ensures a failed predicate cannot strand accepted
// work silently. Caller predicates are required to be cheap and non-panicking.
struct RelayGuard<'a>(&'a Control);
impl Drop for RelayGuard<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            self.0.cancel();
        }
    }
}
fn controlled<C: rvvdk_datamover::Cancellation>(
    c: &C,
    work: impl FnOnce(&Control, &mut Envelope) -> Result<Status, Status>,
    e: &mut Envelope,
) -> Status {
    let control = Control {
        token: Cancellation::default(),
        requested: Arc::new(AtomicBool::new(false)),
        started: Instant::now(),
    };
    let result = catch_unwind(AssertUnwindSafe(|| {
        if c.is_cancelled() {
            control.cancel();
        }
        std::thread::scope(|scope| {
            let (stop, wait) = mpsc::channel::<()>();
            let relay_control = &control;
            #[cfg(test)]
            if tests::FAIL_SPAWN.with(|v| v.replace(false)) {
                return Status::Failed;
            }
            let handle = std::thread::Builder::new()
                .name("rvddk-cancel".into())
                .spawn_scoped(scope, move || {
                    let _guard = RelayGuard(relay_control);
                    loop {
                        if c.is_cancelled() {
                            relay_control.cancel();
                        }
                        match wait.recv_timeout(Duration::from_millis(10)) {
                            Err(mpsc::RecvTimeoutError::Timeout) => {}
                            _ => break,
                        }
                    }
                });
            let handle = match handle {
                Ok(h) => h,
                Err(_) => return Status::Failed,
            };
            #[cfg(test)]
            profile::mark("relay_start");
            let result = catch_unwind(AssertUnwindSafe(|| work(&control, e)));
            #[cfg(test)]
            profile::mark("operation");
            drop(stop); // Wake and join before scope exit, including dispatch unwind.
            if handle.join().is_err() {
                return Status::Uncertain;
            }
            #[cfg(test)]
            profile::mark("relay_join");
            match result {
                Ok(Ok(s)) | Ok(Err(s)) => s,
                Err(_) => Status::Uncertain,
            }
        })
    }));
    result.unwrap_or(Status::Uncertain)
}
pub(crate) fn run<C: rvvdk_datamover::Cancellation>(
    args: &clap::ArgMatches,
    json: bool,
    out: &mut impl Write,
    err: &mut impl Write,
    c: &C,
) -> i32 {
    let (name, args) = args.subcommand().expect("required recovery verb");
    let op = Operation::parse(name).expect("closed recovery verbs");
    let path = args.get_one::<std::path::PathBuf>("config").unwrap();
    let mut e = Envelope::new(Some(op));
    let status = controlled(c, |c, e| dispatch(op, path, c, e), &mut e);
    let code = e.emit(status, json, out, err);
    #[cfg(test)]
    profile::mark("report");
    code
}
pub(crate) fn usage(json: bool, out: &mut impl Write, err: &mut impl Write) -> i32 {
    Envelope::new(None).emit(Status::Invalid, json, out, err)
}
#[cfg(test)]
mod tests;
