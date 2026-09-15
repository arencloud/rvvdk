use rvvdk_core::{Error, Result};

pub const DEFAULT_BUFFER_COUNT: usize = 1;
pub const DEFAULT_BLOCK_SIZE: usize = 1024 * 1024;
pub const DEFAULT_BUFFER_ALIGNMENT: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyOptions {
    block_size: usize,
    buffer_alignment: usize,
    buffer_count: usize,
}

impl Default for CopyOptions {
    fn default() -> Self {
        Self {
            block_size: DEFAULT_BLOCK_SIZE,
            buffer_alignment: DEFAULT_BUFFER_ALIGNMENT,
            buffer_count: DEFAULT_BUFFER_COUNT,
        }
    }
}

impl CopyOptions {
    pub fn new(block_size: usize) -> Result<Self> {
        if block_size == 0 {
            return Err(Error::InvalidAlignment {
                value: 0,
                alignment: 1,
            });
        }

        Ok(Self {
            block_size,
            buffer_alignment: DEFAULT_BUFFER_ALIGNMENT,
            buffer_count: DEFAULT_BUFFER_COUNT,
        })
    }

    pub const fn block_size(&self) -> usize {
        self.block_size
    }
    pub fn with_alignment(block_size: usize, buffer_alignment: usize) -> Result<Self> {
        if block_size == 0 {
            return Err(Error::InvalidAlignment {
                value: 0,
                alignment: 1,
            });
        }

        if buffer_alignment == 0 || !buffer_alignment.is_power_of_two() {
            return Err(Error::InvalidBufferAlignment {
                alignment: buffer_alignment,
            });
        }

        Ok(Self {
            block_size,
            buffer_alignment,
            buffer_count: DEFAULT_BUFFER_COUNT,
        })
    }
    pub const fn buffer_alignment(&self) -> usize {
        self.buffer_alignment
    }

    pub fn with_buffer_pool(
        block_size: usize,
        buffer_alignment: usize,
        buffer_count: usize,
    ) -> Result<Self> {
        if block_size == 0 {
            return Err(Error::InvalidAlignment {
                value: 0,
                alignment: 1,
            });
        }

        if buffer_alignment == 0 || !buffer_alignment.is_power_of_two() {
            return Err(Error::InvalidBufferAlignment {
                alignment: buffer_alignment,
            });
        }

        if buffer_count == 0 {
            return Err(Error::InvalidBufferPoolCapacity);
        }

        Ok(Self {
            block_size,
            buffer_alignment,
            buffer_count,
        })
    }
    pub const fn buffer_count(&self) -> usize {
        self.buffer_count
    }
}
