use std::time::Instant;

use rvvdk_core::{
    BlockDevice, BufferPool, Capabilities, Error, Extent, ExtentKind, RawDisk, Result, VirtualDisk,
};

use crate::concurrent;
use crate::planner::ExtentWorkIter;

use crate::{
    CopyOptions, CopyPlan, CopyReport, CopyStats, ExecutionBackend, ExecutionStrategy,
    NativeCopyReport, NativeCopyStats,
};

#[cfg(target_os = "linux")]
use rvvdk_platform::LinuxFdBackend;

#[cfg(target_os = "linux")]
use crate::io_uring::{
    NativeExtentPlan, copy_extent_plan_with_destination, copy_file_range_with_options,
    evaluate_compatibility,
};

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

    /*
     * M20B planning entry point.
     *
     * This stage deliberately models portable planning only.
     *
     * M20C will make backend selection destination-aware and will
     * evaluate Threaded / IoUring / Auto using LinuxFdBackend
     * compatibility.
     */
    pub fn plan<S>(&self, source: &S) -> Result<CopyPlan>
    where
        S: VirtualDisk,
    {
        let source_size = source.size();

        let extents = source.extents(0, source_size)?;

        crate::extent_validation::validate_extents(&extents, source_size)?;

        CopyPlan::new(
            source_size,
            extents,
            ExecutionBackend::Threaded,
            self.options.block_size(),
            self.options.buffer_alignment(),
        )
    }

    #[cfg(target_os = "linux")]
    pub fn plan_with_destination<S, D>(
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

        let extents = source.extents(0, source_size)?;

        crate::extent_validation::validate_extents(&extents, source_size)?;

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
                    source.raw_fd(),
                    destination.raw_fd(),
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
        S: VirtualDisk,
        D: VirtualDisk,
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
    pub fn execute_plan<S, D>(
        &self,
        plan: &CopyPlan,
        source: &RawDisk<S>,
        destination: &RawDisk<D>,
    ) -> Result<CopyReport>
    where
        S: BlockDevice + LinuxFdBackend,
        D: BlockDevice + LinuxFdBackend,
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

        match plan.backend() {
            ExecutionBackend::Threaded => {
                /*
                 * Threaded plans use the configured buffer alignment.
                 *
                 * Reject execution through a differently configured
                 * DataMover.
                 */
                if plan.alignment() != self.options.buffer_alignment() {
                    return Err(Error::CorruptMetadata(format!(
                        "copy plan alignment mismatch: \
                             plan={}, mover={}",
                        plan.alignment(),
                        self.options.buffer_alignment(),
                    )));
                }

                self.execute_threaded_plan(plan, source, destination)
            }

            ExecutionBackend::IoUring => {
                /*
                 * Native compatibility and runtime alignment are checked
                 * again inside execute_io_uring_plan().
                 *
                 * This protects execution if the native backend
                 * capabilities changed after planning.
                 */
                self.execute_io_uring_plan(plan, source, destination)
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn execute_threaded_plan<S, D>(
        &self,
        plan: &CopyPlan,
        source: &RawDisk<S>,
        destination: &RawDisk<D>,
    ) -> Result<CopyReport>
    where
        S: BlockDevice + LinuxFdBackend,
        D: BlockDevice + LinuxFdBackend,
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
            source_backend.raw_fd(),
            destination_backend.raw_fd(),
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
    pub fn copy_with_report<S, D>(
        &self,
        source: &RawDisk<S>,
        destination: &RawDisk<D>,
    ) -> Result<CopyReport>
    where
        S: BlockDevice + LinuxFdBackend,
        D: BlockDevice + LinuxFdBackend,
    {
        let plan = self.plan_with_destination(source, destination)?;

        self.execute_plan(&plan, source, destination)
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
        S: VirtualDisk,
        D: VirtualDisk,
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
        S: VirtualDisk,
        D: VirtualDisk,
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
        D: VirtualDisk,
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
        D: VirtualDisk,
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
        D: VirtualDisk,
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
        S: VirtualDisk,
        D: VirtualDisk,
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
