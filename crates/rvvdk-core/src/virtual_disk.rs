use crate::{Capabilities, DiskGeometry, Extent, Result};

pub trait VirtualDisk: Send + Sync {
    fn geometry(&self) -> DiskGeometry;

    fn capabilities(&self) -> Capabilities;

    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize>;

    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize>;

    fn flush(&self) -> Result<()>;

    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>>;

    fn size(&self) -> u64 {
        self.geometry().size()
    }

    fn is_read_only(&self) -> bool {
        !self.capabilities().contains(Capabilities::WRITE)
    }
}
