use std::os::fd::RawFd;

use rvvdk_core::{Capabilities, Error, Extent, ExtentKind, Result, VirtualDisk};

use crate::IoUringExecutionOptions;

use super::{IoUringCopyStats, NativeExtentPlan, copy_file_range_with_options};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IoUringExtentCopyStats {
    bytes_read: u64,
    bytes_written: u64,
    bytes_zeroed: u64,
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

    pub const fn blocks_completed(&self) -> u64 {
        self.blocks_completed
    }

    pub const fn extents_processed(&self) -> u64 {
        self.extents_processed
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

        self.extents_processed =
            self.extents_processed
                .checked_add(1)
                .ok_or(Error::RangeOverflow {
                    offset: self.extents_processed,
                    length: 1,
                })?;

        Ok(())
    }

    fn add_zero_extent(&mut self, length: u64) -> Result<()> {
        self.bytes_zeroed = self
            .bytes_zeroed
            .checked_add(length)
            .ok_or(Error::RangeOverflow {
                offset: self.bytes_zeroed,
                length,
            })?;

        self.extents_processed =
            self.extents_processed
                .checked_add(1)
                .ok_or(Error::RangeOverflow {
                    offset: self.extents_processed,
                    length: 1,
                })?;

        Ok(())
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

pub fn copy_extent_plan(
    source_fd: RawFd,
    destination_fd: RawFd,
    plan: &NativeExtentPlan,
    block_size: usize,
    alignment: usize,
    options: IoUringExecutionOptions,
) -> Result<IoUringExtentCopyStats> {
    let mut aggregate = IoUringExtentCopyStats::default();

    for extent in plan.extents() {
        match extent.kind() {
            ExtentKind::Data => {
                let stats = copy_file_range_with_options(
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

            ExtentKind::Zero => {
                return Err(Error::UnsupportedNativeExtent {
                    kind: "zero",
                    offset: extent.offset(),
                    length: extent.length(),
                });
            }

            ExtentKind::Hole => {
                return Err(Error::UnsupportedNativeExtent {
                    kind: "hole",
                    offset: extent.offset(),
                    length: extent.length(),
                });
            }
        }
    }

    Ok(aggregate)
}

pub fn copy_extent_plan_with_destination<D>(
    source_fd: RawFd,
    destination_fd: RawFd,
    destination: &D,
    plan: &NativeExtentPlan,
    block_size: usize,
    alignment: usize,
    options: IoUringExecutionOptions,
) -> Result<IoUringExtentCopyStats>
where
    D: VirtualDisk,
{
    let mut aggregate = IoUringExtentCopyStats::default();

    for extent in plan.extents() {
        match extent.kind() {
            ExtentKind::Data => {
                let stats = copy_file_range_with_options(
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

            ExtentKind::Zero => {
                process_zero_extent(destination, *extent, block_size, &mut aggregate)?;
            }

            ExtentKind::Hole => {
                return Err(Error::UnsupportedNativeExtent {
                    kind: "hole",
                    offset: extent.offset(),
                    length: extent.length(),
                });
            }
        }
    }

    Ok(aggregate)
}

fn process_zero_extent<D>(
    destination: &D,
    extent: Extent,
    block_size: usize,
    stats: &mut IoUringExtentCopyStats,
) -> Result<()>
where
    D: VirtualDisk,
{
    if destination
        .capabilities()
        .contains(Capabilities::WRITE_ZERO)
    {
        destination.write_zero_at(extent.offset(), extent.length())?;

        stats.add_zero_extent(extent.length())?;

        return Ok(());
    }

    write_zero_fallback(destination, extent, block_size, stats)
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

    stats.add_zero_extent(extent.length())
}
