use crate::{Capabilities, DiskGeometry, Error, Extent, Result};

pub trait VirtualDisk: Send + Sync {
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

    fn flush(&self) -> Result<()>;

    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>>;

    fn size(&self) -> u64 {
        self.geometry().size()
    }

    fn is_read_only(&self) -> bool {
        !self.capabilities().contains(Capabilities::WRITE)
    }
}
