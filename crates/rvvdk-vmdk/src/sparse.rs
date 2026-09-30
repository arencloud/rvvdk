//! Allocation-free validation of the first sector of a hosted sparse extent.
//! Header admission does not validate directory/table contents or logical data.
use crate::SECTOR_BYTES;
use std::io::Read;
use thiserror::Error;

pub const SPARSE_HEADER_BYTES: usize = 512;
const MAGIC: u32 = 0x564d444b;
const NEWLINE: u32 = 1;
const REDUNDANT: u32 = 2;
const GT_ENTRIES: u64 = 512;
const GT_BYTES: u64 = GT_ENTRIES * 4;

/// Independent admission limits, not an allocation or process-memory promise.
#[derive(Clone, Copy, Debug)]
pub struct SparseLimits {
    pub capacity_bytes: u64,
    pub grain_bytes: u64,
    pub descriptor_bytes: u64,
    pub directory_entries: u64,
    /// Bounds both advertised overhead and computed full metadata storage.
    pub metadata_bytes: u64,
}
impl Default for SparseLimits {
    fn default() -> Self {
        Self {
            capacity_bytes: 1 << 40,
            grain_bytes: 16 << 20,
            descriptor_bytes: 1 << 20,
            directory_entries: 1 << 20,
            metadata_bytes: 256 << 20,
        }
    }
}
#[derive(Debug, Error)]
pub enum SparseError {
    #[error("sparse header must contain exactly 512 bytes")]
    HeaderLength,
    #[error("invalid hosted sparse magic")]
    Magic,
    #[error("unsupported sparse feature: {0}")]
    Unsupported(&'static str),
    #[error("invalid sparse header: {0}")]
    Invalid(&'static str),
    #[error("sparse arithmetic overflow: {0}")]
    Overflow(&'static str),
    #[error("sparse resource limit exceeded: {0}")]
    Limit(&'static str),
    #[error("sparse metadata outside extent or advertised overhead: {0}")]
    Bounds(&'static str),
    #[error("reading sparse header: {0}")]
    Io(#[from] std::io::Error),
}
type Result<T> = std::result::Result<T, SparseError>;

/// Checked, nonempty, sector-aligned metadata storage range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SparseRegion {
    offset: u64,
    length: u64,
}
impl SparseRegion {
    pub fn offset(&self) -> u64 {
        self.offset
    }
    pub fn length(&self) -> u64 {
        self.length
    }
    pub fn end(&self) -> u64 {
        self.offset + self.length
    }
}
/// Admitted version-1 metadata geometry; not a validated grain map or VirtualDisk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SparseHeader {
    flags: u32,
    capacity_bytes: u64,
    grain_bytes: u64,
    directory_entries: u64,
    descriptor: Option<SparseRegion>,
    primary_directory: SparseRegion,
    redundant_directory: Option<SparseRegion>,
    overhead_bytes: u64,
    minimum_metadata_bytes: u64,
}
fn mul(a: u64, b: u64, field: &'static str) -> Result<u64> {
    a.checked_mul(b).ok_or(SparseError::Overflow(field))
}
fn add(a: u64, b: u64, field: &'static str) -> Result<u64> {
    a.checked_add(b).ok_or(SparseError::Overflow(field))
}
fn bound(value: u64, limit: u64, field: &'static str) -> Result<()> {
    if value > limit {
        Err(SparseError::Limit(field))
    } else {
        Ok(())
    }
}
fn region(sector: u64, length: u64, overhead: u64, field: &'static str) -> Result<SparseRegion> {
    let offset = mul(sector, SECTOR_BYTES, field)?;
    let end = add(offset, length, field)?;
    if offset < SECTOR_BYTES || length == 0 || end > overhead {
        return Err(SparseError::Bounds(field));
    }
    Ok(SparseRegion { offset, length })
}
fn overlaps(a: SparseRegion, b: SparseRegion) -> bool {
    a.offset < b.end() && b.offset < a.end()
}
impl SparseHeader {
    /// Exactly one 512-byte sector. extent_bytes is an observed container length,
    /// not virtual capacity; the caller must keep/revalidate the owned source.
    pub fn parse(bytes: &[u8], extent_bytes: u64) -> Result<Self> {
        Self::parse_with_limits(bytes, extent_bytes, SparseLimits::default())
    }
    pub fn parse_with_limits(
        bytes: &[u8],
        extent_bytes: u64,
        limits: SparseLimits,
    ) -> Result<Self> {
        if bytes.len() != SPARSE_HEADER_BYTES {
            return Err(SparseError::HeaderLength);
        }
        // Fixed offsets after the exact-size check; no packed casts or alignment assumptions.
        let u32_at = |offset| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let u64_at = |offset| u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap());
        if u32_at(0) != MAGIC {
            return Err(SparseError::Magic);
        }
        if u32_at(4) != 1 {
            return Err(SparseError::Unsupported("version (only 1)"));
        }
        let flags = u32_at(8);
        if flags & !(NEWLINE | REDUNDANT) != 0 {
            return Err(SparseError::Unsupported("flags"));
        }
        if bytes[77..79] != [0, 0] {
            return Err(SparseError::Unsupported("compression"));
        }
        if bytes[72] != 0 {
            return Err(SparseError::Unsupported("unclean shutdown"));
        }
        if flags & NEWLINE != 0 && bytes[73..77] != *b"\n \r\n" {
            return Err(SparseError::Invalid("newline check"));
        }
        if bytes[79..].iter().any(|&b| b != 0) {
            return Err(SparseError::Unsupported("reserved header bytes"));
        }
        let capacity = u64_at(12);
        let grain = u64_at(20);
        if grain <= 8 || !grain.is_power_of_two() {
            return Err(SparseError::Invalid("grain size"));
        }
        if capacity == 0 || !capacity.is_multiple_of(grain) {
            return Err(SparseError::Invalid(
                "capacity must be nonzero and grain-aligned",
            ));
        }
        if u32_at(44) != GT_ENTRIES as u32 {
            return Err(SparseError::Unsupported("grain table entry count"));
        }
        let capacity_bytes = mul(capacity, SECTOR_BYTES, "capacity")?;
        let grain_bytes = mul(grain, SECTOR_BYTES, "grain size")?;
        bound(capacity_bytes, limits.capacity_bytes, "capacity bytes")?;
        bound(grain_bytes, limits.grain_bytes, "grain bytes")?;
        let directory_entries = (capacity / grain).div_ceil(GT_ENTRIES);
        bound(
            directory_entries,
            limits.directory_entries,
            "directory entries",
        )?;
        let directory_bytes = mul(
            mul(directory_entries, 4, "directory")?.div_ceil(SECTOR_BYTES),
            SECTOR_BYTES,
            "directory",
        )?;
        let overhead = u64_at(64);
        if overhead == 0 || !overhead.is_multiple_of(grain) {
            return Err(SparseError::Invalid(
                "overhead must be nonzero and grain-aligned",
            ));
        }
        let overhead_bytes = mul(overhead, SECTOR_BYTES, "overhead")?;
        bound(overhead_bytes, limits.metadata_bytes, "metadata bytes")?;
        if overhead_bytes > extent_bytes {
            return Err(SparseError::Bounds("overhead"));
        }
        let descriptor_offset = u64_at(28);
        let descriptor_sectors = u64_at(36);
        let descriptor = match (descriptor_offset, descriptor_sectors) {
            (0, 0) => None,
            (0, _) | (_, 0) => return Err(SparseError::Invalid("descriptor offset/size pair")),
            (offset, sectors) => {
                let length = mul(sectors, SECTOR_BYTES, "descriptor")?;
                bound(length, limits.descriptor_bytes, "descriptor bytes")?;
                Some(region(offset, length, overhead_bytes, "descriptor")?)
            }
        };
        let gd = u64_at(56);
        if gd == u64::MAX {
            return Err(SparseError::Unsupported("footer directory"));
        }
        let primary_directory = region(gd, directory_bytes, overhead_bytes, "primary directory")?;
        let rgd = u64_at(48);
        let redundant_directory = if flags & REDUNDANT != 0 {
            Some(region(
                rgd,
                directory_bytes,
                overhead_bytes,
                "redundant directory",
            )?)
        } else {
            if rgd != 0 {
                return Err(SparseError::Invalid("redundant offset without flag"));
            }
            None
        };
        if descriptor.is_some_and(|d| {
            overlaps(d, primary_directory) || redundant_directory.is_some_and(|r| overlaps(d, r))
        }) || redundant_directory.is_some_and(|r| overlaps(r, primary_directory))
        {
            return Err(SparseError::Invalid("overlapping header metadata regions"));
        }
        let one_copy = add(
            directory_bytes,
            mul(directory_entries, GT_BYTES, "grain tables")?,
            "directory and tables",
        )?;
        let minimum_metadata_bytes = add(
            add(SECTOR_BYTES, descriptor.map_or(0, |d| d.length), "metadata")?,
            mul(
                one_copy,
                if redundant_directory.is_some() { 2 } else { 1 },
                "metadata copies",
            )?,
            "metadata",
        )?;
        bound(
            minimum_metadata_bytes,
            limits.metadata_bytes,
            "metadata bytes",
        )?;
        if minimum_metadata_bytes > overhead_bytes {
            return Err(SparseError::Bounds("minimum metadata storage"));
        }
        Ok(Self {
            flags,
            capacity_bytes,
            grain_bytes,
            directory_entries,
            descriptor,
            primary_directory,
            redundant_directory,
            overhead_bytes,
            minimum_metadata_bytes,
        })
    }
    /// Read exactly one sector from the reader's current position. No seek,
    /// following offsets, allocation, size-hint trust, or extra probe. Interrupts retry.
    pub fn read_from(
        mut reader: impl Read,
        extent_bytes: u64,
        limits: SparseLimits,
    ) -> Result<Self> {
        let mut bytes = [0; SPARSE_HEADER_BYTES];
        reader.read_exact(&mut bytes)?;
        Self::parse_with_limits(&bytes, extent_bytes, limits)
    }
    pub fn flags(&self) -> u32 {
        self.flags
    }
    pub fn capacity_bytes(&self) -> u64 {
        self.capacity_bytes
    }
    pub fn grain_bytes(&self) -> u64 {
        self.grain_bytes
    }
    pub fn grain_count(&self) -> u64 {
        self.capacity_bytes / self.grain_bytes
    }
    pub fn directory_entries(&self) -> u64 {
        self.directory_entries
    }
    pub fn grain_table_bytes(&self) -> u64 {
        GT_BYTES
    }
    pub fn descriptor(&self) -> Option<SparseRegion> {
        self.descriptor
    }
    pub fn primary_directory(&self) -> SparseRegion {
        self.primary_directory
    }
    pub fn redundant_directory(&self) -> Option<SparseRegion> {
        self.redundant_directory
    }
    pub fn overhead_bytes(&self) -> u64 {
        self.overhead_bytes
    }
    pub fn minimum_metadata_bytes(&self) -> u64 {
        self.minimum_metadata_bytes
    }
}
