use crate::control::{Checkpoint, Lifecycle};
use crate::{
    Cancellation, CopyObserver, CopyPhase, CopyPlan, CopyReport, CopyStats, DataMover,
    ExecutionBackend,
};
use rvvdk_core::{BufferPool, CopyOperation, CopyProgress, Error, Result, VirtualDisk};
use std::time::Instant;

impl DataMover {
    /// Execute with coordinator lifecycle events and cooperative cancellation.
    /// Preparation errors also produce a terminal event. Callbacks must not panic.
    pub fn execute_plan_controlled<
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
        C: Cancellation,
        O: CopyObserver + ?Sized,
    >(
        &self,
        plan: &CopyPlan,
        source: &S,
        destination: &D,
        cancellation: &C,
        observer: &O,
    ) -> Result<CopyReport> {
        let life = Lifecycle::new(cancellation, observer, plan.logical_bytes());
        lifecycle(&life, || {
            life.cancelled()?;
            self.prepare_portable(plan, source, destination)?;
            life.backend.set(Some(ExecutionBackend::Threaded));
            life.emit(CopyPhase::Started, CopyProgress::default());
            life.cancelled()?;
            self.threaded_controlled(plan, source, destination, &life)
        })
    }

    #[cfg(target_os = "linux")]
    pub fn execute_raw_plan_controlled<S, D, C: Cancellation, O: CopyObserver + ?Sized>(
        &self,
        plan: &CopyPlan,
        source: &rvvdk_core::RawDisk<S>,
        destination: &rvvdk_core::RawDisk<D>,
        cancellation: &C,
        observer: &O,
    ) -> Result<CopyReport>
    where
        S: rvvdk_core::BlockDevice + rvvdk_platform::LinuxFdBackend,
        D: rvvdk_core::BlockDevice + rvvdk_platform::LinuxFdBackend,
    {
        use crate::preparation::PreparedExecutor;
        let life = Lifecycle::new(cancellation, observer, plan.logical_bytes());
        lifecycle(&life, || {
            life.cancelled()?;
            let prepared = self.prepare_raw(plan, source, destination)?;
            let (plan, source, destination, executor) = prepared.parts();
            let backend = match &executor {
                PreparedExecutor::IoUring { .. } => ExecutionBackend::IoUring,
                _ => ExecutionBackend::Threaded,
            };
            life.backend.set(Some(backend));
            life.emit(CopyPhase::Started, CopyProgress::default());
            life.cancelled()?;
            match executor {
                PreparedExecutor::Threaded => {
                    self.threaded_controlled(plan, source, destination, &life)
                }
                PreparedExecutor::NativeFallback(reason) => self
                    .threaded_controlled(plan, source, destination, &life)
                    .map(|r| r.with_runtime_fallback(Some(reason))),
                PreparedExecutor::IoUring {
                    native_plan,
                    resources,
                    setup_elapsed,
                } => {
                    let started = Instant::now();
                    let stats = resources.execute_controlled(destination, &native_plan, &life)?;
                    life.flushing(stats.progress()).map_err(|e| {
                        crate::failure::execution(
                            "io_uring",
                            CopyOperation::Cancel,
                            None,
                            stats.progress(),
                            e,
                        )
                    })?;
                    destination.flush().map_err(|e| {
                        crate::failure::execution(
                            "io_uring",
                            CopyOperation::Flush,
                            None,
                            stats.progress(),
                            e,
                        )
                    })?;
                    Ok(CopyReport::new(
                        ExecutionBackend::IoUring,
                        CopyStats::from_io_uring_extents(stats, setup_elapsed + started.elapsed()),
                    ))
                }
            }
        })
    }

    fn threaded_controlled<S: VirtualDisk + ?Sized, D: VirtualDisk + ?Sized, C: Checkpoint>(
        &self,
        plan: &CopyPlan,
        source: &S,
        destination: &D,
        control: &C,
    ) -> Result<CopyReport> {
        let started = Instant::now();
        let stats = if self.options.concurrency() == 1 {
            crate::sequential::execute_controlled(
                source,
                destination,
                plan.extents(),
                self.options,
                started,
                &mut (),
                control,
            )?
        } else {
            let pool = BufferPool::new(
                self.options.buffer_count(),
                self.options.block_size(),
                self.options.buffer_alignment(),
            )
            .map_err(|e| {
                crate::failure::execution(
                    "threaded",
                    CopyOperation::Allocate,
                    None,
                    CopyProgress::default(),
                    e,
                )
            })?;
            let stats = crate::concurrent::execute_controlled(
                source,
                destination,
                self.options.concurrency(),
                self.options.queue_capacity(),
                &pool,
                control,
                |sender, checkpoint| {
                    for extent in plan.extents() {
                        for work in
                            crate::planner::ExtentWorkIter::new(*extent, self.options.block_size())
                        {
                            checkpoint()?;
                            sender.send(work?).map_err(|_| Error::WorkQueueClosed)?;
                        }
                    }
                    Ok(())
                },
            )?;
            let mut p = stats.progress();
            p.extents_completed = Some(plan.extent_count() as u64);
            control.flushing(p).map_err(|e| {
                crate::failure::execution("threaded", CopyOperation::Cancel, None, p, e)
            })?;
            destination.flush().map_err(|e| {
                crate::failure::execution("threaded", CopyOperation::Flush, None, p, e)
            })?;
            CopyStats::new(
                stats.bytes_read,
                stats.bytes_written,
                stats.bytes_zeroed,
                stats.bytes_discarded,
                stats.blocks_copied,
                plan.extent_count() as u64,
                started.elapsed(),
            )
        };
        Ok(CopyReport::new(ExecutionBackend::Threaded, stats))
    }
}
fn lifecycle<C: Cancellation, O: CopyObserver + ?Sized>(
    life: &Lifecycle<'_, C, O>,
    run: impl FnOnce() -> Result<CopyReport>,
) -> Result<CopyReport> {
    life.emit(CopyPhase::Preparing, CopyProgress::default());
    let outcome = run().and_then(|report| {
        let stats = report.stats();
        let progress = CopyProgress {
            bytes_read: stats.bytes_read(),
            bytes_written: stats.bytes_written(),
            bytes_zeroed: stats.bytes_zeroed(),
            bytes_discarded: stats.bytes_discarded(),
            blocks_completed: stats.blocks_copied(),
            extents_completed: Some(stats.extents_processed()),
            unconfirmed_io: false,
        };
        life.progress.set(progress);
        life.cancelled().map_err(|e| {
            crate::failure::execution(
                match report.backend() {
                    ExecutionBackend::Threaded => "threaded",
                    #[cfg(target_os = "linux")]
                    ExecutionBackend::IoUring => "io_uring",
                },
                CopyOperation::Cancel,
                None,
                progress,
                e,
            )
        })?;
        life.emit(CopyPhase::Completed, progress);
        Ok(report)
    });
    if let Err(error) = &outcome {
        let progress = error
            .copy_failure()
            .map_or(life.progress.get(), |f| f.progress);
        life.emit(
            if error.is_cancelled() {
                CopyPhase::Cancelled
            } else {
                CopyPhase::Failed
            },
            progress,
        );
    }
    outcome
}
