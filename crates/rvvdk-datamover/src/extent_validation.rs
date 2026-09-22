use rvvdk_core::{Error, Extent, Result};

pub(crate) fn validate_extents(extents: &[Extent], disk_size: u64) -> Result<()> {
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
