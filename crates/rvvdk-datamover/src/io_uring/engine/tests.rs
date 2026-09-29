use std::fs::{File, OpenOptions};
use std::io::ErrorKind;
use std::os::unix::fs::FileExt;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use rvvdk_core::BufferPool;

use super::{faults::Enter, *};

const BLOCK: usize = 4096;
static NEXT: AtomicUsize = AtomicUsize::new(0);

fn file() -> File {
    let path = std::env::temp_dir().join(format!(
        "rvvdk-r03-{}-{}.img",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    std::fs::remove_file(path).unwrap();
    file.write_all_at(&vec![0x5a; 4 * BLOCK], 0).unwrap();
    file
}

fn setup(count: usize) -> (IoUringEngine, BufferPool, IoUringFile) {
    let file = file();
    let endpoint = IoUringFile::new(file.as_fd()).unwrap();
    let pool = BufferPool::new(count, BLOCK, BLOCK).unwrap();
    let mut engine = IoUringEngine::new(count as u32).unwrap();
    for offset in 0..count {
        engine
            .submit_owned_read(&endpoint, (offset * BLOCK) as u64, BLOCK, pool.acquire())
            .unwrap();
    }
    (engine, pool, endpoint)
}

fn bounded(name: &str, test: impl FnOnce()) {
    const CHILD: &str = "RVVDK_LIFETIME_TEST_CHILD";
    if std::env::var(CHILD).as_deref() == Ok(name) {
        test();
        return;
    }
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            &format!("io_uring::engine::tests::{name}"),
            "--nocapture",
        ])
        .env(CHILD, name)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "{name}: child failed");
            return;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("{name}: exceeded ten-second shutdown deadline");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

macro_rules! fault_test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() {
            bounded(stringify!($name), || $body);
        }
    };
}

#[test]
fn creates_engine() {
    let engine = IoUringEngine::new(8).unwrap();
    assert_eq!(engine.queue_depth(), 8);
    assert_eq!(engine.in_flight(), 0);
}

#[test]
fn rejects_zero_queue_depth() {
    assert!(matches!(
        IoUringEngine::new(0),
        Err(Error::InvalidIoUringQueueDepth)
    ));
}

#[test]
fn owned_completion_without_submission_fails() {
    let mut engine = IoUringEngine::new(8).unwrap();
    assert!(matches!(
        engine.wait_owned_completion(),
        Err(Error::IoUringNoInFlight)
    ));
}

fault_test!(
    interrupted_submit_and_wait_retry_without_releasing_buffers,
    {
        let (mut engine, pool, _) = setup(2);
        engine.faults.enter.extend([
            Enter::Before(ErrorKind::Interrupted),
            Enter::After(ErrorKind::Interrupted),
        ]);
        engine.submit().unwrap();
        assert_eq!(pool.available(), 0);
        engine
            .faults
            .enter
            .push_back(Enter::Before(ErrorKind::Interrupted));
        let done = engine.drain().unwrap();
        assert_eq!(done.len(), 2);
        assert!(done.iter().all(|c| c.buffer().iter().all(|b| *b == 0x5a)));
        assert_eq!(pool.available(), 0);
        drop(done);
        assert_eq!(pool.available(), 2);
    }
);

fault_test!(submit_errors_before_and_after_kernel_entry_keep_owners, {
    for fault in [
        Enter::Before(ErrorKind::Other),
        Enter::After(ErrorKind::Other),
    ] {
        let (mut engine, pool, endpoint) = setup(2);
        engine.faults.enter.push_back(fault);
        assert!(matches!(engine.submit(), Err(Error::Io(_))));
        assert_eq!(engine.in_flight(), 2);
        assert_eq!(pool.available(), 0);
        let spare = BufferPool::new(1, BLOCK, BLOCK).unwrap();
        assert!(matches!(
            engine.submit_owned_read(&endpoint, 0, BLOCK, spare.acquire()),
            Err(Error::IoUringEngineShutDown)
        ));
        assert_eq!(spare.available(), 1);
        engine.shutdown().unwrap();
        assert_eq!(pool.available(), 2);
        assert_eq!(engine.quarantined_operations(), 0);
    }
});

fault_test!(wait_error_retains_owners_until_shutdown_completes, {
    let (mut engine, pool, _) = setup(2);
    engine.submit().unwrap();
    engine
        .faults
        .enter
        .push_back(Enter::Before(ErrorKind::Other));
    assert!(matches!(engine.wait_owned_completion(), Err(Error::Io(_))));
    assert_eq!(pool.available(), 0);
    engine.shutdown().unwrap();
    assert_eq!(pool.available(), 2);
});

fault_test!(partial_submission_retains_submitted_and_queued_owners, {
    for fail in [false, true] {
        let (mut engine, pool, _) = setup(2);
        engine.faults.enter.push_back(Enter::Partial(fail));
        let result = engine.submit();
        if fail {
            assert!(matches!(result, Err(Error::Io(_))));
        } else {
            assert_eq!(result.unwrap(), 1);
        }
        assert_eq!(engine.in_flight(), 2);
        assert_eq!(pool.available(), 0);
        engine.shutdown().unwrap();
        assert_eq!(pool.available(), 2);
    }
});

fault_test!(unconfirmed_shutdown_retains_buffers_and_rejects_reuse, {
    let (mut engine, pool, endpoint) = setup(2);
    engine.submit().unwrap();
    engine
        .faults
        .enter
        .push_back(Enter::Before(ErrorKind::Other));
    assert!(matches!(
        engine.shutdown(),
        Err(Error::IoUringShutdownUnconfirmed { operations: 2 })
    ));
    assert_eq!(pool.available(), 0);
    assert_eq!(engine.quarantined_operations(), 2);
    assert!(matches!(
        engine.drain(),
        Err(Error::IoUringShutdownUnconfirmed { operations: 2 })
    ));
    assert!(matches!(
        engine.wait_owned_completion(),
        Err(Error::IoUringShutdownUnconfirmed { operations: 2 })
    ));
    assert!(matches!(
        engine.shutdown(),
        Err(Error::IoUringShutdownUnconfirmed { operations: 2 })
    ));
    assert!(matches!(engine.submit(), Err(Error::IoUringEngineShutDown)));
    let spare = BufferPool::new(1, BLOCK, BLOCK).unwrap();
    assert!(matches!(
        engine.submit_owned_write(&endpoint, 0, BLOCK, spare.acquire()),
        Err(Error::IoUringEngineShutDown)
    ));
    drop(engine);
    assert_eq!(pool.available(), 0);
    assert_eq!(spare.available(), 1);
});

fault_test!(drop_after_fatal_wait_keeps_unconfirmed_owners, {
    let (mut engine, pool, _) = setup(2);
    engine
        .faults
        .enter
        .push_back(Enter::Before(ErrorKind::Other));
    drop(engine);
    assert_eq!(pool.available(), 0);
});

fault_test!(unknown_or_nonfinal_completions_never_release_owners, {
    for change in [(Some(u64::MAX), None, None), (None, None, Some(2))] {
        let (mut engine, pool, _) = setup(2);
        engine.faults.completion = Some(change);
        assert!(matches!(
            engine.wait_owned_completion(),
            Err(Error::IoUringUnknownCompletion { .. })
        ));
        assert_eq!(pool.available(), 0);
        assert!(matches!(
            engine.shutdown(),
            Err(Error::IoUringShutdownUnconfirmed { operations: 2 })
        ));
        assert_eq!(pool.available(), 0);
    }
});

fault_test!(duplicate_completion_does_not_release_another_buffer, {
    let (mut engine, pool, _) = setup(2);
    let first = engine.wait_owned_completion().unwrap();
    let id = first.user_data();
    drop(first);
    assert_eq!(pool.available(), 1);
    engine.faults.completion = Some((Some(id), None, None));
    assert!(matches!(
        engine.wait_owned_completion(),
        Err(Error::IoUringUnknownCompletion { .. })
    ));
    assert!(engine.shutdown().is_err());
    assert_eq!(pool.available(), 1);
});

fault_test!(missing_completion_retains_owners_without_spinning, {
    let (mut engine, pool, _) = setup(2);
    engine.faults.hide_completions = true;
    assert!(matches!(
        engine.wait_owned_completion(),
        Err(Error::IoUringCompletionMissing)
    ));
    assert!(engine.shutdown().is_err());
    assert_eq!(pool.available(), 0);
});

fault_test!(malformed_results_release_only_the_confirmed_operation, {
    for result in [i32::MIN, (BLOCK + 1) as i32] {
        let (mut engine, pool, _) = setup(2);
        engine.faults.completion = Some((None, Some(result), None));
        assert!(matches!(
            engine.wait_owned_completion(),
            Err(Error::CorruptMetadata(_))
        ));
        assert_eq!(pool.available(), 1);
        engine.shutdown().unwrap();
        assert_eq!(pool.available(), 2);
    }
});

fault_test!(negative_cqe_does_not_prevent_draining_other_operations, {
    let read_only = File::open("/dev/zero").unwrap();
    let endpoint = IoUringFile::new(read_only.as_fd()).unwrap();
    let pool = BufferPool::new(2, BLOCK, BLOCK).unwrap();
    let mut engine = IoUringEngine::new(2).unwrap();
    for _ in 0..2 {
        engine
            .submit_owned_write(&endpoint, 0, BLOCK, pool.acquire())
            .unwrap();
    }
    assert!(matches!(engine.shutdown(), Err(Error::Io(_))));
    assert_eq!(pool.available(), 2);
    assert_eq!(engine.quarantined_operations(), 0);
});

fault_test!(queued_fd_survives_caller_close_and_descriptor_reuse, {
    let original = file();
    let original_fd = original.as_raw_fd();
    let endpoint = IoUringFile::new(original.as_fd()).unwrap();
    let pool = BufferPool::new(1, BLOCK, BLOCK).unwrap();
    let mut engine = IoUringEngine::new(1).unwrap();
    engine
        .submit_owned_read(&endpoint, 0, BLOCK, pool.acquire())
        .unwrap();
    drop(endpoint);
    drop(original);
    let replacement = File::open("/dev/zero").unwrap();
    assert_eq!(
        replacement.as_raw_fd(),
        original_fd,
        "isolated child must reuse the original FD"
    );
    let done = engine.wait_owned_completion().unwrap();
    assert!(done.buffer().iter().all(|b| *b == 0x5a));
    drop(done);
    assert_eq!(pool.available(), 1);
});

fault_test!(capacity_error_returns_only_unpublished_buffer, {
    let (mut engine, pool, endpoint) = setup(1);
    engine.submit().unwrap();
    let spare = BufferPool::new(1, BLOCK, BLOCK).unwrap();
    assert!(matches!(
        engine.submit_owned_read(&endpoint, 0, BLOCK, spare.acquire()),
        Err(Error::IoUringQueueFull { .. })
    ));
    assert_eq!(pool.available(), 0);
    assert_eq!(spare.available(), 1);
    engine.shutdown().unwrap();
    assert_eq!(pool.available(), 1);
});

fault_test!(shutdown_is_idempotent_and_returns_all_confirmed_buffers, {
    let (mut engine, pool, _) = setup(2);
    engine.shutdown().unwrap();
    engine.shutdown().unwrap();
    assert_eq!(pool.available(), 2);
    assert_eq!(engine.quarantined_operations(), 0);
    assert!(matches!(engine.submit(), Err(Error::IoUringEngineShutDown)));
});

fault_test!(user_data_wraparound_skips_live_ids, {
    let endpoint = IoUringFile::new(file().as_fd()).unwrap();
    let pool = BufferPool::new(3, BLOCK, BLOCK).unwrap();
    let mut engine = IoUringEngine::new(3).unwrap();
    let first = engine
        .submit_owned_read(&endpoint, 0, BLOCK, pool.acquire())
        .unwrap();
    assert_eq!(first, 1);
    engine.next_user_data = u64::MAX;
    let last = engine
        .submit_owned_read(&endpoint, 0, BLOCK, pool.acquire())
        .unwrap();
    assert_eq!(last, u64::MAX);
    let wrapped = engine
        .submit_owned_read(&endpoint, 0, BLOCK, pool.acquire())
        .unwrap();
    assert_eq!(wrapped, 2, "must skip the still-live operation with id 1");
    assert_eq!(pool.available(), 0);
    let completed = engine.drain().unwrap();
    assert_eq!(completed.len(), 3);
    drop(completed);
    assert_eq!(pool.available(), 3);
});

fault_test!(
    unwinding_after_submission_drains_before_returning_buffers,
    {
        let (mut engine, pool, _) = setup(2);
        engine.faults.enter.push_back(Enter::PanicAfterSubmit);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            engine.submit().unwrap();
        }));
        assert!(result.is_err());
        assert_eq!(pool.available(), 2);
    }
);

fn admission(endpoint: &IoUringFile) -> rvvdk_platform::FileAccess {
    let state = rvvdk_platform::inspect_file(endpoint.as_fd()).unwrap();
    rvvdk_platform::FileAccess::for_identity(state.device, state.inode)
}

fault_test!(
    admission_lasts_from_enqueue_until_confirmation_not_buffer_drop,
    {
        use rvvdk_platform::FileAccessKind::BufferedWrite;
        let (mut engine, pool, endpoint) = setup(1);
        let access = admission(&endpoint);
        assert!(matches!(
            access.try_acquire(0, BLOCK as u64, BufferedWrite),
            Err(Error::ConcurrentFileAccess { .. })
        ));
        engine.submit().unwrap();
        assert!(access.try_acquire(0, BLOCK as u64, BufferedWrite).is_err());
        let completed = engine.wait_owned_completion().unwrap();
        assert_eq!(pool.available(), 0);
        assert!(access.try_acquire(0, BLOCK as u64, BufferedWrite).is_ok());
        drop(completed);
        assert_eq!(pool.available(), 1);
    }
);

fault_test!(unconfirmed_shutdown_quarantines_admission_with_owners, {
    use rvvdk_platform::FileAccessKind::BufferedWrite;
    let (mut engine, _, endpoint) = setup(1);
    let access = admission(&endpoint);
    engine.faults.hide_completions = true;
    assert!(engine.shutdown().is_err());
    drop(engine);
    drop(endpoint);
    assert!(matches!(
        access.try_acquire(0, BLOCK as u64, BufferedWrite),
        Err(Error::ConcurrentFileAccess { .. })
    ));
    assert!(
        access
            .try_acquire(BLOCK as u64, BLOCK as u64, BufferedWrite)
            .is_ok()
    );
});

fault_test!(regular_file_negative_completion_releases_admission, {
    use rvvdk_platform::FileAccessKind::Flush;
    let writable = file();
    let read_only = File::open(format!("/proc/self/fd/{}", writable.as_raw_fd())).unwrap();
    // Exercise infallible OwnedFd conversion and lazy registration too.
    let endpoint = IoUringFile::from(std::os::fd::OwnedFd::from(read_only));
    let access = admission(&endpoint);
    let pool = BufferPool::new(1, BLOCK, BLOCK).unwrap();
    let mut engine = IoUringEngine::new(1).unwrap();
    engine
        .submit_owned_write(&endpoint, 0, BLOCK, pool.acquire())
        .unwrap();
    assert!(access.try_acquire(0, 0, Flush).is_err());
    assert!(matches!(engine.wait_owned_completion(), Err(Error::Io(_))));
    assert!(access.try_acquire(0, 0, Flush).is_ok());
    assert_eq!(pool.available(), 1);
    engine.shutdown().unwrap();
});
