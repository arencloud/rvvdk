//! Bounded streamOptimized envelope and marker admission, not logical disk reads.
//! Grain-table pointers, record ordering and compressed payloads remain unvalidated.
use crate::{DescriptorError, Limits, StreamDescriptor};
use std::io::{Read, Seek, SeekFrom};
use thiserror::Error;

const SECTOR: u64 = 512;
const GRAIN: u64 = 65536;
const FLAGS: u32 = (1 << 16) | (1 << 17);

#[derive(Clone, Copy, Debug)]
pub struct StreamLimits {
    pub capacity_bytes: u64,
    pub extent_bytes: u64,
    pub overhead_bytes: u64,
    pub directory_entries: u64,
    pub compressed_grain_bytes: u64,
    /// Actual bytes requested by envelope acquisition, including header recheck.
    pub read_bytes: u64,
    pub descriptor: Limits,
}
impl Default for StreamLimits {
    fn default() -> Self {
        Self {
            capacity_bytes: 1 << 40,
            extent_bytes: 1 << 41,
            overhead_bytes: 128 << 20,
            directory_entries: 32768,
            compressed_grain_bytes: 128 << 10,
            read_bytes: 2 << 20,
            descriptor: Limits::default(),
        }
    }
}

#[derive(Debug, Error)]
pub enum StreamError {
    #[error("unsupported streamOptimized feature: {0}")]
    Unsupported(&'static str),
    #[error("invalid streamOptimized metadata: {0}")]
    Invalid(&'static str),
    #[error("streamOptimized resource limit: {0}")]
    Limit(&'static str),
    #[error("streamOptimized arithmetic overflow")]
    Overflow,
    #[error("streamOptimized allocation failed")]
    Allocation,
    #[error("{0}")]
    Descriptor(#[from] DescriptorError),
    #[error("streamOptimized I/O: {0}")]
    Io(#[from] std::io::Error),
}
type Result<T> = std::result::Result<T, StreamError>;
fn invalid(field: &'static str) -> StreamError {
    StreamError::Invalid(field)
}
fn add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b).ok_or(StreamError::Overflow)
}
fn mul(a: u64, b: u64) -> Result<u64> {
    a.checked_mul(b).ok_or(StreamError::Overflow)
}
fn limit(value: u64, max: u64, name: &'static str) -> Result<()> {
    if value > max {
        Err(StreamError::Limit(name))
    } else {
        Ok(())
    }
}
fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}
fn u64_at(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().unwrap())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamRegion {
    pub offset: u64,
    pub length: u64,
}
impl StreamRegion {
    fn end(self) -> Result<u64> {
        add(self.offset, self.length)
    }
    fn overlaps(self, other: Self) -> Result<bool> {
        Ok(self.offset < other.end()? && other.offset < self.end()?)
    }
}
fn region(sector: u64, length: u64, end: u64) -> Result<StreamRegion> {
    let r = StreamRegion {
        offset: mul(sector, SECTOR)?,
        length,
    };
    if r.offset < SECTOR || length == 0 || r.end()? > end {
        return Err(invalid("region bounds"));
    }
    Ok(r)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamDirectory {
    /// QEMU-compatible front directories; their entries/tables are not read here.
    Front {
        primary: StreamRegion,
        redundant: Option<StreamRegion>,
    },
    Footer,
}

/// Admitted 64 KiB-grain, version-3 header geometry. No map or payload validity claim.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamHeader {
    flags: u32,
    capacity: u64,
    extent: u64,
    descriptor: StreamRegion,
    overhead: u64,
    directory_bytes: u64,
    directory: StreamDirectory,
}
impl StreamHeader {
    pub fn parse(b: &[u8], extent_bytes: u64, limits: StreamLimits) -> Result<Self> {
        Self::parse_inner(b, extent_bytes, limits, false)
    }
    fn parse_inner(b: &[u8], extent: u64, limits: StreamLimits, footer: bool) -> Result<Self> {
        if b.len() != 512 {
            return Err(invalid("header length"));
        }
        if u32_at(b, 0) != 0x564d444b {
            return Err(invalid("magic"));
        }
        if u32_at(b, 4) != 3 {
            return Err(StreamError::Unsupported("version (only 3)"));
        }
        let flags = u32_at(b, 8);
        if flags & FLAGS != FLAGS || flags & !(FLAGS | 3) != 0 {
            return Err(StreamError::Unsupported("flags"));
        }
        if u64_at(b, 20) != 128 || u32_at(b, 44) != 512 {
            return Err(StreamError::Unsupported("grain/table geometry"));
        }
        if b[77..79] != [1, 0] {
            return Err(StreamError::Unsupported("compression algorithm"));
        }
        if b[72] != 0 {
            return Err(StreamError::Unsupported("unclean shutdown"));
        }
        if flags & 1 != 0 && b[73..77] != *b"\n \r\n" {
            return Err(invalid("newline check"));
        }
        if b[79..].iter().any(|v| *v != 0) {
            return Err(StreamError::Unsupported("reserved header bytes"));
        }
        if extent < 1024 || !extent.is_multiple_of(SECTOR) {
            return Err(invalid("extent length"));
        }
        limit(extent, limits.extent_bytes, "extent bytes")?;
        let capacity = mul(u64_at(b, 12), SECTOR)?;
        if capacity == 0 || !capacity.is_multiple_of(GRAIN) {
            return Err(invalid("capacity"));
        }
        limit(capacity, limits.capacity_bytes, "capacity bytes")?;
        let entries = (capacity / GRAIN).div_ceil(512);
        limit(entries, limits.directory_entries, "directory entries")?;
        let directory_bytes = mul(mul(entries, 4)?.div_ceil(SECTOR), SECTOR)?;
        let overhead = mul(u64_at(b, 64), SECTOR)?;
        if overhead == 0 || !overhead.is_multiple_of(GRAIN) || overhead > extent {
            return Err(invalid("overhead"));
        }
        limit(overhead, limits.overhead_bytes, "overhead bytes")?;
        let desc_bytes = mul(u64_at(b, 36), SECTOR)?;
        limit(
            desc_bytes,
            limits.descriptor.descriptor_bytes as u64,
            "descriptor bytes",
        )?;
        let descriptor = region(u64_at(b, 28), desc_bytes, overhead)?;
        let gd = u64_at(b, 56);
        let directory = if gd == u64::MAX {
            if footer || flags & 2 != 0 || add(overhead, 1536)? > extent {
                return Err(invalid("footer directory profile"));
            }
            StreamDirectory::Footer
        } else {
            let bound = if footer {
                extent
                    .checked_sub(1536)
                    .ok_or_else(|| invalid("footer length"))?
            } else {
                overhead
            };
            let primary = region(gd, directory_bytes, bound)?;
            if descriptor.overlaps(primary)? || (footer && primary.offset < add(overhead, SECTOR)?)
            {
                return Err(invalid("directory overlap/location"));
            }
            let redundant = if flags & 2 != 0 {
                if footer {
                    return Err(invalid("redundant footer"));
                }
                let r = region(u64_at(b, 48), directory_bytes, overhead)?;
                if r.overlaps(primary)? || r.overlaps(descriptor)? {
                    return Err(invalid("redundant overlap"));
                }
                Some(r)
            } else {
                None
            }; // Specification says to ignore rgdOffset without the flag.
            if !footer {
                let minimum = add(
                    add(SECTOR, desc_bytes)?,
                    mul(
                        add(directory_bytes, mul(entries, 2048)?)?,
                        if redundant.is_some() { 2 } else { 1 },
                    )?,
                )?;
                if minimum > overhead {
                    return Err(invalid("front metadata footprint"));
                }
            }
            StreamDirectory::Front { primary, redundant }
        };
        Ok(Self {
            flags,
            capacity,
            extent,
            descriptor,
            overhead,
            directory_bytes,
            directory,
        })
    }
    pub fn capacity_bytes(&self) -> u64 {
        self.capacity
    }
    pub fn extent_bytes(&self) -> u64 {
        self.extent
    }
    pub fn grain_bytes(&self) -> u64 {
        GRAIN
    }
    pub fn flags(&self) -> u32 {
        self.flags
    }
    pub fn descriptor(&self) -> StreamRegion {
        self.descriptor
    }
    pub fn overhead_bytes(&self) -> u64 {
        self.overhead
    }
    pub fn directory(&self) -> StreamDirectory {
        self.directory
    }
    pub fn directory_bytes(&self) -> u64 {
        self.directory_bytes
    }
}

/// A locally bounded marker, not proof that it is reachable in a valid record sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamMarker {
    Grain {
        logical_offset: u64,
        payload: StreamRegion,
        next_offset: u64,
    },
    GrainTable(StreamRegion),
    GrainDirectory(StreamRegion),
    Footer(StreamRegion),
    End,
}
impl StreamMarker {
    /// Exactly 16 prefix bytes; metadata-marker padding is ignored as specified.
    /// No compressed bytes are read or decompressed, and no grain pointers followed.
    pub fn parse(
        b: &[u8],
        offset: u64,
        header: &StreamHeader,
        limits: StreamLimits,
    ) -> Result<Self> {
        if b.len() != 16 || !offset.is_multiple_of(SECTOR) || offset < header.overhead {
            return Err(invalid("marker position/prefix"));
        }
        let value = u64_at(b, 0);
        let size = u32_at(b, 8) as u64;
        if size != 0 {
            limit(
                size,
                limits.compressed_grain_bytes,
                "compressed grain bytes",
            )?;
            let logical_offset = mul(value, SECTOR)?;
            if !logical_offset.is_multiple_of(GRAIN)
                || add(logical_offset, GRAIN)? > header.capacity
            {
                return Err(invalid("grain logical range"));
            }
            let payload = StreamRegion {
                offset: add(offset, 12)?,
                length: size,
            };
            let next_offset = mul(payload.end()?.div_ceil(SECTOR), SECTOR)?;
            let end = if header.directory == StreamDirectory::Footer {
                header.extent - 1536
            } else {
                header.extent
            };
            if next_offset > end {
                return Err(invalid("grain physical range"));
            }
            return Ok(Self::Grain {
                logical_offset,
                payload,
                next_offset,
            });
        }
        let kind = u32_at(b, 12);
        let expected = match kind {
            0 => 0,
            1 => 4,
            2 => header.directory_bytes / SECTOR,
            3 => 1,
            _ => return Err(StreamError::Unsupported("marker type")),
        };
        if value != expected {
            return Err(invalid("marker sector count"));
        }
        let r = StreamRegion {
            offset: add(offset, SECTOR)?,
            length: mul(value, SECTOR)?,
        };
        if r.end()? > header.extent {
            return Err(invalid("marker physical range"));
        }
        if matches!(kind, 1 | 2)
            && header.directory == StreamDirectory::Footer
            && r.end()? > header.extent - 1536
        {
            return Err(invalid("metadata overlaps terminal records"));
        }
        match kind {
            0 if r.offset == header.extent => Ok(Self::End),
            1 => Ok(Self::GrainTable(r)),
            2 => Ok(Self::GrainDirectory(r)),
            3 if add(r.end()?, SECTOR)? == header.extent => Ok(Self::Footer(r)),
            _ => Err(invalid("terminal marker position")),
        }
    }
}

/// Header/descriptor/tail envelope only. It is deliberately not a BlockDevice or map.
#[derive(Clone, Debug)]
pub struct StreamEnvelope {
    pub header: StreamHeader,
    pub primary_directory: StreamRegion,
    pub cid: u32,
    pub metadata_bytes_read: u64,
}
impl StreamEnvelope {
    /// Caller must keep the source quiescent. Length/header rechecks are not a snapshot.
    /// Reads no directory entries, grain tables, compressed records or payloads.
    pub fn read_from(
        mut source: impl Read + Seek,
        extent: u64,
        limits: StreamLimits,
    ) -> Result<Self> {
        if source.seek(SeekFrom::End(0))? != extent {
            return Err(invalid("observed length"));
        }
        let mut reads = 0;
        let mut read = |at, bytes: &mut [u8]| -> Result<()> {
            reads = add(reads, bytes.len() as u64)?;
            limit(reads, limits.read_bytes, "read bytes")?;
            if add(at, bytes.len() as u64)? > extent {
                return Err(invalid("read bounds"));
            }
            source.seek(SeekFrom::Start(at))?;
            source.read_exact(bytes)?;
            Ok(())
        };
        let mut first = [0; 512];
        read(0, &mut first)?;
        let header = StreamHeader::parse(&first, extent, limits)?;
        let total = add(
            add(1024, header.descriptor.length)?,
            if header.directory == StreamDirectory::Footer {
                2048
            } else {
                0
            },
        )?;
        limit(total, limits.read_bytes, "read bytes")?;
        let length =
            usize::try_from(header.descriptor.length).map_err(|_| StreamError::Allocation)?;
        let mut text = Vec::new();
        text.try_reserve_exact(length)
            .map_err(|_| StreamError::Allocation)?;
        text.resize(length, 0);
        read(header.descriptor.offset, &mut text)?;
        let descriptor = StreamDescriptor::parse_with_limits(&text, limits.descriptor)?;
        if descriptor.size_bytes() != header.capacity {
            return Err(invalid("descriptor capacity"));
        }
        let cid = descriptor.cid();
        let primary_directory = match header.directory {
            StreamDirectory::Front { primary, .. } => primary,
            StreamDirectory::Footer => {
                let mut tail = [0; 1536];
                read(extent - 1536, &mut tail)?;
                if !matches!(
                    StreamMarker::parse(&tail[..16], extent - 1536, &header, limits)?,
                    StreamMarker::Footer(_)
                ) || StreamMarker::parse(&tail[1024..1040], extent - 512, &header, limits)?
                    != StreamMarker::End
                {
                    return Err(invalid("footer/end markers"));
                }
                let footer = StreamHeader::parse_inner(&tail[512..1024], extent, limits, true)?;
                if first[..48] != tail[512..560] || first[64..79] != tail[576..591] {
                    return Err(invalid("header/footer disagreement"));
                }
                let StreamDirectory::Front { primary, .. } = footer.directory else {
                    unreachable!()
                };
                if primary.end()? != extent - 1536 {
                    return Err(invalid("directory/footer adjacency"));
                }
                let mut marker = [0; 512];
                read(primary.offset - SECTOR, &mut marker)?;
                if StreamMarker::parse(&marker[..16], primary.offset - SECTOR, &header, limits)?
                    != StreamMarker::GrainDirectory(primary)
                {
                    return Err(invalid("directory marker"));
                }
                primary
            }
        };
        let mut last = [0; 512];
        read(0, &mut last)?;
        if last != first {
            return Err(invalid("header changed"));
        }
        if source.seek(SeekFrom::End(0))? != extent {
            return Err(invalid("length changed"));
        }
        Ok(Self {
            header,
            primary_directory,
            cid,
            metadata_bytes_read: reads,
        })
    }
}
