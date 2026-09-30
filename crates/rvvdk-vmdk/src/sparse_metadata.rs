//! Eager, bounded metadata validation. No logical disk reads or writes.
use crate::{
    BackingError, BackingResolver, CreateType, DescriptorError, ExtentBacking, Limits,
    SECTOR_BYTES, SparseDescriptor, SparseError, SparseHeader, SparseLimits,
};
use rvvdk_core::{BlockDevice, Capabilities, CopyEndpoint};
use std::sync::Arc;
use thiserror::Error;

#[derive(Clone, Copy, Debug)]
pub struct SparseMetadataLimits {
    pub header: SparseLimits,
    pub descriptor: Limits,
    /// Loader-owned buffers/map/sort scratch; parser/resolver/allocator overhead excluded.
    pub memory_bytes: u64,
    /// Total physical metadata bytes requested, including the final header recheck.
    pub read_bytes: u64,
}
impl Default for SparseMetadataLimits {
    fn default() -> Self {
        Self {
            header: SparseLimits::default(),
            descriptor: Limits::default(),
            memory_bytes: 128 << 20,
            read_bytes: 256 << 20,
        }
    }
}
#[derive(Debug, Error)]
pub enum SparseMetadataError {
    #[error("{0}")]
    Header(#[from] SparseError),
    #[error("{0}")]
    Descriptor(#[from] DescriptorError),
    #[error("{0}")]
    Resolve(#[from] BackingError),
    #[error("sparse metadata I/O: {0}")]
    Device(#[from] rvvdk_core::Error),
    #[error("invalid sparse metadata: {0}")]
    Invalid(&'static str),
    #[error("sparse metadata resource limit: {0}")]
    Limit(&'static str),
    #[error("sparse metadata allocation failed")]
    Allocation,
}
type Result<T> = std::result::Result<T, SparseMetadataError>;
fn invalid(reason: &'static str) -> SparseMetadataError {
    SparseMetadataError::Invalid(reason)
}
fn add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b)
        .ok_or(SparseMetadataError::Limit("arithmetic"))
}
fn mul(a: u64, b: u64) -> Result<u64> {
    a.checked_mul(b)
        .ok_or(SparseMetadataError::Limit("arithmetic"))
}
fn allocated<T: Default + Clone>(n: u64) -> Result<Vec<T>> {
    let n = usize::try_from(n).map_err(|_| SparseMetadataError::Limit("address space"))?;
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| SparseMetadataError::Allocation)?;
    v.resize(n, T::default());
    Ok(v)
}
fn check_endpoint(initial: CopyEndpoint, live: CopyEndpoint) -> Result<()> {
    if !live.capabilities.contains(Capabilities::READ)
        || live.size != initial.size
        || initial.identity.is_some() && live.identity != initial.identity
    {
        return Err(invalid("source endpoint changed or unreadable"));
    }
    Ok(())
}
fn word(bytes: &[u8], index: usize) -> u32 {
    u32::from_le_bytes(bytes[index * 4..index * 4 + 4].try_into().unwrap())
}
/// Retained immutable version-1 map. Zero entries mean unallocated, not a public
/// logical-zero guarantee. Source quiescence remains a caller requirement.
pub struct SparseMetadata {
    source: Arc<dyn BlockDevice>,
    initial: CopyEndpoint,
    header: SparseHeader,
    grains: Vec<u32>,
    cid: u32,
    extent_index: usize,
    logical_offset: u64,
    reserved_memory: u64,
    read_bytes: u64,
}
impl SparseMetadata {
    /// Resolve exactly the selected descriptor reference. No embedded reference
    /// is followed; embedded text must agree with the supplied base descriptor.
    pub fn load(
        descriptor: &SparseDescriptor<'_>,
        extent_index: usize,
        resolver: &dyn BackingResolver,
        limits: SparseMetadataLimits,
    ) -> Result<Self> {
        let extent = descriptor
            .extents()
            .get(extent_index)
            .ok_or(invalid("extent index"))?;
        let ExtentBacking::Sparse { file_name } = extent.backing() else {
            return Err(invalid("extent type"));
        };
        if limits.read_bytes < 512 {
            return Err(SparseMetadataError::Limit("read bytes"));
        }
        let source = resolver.resolve(file_name)?;
        let initial = source.copy_endpoint()?;
        check_endpoint(initial, initial)?;
        let mut raw = [0; 512];
        source.read_exact_at(0, &mut raw)?;
        let header = SparseHeader::parse_with_limits(&raw, initial.size, limits.header)?;
        if header.capacity_bytes() != extent.size_bytes() {
            return Err(invalid("descriptor/header capacity mismatch"));
        }
        let copies = if header.redundant_directory().is_some() {
            2
        } else {
            1
        };
        let tables = mul(header.directory_entries(), copies)?;
        let dirs = mul(header.primary_directory().length(), copies)?;
        let desc = header.descriptor().map_or(0, |r| r.length());
        if desc > limits.descriptor.descriptor_bytes as u64 {
            return Err(SparseMetadataError::Limit("descriptor bytes"));
        }
        // Simultaneous buffers: descriptor, both GDs, map + sort scratch, regions.
        // Fixed stack storage: header/recheck (1 KiB) + two GTs (4 KiB).
        let ranges = add(tables, 4)?;
        let reserved_memory = add(
            add(add(desc, dirs)?, mul(header.grain_count(), 8)?)?,
            mul(ranges, 16)?,
        )?;
        let read_bytes = add(add(add(1024, desc)?, dirs)?, mul(tables, 2048)?)?;
        if reserved_memory > limits.memory_bytes {
            return Err(SparseMetadataError::Limit("memory bytes"));
        }
        if read_bytes > limits.read_bytes {
            return Err(SparseMetadataError::Limit("read bytes"));
        }
        // Admission precedes every metadata-sized allocation and every offset-following read.
        let mut text = allocated::<u8>(desc)?;
        if let Some(r) = header.descriptor() {
            source.read_exact_at(r.offset(), &mut text)?;
        }
        if text.iter().all(|&b| b == 0) {
            if descriptor.create_type() == CreateType::MonolithicSparse {
                return Err(invalid("monolithic embedded descriptor missing"));
            }
        } else {
            let embedded = SparseDescriptor::parse_with_limits(&text, limits.descriptor)?;
            if !descriptor.same_mapping(&embedded) {
                return Err(invalid("embedded/external descriptor mismatch"));
            }
        }
        let mut primary = allocated::<u8>(header.primary_directory().length())?;
        source.read_exact_at(header.primary_directory().offset(), &mut primary)?;
        let mut redundant =
            allocated::<u8>(header.redundant_directory().map_or(0, |r| r.length()))?;
        if let Some(r) = header.redundant_directory() {
            source.read_exact_at(r.offset(), &mut redundant)?;
        }
        let count = usize::try_from(header.directory_entries())
            .map_err(|_| SparseMetadataError::Limit("address space"))?;
        let mut regions = allocated::<(u64, u64)>(ranges)?;
        regions.clear();
        regions.push((0, 512));
        for r in [
            header.descriptor(),
            Some(header.primary_directory()),
            header.redundant_directory(),
        ]
        .into_iter()
        .flatten()
        {
            regions.push((r.offset(), r.end()));
        }
        for directory in [&primary, &redundant] {
            if directory.is_empty() {
                continue;
            }
            if directory[count * 4..].iter().any(|&b| b != 0) {
                return Err(invalid("nonzero directory padding"));
            }
            for i in 0..count {
                let offset = u64::from(word(directory, i)) * SECTOR_BYTES;
                let end = offset + 2048; // u32 sector pointer cannot overflow u64.
                if offset < 512 || end > header.overhead_bytes() {
                    return Err(invalid("grain table outside metadata"));
                }
                regions.push((offset, end));
            }
        }
        regions.sort_unstable();
        if regions.windows(2).any(|w| w[0].1 > w[1].0) {
            return Err(invalid("overlapping metadata regions"));
        }
        let mut grains = allocated::<u32>(header.grain_count())?;
        let mut a = [0; 2048];
        let mut b = [0; 2048];
        for i in 0..count {
            source.read_exact_at(u64::from(word(&primary, i)) * SECTOR_BYTES, &mut a)?;
            if !redundant.is_empty() {
                source.read_exact_at(u64::from(word(&redundant, i)) * SECTOR_BYTES, &mut b)?;
                if a != b {
                    return Err(invalid("redundant grain tables disagree"));
                }
            }
            for j in 0..512 {
                let value = word(&a, j);
                let index = i * 512 + j;
                if index >= grains.len() {
                    if value != 0 {
                        return Err(invalid("nonzero unused grain entry"));
                    }
                    continue;
                }
                if value != 0 {
                    let offset = u64::from(value) * SECTOR_BYTES;
                    let end = add(offset, header.grain_bytes())?;
                    if offset < header.overhead_bytes()
                        || !offset.is_multiple_of(header.grain_bytes())
                        || end > initial.size
                    {
                        return Err(invalid("invalid data grain range"));
                    }
                }
                grains[index] = value;
            }
        }
        let mut physical = allocated::<u32>(header.grain_count())?;
        physical.copy_from_slice(&grains);
        physical.sort_unstable();
        if physical.windows(2).any(|w| w[0] != 0 && w[0] == w[1]) {
            return Err(invalid("duplicate physical grain"));
        }
        let mut live_header = [0; 512];
        source.read_exact_at(0, &mut live_header)?;
        if raw != live_header {
            return Err(invalid("header changed during acquisition"));
        }
        check_endpoint(initial, source.copy_endpoint()?)?;
        Ok(Self {
            source,
            initial,
            header,
            grains,
            cid: descriptor.cid(),
            extent_index,
            logical_offset: extent.logical_offset(),
            reserved_memory,
            read_bytes,
        })
    }
    pub fn header(&self) -> &SparseHeader {
        &self.header
    }
    pub fn grain_sectors(&self) -> &[u32] {
        &self.grains
    }
    pub fn cid(&self) -> u32 {
        self.cid
    }
    pub fn extent_index(&self) -> usize {
        self.extent_index
    }
    pub fn logical_offset(&self) -> u64 {
        self.logical_offset
    }
    pub fn reserved_memory_bytes(&self) -> u64 {
        self.reserved_memory
    }
    pub fn metadata_read_bytes(&self) -> u64 {
        self.read_bytes
    }
    pub fn initial_endpoint(&self) -> CopyEndpoint {
        self.initial
    }
    /// Endpoint observations only, not metadata replay or a content snapshot.
    pub fn revalidate(&self) -> Result<()> {
        check_endpoint(self.initial, self.source.copy_endpoint()?)
    }
}
