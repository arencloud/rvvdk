use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiskGeometry {
    size: u64,
    logical_block_size: u32,
    physical_block_size: u32,
}

impl DiskGeometry {
    pub fn new(size: u64, logical_block_size: u32, physical_block_size: u32) -> Result<Self> {
        if logical_block_size == 0 {
            return Err(Error::InvalidAlignment {
                value: logical_block_size as u64,
                alignment: 1,
            });
        }

        if physical_block_size == 0 {
            return Err(Error::InvalidAlignment {
                value: physical_block_size as u64,
                alignment: 1,
            });
        }

        if !physical_block_size.is_multiple_of(logical_block_size) {
            return Err(Error::InvalidAlignment {
                value: physical_block_size as u64,
                alignment: logical_block_size as u64,
            });
        }

        Ok(Self {
            size,
            logical_block_size,
            physical_block_size,
        })
    }

    pub const fn size(&self) -> u64 {
        self.size
    }

    pub const fn logical_block_size(&self) -> u32 {
        self.logical_block_size
    }

    pub const fn physical_block_size(&self) -> u32 {
        self.physical_block_size
    }
}
