use std::time::Instant;

use rvvdk_core::{BufferPool, Error, Extent, Result, VirtualDisk};

use crate::planner::ExtentWorkIter;
#[cfg(target_os = "linux")]
use crate::preparation::{PreparedExecution, PreparedExecutor};
use crate::{concurrent, sequential};

use crate::{
    CopyOptions, CopyPlan, CopyReport, CopyStats, ExecutionBackend, ExecutionSelection,
    ExecutionStrategy, ProgressCompleted, ProgressObserver, ProgressSnapshot, ProgressTotals,
};

#[cfg(target_os = "linux")]
use crate::{NativeCopyReport, NativeCopyStats};
#[cfg(target_os = "linux")]
use rvvdk_core::{BlockDevice, ExtentKind, RawDisk};
#[cfg(target_os = "linux")]
use rvvdk_platform::LinuxFdBackend;

#[cfg(target_os = "linux")]
use crate::io_uring::{
    NativeExtentPlan, copy_extent_plan_with_destination, copy_file_range_with_options,
    evaluate_compatibility,
};

pub struct DataMover {
    pub(crate) options: CopyOptions,
    execution_strategy: ExecutionStrategy,
}

impl DataMover {
    pub fn new(options: CopyOptions) -> Self {
        Self {
            options,
            execution_strategy: ExecutionStrategy::Threaded,
        }
    }

    pub fn with_execution_strategy(
        options: CopyOptions,
        execution_strategy: ExecutionStrategy,
    ) -> Self {
        Self {
            options,
            execution_strategy,
        }
    }

    pub const fn execution_strategy(&self) -> ExecutionStrategy {
        self.execution_strategy
    }

    /// Build a portable logical plan. Auto selects threaded execution here;
    /// explicit io_uring requires the RAW entry points instead.
    pub fn plan<S>(&self, source: &S) -> Result<CopyPlan>
    where
        S: VirtualDisk + ?Sized,
    {
        let selection = ExecutionSelection::portable(self.execution_strategy)?;
        let source_size = source.size();

        let extents = source.extents(0, source_size)?;

        CopyPlan::new(
            source_size,
            extents,
            selection,
            self.options.block_size(),
            self.options.buffer_alignment(),
        )
    }

    /// Plan a logical copy through VirtualDisk methods, including trait objects.
    /// Auto uses threaded execution; no native descriptor can bypass translation.
    pub fn plan_with_destination<S, D>(&self, source: &S, destination: &D) -> Result<CopyPlan>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
    {
        self.require_portable_execution()?;
        crate::preflight::virtual_pair(source, destination, source.size())?;
        self.plan(source)
    }

    /// Execute a portable plan after live endpoint and structural validation.
    /// Native RAW plans are rejected before I/O or observer notification.
    pub fn execute_plan<S, D>(
        &self,
        plan: &CopyPlan,
        source: &S,
        destination: &D,
    ) -> Result<CopyReport>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
    {
        let prepared = self.prepare_portable(plan, source, destination)?;
        let (plan, source, destination, _) = prepared.parts();
        self.execute_threaded_plan(plan, source, destination)
    }

    pub fn execute_plan_with_observer<S, D, O>(
        &self,
        plan: &CopyPlan,
        source: &S,
        destination: &D,
        observer: &O,
    ) -> Result<CopyReport>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
        O: ProgressObserver + ?Sized,
    {
        let prepared = self.prepare_portable(plan, source, destination)?;
        let (plan, source, destination, _) = prepared.parts();
        Self::observe_plan(plan, observer, || {
            if self.options.concurrency() == 1 {
                self.execute_threaded_plan_with_observer(plan, source, destination, observer)
            } else {
                self.execute_threaded_plan(plan, source, destination)
            }
        })
    }

    pub fn copy_with_report<S, D>(&self, source: &S, destination: &D) -> Result<CopyReport>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
    {
        let plan = self.plan_with_destination(source, destination)?;
        self.execute_plan(&plan, source, destination)
    }

    /// Linux RAW adapter: permits FD execution only for explicitly exposed RAW backends.
    #[cfg(target_os = "linux")]
    pub fn plan_raw_with_destination<S, D>(
        &self,
        source: &RawDisk<S>,
        destination: &RawDisk<D>,
    ) -> Result<CopyPlan>
    where
        S: BlockDevice + LinuxFdBackend,
        D: BlockDevice + LinuxFdBackend,
    {
        let source_size = source.size();

        let destination_size = destination.size();

        if destination_size < source_size {
            return Err(Error::OutOfBounds {
                offset: 0,
                length: source_size,
                size: destination_size,
            });
        }

        let endpoints = crate::preflight::raw_pair(source, destination, source_size)?;

        let extents = source.extents(0, source_size)?;

        let source_backend = source.device();

        let destination_backend = destination.device();

        let compatibility = evaluate_compatibility(source_backend, destination_backend);

        let selection =
            ExecutionSelection::raw(self.execution_strategy, compatibility.compatible())?;
        let backend = selection.selected();
        let alignment = match backend {
            ExecutionBackend::Threaded => self.options.buffer_alignment(),
            ExecutionBackend::IoUring => compatibility
                .alignment()
                .max(self.options.buffer_alignment()),
        };

        if backend == ExecutionBackend::IoUring
            && extents
                .iter()
                .any(|extent| extent.kind() == ExtentKind::Data)
        {
            endpoints.validate_native_binding()?;
        }

        CopyPlan::new(
            source_size,
            extents,
            selection,
            self.options.block_size(),
            alignment,
        )
    }

    #[cfg(target_os = "linux")]
    pub fn copy_native_with_report<S, D>(
        &self,
        source: &S,
        destination: &D,
        offset: u64,
        length: u64,
    ) -> Result<NativeCopyReport>
    where
        S: LinuxFdBackend,
        D: LinuxFdBackend,
    {
        match self.execution_strategy {
            ExecutionStrategy::Threaded => Err(Error::NativeExecutionNotSelected),

            ExecutionStrategy::IoUring(execution_options)
            | ExecutionStrategy::Auto(execution_options) => {
                let compatibility = evaluate_compatibility(source, destination);

                if !compatibility.compatible() {
                    return Err(Error::NativeExecutionUnsupported);
                }

                let alignment = compatibility
                    .alignment()
                    .max(self.options.buffer_alignment());

                let stats = copy_file_range_with_options(
                    source.as_fd(),
                    destination.as_fd(),
                    offset,
                    length,
                    self.options.block_size(),
                    alignment,
                    execution_options,
                )?;

                Ok(NativeCopyReport::new(
                    ExecutionBackend::IoUring,
                    NativeCopyStats::from(stats),
                ))
            }
        }
    }

    #[cfg(target_os = "linux")]
    pub fn copy_native<S, D>(
        &self,
        source: &S,
        destination: &D,
        offset: u64,
        length: u64,
    ) -> Result<NativeCopyStats>
    where
        S: LinuxFdBackend,
        D: LinuxFdBackend,
    {
        self.copy_native_with_report(source, destination, offset, length)
            .map(|report| *report.stats())
    }

    pub fn copy<S, D>(&self, source: &S, destination: &D) -> Result<CopyStats>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
    {
        self.require_portable_execution()?;
        let source_size = source.size();

        let destination_size = destination.size();

        if destination_size < source_size {
            return Err(Error::OutOfBounds {
                offset: 0,
                length: source_size,
                size: destination_size,
            });
        }

        crate::preflight::virtual_pair(source, destination, source_size)?;

        let started = Instant::now();

        let extents = source.extents(0, source_size)?;
        crate::extent_validation::validate_extents(&extents, source_size)?;

        if self.options.concurrency() > 1 {
            return self.copy_concurrent(source, destination, extents, started);
        }

        sequential::execute(
            source,
            destination,
            &extents,
            self.options,
            started,
            &mut (),
        )
    }

    #[cfg(target_os = "linux")]
    pub fn execute_raw_plan<S, D>(
        &self,
        plan: &CopyPlan,
        source: &RawDisk<S>,
        destination: &RawDisk<D>,
    ) -> Result<CopyReport>
    where
        S: BlockDevice + LinuxFdBackend,
        D: BlockDevice + LinuxFdBackend,
    {
        let prepared = self.prepare_raw(plan, source, destination)?;
        self.execute_prepared_raw_plan(&prepared)
    }

    /// Dispatch only a fresh preparation tied to this plan and endpoint borrows.
    #[cfg(target_os = "linux")]
    fn execute_prepared_raw_plan<S, D>(
        &self,
        prepared: &PreparedExecution<'_, RawDisk<S>, RawDisk<D>>,
    ) -> Result<CopyReport>
    where
        S: BlockDevice + LinuxFdBackend,
        D: BlockDevice + LinuxFdBackend,
    {
        let (plan, source, destination, executor) = prepared.parts();
        match executor {
            PreparedExecutor::Threaded => self.execute_threaded_plan(plan, source, destination),
            PreparedExecutor::IoUring {
                native_plan,
                options,
            } => self.execute_io_uring_plan(plan, source, destination, native_plan, *options),
        }
    }

    #[cfg(target_os = "linux")]
    pub fn execute_raw_plan_with_observer<S, D, O>(
        &self,
        plan: &CopyPlan,
        source: &RawDisk<S>,
        destination: &RawDisk<D>,
        observer: &O,
    ) -> Result<CopyReport>
    where
        S: BlockDevice + LinuxFdBackend,
        D: BlockDevice + LinuxFdBackend,
        O: ProgressObserver + ?Sized,
    {
        let prepared = self.prepare_raw(plan, source, destination)?;
        let (plan, source, destination, _) = prepared.parts();
        Self::observe_plan(plan, observer, || {
            if plan.backend() == ExecutionBackend::Threaded && self.options.concurrency() == 1 {
                self.execute_threaded_plan_with_observer(plan, source, destination, observer)
            } else {
                self.execute_prepared_raw_plan(&prepared)
            }
        })
    }

    fn observe_plan<O: ProgressObserver + ?Sized>(
        plan: &CopyPlan,
        observer: &O,
        execute: impl FnOnce() -> Result<CopyReport>,
    ) -> Result<CopyReport> {
        observer.on_progress(&ProgressSnapshot::initial(plan));
        let report = execute()?;
        let stats = report.stats();
        let completed = ProgressCompleted::new(
            plan.logical_bytes(),
            stats.bytes_read(),
            stats.bytes_written(),
            stats.bytes_zeroed(),
            stats.bytes_discarded(),
            stats.extents_processed(),
        );

        observer.on_progress(&ProgressSnapshot::new(
            report.backend(),
            ProgressTotals::from_plan(plan),
            completed,
            stats.elapsed(),
        ));

        Ok(report)
    }

    fn execute_threaded_plan_with_observer<S, D, O>(
        &self,
        plan: &CopyPlan,
        source: &S,
        destination: &D,
        observer: &O,
    ) -> Result<CopyReport>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
        O: ProgressObserver + ?Sized,
    {
        let started = Instant::now();
        let mut progress = sequential::Observed::new(plan, observer, started);
        let stats = sequential::execute(
            source,
            destination,
            plan.extents(),
            self.options,
            started,
            &mut progress,
        )?;
        Ok(CopyReport::new(ExecutionBackend::Threaded, stats))
    }

    fn execute_threaded_plan<S, D>(
        &self,
        plan: &CopyPlan,
        source: &S,
        destination: &D,
    ) -> Result<CopyReport>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
    {
        let started = Instant::now();

        if self.options.concurrency() > 1 {
            let stats =
                self.copy_concurrent(source, destination, plan.extents().to_vec(), started)?;

            return Ok(CopyReport::new(ExecutionBackend::Threaded, stats));
        }

        let stats = sequential::execute(
            source,
            destination,
            plan.extents(),
            self.options,
            started,
            &mut (),
        )?;
        Ok(CopyReport::new(ExecutionBackend::Threaded, stats))
    }

    #[cfg(target_os = "linux")]
    fn execute_io_uring_plan<S, D>(
        &self,
        plan: &CopyPlan,
        source: &RawDisk<S>,
        destination: &RawDisk<D>,
        native_plan: &NativeExtentPlan,
        execution_options: crate::IoUringExecutionOptions,
    ) -> Result<CopyReport>
    where
        S: BlockDevice + LinuxFdBackend,
        D: BlockDevice + LinuxFdBackend,
    {
        let source_backend = source.device();
        let destination_backend = destination.device();

        let started = Instant::now();

        let stats = copy_extent_plan_with_destination(
            source_backend.as_fd(),
            destination_backend.as_fd(),
            destination,
            native_plan,
            plan.block_size(),
            plan.alignment(),
            execution_options,
        )?;

        destination.flush()?;

        Ok(CopyReport::new(
            ExecutionBackend::IoUring,
            CopyStats::from_io_uring_extents(stats, started.elapsed()),
        ))
    }

    #[cfg(target_os = "linux")]
    pub fn copy_raw_with_report<S, D>(
        &self,
        source: &RawDisk<S>,
        destination: &RawDisk<D>,
    ) -> Result<CopyReport>
    where
        S: BlockDevice + LinuxFdBackend,
        D: BlockDevice + LinuxFdBackend,
    {
        let plan = self.plan_raw_with_destination(source, destination)?;

        self.execute_raw_plan(&plan, source, destination)
    }

    fn copy_concurrent<S, D>(
        &self,
        source: &S,
        destination: &D,
        extents: Vec<Extent>,
        started: Instant,
    ) -> Result<CopyStats>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
    {
        let pool = BufferPool::new(
            self.options.buffer_count(),
            self.options.block_size(),
            self.options.buffer_alignment(),
        )?;

        let stats = concurrent::execute(
            source,
            destination,
            self.options.concurrency(),
            self.options.queue_capacity(),
            &pool,
            |sender| {
                for extent in &extents {
                    for work in ExtentWorkIter::new(*extent, self.options.block_size()) {
                        let work = work?;

                        sender.send(work).map_err(|_| Error::WorkQueueClosed)?;
                    }
                }

                Ok(())
            },
        )?;

        destination.flush()?;

        Ok(CopyStats::new(
            stats.bytes_read,
            stats.bytes_written,
            stats.bytes_zeroed,
            stats.bytes_discarded,
            stats.blocks_copied,
            extents.len() as u64,
            started.elapsed(),
        ))
    }
}
