use crate::{
    BlockDevice, Capabilities, DiskGeometry, Error, Extent, ExtentKind, Result, VirtualDisk,
};

pub struct RawDisk<D>
where
    D: BlockDevice,
{
    device: D,
}

impl<D> RawDisk<D>
where
    D: BlockDevice,
{
    pub fn new(device: D) -> Self {
        Self { device }
    }

    pub fn device(&self) -> &D {
        &self.device
    }

    pub fn into_inner(self) -> D {
        self.device
    }
}

impl<D> VirtualDisk for RawDisk<D>
where
    D: BlockDevice,
{
    fn geometry(&self) -> DiskGeometry {
        self.device.geometry()
    }

    fn capabilities(&self) -> Capabilities {
        self.device.capabilities()
    }

    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        self.device.read_at(offset, buffer)
    }

    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize> {
        self.device.write_at(offset, buffer)
    }

    fn flush(&self) -> Result<()> {
        self.device.flush()
    }

    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
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

        if length == 0 {
            return Ok(Vec::new());
        }

        Ok(vec![Extent::new(offset, length, ExtentKind::Data)?])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MemoryBlockDevice;

    #[test]
    fn exposes_device_geometry() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        let disk = RawDisk::new(device);

        assert_eq!(disk.size(), 4096);
    }

    #[test]
    fn writes_and_reads_data() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        let disk = RawDisk::new(device);

        disk.write_at(1024, b"rvvdk").unwrap();

        let mut buffer = [0_u8; 5];

        disk.read_at(1024, &mut buffer).unwrap();

        assert_eq!(&buffer, b"rvvdk");
    }

    #[test]
    fn exposes_full_range_as_data_extent() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        let disk = RawDisk::new(device);

        let extents = disk.extents(0, 4096).unwrap();

        assert_eq!(extents.len(), 1);

        assert_eq!(extents[0].kind(), ExtentKind::Data,);

        assert_eq!(extents[0].offset(), 0);
        assert_eq!(extents[0].length(), 4096);
    }

    #[test]
    fn exposes_partial_range_as_data_extent() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        let disk = RawDisk::new(device);

        let extents = disk.extents(1024, 512).unwrap();

        assert_eq!(extents.len(), 1);
        assert_eq!(extents[0].offset(), 1024);
        assert_eq!(extents[0].length(), 512);
    }

    #[test]
    fn empty_range_has_no_extents() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        let disk = RawDisk::new(device);

        let extents = disk.extents(4096, 0).unwrap();

        assert!(extents.is_empty());
    }

    #[test]
    fn rejects_extent_outside_disk() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        let disk = RawDisk::new(device);

        let result = disk.extents(4000, 512);

        assert!(matches!(result, Err(Error::OutOfBounds { .. })));
    }

    #[test]
    fn preserves_device_capabilities() {
        let device = MemoryBlockDevice::read_only(4096).unwrap();

        let disk = RawDisk::new(device);

        assert!(disk.capabilities().contains(Capabilities::READ));

        assert!(!disk.capabilities().contains(Capabilities::WRITE));

        assert!(disk.is_read_only());
    }

    #[test]
    fn returns_underlying_device() {
        let device = MemoryBlockDevice::new(4096).unwrap();

        let disk = RawDisk::new(device);

        let device = disk.into_inner();

        assert_eq!(device.size(), 4096);
    }
}
