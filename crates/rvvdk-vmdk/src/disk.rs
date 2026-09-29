use crate::{BackingError, ResolvedDescriptor, ResolvedExtentBacking};
use rvvdk_core::{
    Capabilities, CopyEndpoint, DiskGeometry, DiskRange, Error, Extent, ExtentKind, Result,
    VirtualDisk,
};

/// Read-only logical view over retained FLAT/ZERO sources. No native RAW endpoint.
/// Call revalidate before standalone read sessions; DataMover preflight does this
/// through copy_endpoint. Retained handles do not freeze backing contents.
pub struct VmdkDisk {
    resolved: ResolvedDescriptor,
    geometry: DiskGeometry,
}
impl VmdkDisk {
    pub fn new(resolved: ResolvedDescriptor) -> std::result::Result<Self, BackingError> {
        resolved.revalidate()?;
        // Fixed logical geometry, independent of informational CHS or host alignment.
        let geometry = DiskGeometry::new(resolved.size_bytes(), 512, 512)
            .expect("fixed 512-byte geometry is valid");
        Ok(Self { resolved, geometry })
    }
    pub fn resolved(&self) -> &ResolvedDescriptor {
        &self.resolved
    }
    pub fn revalidate(&self) -> std::result::Result<(), BackingError> {
        self.resolved.revalidate()
    }
    fn validate_live(&self) -> Result<()> {
        self.revalidate()
            .map_err(|e| Error::EndpointChanged(e.to_string()))
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
    fn first_extent(&self, offset: u64) -> usize {
        self.resolved
            .extents()
            .partition_point(|e| e.logical_offset() + e.size_bytes() <= offset)
    }
}
impl VirtualDisk for VmdkDisk {
    fn geometry(&self) -> DiskGeometry {
        self.geometry
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::READ | Capabilities::EXTENTS | Capabilities::SPARSE
    }
    fn copy_endpoint(&self) -> Result<CopyEndpoint> {
        self.validate_live()?;
        // A composite logical disk has no single physical backing identity.
        Ok(CopyEndpoint {
            size: self.size(),
            capabilities: self.capabilities(),
            identity: None,
        })
    }
    fn validate_destination_identity(&self, destination: CopyEndpoint) -> Result<()> {
        // virtual_pair has just called copy_endpoint, validating known identities.
        // A direct caller must follow the same preflight ordering.
        if self.resolved.backings().is_empty() {
            return Ok(());
        }
        let identity = destination.identity.ok_or(Error::InvalidEndpoint {
            reason: "VMDK destination identity is unknown",
        })?;
        for backing in self.resolved.backings() {
            let source = backing
                .initial_endpoint()
                .identity
                .ok_or(Error::InvalidEndpoint {
                    reason: "VMDK backing identity is unknown",
                })?;
            if source == identity {
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
        self.range(offset, length)?;
        if buffer.is_empty() {
            return Ok(0);
        }
        let mut logical = offset;
        let mut done = 0;
        let first = self.first_extent(offset);
        for extent in &self.resolved.extents()[first..] {
            let within = logical - extent.logical_offset();
            let count = (extent.size_bytes() - within).min(length - done as u64) as usize;
            let target = &mut buffer[done..done + count];
            match extent.backing() {
                ResolvedExtentBacking::Zero => target.fill(0),
                ResolvedExtentBacking::Flat {
                    backing_index,
                    offset_bytes,
                } => {
                    self.resolved.backings()[backing_index]
                        .read_exact_at(offset_bytes + within, target)?;
                }
            }
            done += count;
            logical += count as u64;
            if done == buffer.len() {
                break;
            }
        }
        Ok(done)
    }
    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
        let range = self.range(offset, length)?;
        let mut out: Vec<Extent> = Vec::new();
        if range.is_empty() {
            return Ok(out);
        }
        for e in &self.resolved.extents()[self.first_extent(offset)..] {
            if e.logical_offset() >= range.end() {
                break;
            }
            let start = e.logical_offset().max(offset);
            let end = (e.logical_offset() + e.size_bytes()).min(range.end());
            let kind = match e.backing() {
                ResolvedExtentBacking::Zero => ExtentKind::Zero,
                ResolvedExtentBacking::Flat { .. } => ExtentKind::Data,
            };
            if let Some(last) = out.last_mut().filter(|last| last.kind() == kind) {
                *last = Extent::new(last.offset(), end - last.offset(), kind)?;
            } else {
                out.push(Extent::new(start, end - start, kind)?);
            }
        }
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
