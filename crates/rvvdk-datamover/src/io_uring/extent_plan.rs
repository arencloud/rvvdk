use crate::extent_validation::validate_extents;
use rvvdk_core::{Extent, ExtentKind, Result};

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

    pub fn has_holes(&self) -> bool {
        self.extents
            .iter()
            .any(|extent| extent.kind() == ExtentKind::Hole)
    }

    pub fn supports_data_and_zero(&self) -> bool {
        self.extents
            .iter()
            .all(|extent| matches!(extent.kind(), ExtentKind::Data | ExtentKind::Zero))
    }
}
