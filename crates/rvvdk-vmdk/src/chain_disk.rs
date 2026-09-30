//! Logical reads from a validated, quiescent sparse chain.
use crate::{SparseChain, SparseChainError, SparseMetadata};
use rvvdk_core::{
    Capabilities, CopyEndpoint, DiskGeometry, DiskRange, Error, Extent, ExtentKind, Result,
    VirtualDisk,
};

#[derive(Clone, Copy, Debug)]
pub struct SparseChainDiskLimits {
    /// Maximum coalesced logical entries returned by one extents() query.
    pub output_extents: usize,
}
impl Default for SparseChainDiskLimits {
    fn default() -> Self {
        Self {
            output_extents: 65536,
        }
    }
}
/// Read-only sparse chain. Keep every source quiescent and revalidate before read
/// sessions. Endpoint observations do not provide snapshot consistency.
pub struct SparseChainDisk {
    chain: SparseChain,
    limits: SparseChainDiskLimits,
}
#[derive(Clone, Copy)]
struct Run<'a> {
    offset: u64,
    length: u64,
    data: Option<(&'a SparseMetadata, u64)>,
}
impl SparseChainDisk {
    pub fn new(chain: SparseChain) -> Self {
        Self::with_limits(chain, SparseChainDiskLimits::default())
    }
    pub fn with_limits(chain: SparseChain, limits: SparseChainDiskLimits) -> Self {
        Self { chain, limits }
    }
    pub fn chain(&self) -> &SparseChain {
        &self.chain
    }
    pub fn revalidate(&self) -> std::result::Result<(), SparseChainError> {
        self.chain.revalidate()
    }
    fn range(&self, offset: u64, length: u64) -> Result<DiskRange> {
        let range = DiskRange::new(offset, length)?;
        if range.end() > self.size() {
            return Err(Error::OutOfBounds {
                offset,
                length,
                size: self.size(),
            });
        }
        Ok(range)
    }
    /// Clip at every consulted child/ancestor boundary. A larger parent grain
    /// must never hide an allocated child grain just beyond the current position.
    fn resolve(&self, pos: u64, end: u64) -> Run<'_> {
        let mut stop = end;
        for layer in self.chain.layers() {
            let maps = layer.metadata();
            let index =
                maps.partition_point(|m| m.logical_offset() + m.header().capacity_bytes() <= pos);
            let m = &maps[index]; // validated layers cover the same nonempty capacity
            let local = pos - m.logical_offset();
            let grain = m.header().grain_bytes();
            let index = (local / grain) as usize;
            stop = stop.min(m.logical_offset() + (index as u64 + 1) * grain);
            let entry = m.grain_sectors()[index];
            if entry != 0 {
                return Run {
                    offset: pos,
                    length: stop - pos,
                    data: Some((m, u64::from(entry) * 512 + local % grain)),
                };
            }
        }
        Run {
            offset: pos,
            length: stop - pos,
            data: None,
        }
    }
    /// Constant auxiliary space; coalesce only the same retained map and adjacent
    /// physical bytes, or adjacent fully resolved zero ranges.
    fn runs(&self, range: DiskRange, mut visit: impl FnMut(Run<'_>) -> Result<()>) -> Result<()> {
        let mut pos = range.offset();
        let mut pending: Option<Run<'_>> = None;
        while pos < range.end() {
            let next = self.resolve(pos, range.end());
            pos += next.length;
            if let Some(previous) = pending.as_mut() {
                let adjacent = match (previous.data, next.data) {
                    (None, None) => true,
                    (Some((a, offset)), Some((b, next_offset))) => {
                        std::ptr::eq(a, b) && offset + previous.length == next_offset
                    }
                    _ => false,
                };
                if adjacent {
                    previous.length += next.length;
                    continue;
                }
                visit(*previous)?;
            }
            pending = Some(next);
        }
        if let Some(run) = pending {
            visit(run)?;
        }
        Ok(())
    }
}
impl VirtualDisk for SparseChainDisk {
    fn geometry(&self) -> DiskGeometry {
        DiskGeometry::new(self.chain.layers()[0].size_bytes(), 512, 512)
            .expect("validated sector geometry")
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::READ | Capabilities::EXTENTS | Capabilities::SPARSE
    }
    fn copy_endpoint(&self) -> Result<CopyEndpoint> {
        self.revalidate()
            .map_err(|e| Error::EndpointChanged(e.to_string()))?;
        Ok(CopyEndpoint {
            size: self.size(),
            capabilities: self.capabilities(),
            identity: None,
        })
    }
    fn validate_destination_identity(&self, destination: CopyEndpoint) -> Result<()> {
        self.revalidate()
            .map_err(|e| Error::EndpointChanged(e.to_string()))?;
        let id = destination.identity.ok_or(Error::InvalidEndpoint {
            reason: "chain destination identity is unknown",
        })?;
        for layer in self.chain.layers() {
            if layer.descriptor_endpoint().identity == Some(id)
                || layer
                    .metadata()
                    .iter()
                    .any(|m| m.initial_endpoint().identity == Some(id))
            {
                return Err(Error::AliasedEndpoints);
            }
        }
        Ok(())
    }
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        let length = u64::try_from(buffer.len()).map_err(|_| Error::RangeOverflow {
            offset,
            length: u64::MAX,
        })?;
        self.runs(self.range(offset, length)?, |run| {
            let start = (run.offset - offset) as usize;
            let out = &mut buffer[start..start + run.length as usize];
            match run.data {
                Some((map, physical)) => map.read_physical(physical, out)?,
                None => out.fill(0),
            };
            Ok(())
        })?;
        Ok(buffer.len())
    }
    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
        let range = self.range(offset, length)?;
        let mut previous = None;
        let mut count = 0usize;
        self.runs(range, |run| {
            let data = run.data.is_some();
            if previous != Some(data) {
                if count == self.limits.output_extents {
                    return Err(Error::InvalidEndpoint {
                        reason: "chain output extent limit",
                    });
                }
                count += 1;
                previous = Some(data);
            }
            Ok(())
        })?;
        let bytes = count
            .checked_mul(std::mem::size_of::<Extent>())
            .ok_or(Error::MemoryAccountingOverflow)?;
        let mut out: Vec<Extent> = Vec::new();
        out.try_reserve_exact(count)
            .map_err(|_| Error::BufferAllocation {
                size: bytes,
                alignment: std::mem::align_of::<Extent>(),
            })?;
        self.runs(range, |run| {
            let kind = if run.data.is_some() {
                ExtentKind::Data
            } else {
                ExtentKind::Zero
            };
            if let Some(last) = out.last_mut().filter(|last| last.kind() == kind) {
                *last = Extent::new(last.offset(), run.offset + run.length - last.offset(), kind)?;
            } else {
                out.push(Extent::new(run.offset, run.length, kind)?);
            }
            Ok(())
        })?;
        Ok(out)
    }
    fn write_at(&self, _: u64, _: &[u8]) -> Result<usize> {
        Err(Error::Unsupported)
    }
    fn write_zero_at(&self, _: u64, _: u64) -> Result<()> {
        Err(Error::Unsupported)
    }
    fn discard(&self, _: u64, _: u64) -> Result<()> {
        Err(Error::Unsupported)
    }
    fn flush(&self) -> Result<()> {
        Err(Error::Unsupported)
    }
}
