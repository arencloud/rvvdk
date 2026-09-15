use crate::{Capabilities, DiskGeometry, Error, Extent, Result};

pub trait BlockDevice: Send + Sync {
    fn geometry(&self) -> DiskGeometry;

    fn capabilities(&self) -> Capabilities;

    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize>;

    fn read_exact_at(&self, mut offset: u64, mut buffer: &mut [u8]) -> Result<()> {
        while !buffer.is_empty() {
            let read = self.read_at(offset, buffer)?;

            if read == 0 {
                return Err(Error::UnexpectedEof {
                    offset,
                    remaining: buffer.len(),
                });
            }

            let read_u64 = u64::try_from(read).map_err(|_| Error::RangeOverflow {
                offset,
                length: u64::MAX,
            })?;

            offset = offset.checked_add(read_u64).ok_or(Error::RangeOverflow {
                offset,
                length: read_u64,
            })?;

            buffer = &mut buffer[read..];
        }

        Ok(())
    }

    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize>;

    fn write_all_at(&self, mut offset: u64, mut buffer: &[u8]) -> Result<()> {
        while !buffer.is_empty() {
            let written = self.write_at(offset, buffer)?;

            if written == 0 {
                return Err(Error::WriteZero {
                    offset,
                    remaining: buffer.len(),
                });
            }

            let written_u64 = u64::try_from(written).map_err(|_| Error::RangeOverflow {
                offset,
                length: u64::MAX,
            })?;

            offset = offset
                .checked_add(written_u64)
                .ok_or(Error::RangeOverflow {
                    offset,
                    length: written_u64,
                })?;

            buffer = &buffer[written..];
        }

        Ok(())
    }

    fn write_zero_at(&self, offset: u64, length: u64) -> Result<()> {
        if !self.capabilities().contains(Capabilities::WRITE_ZERO) {
            return Err(Error::Unsupported);
        }

        let length_usize =
            usize::try_from(length).map_err(|_| Error::RangeOverflow { offset, length })?;

        self.validate_range(offset, length_usize)?;

        if length == 0 {
            return Ok(());
        }

        Err(Error::Unsupported)
    }

    fn discard(&self, offset: u64, length: u64) -> Result<()> {
        if !self.capabilities().contains(Capabilities::DISCARD) {
            return Err(Error::Unsupported);
        }

        let length_usize =
            usize::try_from(length).map_err(|_| Error::RangeOverflow { offset, length })?;

        self.validate_range(offset, length_usize)?;

        if length == 0 {
            return Ok(());
        }

        Err(Error::Unsupported)
    }

    fn extents(&self, _offset: u64, _length: u64) -> Result<Vec<Extent>> {
        Err(Error::Unsupported)
    }

    fn flush(&self) -> Result<()>;

    fn size(&self) -> u64 {
        self.geometry().size()
    }

    fn is_read_only(&self) -> bool {
        !self.capabilities().contains(Capabilities::WRITE)
    }

    fn validate_range(&self, offset: u64, length: usize) -> Result<()> {
        let length = u64::try_from(length).map_err(|_| Error::RangeOverflow {
            offset,
            length: u64::MAX,
        })?;

        let end = offset
            .checked_add(length)
            .ok_or(Error::RangeOverflow { offset, length })?;

        if end > self.size() {
            return Err(Error::OutOfBounds {
                offset,
                length,
                size: self.size(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBlockDevice {
        geometry: DiskGeometry,
        capabilities: Capabilities,
    }

    impl TestBlockDevice {
        fn new(size: u64, capabilities: Capabilities) -> Self {
            Self {
                geometry: DiskGeometry::new(size, 512, 4096).unwrap(),
                capabilities,
            }
        }
    }

    impl BlockDevice for TestBlockDevice {
        fn geometry(&self) -> DiskGeometry {
            self.geometry
        }

        fn capabilities(&self) -> Capabilities {
            self.capabilities
        }

        fn read_at(&self, _offset: u64, _buffer: &mut [u8]) -> Result<usize> {
            Ok(0)
        }

        fn write_at(&self, _offset: u64, _buffer: &[u8]) -> Result<usize> {
            Ok(0)
        }

        fn flush(&self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn reports_size() {
        let device = TestBlockDevice::new(1024 * 1024, Capabilities::READ);

        assert_eq!(device.size(), 1024 * 1024);
    }

    #[test]
    fn detects_read_only_device() {
        let device = TestBlockDevice::new(1024 * 1024, Capabilities::READ);

        assert!(device.is_read_only());
    }

    #[test]
    fn detects_writable_device() {
        let device = TestBlockDevice::new(1024 * 1024, Capabilities::READ | Capabilities::WRITE);

        assert!(!device.is_read_only());
    }

    #[test]
    fn accepts_valid_range() {
        let device = TestBlockDevice::new(1024, Capabilities::READ);

        assert!(device.validate_range(512, 512).is_ok());
    }

    #[test]
    fn rejects_out_of_bounds_range() {
        let device = TestBlockDevice::new(1024, Capabilities::READ);

        let result = device.validate_range(900, 200);

        assert!(matches!(result, Err(Error::OutOfBounds { .. })));
    }

    #[test]
    fn rejects_overflowing_range() {
        let device = TestBlockDevice::new(u64::MAX, Capabilities::READ);

        let result = device.validate_range(u64::MAX, 1);

        assert!(matches!(result, Err(Error::RangeOverflow { .. })));
    }

    use std::sync::Mutex;

    struct PartialBlockDevice {
        data: Mutex<Vec<u8>>,
        geometry: DiskGeometry,
    }

    impl PartialBlockDevice {
        fn new(size: usize) -> Self {
            Self {
                data: Mutex::new(vec![0_u8; size]),
                geometry: DiskGeometry::new(size as u64, 512, 4096).unwrap(),
            }
        }
    }

    impl BlockDevice for PartialBlockDevice {
        fn geometry(&self) -> DiskGeometry {
            self.geometry
        }

        fn capabilities(&self) -> Capabilities {
            Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH
        }

        fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
            self.validate_range(offset, buffer.len())?;

            if buffer.is_empty() {
                return Ok(0);
            }

            let transfer = buffer.len().min(2);

            let start = usize::try_from(offset).unwrap();

            let end = start + transfer;

            let data = self.data.lock().unwrap();

            buffer[..transfer].copy_from_slice(&data[start..end]);

            Ok(transfer)
        }

        fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize> {
            self.validate_range(offset, buffer.len())?;

            if buffer.is_empty() {
                return Ok(0);
            }

            let transfer = buffer.len().min(2);

            let start = usize::try_from(offset).unwrap();

            let end = start + transfer;

            let mut data = self.data.lock().unwrap();

            data[start..end].copy_from_slice(&buffer[..transfer]);

            Ok(transfer)
        }

        fn flush(&self) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn exact_io_handles_partial_operations() {
        let device = PartialBlockDevice::new(4096);

        device.write_all_at(100, b"rvvdk").unwrap();

        let mut buffer = [0_u8; 5];

        device.read_exact_at(100, &mut buffer).unwrap();

        assert_eq!(&buffer, b"rvvdk",);
    }
}
