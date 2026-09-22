use std::fs::{File, OpenOptions};
use std::os::fd::{AsRawFd, RawFd};
use std::os::unix::fs::FileExt;
use std::path::Path;

use rvvdk_core::{BlockDevice, Capabilities, DiskGeometry, Error, Extent, Result};

use rvvdk_platform::{LinuxFdBackend, LinuxFdCapabilities};

const ZERO_CHUNK_SIZE: usize = 64 * 1024;

pub struct TestExtentBlockDevice {
    file: File,
    geometry: DiskGeometry,
    capabilities: Capabilities,
    extents: Vec<Extent>,
}

impl TestExtentBlockDevice {
    pub fn create<P>(
        path: P,
        size: u64,
        capabilities: Capabilities,
        extents: Vec<Extent>,
    ) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        file.set_len(size)?;

        let geometry = DiskGeometry::new(size, 512, 4096)?;

        Ok(Self {
            file,
            geometry,
            capabilities,
            extents,
        })
    }

    pub fn open_read_only<P>(
        path: P,
        size: u64,
        capabilities: Capabilities,
        extents: Vec<Extent>,
    ) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = OpenOptions::new().read(true).open(path)?;

        let metadata = file.metadata()?;

        if metadata.len() < size {
            return Err(Error::OutOfBounds {
                offset: 0,
                length: size,
                size: metadata.len(),
            });
        }

        let geometry = DiskGeometry::new(size, 512, 4096)?;

        Ok(Self {
            file,
            geometry,
            capabilities,
            extents,
        })
    }

    fn zero_range(&self, offset: u64, length: u64) -> Result<()> {
        let length_usize =
            usize::try_from(length).map_err(|_| Error::RangeOverflow { offset, length })?;

        self.validate_range(offset, length_usize)?;

        if length == 0 {
            return Ok(());
        }

        let zeros = vec![0_u8; ZERO_CHUNK_SIZE];

        let end = offset
            .checked_add(length)
            .ok_or(Error::RangeOverflow { offset, length })?;

        let mut current = offset;

        while current < end {
            let remaining = end - current;

            let request_u64 = remaining.min(ZERO_CHUNK_SIZE as u64);

            let request = usize::try_from(request_u64).map_err(|_| Error::RangeOverflow {
                offset: current,
                length: request_u64,
            })?;

            self.file.write_all_at(&zeros[..request], current)?;

            current = current
                .checked_add(request_u64)
                .ok_or(Error::RangeOverflow {
                    offset: current,
                    length: request_u64,
                })?;
        }

        Ok(())
    }
}

impl BlockDevice for TestExtentBlockDevice {
    fn geometry(&self) -> DiskGeometry {
        self.geometry
    }

    fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        self.validate_range(offset, buffer.len())?;

        if buffer.is_empty() {
            return Ok(0);
        }

        Ok(self.file.read_at(buffer, offset)?)
    }

    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize> {
        if !self.capabilities.contains(Capabilities::WRITE) {
            return Err(Error::Unsupported);
        }

        self.validate_range(offset, buffer.len())?;

        if buffer.is_empty() {
            return Ok(0);
        }

        Ok(self.file.write_at(buffer, offset)?)
    }

    fn write_zero_at(&self, offset: u64, length: u64) -> Result<()> {
        if !self.capabilities.contains(Capabilities::WRITE_ZERO) {
            return Err(Error::Unsupported);
        }

        self.zero_range(offset, length)
    }

    fn discard(&self, offset: u64, length: u64) -> Result<()> {
        if !self.capabilities.contains(Capabilities::DISCARD) {
            return Err(Error::Unsupported);
        }

        /*
         * M19E validates logical disk semantics rather than physical
         * filesystem allocation.
         *
         * Represent discard as a logically zero range so both
         * execution engines can be compared deterministically.
         */
        self.zero_range(offset, length)
    }

    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
        let end = offset
            .checked_add(length)
            .ok_or(Error::RangeOverflow { offset, length })?;

        if end > self.geometry.size() {
            return Err(Error::OutOfBounds {
                offset,
                length,
                size: self.geometry.size(),
            });
        }

        if length == 0 {
            return Ok(Vec::new());
        }

        if !self.capabilities.contains(Capabilities::EXTENTS) {
            return Err(Error::Unsupported);
        }

        /*
         * The deterministic M19E backend intentionally exposes one
         * predefined complete-disk extent map.
         *
         * Partial extent-map queries are rejected rather than
         * returning metadata that does not precisely describe the
         * requested range.
         */
        if offset != 0 || length != self.geometry.size() {
            return Err(Error::Unsupported);
        }

        Ok(self.extents.clone())
    }

    fn flush(&self) -> Result<()> {
        if self.capabilities.contains(Capabilities::FLUSH) {
            self.file.sync_all()?;
        }

        Ok(())
    }
}

impl LinuxFdBackend for TestExtentBlockDevice {
    fn raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }

    fn linux_fd_capabilities(&self) -> LinuxFdCapabilities {
        /*
         * The deterministic parity backend uses ordinary buffered file
         * descriptors rather than O_DIRECT.
         *
         * Buffered I/O imposes no additional native alignment
         * requirement beyond byte alignment.
         */
        LinuxFdCapabilities::new(false, 1, 1)
    }
}
