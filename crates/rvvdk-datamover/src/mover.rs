use std::time::Instant;

use rvvdk_core::{BufferPool, Capabilities, Error, Extent, ExtentKind, Result, VirtualDisk};

use crate::concurrent;
use crate::planner::ExtentWorkIter;

use crate::{
    CopyOptions, CopyPlan, CopyReport, CopyStats, ExecutionBackend, ExecutionStrategy,
    ProgressCompleted, ProgressObserver, ProgressSnapshot, ProgressState, ProgressTotals,
};

#[cfg(target_os = "linux")]
use crate::{NativeCopyReport, NativeCopyStats};
#[cfg(target_os = "linux")]
use rvvdk_core::{BlockDevice, RawDisk};
#[cfg(target_os = "linux")]
use rvvdk_platform::LinuxFdBackend;

#[cfg(target_os = "linux")]
use crate::io_uring::{
    NativeExtentPlan, copy_extent_plan_with_destination, copy_file_range_with_options,
    evaluate_compatibility,
};

const DEFAULT_PROGRESS_INTERVAL_BYTES: u64 = 64 * 1024 * 1024;

pub struct DataMover {
    options: CopyOptions,
    execution_strategy: ExecutionStrategy,
}

#[derive(Debug, Default)]
struct MutableStats {
    bytes_read: u64,
    bytes_written: u64,
    bytes_zeroed: u64,
    bytes_discarded: u64,
    blocks_copied: u64,
    extents_processed: u64,
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
        self.require_portable_execution()?;
        let source_size = source.size();

        let extents = source.extents(0, source_size)?;

        CopyPlan::new(
            source_size,
            extents,
            ExecutionBackend::Threaded,
            self.options.block_size(),
            self.options.buffer_alignment(),
        )
    }

    fn require_portable_execution(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        if matches!(self.execution_strategy, ExecutionStrategy::IoUring(_)) {
            return Err(Error::NativeExecutionUnsupported);
        }
        Ok(())
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
        self.validate_portable_plan(plan, source, destination)?;
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
        self.validate_portable_plan(plan, source, destination)?;
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

    fn validate_portable_plan<S, D>(
        &self,
        plan: &CopyPlan,
        source: &S,
        destination: &D,
    ) -> Result<()>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
    {
        self.require_portable_execution()?;
        if plan.backend() != ExecutionBackend::Threaded {
            return Err(Error::NativeExecutionUnsupported);
        }
        self.validate_plan(plan, source, destination, || {
            crate::preflight::virtual_pair(source, destination, plan.logical_bytes()).map(|_| ())
        })
    }

    #[cfg(target_os = "linux")]
    fn validate_raw_plan<S, D>(
        &self,
        plan: &CopyPlan,
        source: &RawDisk<S>,
        destination: &RawDisk<D>,
    ) -> Result<()>
    where
        S: BlockDevice + LinuxFdBackend,
        D: BlockDevice + LinuxFdBackend,
    {
        self.validate_plan(plan, source, destination, || {
            let endpoints = crate::preflight::raw_pair(source, destination, plan.logical_bytes())?;
            if plan.backend() == ExecutionBackend::IoUring
                && plan
                    .extents()
                    .iter()
                    .any(|extent| extent.kind() == ExtentKind::Data)
            {
                endpoints.validate_native_binding()?;
            }
            Ok(())
        })
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

        let (backend, alignment) = match self.execution_strategy {
            ExecutionStrategy::Threaded => {
                (ExecutionBackend::Threaded, self.options.buffer_alignment())
            }

            ExecutionStrategy::IoUring(_) => {
                if !compatibility.compatible() {
                    return Err(Error::NativeExecutionUnsupported);
                }

                (
                    ExecutionBackend::IoUring,
                    compatibility
                        .alignment()
                        .max(self.options.buffer_alignment()),
                )
            }

            ExecutionStrategy::Auto(_) => {
                if compatibility.compatible() {
                    (
                        ExecutionBackend::IoUring,
                        compatibility
                            .alignment()
                            .max(self.options.buffer_alignment()),
                    )
                } else {
                    (ExecutionBackend::Threaded, self.options.buffer_alignment())
                }
            }
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
            backend,
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

        let pool = BufferPool::new(
            self.options.buffer_count(),
            self.options.block_size(),
            self.options.buffer_alignment(),
        )?;

        let mut buffer = pool.acquire();

        let mut stats = MutableStats::default();

        for extent in extents {
            self.process_extent(
                source,
                destination,
                extent,
                buffer.as_mut_slice(),
                &mut stats,
            )?;

            stats.extents_processed += 1;
        }

        destination.flush()?;

        Ok(CopyStats::new(
            stats.bytes_read,
            stats.bytes_written,
            stats.bytes_zeroed,
            stats.bytes_discarded,
            stats.blocks_copied,
            stats.extents_processed,
            started.elapsed(),
        ))
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
        self.validate_raw_plan(plan, source, destination)?;
        self.execute_validated_raw_plan(plan, source, destination)
    }

    /// Validate structural plan invariants before any execution or observation.
    fn validate_plan<S, D>(
        &self,
        plan: &CopyPlan,
        source: &S,
        destination: &D,
        preflight: impl FnOnce() -> Result<()>,
    ) -> Result<()>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
    {
        let source_size = source.size();

        let destination_size = destination.size();

        /*
         * The source geometry must still match the geometry captured by
         * the plan.
         */
        if source_size != plan.logical_bytes() {
            return Err(Error::CorruptMetadata(format!(
                "copy plan source size mismatch: \
                     plan={}, source={source_size}",
                plan.logical_bytes(),
            )));
        }

        /*
         * The destination must still be large enough for the complete
         * logical source.
         */
        if destination_size < plan.logical_bytes() {
            return Err(Error::OutOfBounds {
                offset: 0,
                length: plan.logical_bytes(),
                size: destination_size,
            });
        }

        preflight()?;

        /*
         * The DataMover configuration used for execution must match the
         * block size captured during planning.
         */
        if plan.block_size() != self.options.block_size() {
            return Err(Error::CorruptMetadata(format!(
                "copy plan block size mismatch: \
                     plan={}, mover={}",
                plan.block_size(),
                self.options.block_size(),
            )));
        }

        /*
         * Re-read only the source extent metadata.
         *
         * We deliberately do not read or hash Data contents here.
         * CopyPlan represents a structural execution plan, not a snapshot
         * of the source payload.
         */
        let current_extents = source.extents(0, source_size)?;

        crate::extent_validation::validate_extents(&current_extents, source_size)?;

        let current_fingerprint = crate::plan::extent_fingerprint(&current_extents);

        /*
         * Reject a stale plan when the source extent structure changed
         * between planning and execution.
         *
         * The fingerprint covers:
         *
         *     offset
         *     length
         *     ExtentKind
         */
        if current_fingerprint != plan.extent_fingerprint() {
            return Err(Error::CorruptMetadata(format!(
                "copy plan extent map changed: \
                     planned={:#018x}, current={:#018x}",
                plan.extent_fingerprint(),
                current_fingerprint,
            )));
        }

        if plan.backend() == ExecutionBackend::Threaded
            && plan.alignment() != self.options.buffer_alignment()
        {
            return Err(Error::CorruptMetadata(format!(
                "copy plan alignment mismatch: plan={}, mover={}",
                plan.alignment(),
                self.options.buffer_alignment(),
            )));
        }

        Ok(())
    }

    /// Dispatch only after the caller has validated the structural plan.
    #[cfg(target_os = "linux")]
    fn execute_validated_raw_plan<S, D>(
        &self,
        plan: &CopyPlan,
        source: &RawDisk<S>,
        destination: &RawDisk<D>,
    ) -> Result<CopyReport>
    where
        S: BlockDevice + LinuxFdBackend,
        D: BlockDevice + LinuxFdBackend,
    {
        match plan.backend() {
            ExecutionBackend::Threaded => self.execute_threaded_plan(plan, source, destination),
            // Native compatibility and runtime alignment are rechecked before native I/O.
            ExecutionBackend::IoUring => self.execute_io_uring_plan(plan, source, destination),
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
        self.validate_raw_plan(plan, source, destination)?;
        Self::observe_plan(plan, observer, || {
            if plan.backend() == ExecutionBackend::Threaded && self.options.concurrency() == 1 {
                self.execute_threaded_plan_with_observer(plan, source, destination, observer)
            } else {
                self.execute_validated_raw_plan(plan, source, destination)
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
        let pool = BufferPool::new(
            self.options.buffer_count(),
            plan.block_size(),
            plan.alignment(),
        )?;
        let mut buffer = pool.acquire();
        let mut stats = MutableStats::default();
        let mut progress = ProgressState::from_plan(plan);
        let mut last_emitted = 0_u64;

        for extent in plan.extents() {
            match extent.kind() {
                ExtentKind::Data => {
                    let mut offset = extent.offset();
                    let end = extent.end();

                    while offset < end {
                        let remaining = end - offset;
                        let length = remaining.min(self.options.block_size() as u64);
                        let request_size = usize::try_from(length)
                            .map_err(|_| Error::RangeOverflow { offset, length })?;
                        let current_buffer = &mut buffer.as_mut_slice()[..request_size];

                        source.read_exact_at(offset, current_buffer)?;
                        destination.write_all_at(offset, current_buffer)?;

                        offset = offset
                            .checked_add(length)
                            .ok_or(Error::RangeOverflow { offset, length })?;

                        stats.bytes_read += length;
                        stats.bytes_written += length;
                        stats.blocks_copied += 1;
                        progress.complete_data(length);

                        Self::emit_progress_if_needed(
                            &progress,
                            observer,
                            started,
                            &mut last_emitted,
                            false,
                        );
                    }
                }

                ExtentKind::Zero => {
                    if destination
                        .capabilities()
                        .contains(Capabilities::WRITE_ZERO)
                    {
                        destination.write_zero_at(extent.offset(), extent.length())?;
                        stats.bytes_zeroed += extent.length();
                        progress.complete_zero(extent.length());
                    } else {
                        buffer.as_mut_slice().fill(0);
                        let mut offset = extent.offset();
                        let end = extent.end();

                        while offset < end {
                            let remaining = end - offset;
                            let length = remaining.min(self.options.block_size() as u64);
                            let request_size = usize::try_from(length)
                                .map_err(|_| Error::RangeOverflow { offset, length })?;

                            destination
                                .write_all_at(offset, &buffer.as_mut_slice()[..request_size])?;

                            offset = offset
                                .checked_add(length)
                                .ok_or(Error::RangeOverflow { offset, length })?;

                            stats.bytes_written += length;
                            stats.blocks_copied += 1;
                            progress.complete_fallback_write(length);

                            Self::emit_progress_if_needed(
                                &progress,
                                observer,
                                started,
                                &mut last_emitted,
                                false,
                            );
                        }
                    }
                }

                ExtentKind::Hole => {
                    let capabilities = destination.capabilities();

                    if capabilities.contains(Capabilities::DISCARD) {
                        destination.discard(extent.offset(), extent.length())?;
                        stats.bytes_discarded += extent.length();
                        progress.complete_discard(extent.length());
                    } else if capabilities.contains(Capabilities::WRITE_ZERO) {
                        destination.write_zero_at(extent.offset(), extent.length())?;
                        stats.bytes_zeroed += extent.length();
                        progress.complete_zero(extent.length());
                    } else {
                        buffer.as_mut_slice().fill(0);
                        let mut offset = extent.offset();
                        let end = extent.end();

                        while offset < end {
                            let remaining = end - offset;
                            let length = remaining.min(self.options.block_size() as u64);
                            let request_size = usize::try_from(length)
                                .map_err(|_| Error::RangeOverflow { offset, length })?;

                            destination
                                .write_all_at(offset, &buffer.as_mut_slice()[..request_size])?;

                            offset = offset
                                .checked_add(length)
                                .ok_or(Error::RangeOverflow { offset, length })?;

                            stats.bytes_written += length;
                            stats.blocks_copied += 1;
                            progress.complete_fallback_write(length);

                            Self::emit_progress_if_needed(
                                &progress,
                                observer,
                                started,
                                &mut last_emitted,
                                false,
                            );
                        }
                    }
                }
            }

            stats.extents_processed += 1;
            progress.complete_extent();

            if progress.completed().logical_bytes_completed() < progress.totals().logical_bytes() {
                Self::emit_progress_if_needed(
                    &progress,
                    observer,
                    started,
                    &mut last_emitted,
                    true,
                );
            }
        }

        destination.flush()?;

        Ok(CopyReport::new(
            ExecutionBackend::Threaded,
            CopyStats::new(
                stats.bytes_read,
                stats.bytes_written,
                stats.bytes_zeroed,
                stats.bytes_discarded,
                stats.blocks_copied,
                stats.extents_processed,
                started.elapsed(),
            ),
        ))
    }

    fn emit_progress_if_needed<O>(
        progress: &ProgressState,
        observer: &O,
        started: Instant,
        last_emitted: &mut u64,
        force: bool,
    ) where
        O: ProgressObserver + ?Sized,
    {
        let logical_completed = progress.completed().logical_bytes_completed();
        let interval_reached =
            logical_completed.saturating_sub(*last_emitted) >= DEFAULT_PROGRESS_INTERVAL_BYTES;

        if logical_completed == *last_emitted || (!force && !interval_reached) {
            return;
        }

        observer.on_progress(&progress.snapshot(ExecutionBackend::Threaded, started.elapsed()));

        *last_emitted = logical_completed;
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

        let pool = BufferPool::new(
            self.options.buffer_count(),
            plan.block_size(),
            plan.alignment(),
        )?;

        let mut buffer = pool.acquire();

        let mut stats = MutableStats::default();

        for extent in plan.extents() {
            self.process_extent(
                source,
                destination,
                *extent,
                buffer.as_mut_slice(),
                &mut stats,
            )?;

            stats.extents_processed += 1;
        }

        destination.flush()?;

        Ok(CopyReport::new(
            ExecutionBackend::Threaded,
            CopyStats::new(
                stats.bytes_read,
                stats.bytes_written,
                stats.bytes_zeroed,
                stats.bytes_discarded,
                stats.blocks_copied,
                stats.extents_processed,
                started.elapsed(),
            ),
        ))
    }

    #[cfg(target_os = "linux")]
    fn execute_io_uring_plan<S, D>(
        &self,
        plan: &CopyPlan,
        source: &RawDisk<S>,
        destination: &RawDisk<D>,
    ) -> Result<CopyReport>
    where
        S: BlockDevice + LinuxFdBackend,
        D: BlockDevice + LinuxFdBackend,
    {
        let execution_options = match self.execution_strategy {
            ExecutionStrategy::IoUring(options) | ExecutionStrategy::Auto(options) => options,

            ExecutionStrategy::Threaded => {
                return Err(Error::NativeExecutionNotSelected);
            }
        };

        let native_plan = NativeExtentPlan::new(plan.extents().to_vec(), plan.logical_bytes())?;

        let source_backend = source.device();

        let destination_backend = destination.device();

        let compatibility = evaluate_compatibility(source_backend, destination_backend);

        if !compatibility.compatible() {
            return Err(Error::NativeExecutionUnsupported);
        }

        let runtime_alignment = compatibility
            .alignment()
            .max(self.options.buffer_alignment());

        if runtime_alignment != plan.alignment() {
            return Err(Error::CorruptMetadata(format!(
                "copy plan alignment mismatch: \
                     plan={}, runtime={runtime_alignment}",
                plan.alignment(),
            )));
        }

        let started = Instant::now();

        let stats = copy_extent_plan_with_destination(
            source_backend.as_fd(),
            destination_backend.as_fd(),
            destination,
            &native_plan,
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

    fn process_extent<S, D>(
        &self,
        source: &S,
        destination: &D,
        extent: Extent,
        buffer: &mut [u8],
        stats: &mut MutableStats,
    ) -> Result<()>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
    {
        match extent.kind() {
            ExtentKind::Data => self.copy_data_extent(source, destination, extent, buffer, stats),

            ExtentKind::Zero => self.zero_extent(destination, extent, buffer, stats),

            ExtentKind::Hole => self.hole_extent(destination, extent, buffer, stats),
        }
    }

    fn copy_data_extent<S, D>(
        &self,
        source: &S,
        destination: &D,
        extent: Extent,
        buffer: &mut [u8],
        stats: &mut MutableStats,
    ) -> Result<()>
    where
        S: VirtualDisk + ?Sized,
        D: VirtualDisk + ?Sized,
    {
        let mut offset = extent.offset();

        let end = extent.end();

        while offset < end {
            let remaining = end - offset;

            let request_size_u64 = remaining.min(self.options.block_size() as u64);

            let request_size =
                usize::try_from(request_size_u64).map_err(|_| Error::RangeOverflow {
                    offset,
                    length: request_size_u64,
                })?;

            let current_buffer = &mut buffer[..request_size];

            source.read_exact_at(offset, current_buffer)?;

            destination.write_all_at(offset, current_buffer)?;

            offset = offset
                .checked_add(request_size_u64)
                .ok_or(Error::RangeOverflow {
                    offset,
                    length: request_size_u64,
                })?;

            stats.bytes_read += request_size_u64;

            stats.bytes_written += request_size_u64;

            stats.blocks_copied += 1;
        }

        Ok(())
    }

    fn zero_extent<D>(
        &self,
        destination: &D,
        extent: Extent,
        buffer: &mut [u8],
        stats: &mut MutableStats,
    ) -> Result<()>
    where
        D: VirtualDisk + ?Sized,
    {
        if destination
            .capabilities()
            .contains(Capabilities::WRITE_ZERO)
        {
            destination.write_zero_at(extent.offset(), extent.length())?;

            stats.bytes_zeroed += extent.length();

            return Ok(());
        }

        self.write_zero_fallback(destination, extent, buffer, stats)
    }

    fn hole_extent<D>(
        &self,
        destination: &D,
        extent: Extent,
        buffer: &mut [u8],
        stats: &mut MutableStats,
    ) -> Result<()>
    where
        D: VirtualDisk + ?Sized,
    {
        let capabilities = destination.capabilities();

        if capabilities.contains(Capabilities::DISCARD) {
            destination.discard(extent.offset(), extent.length())?;

            stats.bytes_discarded += extent.length();

            return Ok(());
        }

        if capabilities.contains(Capabilities::WRITE_ZERO) {
            destination.write_zero_at(extent.offset(), extent.length())?;

            stats.bytes_zeroed += extent.length();

            return Ok(());
        }

        self.write_zero_fallback(destination, extent, buffer, stats)
    }

    fn write_zero_fallback<D>(
        &self,
        destination: &D,
        extent: Extent,
        buffer: &mut [u8],
        stats: &mut MutableStats,
    ) -> Result<()>
    where
        D: VirtualDisk + ?Sized,
    {
        buffer.fill(0);

        let mut offset = extent.offset();

        let end = extent.end();

        while offset < end {
            let remaining = end - offset;

            let request_size_u64 = remaining.min(self.options.block_size() as u64);

            let request_size =
                usize::try_from(request_size_u64).map_err(|_| Error::RangeOverflow {
                    offset,
                    length: request_size_u64,
                })?;

            destination.write_all_at(offset, &buffer[..request_size])?;

            offset = offset
                .checked_add(request_size_u64)
                .ok_or(Error::RangeOverflow {
                    offset,
                    length: request_size_u64,
                })?;

            stats.bytes_written += request_size_u64;

            stats.blocks_copied += 1;
        }

        Ok(())
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
