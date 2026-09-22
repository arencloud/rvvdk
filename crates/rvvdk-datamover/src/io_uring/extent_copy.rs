use std::os::fd::RawFd;

use rvvdk_core::{Error, ExtentKind, Result};

use crate::IoUringExecutionOptions;

use super::{IoUringCopyStats, NativeExtentPlan, copy_file_range_with_options};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct IoUringExtentCopyStats {
    bytes_read: u64,
    bytes_written: u64,
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
