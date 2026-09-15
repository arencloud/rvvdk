use std::fs::File;
use std::os::unix::fs::FileExt;
use std::path::Path;

use rvvdk_core::{BlockDevice, Capabilities, DiskGeometry, Error, Result};

pub struct LocalFileBlockDevice {
    file: File,
    geometry: DiskGeometry,
    capabilities: Capabilities,
}

impl LocalFileBlockDevice {
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::options().read(true).open(path)?;

        Self::from_file(file, Capabilities::READ)
    }

    pub fn open_read_write(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::options().read(true).write(true).open(path)?;

        Self::from_file(
            file,
            Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH,
        )
    }

    fn from_file(file: File, capabilities: Capabilities) -> Result<Self> {
        let metadata = file.metadata()?;

        if !metadata.is_file() {
            return Err(Error::Unsupported);
        }

        let geometry = DiskGeometry::new(metadata.len(), 512, 4096)?;

        Ok(Self {
            file,
            geometry,
            capabilities,
        })
    }
}

impl BlockDevice for LocalFileBlockDevice {
    fn geometry(&self) -> DiskGeometry {
        self.geometry
    }

    fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        if !self.capabilities.contains(Capabilities::READ) {
            return Err(Error::Unsupported);
        }

        self.validate_range(offset, buffer.len())?;

        Ok(self.file.read_at(buffer, offset)?)
    }

    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize> {
        if !self.capabilities.contains(Capabilities::WRITE) {
            return Err(Error::Unsupported);
        }

        self.validate_range(offset, buffer.len())?;

        Ok(self.file.write_at(buffer, offset)?)
    }

    fn flush(&self) -> Result<()> {
        if !self.capabilities.contains(Capabilities::FLUSH) {
            return Err(Error::Unsupported);
        }

        self.file.sync_data()?;

        Ok(())
    }
}
