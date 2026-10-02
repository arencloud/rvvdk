use super::*;
use crate::{
    contract::{EndpointIdentity, PinProvenance},
    ownership::JobState,
};
use std::{
    os::unix::fs::DirBuilderExt,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "rvddk-worker-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        Self(p)
    }
    fn store(&self) -> JobStore {
        JobStore::open(&self.0).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn id() -> ArtifactId {
    ArtifactId::new([3; 16]).unwrap()
}
fn source() -> SourceSelection {
    SourceSelection::new(
        EndpointIdentity::pinned(
            "https://example.invalid",
            &"a".repeat(64),
            PinProvenance::TrustOnFirstUse,
        )
        .unwrap(),
        "vm-synthetic",
        "01234567-89ab-cdef-0123-456789abcdef",
        2000,
        "synthetic",
        1024,
    )
    .unwrap()
}
#[tokio::test]
async fn cancelled_waiter_retains_lock_until_accepted_work_is_drained() {
    let f = Fixture::new();
    let (started, ready) = oneshot::channel();
    let mut started = Some(started);
    let mut worker = Worker::start_hook(f.store(), id(), source(), move |command| {
        if matches!(command, Command::Acquire) {
            started.take().unwrap().send(()).unwrap();
            std::thread::sleep(Duration::from_millis(150));
        }
        Ok(())
    });
    (&mut worker.ready).await.unwrap().unwrap();
    worker.client.apply(Command::Prepare).await.unwrap();
    let client = worker.client.clone();
    let task = tokio::spawn(async move { client.apply(Command::Acquire).await });
    ready.await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert!(matches!(JobStore::open(&f.0), Err(OwnershipError::Busy)));
    let report = worker.finish().await.unwrap();
    assert_eq!(report.state, JobState::AcquireIntent);
    assert!(JobStore::open(&f.0).is_ok());
}
#[tokio::test]
async fn worker_failure_stops_later_commands_and_returns_conservative_assessment() {
    let f = Fixture::new();
    let mut worker = Worker::start_hook(f.store(), id(), source(), |command| {
        if matches!(command, Command::Held(_)) {
            Err(OwnershipError::Io)
        } else {
            Ok(())
        }
    });
    (&mut worker.ready).await.unwrap().unwrap();
    worker.client.apply(Command::Prepare).await.unwrap();
    worker.client.apply(Command::Acquire).await.unwrap();
    assert_eq!(
        worker
            .client
            .apply(Command::Held(Zeroizing::new("synthetic".into())))
            .await,
        Err(OwnershipError::Io)
    );
    assert_eq!(
        worker.client.apply(Command::Abort).await,
        Err(OwnershipError::Uncertain)
    );
    assert_eq!(
        worker.finish().await.unwrap().state,
        JobState::AcquireIntent
    );
}
#[tokio::test]
async fn worker_panic_returns_uncertainty_and_releases_owned_handles() {
    let f = Fixture::new();
    let mut worker = Worker::start_hook(f.store(), id(), source(), |_| {
        panic!("synthetic worker panic")
    });
    (&mut worker.ready).await.unwrap().unwrap();
    assert_eq!(
        worker.client.apply(Command::Prepare).await,
        Err(OwnershipError::Uncertain)
    );
    assert!(matches!(
        worker.finish().await,
        Err(OwnershipError::Uncertain)
    ));
    assert_eq!(
        f.store().recover(id(), &source()).unwrap().state,
        JobState::Prepared
    );
}

#[tokio::test]
async fn payload_commands_bound_chunks_and_never_validate_failed_writes() {
    for size in [0, 513, CHUNK_BYTES + 1] {
        let f = Fixture::new();
        let mut worker = Worker::start(f.store(), id(), source());
        (&mut worker.ready).await.unwrap().unwrap();
        for command in [
            Command::Prepare,
            Command::Acquire,
            Command::Held(Zeroizing::new("synthetic".into())),
            Command::PayloadOpen(512),
        ] {
            worker.client.apply(command).await.unwrap();
        }
        assert!(
            worker
                .client
                .apply(Command::PayloadWrite(vec![0; size]))
                .await
                .is_err()
        );
        worker.client.apply(Command::Abort).await.unwrap();
        worker.client.apply(Command::Aborted).await.unwrap();
        let outcome = worker.finish_payload().await.unwrap();
        assert_eq!(outcome.progress.written, 0);
        assert!(!outcome.progress.verified);
        assert!(outcome.payload_error.is_some());
        assert_eq!(outcome.recovery.unwrap().state, JobState::AbortedLease);
    }
}
