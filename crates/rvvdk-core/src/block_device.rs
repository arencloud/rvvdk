use crate::{Capabilities, DiskGeometry, Error, Result};

pub trait BlockDevice: Send + Sync {
    fn geometry(&self) -> DiskGeometry;

    fn capabilities(&self) -> Capabilities;

    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize>;

    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize>;

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
}
