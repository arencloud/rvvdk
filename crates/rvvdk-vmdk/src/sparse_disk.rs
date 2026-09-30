//! Read-only logical access to validated, quiescent base sparse sources.
use crate::{
    BackingResolver, SparseDescriptor, SparseMetadata, SparseMetadataError, SparseMetadataLimits,
};
use rvvdk_core::{
    Capabilities, CopyEndpoint, DiskGeometry, DiskRange, Error, Extent, ExtentKind, Result,
    VirtualDisk,
};

#[derive(Clone, Copy, Debug)]
pub struct SparseDiskLimits {
    pub metadata: SparseMetadataLimits,
    /// Maximum resolved extents/handles, including repeated references.
    pub extents: usize,
    /// Conservative sum of per-extent loader reservations plus metadata structs.
    pub memory_bytes: u64,
    /// Sum of complete metadata reads across all extents.
    pub read_bytes: u64,
    /// Maximum coalesced entries returned by one extents() query.
    pub output_extents: usize,
}
impl Default for SparseDiskLimits {
    fn default() -> Self {
        Self {
            metadata: SparseMetadataLimits::default(),
            extents: 128,
            memory_bytes: 128 << 20,
            read_bytes: 256 << 20,
            output_extents: 65536,
        }
    }
}

/// Owned base sparse disk. No parent resolution, writes, or native RAW endpoint.
/// Standalone callers must revalidate before read sessions and keep all sources
/// quiescent throughout use. Endpoint observations do not freeze file contents.
pub struct SparseDisk {
    metadata: Vec<SparseMetadata>,
    geometry: DiskGeometry,
    cid: u32,
    reserved_memory: u64,
    read_bytes: u64,
    output_extents: usize,
}
impl SparseDisk {
    pub fn load(
        descriptor: &SparseDescriptor<'_>,
        resolver: &dyn BackingResolver,
        limits: SparseDiskLimits,
    ) -> std::result::Result<Self, SparseMetadataError> {
        let count = descriptor.extents().len();
        if count > limits.extents {
            return Err(SparseMetadataError::Limit("disk extents"));
        }
        let mut reserved_memory = (count as u64)
            .checked_mul(std::mem::size_of::<SparseMetadata>() as u64)
            .ok_or(SparseMetadataError::Limit("disk memory arithmetic"))?;
        if reserved_memory > limits.memory_bytes {
            return Err(SparseMetadataError::Limit("disk memory bytes"));
        }
        let mut metadata = Vec::new();
        metadata
            .try_reserve_exact(count)
            .map_err(|_| SparseMetadataError::Allocation)?;
        let mut read_bytes = 0;
        for index in 0..count {
            let mut admission = limits.metadata;
            admission.memory_bytes = admission
                .memory_bytes
                .min(limits.memory_bytes - reserved_memory);
            admission.read_bytes = admission.read_bytes.min(limits.read_bytes - read_bytes);
            let m = SparseMetadata::load(descriptor, index, resolver, admission)?;
            // The loader admitted both amounts against the remaining aggregate.
            reserved_memory += m.reserved_memory_bytes();
            read_bytes += m.metadata_read_bytes();
            metadata.push(m);
        }
        let disk = Self {
            metadata,
            geometry: DiskGeometry::new(descriptor.size_bytes(), 512, 512)
                .expect("fixed sector geometry"),
            cid: descriptor.cid(),
            reserved_memory,
            read_bytes,
            output_extents: limits.output_extents,
        };
        // Loading later extents may have changed an earlier source observation.
        disk.revalidate()?;
        Ok(disk)
    }
    pub fn metadata(&self) -> &[SparseMetadata] {
        &self.metadata
    }
    pub fn cid(&self) -> u32 {
        self.cid
    }
    pub fn reserved_memory_bytes(&self) -> u64 {
        self.reserved_memory
    }
    pub fn metadata_read_bytes(&self) -> u64 {
        self.read_bytes
    }
    pub fn revalidate(&self) -> std::result::Result<(), SparseMetadataError> {
        for m in &self.metadata {
            m.revalidate()?;
        }
        Ok(())
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
    /// Walk clipped runs without allocating. Data runs require physical adjacency;
    /// zero runs require only unallocated base grains. Never cross a backing here.
    fn runs(
        &self,
        range: DiskRange,
        mut visit: impl FnMut(u64, u64, Option<u64>, &SparseMetadata) -> Result<()>,
    ) -> Result<()> {
        if range.is_empty() {
            return Ok(());
        }
        let first = self.metadata.partition_point(|m| {
            m.logical_offset() + m.header().capacity_bytes() <= range.offset()
        });
        for m in &self.metadata[first..] {
            if m.logical_offset() >= range.end() {
                break;
            }
            let grain = m.header().grain_bytes();
            let sectors = grain / 512;
            let end = range
                .end()
                .min(m.logical_offset() + m.header().capacity_bytes())
                - m.logical_offset();
            let mut pos = range.offset().saturating_sub(m.logical_offset());
            while pos < end {
                let index = (pos / grain) as usize;
                let entry = m.grain_sectors()[index];
                let physical = (entry != 0).then(|| u64::from(entry) * 512 + pos % grain);
                let mut next = index + 1;
                let mut stop = ((next as u64) * grain).min(end);
                while stop < end {
                    let value = m.grain_sectors()[next];
                    let adjacent = if entry == 0 {
                        value == 0
                    } else {
                        value != 0
                            && u64::from(value) == u64::from(m.grain_sectors()[next - 1]) + sectors
                    };
                    if !adjacent {
                        break;
                    }
                    next += 1;
                    stop = ((next as u64) * grain).min(end);
                }
                visit(m.logical_offset() + pos, stop - pos, physical, m)?;
                pos = stop;
            }
        }
        Ok(())
    }
}
impl VirtualDisk for SparseDisk {
    fn geometry(&self) -> DiskGeometry {
        self.geometry
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
        let identity = destination.identity.ok_or(Error::InvalidEndpoint {
            reason: "sparse destination identity is unknown",
        })?;
        for m in &self.metadata {
            let source = m
                .initial_endpoint()
                .identity
                .ok_or(Error::InvalidEndpoint {
                    reason: "sparse backing identity is unknown",
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
        let range = self.range(offset, length)?;
        self.runs(range, |logical, size, physical, m| {
            let start = (logical - offset) as usize;
            let target = &mut buffer[start..start + size as usize];
            if let Some(physical) = physical {
                m.read_physical(physical, target)?;
            } else {
                target.fill(0);
            }
            Ok(())
        })?;
        Ok(buffer.len())
    }
    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
        let range = self.range(offset, length)?;
        // Count coalesced logical kinds first, then allocate exactly that payload.
        // Both passes use immutable metadata and perform no backing I/O.
        let mut count = 0usize;
        let mut previous = None;
        self.runs(range, |_, _, physical, _| {
            let data = physical.is_some();
            if previous != Some(data) {
                if count == self.output_extents {
                    return Err(Error::InvalidEndpoint {
                        reason: "sparse output extent limit",
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
        self.runs(range, |start, length, physical, _| {
            let kind = if physical.is_some() {
                ExtentKind::Data
            } else {
                ExtentKind::Zero
            };
            if let Some(last) = out.last_mut().filter(|last| last.kind() == kind) {
                *last = Extent::new(last.offset(), start + length - last.offset(), kind)?;
            } else {
                out.push(Extent::new(start, length, kind)?);
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
