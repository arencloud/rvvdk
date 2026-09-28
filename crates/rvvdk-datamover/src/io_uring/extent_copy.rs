use std::os::fd::BorrowedFd;

use rvvdk_core::{Error, Extent, ExtentKind, Result, VirtualDisk};

use crate::IoUringExecutionOptions;
use crate::policy::{self, Operation};

use super::copy::copy_file_range_preflighted;
use super::validation::{validate_copy_configuration, validate_file_range};
use super::{IoUringCopyStats, NativeExtentPlan};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IoUringExtentCopyStats {
    bytes_read: u64,
    bytes_written: u64,
    bytes_zeroed: u64,
    bytes_discarded: u64,
    blocks_completed: u64,
    extents_processed: u64,
}

impl IoUringExtentCopyStats {
    pub const fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    pub const fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    pub const fn bytes_zeroed(&self) -> u64 {
        self.bytes_zeroed
    }

    pub const fn bytes_discarded(&self) -> u64 {
        self.bytes_discarded
    }

    pub const fn blocks_completed(&self) -> u64 {
        self.blocks_completed
    }

    pub const fn extents_processed(&self) -> u64 {
        self.extents_processed
    }

    fn complete_extent(&mut self) -> Result<()> {
        self.extents_processed =
            self.extents_processed
                .checked_add(1)
                .ok_or(Error::RangeOverflow {
                    offset: self.extents_processed,
                    length: 1,
                })?;

        Ok(())
    }

    fn add_data_extent(&mut self, stats: IoUringCopyStats) -> Result<()> {
        self.bytes_read =
            self.bytes_read
                .checked_add(stats.bytes_read())
                .ok_or(Error::RangeOverflow {
                    offset: self.bytes_read,
                    length: stats.bytes_read(),
                })?;

        self.bytes_written = self
            .bytes_written
            .checked_add(stats.bytes_written())
            .ok_or(Error::RangeOverflow {
                offset: self.bytes_written,
                length: stats.bytes_written(),
            })?;

        self.blocks_completed = self
            .blocks_completed
            .checked_add(stats.blocks_completed())
            .ok_or(Error::RangeOverflow {
                offset: self.blocks_completed,
                length: stats.blocks_completed(),
            })?;

        self.complete_extent()
    }

    fn add_zero_extent(&mut self, length: u64) -> Result<()> {
        self.bytes_zeroed = self
            .bytes_zeroed
            .checked_add(length)
            .ok_or(Error::RangeOverflow {
                offset: self.bytes_zeroed,
                length,
            })?;

        self.complete_extent()
    }

    fn add_discard_extent(&mut self, length: u64) -> Result<()> {
        self.bytes_discarded =
            self.bytes_discarded
                .checked_add(length)
                .ok_or(Error::RangeOverflow {
                    offset: self.bytes_discarded,
                    length,
                })?;

        self.complete_extent()
    }

    fn add_fallback_write(&mut self, length: u64) -> Result<()> {
        self.bytes_written =
            self.bytes_written
                .checked_add(length)
                .ok_or(Error::RangeOverflow {
                    offset: self.bytes_written,
                    length,
                })?;

        self.blocks_completed =
            self.blocks_completed
                .checked_add(1)
                .ok_or(Error::RangeOverflow {
                    offset: self.blocks_completed,
                    length: 1,
                })?;

        Ok(())
    }
}

/// Copy a data-only plan. Validate configuration, range, and every extent kind
/// before allocation or I/O; an unsupported later extent cannot cause a partial copy.
pub fn copy_extent_plan(
    source_fd: BorrowedFd<'_>,
    destination_fd: BorrowedFd<'_>,
    plan: &NativeExtentPlan,
    block_size: usize,
    alignment: usize,
    options: IoUringExecutionOptions,
) -> Result<IoUringExtentCopyStats> {
    validate_copy_configuration(block_size, alignment)?;
    validate_file_range(0, plan.disk_size())?;
    for extent in plan.extents() {
        let kind = match extent.kind() {
            ExtentKind::Data => continue,
            ExtentKind::Zero => "zero",
            ExtentKind::Hole => "hole",
        };
        return Err(Error::UnsupportedNativeExtent {
            kind,
            offset: extent.offset(),
            length: extent.length(),
        });
    }

    if plan.is_empty() {
        return Ok(IoUringExtentCopyStats::default());
    }
    crate::preflight::files(source_fd, destination_fd, 0, plan.disk_size())?;

    let mut aggregate = IoUringExtentCopyStats::default();
    for extent in plan.extents() {
        let stats = copy_file_range_preflighted(
            source_fd,
            destination_fd,
            extent.offset(),
            extent.length(),
            block_size,
            alignment,
            options,
        )?;
        aggregate.add_data_extent(stats)?;
    }

    Ok(aggregate)
}

/// Copy Data/Zero/Hole extents using destination capabilities. Configuration and
/// range validation precede all backend calls, including for empty plans. Backend
/// access, capacity, and known identity are checked before execution. Data plans
/// require the destination backend to identify the supplied native descriptor.
/// Direct-I/O alignment/tail compatibility remains a separate requirement.
pub fn copy_extent_plan_with_destination<D>(
    source_fd: BorrowedFd<'_>,
    destination_fd: BorrowedFd<'_>,
    destination: &D,
    plan: &NativeExtentPlan,
    block_size: usize,
    alignment: usize,
    options: IoUringExecutionOptions,
) -> Result<IoUringExtentCopyStats>
where
    D: VirtualDisk,
{
    validate_copy_configuration(block_size, alignment)?;
    validate_file_range(0, plan.disk_size())?;

    if plan.is_empty() {
        return Ok(IoUringExtentCopyStats::default());
    }
    let source_info = crate::preflight::file(source_fd, "source")?;
    let destination_info = destination
        .copy_endpoint()
        .map_err(|e| crate::preflight::context("destination", e))?;
    crate::preflight::pair(source_info, destination_info, 0, plan.disk_size(), false)?;
    if plan
        .extents()
        .iter()
        .any(|extent| extent.kind() == ExtentKind::Data)
    {
        let fd_info = crate::preflight::file(destination_fd, "destination descriptor")?;
        crate::preflight::pair(source_info, fd_info, 0, plan.disk_size(), false)?;
        if destination_info.identity.is_none() {
            return Err(crate::preflight::context(
                "destination",
                Error::InvalidEndpoint {
                    reason: "native Data execution requires known backing identity",
                },
            ));
        }
        if destination_info.identity != fd_info.identity {
            return Err(crate::preflight::context(
                "destination",
                Error::EndpointMismatch,
            ));
        }
    }

    let mut aggregate = IoUringExtentCopyStats::default();

    for extent in plan.extents() {
        match policy::select(extent.kind(), || destination.capabilities()) {
            Operation::Copy => {
                let stats = copy_file_range_preflighted(
                    source_fd,
                    destination_fd,
                    extent.offset(),
                    extent.length(),
                    block_size,
                    alignment,
                    options,
                )?;

                aggregate.add_data_extent(stats)?;
            }

            Operation::Zero => {
                destination.write_zero_at(extent.offset(), extent.length())?;
                aggregate.add_zero_extent(extent.length())?;
            }
            Operation::Discard => {
                destination.discard(extent.offset(), extent.length())?;
                aggregate.add_discard_extent(extent.length())?;
            }
            Operation::WriteZero => {
                write_zero_fallback(destination, *extent, block_size, &mut aggregate)?
            }
        }
    }
    Ok(aggregate)
}

fn write_zero_fallback<D>(
    destination: &D,
    extent: Extent,
    block_size: usize,
    stats: &mut IoUringExtentCopyStats,
) -> Result<()>
where
    D: VirtualDisk,
{
    let buffer = vec![0_u8; block_size];

    let mut offset = extent.offset();

    let end = extent.end();

    while offset < end {
        let remaining = end - offset;

        let request_length_u64 = remaining.min(block_size as u64);

        let request_length =
            usize::try_from(request_length_u64).map_err(|_| Error::RangeOverflow {
                offset,
                length: request_length_u64,
            })?;

        destination.write_all_at(offset, &buffer[..request_length])?;

        offset = offset
            .checked_add(request_length_u64)
            .ok_or(Error::RangeOverflow {
                offset,
                length: request_length_u64,
            })?;

        stats.add_fallback_write(request_length_u64)?;
    }

    /*
     * Fallback zeroing was performed through ordinary writes.
     *
     * Match the established threaded CopyStats semantics:
     *
     *   bytes_written     += length
     *   blocks_completed  += number of writes
     *   bytes_zeroed      += 0
     *
     * The extent itself still counts as processed.
     */
    stats.complete_extent()
}
