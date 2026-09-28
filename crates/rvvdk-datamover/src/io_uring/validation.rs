use std::alloc::Layout;

use rvvdk_core::{Error, Result};

// Validate without allocating: every entry point, including empty and all-zero
// plans, must have the same configuration contract before touching a backend.
pub(super) fn validate_copy_configuration(block_size: usize, alignment: usize) -> Result<()> {
    if block_size == 0 {
        return Err(Error::InvalidAlignment {
            value: 0,
            alignment: 1,
        });
    }
    if !alignment.is_power_of_two() {
        return Err(Error::InvalidBufferAlignment { alignment });
    }
    // Read/Write SQEs encode length as u32. Reject before allocating the pool.
    u32::try_from(block_size).map_err(|_| Error::RangeOverflow {
        offset: 0,
        length: block_size as u64,
    })?;
    Layout::from_size_align(block_size, alignment).map_err(|_| Error::BufferAllocation {
        size: block_size,
        alignment,
    })?;
    Ok(())
}

// This API addresses explicit file ranges in the nonnegative signed 64-bit
// domain, including a representable exclusive end. In particular, u64::MAX
// must never reach Read/Write: io_uring interprets it as the current-position
// sentinel instead of the caller's absolute offset.
pub(super) fn validate_file_range(offset: u64, length: u64) -> Result<u64> {
    offset
        .checked_add(length)
        .filter(|end| *end <= i64::MAX as u64)
        .ok_or(Error::RangeOverflow { offset, length })
}
