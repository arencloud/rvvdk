use std::process::Command;
use std::sync::{Condvar, mpsc};
use std::time::{Duration, Instant};

use rvvdk_core::{Capabilities, DiskGeometry, Error, Extent};

use super::*;

const BLOCK: usize = 4096;
const CHILD: &str = "RVVDK_SHUTDOWN_TEST_CHILD";

// A regression must fail within a deadline, never leave a hung worker in the
// test runner. Each child runs just this test and is killed/reaped on timeout.
fn bounded(name: &str, test: impl FnOnce()) {
    if std::env::var(CHILD).as_deref() == Ok(name) {
        test();
        return;
    }
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            &format!("concurrent::tests::{name}"),
            "--nocapture",
        ])
        .env(CHILD, name)
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "{name}: child failed: {status}");
            return;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("{name}: worker shutdown exceeded five seconds");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Fault {
    None,
    Read,
    Write,
    FirstWrite,
    SecondWrite,
    Zero,
    Discard,
    Panic,
}

struct Gate {
    entered: mpsc::Sender<()>,
    released: Mutex<bool>,
    ready: Condvar,
}

impl Gate {
    fn release(&self) {
        *self.released.lock().unwrap() = true;
        self.ready.notify_all();
    }

    fn wait(&self) {
        self.entered.send(()).unwrap();
        let released = self.released.lock().unwrap();
        drop(
            self.ready
                .wait_while(released, |released| !*released)
                .unwrap(),
        );
    }
}

struct FaultDisk<'a> {
    fault: Fault,
    gate: Option<&'a Gate>,
}

impl FaultDisk<'_> {
    fn check(&self, operation: Fault) -> Result<()> {
        if self.fault == operation || self.fault == Fault::Panic {
            if let Some(gate) = self.gate {
                gate.wait();
            }
            assert!(self.fault != Fault::Panic, "injected worker panic");
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "injected backend failure",
            )));
        }
        Ok(())
    }
}

impl VirtualDisk for FaultDisk<'_> {
    fn geometry(&self) -> DiskGeometry {
        DiskGeometry::new((1024 * BLOCK) as u64, 512, BLOCK as u32).unwrap()
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::READ
            | Capabilities::WRITE
            | Capabilities::WRITE_ZERO
            | Capabilities::DISCARD
            | Capabilities::FLUSH
    }
    fn read_at(&self, _: u64, buffer: &mut [u8]) -> Result<usize> {
        self.check(Fault::Read)?;
        buffer.fill(0x5a);
        Ok(buffer.len())
    }
    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize> {
        if self.fault == Fault::SecondWrite {
            self.gate.unwrap().wait();
            return if offset == 0 {
                Ok(buffer.len())
            } else {
                Err(Error::Io(std::io::Error::from_raw_os_error(13)))
            };
        }
        if self.fault == Fault::FirstWrite {
            if offset == 0 {
                return FaultDisk {
                    fault: Fault::Write,
                    gate: None,
                }
                .check(Fault::Write)
                .map(|()| buffer.len());
            }
            // Healthy peers must stop rather than draining this slow workload.
            std::thread::sleep(Duration::from_millis(1));
        }
        self.check(Fault::Write)?;
        Ok(buffer.len())
    }
    fn write_zero_at(&self, _: u64, _: u64) -> Result<()> {
        self.check(Fault::Zero)
    }
    fn discard(&self, _: u64, _: u64) -> Result<()> {
        self.check(Fault::Discard)
    }
    fn flush(&self) -> Result<()> {
        panic!("failed copy must not flush")
    }
    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
        Ok(vec![Extent::new(
            offset,
            length,
            rvvdk_core::ExtentKind::Data,
        )?])
    }
}

fn assert_backend_error<T: std::fmt::Debug>(result: Result<T>) {
    let error = result.unwrap_err();
    let cause = &error.copy_failure().expect("copy failure context").cause;
    match cause {
        Error::Io(error) => {
            assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
            assert_eq!(error.to_string(), "injected backend failure");
        }
        error => panic!("original backend error was replaced: {error}"),
    }
}

fn full_queue_failure(fault: Fault, kind: WorkKind) {
    for workers in [2, 4] {
        for capacity in [1, 4] {
            let (entered, waiting) = mpsc::channel();
            let gate = Gate {
                entered,
                released: Mutex::new(false),
                ready: Condvar::new(),
            };
            let source = FaultDisk {
                fault: if fault == Fault::Read {
                    fault
                } else {
                    Fault::None
                },
                gate: Some(&gate),
            };
            let destination = FaultDisk {
                fault: if fault == Fault::Read {
                    Fault::None
                } else {
                    fault
                },
                gate: Some(&gate),
            };
            let pool = BufferPool::new(workers, BLOCK, BLOCK).unwrap();
            let work = WorkItem::new(0, BLOCK, kind);
            let result = execute(&source, &destination, workers, capacity, &pool, |sender| {
                for _ in 0..workers {
                    sender.send(work).unwrap();
                }
                for _ in 0..workers {
                    waiting.recv().unwrap();
                }
                // All workers are inside the failing backend operation. Fill
                // the queue and prove the next send would block before release.
                for _ in 0..capacity {
                    sender.try_send(work).unwrap();
                }
                assert!(matches!(
                    sender.try_send(work),
                    Err(mpsc::TrySendError::Full(_))
                ));
                gate.release();
                sender.send(work).map_err(|_| Error::WorkQueueClosed)
            });
            assert_backend_error(result);
            assert_eq!(pool.available(), workers, "all buffer guards must return");
        }
    }
}

#[test]
fn read_failure_wakes_full_queue() {
    bounded("read_failure_wakes_full_queue", || {
        full_queue_failure(Fault::Read, WorkKind::Copy)
    });
}

#[test]
fn write_failure_wakes_full_queue() {
    bounded("write_failure_wakes_full_queue", || {
        full_queue_failure(Fault::Write, WorkKind::Copy)
    });
}

#[test]
fn zero_failure_wakes_full_queue() {
    bounded("zero_failure_wakes_full_queue", || {
        full_queue_failure(Fault::Zero, WorkKind::Zero)
    });
}

#[test]
fn discard_failure_wakes_full_queue() {
    bounded("discard_failure_wakes_full_queue", || {
        full_queue_failure(Fault::Discard, WorkKind::Discard)
    });
}

#[test]
fn error_returns_buffers_with_more_workers_than_buffers() {
    bounded(
        "error_returns_buffers_with_more_workers_than_buffers",
        || {
            let source = FaultDisk {
                fault: Fault::None,
                gate: None,
            };
            let destination = FaultDisk {
                fault: Fault::Write,
                gate: None,
            };
            let pool = BufferPool::new(1, BLOCK, BLOCK).unwrap();
            let result = execute(&source, &destination, 8, 1, &pool, |sender| {
                for _ in 0..1024 {
                    sender
                        .send(WorkItem::new(0, BLOCK, WorkKind::Copy))
                        .map_err(|_| Error::WorkQueueClosed)?;
                }
                Ok(())
            });
            assert_backend_error(result);
            assert_eq!(pool.available(), 1);
        },
    );
}

#[test]
fn producer_error_wakes_idle_workers() {
    bounded("producer_error_wakes_idle_workers", || {
        let disk = FaultDisk {
            fault: Fault::None,
            gate: None,
        };
        let pool = BufferPool::new(2, BLOCK, BLOCK).unwrap();
        let result = execute(&disk, &disk, 2, 1, &pool, |_| Err(Error::Unsupported));
        assert!(matches!(
            result.unwrap_err().copy_failure().unwrap().cause,
            Error::Unsupported
        ));
        assert_eq!(pool.available(), 2);
    });
}

#[test]
fn worker_panics_unblock_producer_and_propagate() {
    bounded("worker_panics_unblock_producer_and_propagate", || {
        let disk = FaultDisk {
            fault: Fault::Panic,
            gate: None,
        };
        let pool = BufferPool::new(2, BLOCK, BLOCK).unwrap();
        let result = std::panic::catch_unwind(|| {
            execute(&disk, &disk, 2, 1, &pool, |sender| {
                for _ in 0..1024 {
                    sender
                        .send(WorkItem::new(0, BLOCK, WorkKind::Copy))
                        .map_err(|_| Error::WorkQueueClosed)?;
                }
                Ok(())
            })
        });
        assert!(result.is_err());
        assert_eq!(pool.available(), 2);
    });
}

#[test]
fn one_worker_failure_stops_healthy_peers() {
    bounded("one_worker_failure_stops_healthy_peers", || {
        let source = FaultDisk {
            fault: Fault::None,
            gate: None,
        };
        let destination = FaultDisk {
            fault: Fault::FirstWrite,
            gate: None,
        };
        let pool = BufferPool::new(4, BLOCK, BLOCK).unwrap();
        let result = execute(&source, &destination, 4, 64, &pool, |sender| {
            for block in 0..100_000 {
                sender
                    .send(WorkItem::new(block * BLOCK as u64, BLOCK, WorkKind::Copy))
                    .map_err(|_| Error::WorkQueueClosed)?;
            }
            Ok(())
        });
        assert_backend_error(result);
        assert_eq!(pool.available(), 4);
    });
}

#[test]
fn public_copy_preserves_worker_error_without_flushing() {
    bounded(
        "public_copy_preserves_worker_error_without_flushing",
        || {
            let source = FaultDisk {
                fault: Fault::None,
                gate: None,
            };
            let destination = FaultDisk {
                fault: Fault::Write,
                gate: None,
            };
            let options = crate::CopyOptions::with_execution(BLOCK, BLOCK, 2, 1).unwrap();
            assert_backend_error(crate::DataMover::new(options).copy(&source, &destination));
        },
    );
}

#[test]
fn first_recorded_error_survives_later_errors() {
    let failure = Failure::default();
    failure.record(Error::Unsupported);
    failure.record(Error::WorkQueueClosed);
    failure.record(Error::Io(std::io::Error::other("later backend failure")));
    assert!(failure.is_stopped());
    assert!(matches!(
        failure.first.into_inner().unwrap(),
        Some(Error::Unsupported)
    ));
}

#[test]
fn successful_producer_drains_queued_work() {
    bounded("successful_producer_drains_queued_work", || {
        let disk = FaultDisk {
            fault: Fault::None,
            gate: None,
        };
        let pool = BufferPool::new(2, BLOCK, BLOCK).unwrap();
        let stats = execute(&disk, &disk, 2, 64, &pool, |sender| {
            for _ in 0..1000 {
                sender
                    .send(WorkItem::new(0, BLOCK, WorkKind::Copy))
                    .unwrap();
            }
            Ok(())
        })
        .unwrap();
        assert_eq!(stats.blocks_copied, 1000);
        assert_eq!(stats.bytes_read, 1000 * BLOCK as u64);
        assert_eq!(stats.bytes_written, 1000 * BLOCK as u64);
        assert_eq!(pool.available(), 2);
    });
}

#[test]
fn failed_copy_aggregates_healthy_and_failing_workers_after_join() {
    bounded(
        "failed_copy_aggregates_healthy_and_failing_workers_after_join",
        || {
            let (entered, waiting) = mpsc::channel();
            let gate = Gate {
                entered,
                released: Mutex::new(false),
                ready: Condvar::new(),
            };
            let source = FaultDisk {
                fault: Fault::None,
                gate: None,
            };
            let destination = FaultDisk {
                fault: Fault::SecondWrite,
                gate: Some(&gate),
            };
            let pool = BufferPool::new(2, BLOCK, BLOCK).unwrap();
            let error = execute(&source, &destination, 2, 2, &pool, |sender| {
                sender
                    .send(WorkItem::new(0, BLOCK, WorkKind::Copy))
                    .unwrap();
                sender
                    .send(WorkItem::new(BLOCK as u64, BLOCK, WorkKind::Copy))
                    .unwrap();
                waiting.recv().unwrap();
                waiting.recv().unwrap();
                gate.release();
                Ok(())
            })
            .unwrap_err();
            let f = error.copy_failure().unwrap();
            assert_eq!(f.operation, CopyOperation::Write);
            assert_eq!(f.range.unwrap().offset(), BLOCK as u64);
            assert_eq!(
                (
                    f.progress.bytes_read,
                    f.progress.bytes_written,
                    f.progress.blocks_completed
                ),
                (8192, 4096, 1)
            );
            assert_eq!(f.progress.extents_completed, None);
            assert!(f.progress.unconfirmed_io);
            assert_eq!(pool.available(), 2);
        },
    );
}

#[test]
fn first_producer_error_keeps_uncertainty_from_a_later_worker_failure() {
    let failure = Failure::default();
    failure.record(Error::Unsupported);
    failure.record(work_failure(
        CopyOperation::Write,
        WorkItem::new(4096, BLOCK, WorkKind::Copy),
        Error::WriteZero {
            offset: 4096,
            remaining: BLOCK,
        },
    ));
    let progress = CopyProgress {
        unconfirmed_io: failure.unconfirmed_io.load(Ordering::Relaxed),
        ..CopyProgress::default()
    };
    let error =
        crate::failure::worker_total(failure.first.into_inner().unwrap().unwrap(), progress);
    let f = error.copy_failure().unwrap();
    assert_eq!(f.operation, CopyOperation::Schedule);
    assert!(matches!(f.cause, Error::Unsupported));
    assert!(f.progress.unconfirmed_io);
}
