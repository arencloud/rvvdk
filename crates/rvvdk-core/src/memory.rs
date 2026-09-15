use std::sync::RwLock;

use crate::{BlockDevice, Capabilities, DiskGeometry, Error, Result};

pub struct MemoryBlockDevice {
    data: RwLock<Vec<u8>>,
    geometry: DiskGeometry,
    capabilities: Capabilities,
}

impl MemoryBlockDevice {
    pub fn new(size: usize) -> Result<Self> {
        Self::with_capabilities(
            size,
            Capabilities::READ
                | Capabilities::WRITE
                | Capabilities::FLUSH
                | Capabilities::WRITE_ZERO
                | Capabilities::DISCARD,
        )
    }

    pub fn read_only(size: usize) -> Result<Self> {
        Self::with_capabilities(size, Capabilities::READ)
    }

    pub fn with_capabilities(size: usize, capabilities: Capabilities) -> Result<Self> {
        let size_u64 = u64::try_from(size).map_err(|_| Error::RangeOverflow {
            offset: 0,
            length: u64::MAX,
        })?;

        let geometry = DiskGeometry::new(size_u64, 512, 4096)?;

        Ok(Self {
            data: RwLock::new(vec![0; size]),
            geometry,
            capabilities,
        })
    }
}

impl BlockDevice for MemoryBlockDevice {
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

        let start = usize::try_from(offset).map_err(|_| Error::RangeOverflow {
            offset,
            length: buffer.len() as u64,
        })?;

        let end = start
            .checked_add(buffer.len())
            .ok_or(Error::RangeOverflow {
                offset,
                length: buffer.len() as u64,
            })?;

        let data = self
            .data
            .read()
            .map_err(|_| Error::CorruptMetadata("memory block device lock poisoned".into()))?;

        buffer.copy_from_slice(&data[start..end]);

        Ok(buffer.len())
    }

    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize> {
        if !self.capabilities.contains(Capabilities::WRITE) {
            return Err(Error::Unsupported);
        }

        self.validate_range(offset, buffer.len())?;

        let start = usize::try_from(offset).map_err(|_| Error::RangeOverflow {
            offset,
            length: buffer.len() as u64,
        })?;

        let end = start
            .checked_add(buffer.len())
            .ok_or(Error::RangeOverflow {
                offset,
                length: buffer.len() as u64,
            })?;

        let mut data = self
            .data
            .write()
            .map_err(|_| Error::CorruptMetadata("memory block device lock poisoned".into()))?;

        data[start..end].copy_from_slice(buffer);

        Ok(buffer.len())
    }

    fn write_zero_at(&self, offset: u64, length: u64) -> Result<()> {
        if !self.capabilities.contains(Capabilities::WRITE_ZERO) {
            return Err(Error::Unsupported);
        }

        let length_usize =
            usize::try_from(length).map_err(|_| Error::RangeOverflow { offset, length })?;

        self.validate_range(offset, length_usize)?;

        if length == 0 {
            return Ok(());
        }

        let start = usize::try_from(offset).map_err(|_| Error::RangeOverflow { offset, length })?;

        let end = start
            .checked_add(length_usize)
            .ok_or(Error::RangeOverflow { offset, length })?;

        let mut data = self
            .data
            .write()
            .map_err(|_| Error::CorruptMetadata("memory block device lock poisoned".into()))?;

        data[start..end].fill(0);

        Ok(())
    }

    fn discard(&self, offset: u64, length: u64) -> Result<()> {
        if !self.capabilities.contains(Capabilities::DISCARD) {
            return Err(Error::Unsupported);
        }

        let length_usize =
            usize::try_from(length).map_err(|_| Error::RangeOverflow { offset, length })?;

        self.validate_range(offset, length_usize)?;

        if length == 0 {
            return Ok(());
        }

        let start = usize::try_from(offset).map_err(|_| Error::RangeOverflow { offset, length })?;

        let end = start
            .checked_add(length_usize)
            .ok_or(Error::RangeOverflow { offset, length })?;

        let mut data = self
            .data
            .write()
            .map_err(|_| Error::CorruptMetadata("memory block device lock poisoned".into()))?;

        data[start..end].fill(0);

        Ok(())
    }

    fn flush(&self) -> Result<()> {
        if !self.capabilities.contains(Capabilities::FLUSH) {
            return Err(Error::Unsupported);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_memory_device() {
        let device = MemoryBlockDevice::new(1024 * 1024).unwrap();

        assert_eq!(device.size(), 1024 * 1024);

        assert!(device.capabilities().contains(Capabilities::READ));

        assert!(device.capabilities().contains(Capabilities::WRITE));

        assert!(device.capabilities().contains(Capabilities::FLUSH));
    }

    #[test]
    fn new_device_is_zero_initialized() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        let mut buffer = [0xff; 512];

        let bytes_read = device.read_at(0, &mut buffer).unwrap();

        assert_eq!(bytes_read, 512);
        assert_eq!(buffer, [0; 512]);
    }

    #[test]
    fn writes_and_reads_data() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        let input = b"rvvdk";

        let bytes_written = device.write_at(1024, input).unwrap();

        assert_eq!(bytes_written, input.len());

        let mut output = [0; 5];

        let bytes_read = device.read_at(1024, &mut output).unwrap();

        assert_eq!(bytes_read, input.len());
        assert_eq!(&output, input);
    }

    #[test]
    fn preserves_unmodified_data() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        device.write_at(100, &[1, 2, 3, 4]).unwrap();

        let mut before = [0xff; 4];
        let mut after = [0xff; 4];

        device.read_at(96, &mut before).unwrap();

        device.read_at(104, &mut after).unwrap();

        assert_eq!(before, [0, 0, 0, 0]);
        assert_eq!(after, [0, 0, 0, 0]);
    }

    #[test]
    fn rejects_out_of_bounds_read() {
        let device = MemoryBlockDevice::new(1024).unwrap();

        let mut buffer = [0; 512];

        let result = device.read_at(800, &mut buffer);

        assert!(matches!(result, Err(Error::OutOfBounds { .. })));
    }

    #[test]
    fn rejects_out_of_bounds_write() {
        let device = MemoryBlockDevice::new(1024).unwrap();

        let buffer = [0; 512];

        let result = device.write_at(800, &buffer);

        assert!(matches!(result, Err(Error::OutOfBounds { .. })));
    }

    #[test]
    fn rejects_write_on_read_only_device() {
        let device = MemoryBlockDevice::read_only(4096).unwrap();

        let result = device.write_at(0, &[1, 2, 3, 4]);

        assert!(matches!(result, Err(Error::Unsupported)));
    }

    #[test]
    fn allows_read_on_read_only_device() {
        let device = MemoryBlockDevice::read_only(4096).unwrap();

        let mut buffer = [0; 512];

        let result = device.read_at(0, &mut buffer);

        assert!(result.is_ok());
    }

    #[test]
    fn flush_succeeds_when_supported() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        assert!(device.flush().is_ok());
    }

    #[test]
    fn flush_fails_when_not_supported() {
        let device = MemoryBlockDevice::read_only(4096).unwrap();

        assert!(matches!(device.flush(), Err(Error::Unsupported)));
    }

    #[test]
    fn zero_length_read_is_valid() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        let mut buffer = [];

        assert_eq!(device.read_at(4096, &mut buffer).unwrap(), 0);
    }

    #[test]
    fn zero_length_write_is_valid() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        let buffer = [];

        assert_eq!(device.write_at(4096, &buffer).unwrap(), 0);
    }

    #[test]
    fn read_exact_at_reads_complete_buffer() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        device.write_at(100, b"rvvdk").unwrap();

        let mut buffer = [0_u8; 5];

        device.read_exact_at(100, &mut buffer).unwrap();

        assert_eq!(&buffer, b"rvvdk");
    }

    #[test]
    fn write_all_at_writes_complete_buffer() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        device.write_all_at(200, b"rvvdk").unwrap();

        let mut buffer = [0_u8; 5];

        device.read_exact_at(200, &mut buffer).unwrap();

        assert_eq!(&buffer, b"rvvdk");
    }

    #[test]
    fn write_zero_clears_requested_range() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        device.write_all_at(100, b"abcdefghij").unwrap();

        device.write_zero_at(103, 4).unwrap();

        let mut buffer = [0_u8; 10];

        device.read_exact_at(100, &mut buffer).unwrap();

        assert_eq!(buffer, [b'a', b'b', b'c', 0, 0, 0, 0, b'h', b'i', b'j',]);
    }

    #[test]
    fn discard_clears_memory_backend_range() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        device.write_all_at(100, b"abcdefghij").unwrap();

        device.discard(102, 5).unwrap();

        let mut buffer = [0_u8; 10];

        device.read_exact_at(100, &mut buffer).unwrap();

        assert_eq!(buffer, [b'a', b'b', 0, 0, 0, 0, 0, b'h', b'i', b'j',]);
    }

    #[test]
    fn write_zero_rejects_out_of_bounds_range() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        let result = device.write_zero_at(4000, 512);

        assert!(matches!(result, Err(Error::OutOfBounds { .. })));
    }

    #[test]
    fn read_only_device_rejects_write_zero() {
        let device = MemoryBlockDevice::read_only(4096).unwrap();

        assert!(matches!(
            device.write_zero_at(0, 512),
            Err(Error::Unsupported)
        ));
    }

    #[test]
    fn read_only_device_rejects_discard() {
        let device = MemoryBlockDevice::read_only(4096).unwrap();

        assert!(matches!(device.discard(0, 512), Err(Error::Unsupported)));
    }
}
