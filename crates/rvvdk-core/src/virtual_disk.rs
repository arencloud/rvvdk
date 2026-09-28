use crate::{Capabilities, DiskGeometry, Error, Extent, Result};

pub trait VirtualDisk: Send + Sync {
    fn geometry(&self) -> DiskGeometry;

    fn capabilities(&self) -> Capabilities;

    /// Return current access, capacity, and known backing identity for preflight.
    fn copy_endpoint(&self) -> Result<crate::CopyEndpoint> {
        Ok(crate::CopyEndpoint {
            size: self.size(),
            capabilities: self.capabilities(),
            identity: None,
        })
    }

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

    /// On success, the complete requested logical range reads zero. Preserve
    /// bytes outside the range and disk size; durability requires flush.
    fn write_zero_at(&self, offset: u64, length: u64) -> Result<()>;

    /// Request discard without assuming its read-back contents. With both
    /// DISCARD and DISCARD_ZEROES advertised, success must make the entire
    /// requested logical range read zero and preserve surrounding bytes/size.
    /// A failed call may have partial effects; callers must not assume rollback.
    fn discard(&self, offset: u64, length: u64) -> Result<()>;

    fn flush(&self) -> Result<()>;

    /// Describe logical contents. Zero and Hole both guarantee zero reads;
    /// physical unallocation that exposes parent data must not be reported as Hole.
    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>>;

    fn size(&self) -> u64 {
        self.geometry().size()
    }

    fn is_read_only(&self) -> bool {
        !self.capabilities().contains(Capabilities::WRITE)
    }
}
