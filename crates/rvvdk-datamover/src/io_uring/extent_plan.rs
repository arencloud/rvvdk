use rvvdk_core::{Error, Extent, ExtentKind, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeExtentPlan {
    extents: Vec<Extent>,
    disk_size: u64,
}

impl NativeExtentPlan {
    pub fn new(extents: Vec<Extent>, disk_size: u64) -> Result<Self> {
        validate_extents(&extents, disk_size)?;

        Ok(Self { extents, disk_size })
    }

    pub fn extents(&self) -> &[Extent] {
        &self.extents
    }

    pub const fn disk_size(&self) -> u64 {
        self.disk_size
    }

    pub fn len(&self) -> usize {
        self.extents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.extents.is_empty()
    }

    pub fn is_data_only(&self) -> bool {
        self.extents
            .iter()
            .all(|extent| extent.kind() == ExtentKind::Data)
    }
}

fn validate_extents(extents: &[Extent], disk_size: u64) -> Result<()> {
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
