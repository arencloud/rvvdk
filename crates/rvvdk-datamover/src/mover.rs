use std::time::Instant;

use rvvdk_core::{Capabilities, Error, Extent, ExtentKind, Result, VirtualDisk};

use crate::{CopyOptions, CopyStats};

pub struct DataMover {
    options: CopyOptions,
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
    fn validate_extents(&self, extents: &[Extent], disk_size: u64) -> Result<()> {
        if disk_size == 0 {
            if extents.is_empty() {
                return Ok(());
            }

            return Err(Error::CorruptMetadata(
                "zero-sized disk returned extents".into(),
            ));
        }

        if extents.is_empty() {
            return Err(Error::CorruptMetadata(
                "non-empty disk returned no extents".into(),
            ));
        }

        let mut expected_offset = 0_u64;

        for extent in extents {
            if extent.offset() != expected_offset {
                return Err(Error::CorruptMetadata(format!(
                    "invalid extent map: expected offset \
                     {expected_offset}, got {}",
                    extent.offset(),
                )));
            }

            if extent.end() > disk_size {
                return Err(Error::CorruptMetadata(format!(
                    "extent exceeds disk size: \
                     end={}, size={disk_size}",
                    extent.end(),
                )));
            }

            expected_offset = extent.end();
        }

        if expected_offset != disk_size {
            return Err(Error::CorruptMetadata(format!(
                "extent map ends at \
                 {expected_offset}, disk size is \
                 {disk_size}",
            )));
        }

        Ok(())
    }
    pub const fn new(options: CopyOptions) -> Self {
        Self { options }
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

        self.validate_extents(&extents, source_size)?;

        let mut buffer = vec![0_u8; self.options.block_size()];

        let mut stats = MutableStats::default();

        for extent in extents {
            self.process_extent(source, destination, extent, &mut buffer, &mut stats)?;

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
}
