use std::time::Instant;

use rvvdk_core::{
    BlockDevice, BufferPool, Capabilities, Error, Extent, ExtentKind, RawDisk, Result, VirtualDisk,
};

use crate::concurrent;
use crate::planner::ExtentWorkIter;

use crate::{
    CopyOptions, CopyReport, CopyStats, ExecutionBackend, ExecutionStrategy, NativeCopyReport,
    NativeCopyStats,
};

#[cfg(target_os = "linux")]
use rvvdk_platform::LinuxFdBackend;

#[cfg(target_os = "linux")]
use crate::io_uring::{
    NativeExtentPlan, copy_extent_plan, copy_file_range_with_options, evaluate_compatibility,
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
    pub fn copy_with_report<S, D>(
        &self,
        source: &RawDisk<S>,
        destination: &RawDisk<D>,
    ) -> Result<CopyReport>
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

        let plan = NativeExtentPlan::new(extents, source_size)?;

        match self.execution_strategy {
            ExecutionStrategy::Threaded => {
                let stats = self.copy(source, destination)?;

                Ok(CopyReport::new(ExecutionBackend::Threaded, stats))
            }

            ExecutionStrategy::IoUring(execution_options) => {
                let source_backend = source.device();

                let destination_backend = destination.device();

                let compatibility = evaluate_compatibility(source_backend, destination_backend);

                if !compatibility.compatible() {
                    return Err(Error::NativeExecutionUnsupported);
                }

                let alignment = compatibility
                    .alignment()
                    .max(self.options.buffer_alignment());

                let started = Instant::now();

                let stats = copy_extent_plan(
                    source_backend.raw_fd(),
                    destination_backend.raw_fd(),
                    &plan,
                    self.options.block_size(),
                    alignment,
                    execution_options,
                )?;

                destination.flush()?;

                Ok(CopyReport::new(
                    ExecutionBackend::IoUring,
                    CopyStats::from_io_uring_extents(stats, started.elapsed()),
                ))
            }

            ExecutionStrategy::Auto(execution_options) => {
                let source_backend = source.device();

                let destination_backend = destination.device();

                let compatibility = evaluate_compatibility(source_backend, destination_backend);

                if compatibility.compatible() && plan.is_data_only() {
                    let alignment = compatibility
                        .alignment()
                        .max(self.options.buffer_alignment());

                    let started = Instant::now();

                    let stats = copy_extent_plan(
                        source_backend.raw_fd(),
                        destination_backend.raw_fd(),
                        &plan,
                        self.options.block_size(),
                        alignment,
                        execution_options,
                    )?;

                    destination.flush()?;

                    Ok(CopyReport::new(
                        ExecutionBackend::IoUring,
                        CopyStats::from_io_uring_extents(stats, started.elapsed()),
                    ))
                } else {
                    let stats = self.copy(source, destination)?;

                    Ok(CopyReport::new(ExecutionBackend::Threaded, stats))
                }
            }
        }
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
