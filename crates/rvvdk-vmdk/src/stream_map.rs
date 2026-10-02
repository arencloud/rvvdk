//! Bounded eager stream index. Compressed contents remain unvalidated.
use crate::{
    StreamDirectory, StreamEnvelope, StreamError, StreamHeader, StreamLimits, StreamMarker,
    StreamRegion,
};
use std::io::{Read, Seek, SeekFrom};

const SECTOR: u64 = 512;
const GRAIN: u64 = 65536;
const TABLE: u64 = 2048;
type Result<T> = std::result::Result<T, StreamError>;
fn invalid(s: &'static str) -> StreamError {
    StreamError::Invalid(s)
}
fn add(a: u64, b: u64) -> Result<u64> {
    a.checked_add(b).ok_or(StreamError::Overflow)
}
fn mul(a: u64, b: u64) -> Result<u64> {
    a.checked_mul(b).ok_or(StreamError::Overflow)
}
fn limit(n: u64, max: u64, s: &'static str) -> Result<()> {
    if n > max {
        Err(StreamError::Limit(s))
    } else {
        Ok(())
    }
}
fn word(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes(b[i * 4..i * 4 + 4].try_into().unwrap())
}
fn allocated<T: Clone>(n: u64, value: T) -> Result<Vec<T>> {
    let n = usize::try_from(n).map_err(|_| StreamError::Allocation)?;
    let mut out = Vec::new();
    out.try_reserve_exact(n)
        .map_err(|_| StreamError::Allocation)?;
    out.resize(n, value);
    Ok(out)
}

#[derive(Clone, Copy, Debug)]
pub struct StreamMapLimits {
    pub envelope: StreamLimits,
    /// Directory buffers, overlap scratch and retained index slots. Envelope
    /// parsing, fixed stack buffers and allocator overhead are bounded separately.
    pub memory_bytes: u64,
    /// All requested metadata bytes, including envelope and final header recheck.
    pub read_bytes: u64,
    /// Primary GTE slots examined across both passes, including unused slots.
    pub table_entries: u64,
    pub allocated_grains: u64,
}
impl Default for StreamMapLimits {
    fn default() -> Self {
        Self {
            envelope: StreamLimits::default(),
            memory_bytes: 128 << 20,
            read_bytes: 256 << 20,
            table_entries: 32 << 20,
            allocated_grains: 4 << 20,
        }
    }
}

/// A record location, not decoded bytes. Fields cannot be constructed by callers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamGrain {
    index: u32,
    sector: u32,
    compressed_bytes: u32,
}
impl StreamGrain {
    pub fn index(self) -> u64 {
        self.index as u64
    }
    pub fn logical_offset(self) -> u64 {
        self.index() * GRAIN
    }
    pub fn marker_offset(self) -> u64 {
        self.sector as u64 * SECTOR
    }
    pub fn payload(self) -> StreamRegion {
        StreamRegion {
            offset: self.marker_offset() + 12,
            length: self.compressed_bytes as u64,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StreamMapStats {
    pub metadata_bytes_read: u64,
    pub table_entries_examined: u64,
    pub allocated_grains: u64,
    /// Requested slot bytes at peak, with the exclusions stated by StreamMapLimits.
    pub reserved_bytes: u64,
}

/// Validated metadata topology for a quiescent source; no payload/BlockDevice claim.
pub struct StreamMap {
    header: StreamHeader,
    cid: u32,
    grains: Vec<StreamGrain>,
    stats: StreamMapStats,
}
impl StreamMap {
    pub fn header(&self) -> &StreamHeader {
        &self.header
    }
    pub fn cid(&self) -> u32 {
        self.cid
    }
    pub fn grains(&self) -> &[StreamGrain] {
        &self.grains
    }
    pub fn stats(&self) -> StreamMapStats {
        self.stats
    }
    /// None means an unallocated base grain. This API performs no logical reads.
    pub fn grain(&self, index: u64) -> Result<Option<StreamGrain>> {
        if index >= self.header.capacity_bytes() / GRAIN {
            return Err(invalid("grain index"));
        }
        Ok(self
            .grains
            .binary_search_by_key(&index, |g| g.index())
            .ok()
            .map(|i| self.grains[i]))
    }

    /// The caller must keep the source quiescent throughout acquisition and later
    /// use. Header/length rechecks are not a snapshot or a persistent identity bind.
    /// Two table passes count before allocating the exact sparse index size.
    pub fn read_from(
        mut source: impl Read + Seek,
        extent: u64,
        limits: StreamMapLimits,
    ) -> Result<Self> {
        let mut envelope_limits = limits.envelope;
        envelope_limits.read_bytes = envelope_limits.read_bytes.min(limits.read_bytes);
        let envelope = StreamEnvelope::read_from(&mut source, extent, envelope_limits)?;
        let header = &envelope.header;
        let grain_count = header.capacity_bytes() / GRAIN;
        limit(grain_count, u32::MAX as u64 + 1, "logical grains")?;
        let entries = grain_count.div_ceil(512);
        let redundant = match header.directory() {
            StreamDirectory::Front { redundant, .. } => redundant,
            StreamDirectory::Footer => None,
        };
        let footer = header.directory() == StreamDirectory::Footer;
        let copies = if redundant.is_some() { 2 } else { 1 };
        let directory_bytes = header.directory_bytes();
        let directory_memory = mul(directory_bytes, copies)?;
        limit(directory_memory, limits.memory_bytes, "map memory bytes")?;
        let mut primary = allocated(directory_bytes, 0_u8)?;
        let mut secondary = allocated(
            if redundant.is_some() {
                directory_bytes
            } else {
                0
            },
            0_u8,
        )?;
        let mut reader = Budget {
            source: &mut source,
            extent,
            used: envelope.metadata_bytes_read,
            max: limits.read_bytes,
        };
        let initial_read = add(reader.used, add(mul(directory_bytes, copies)?, SECTOR)?)?;
        limit(initial_read, limits.read_bytes, "map read bytes")?;
        reader.read(envelope.primary_directory.offset, &mut primary)?;
        if let Some(r) = redundant {
            reader.read(r.offset, &mut secondary)?;
        }
        if primary[entries as usize * 4..].iter().any(|b| *b != 0)
            || (!secondary.is_empty() && secondary[entries as usize * 4..].iter().any(|b| *b != 0))
        {
            return Err(invalid("directory unused slots"));
        }
        let tables = primary[..entries as usize * 4]
            .chunks_exact(4)
            .filter(|slot| *slot != [0, 0, 0, 0])
            .count() as u64;
        let region_slots = add(mul(tables, copies)?, 4)?;
        let scratch = add(
            directory_memory,
            mul(region_slots, std::mem::size_of::<Region>() as u64)?,
        )?;
        limit(scratch, limits.memory_bytes, "map memory bytes")?;
        let mut regions = allocated(region_slots, Region::default())?;
        let mut region_count = 0;
        // Every followed table region is checked and included in alias detection
        // before any table I/O. Footer regions include the preceding marker.
        let mut record_region = |start, length| -> Result<()> {
            let end = add(start, length)?;
            if length == 0 || end > extent {
                return Err(invalid("metadata region bounds"));
            }
            regions[region_count] = Region { start, end };
            region_count += 1;
            Ok(())
        };
        record_region(0, SECTOR)?;
        record_region(header.descriptor().offset, header.descriptor().length)?;
        record_region(envelope.primary_directory.offset, directory_bytes)?;
        if let Some(r) = redundant {
            record_region(r.offset, r.length)?;
        }
        let mut last_footer_end = header.overhead_bytes();
        for i in 0..entries as usize {
            let p = word(&primary, i);
            let r = if redundant.is_some() {
                word(&secondary, i)
            } else {
                0
            };
            if redundant.is_some() && (p == 0) != (r == 0) {
                return Err(invalid("table presence disagreement"));
            }
            for sector in [p, r] {
                if sector == 0 {
                    continue;
                }
                if sector == 1 {
                    return Err(invalid("reserved table pointer"));
                }
                let at = sector as u64 * SECTOR;
                let end = add(at, TABLE)?;
                if footer {
                    let marker = at.checked_sub(SECTOR).ok_or(StreamError::Overflow)?;
                    if marker < last_footer_end || end > envelope.primary_directory.offset - SECTOR
                    {
                        return Err(invalid("footer table location/order"));
                    }
                    record_region(marker, TABLE + SECTOR)?;
                    last_footer_end = end;
                } else {
                    if end > header.overhead_bytes() {
                        return Err(invalid("front table location"));
                    }
                    record_region(at, TABLE)?;
                }
            }
        }
        regions[..region_count].sort_unstable_by_key(|r| r.start);
        if regions[..region_count]
            .windows(2)
            .any(|w| w[0].end > w[1].start)
        {
            return Err(invalid("metadata alias/overlap"));
        }
        let work = mul(mul(tables, 512)?, 2)?;
        limit(work, limits.table_entries, "table entries examined")?;
        let fixed_read = add(
            reader.used,
            add(
                SECTOR,
                add(
                    mul(mul(mul(tables, TABLE)?, copies)?, 2)?,
                    if footer { mul(tables, 16)? } else { 0 },
                )?,
            )?,
        )?;
        limit(fixed_read, limits.read_bytes, "map read bytes")?;
        let mut table = [0; 2048];
        let mut redundant_table = [0; 2048];
        let mut populated = 0;
        let mut previous_sector = 0;
        for i in 0..entries as usize {
            if word(&primary, i) == 0 {
                continue;
            }
            read_table(
                &mut reader,
                &primary,
                &secondary,
                i,
                &mut table,
                &mut redundant_table,
            )?;
            for j in 0..512 {
                let sector = word(&table, j);
                let index = i as u64 * 512 + j as u64;
                if index >= grain_count && sector != 0 {
                    return Err(invalid("table unused slots"));
                }
                if sector == 0 {
                    continue;
                }
                if sector == 1
                    || sector as u64 * SECTOR < header.overhead_bytes()
                    || add(sector as u64 * SECTOR, SECTOR)? > extent
                {
                    return Err(invalid("grain pointer bounds"));
                }
                if sector <= previous_sector {
                    return Err(invalid("grain pointer order/alias"));
                }
                previous_sector = sector;
                populated += 1;
                limit(populated, limits.allocated_grains, "allocated grains")?;
            }
        }
        let reserved_bytes = add(
            scratch,
            mul(populated, std::mem::size_of::<StreamGrain>() as u64)?,
        )?;
        limit(reserved_bytes, limits.memory_bytes, "map memory bytes")?;
        limit(
            add(fixed_read, mul(populated, 12)?)?,
            limits.read_bytes,
            "map read bytes",
        )?;
        let mut grains = allocated(
            populated,
            StreamGrain {
                index: 0,
                sector: 0,
                compressed_bytes: 0,
            },
        )?;
        let mut written = 0;
        let mut cursor = header.overhead_bytes();
        for i in 0..entries as usize {
            let sector = word(&primary, i);
            if sector == 0 {
                continue;
            }
            read_table(
                &mut reader,
                &primary,
                &secondary,
                i,
                &mut table,
                &mut redundant_table,
            )?;
            for j in 0..512 {
                let at = word(&table, j);
                if at == 0 {
                    continue;
                }
                let index = i as u64 * 512 + j as u64;
                if written == grains.len() || index >= grain_count {
                    return Err(invalid("grain count changed"));
                }
                if at as u64 * SECTOR != cursor {
                    return Err(invalid("grain record sequence"));
                }
                let mut prefix = [0; 16];
                // Only the 12-byte grain header is read; the compressed payload
                // and sector padding are skipped. Grain parsing ignores bytes 12..16.
                reader.read(cursor, &mut prefix[..12])?;
                let StreamMarker::Grain {
                    logical_offset,
                    payload,
                    next_offset,
                } = StreamMarker::parse(&prefix, cursor, header, limits.envelope)?
                else {
                    return Err(invalid("expected grain record"));
                };
                if logical_offset != index * GRAIN {
                    return Err(invalid("grain LBA binding"));
                }
                grains[written] = StreamGrain {
                    index: index as u32,
                    sector: at,
                    compressed_bytes: payload.length as u32,
                };
                written += 1;
                cursor = next_offset;
            }
            if footer {
                let region = StreamRegion {
                    offset: sector as u64 * SECTOR,
                    length: TABLE,
                };
                if cursor != region.offset - SECTOR {
                    return Err(invalid("table record sequence"));
                }
                let mut prefix = [0; 16];
                reader.read(cursor, &mut prefix)?;
                if StreamMarker::parse(&prefix, cursor, header, limits.envelope)?
                    != StreamMarker::GrainTable(region)
                {
                    return Err(invalid("table marker"));
                }
                cursor = region.offset + region.length;
            }
        }
        let expected_end = if footer {
            envelope.primary_directory.offset - SECTOR
        } else {
            extent
        };
        if cursor != expected_end || written != grains.len() {
            return Err(invalid("unowned records or changed grain count"));
        }
        let mut last = [0; 512];
        reader.read(0, &mut last)?;
        if StreamHeader::parse(&last, extent, limits.envelope)? != *header {
            return Err(invalid("map header changed"));
        }
        if reader.source.seek(SeekFrom::End(0))? != extent {
            return Err(invalid("map length changed"));
        }
        let stats = StreamMapStats {
            metadata_bytes_read: reader.used,
            table_entries_examined: work,
            allocated_grains: populated,
            reserved_bytes,
        };
        Ok(Self {
            header: envelope.header,
            cid: envelope.cid,
            grains,
            stats,
        })
    }
}

#[derive(Clone, Copy, Default)]
struct Region {
    start: u64,
    end: u64,
}
struct Budget<'a, S> {
    source: &'a mut S,
    extent: u64,
    used: u64,
    max: u64,
}
impl<S: Read + Seek> Budget<'_, S> {
    fn read(&mut self, at: u64, buffer: &mut [u8]) -> Result<()> {
        let used = add(self.used, buffer.len() as u64)?;
        limit(used, self.max, "map read bytes")?;
        if add(at, buffer.len() as u64)? > self.extent {
            return Err(invalid("map read bounds"));
        }
        self.source.seek(SeekFrom::Start(at))?;
        self.source.read_exact(buffer)?;
        self.used = used;
        Ok(())
    }
}
fn read_table<S: Read + Seek>(
    reader: &mut Budget<'_, S>,
    primary: &[u8],
    secondary: &[u8],
    index: usize,
    table: &mut [u8; 2048],
    redundant: &mut [u8; 2048],
) -> Result<()> {
    reader.read(word(primary, index) as u64 * SECTOR, table)?;
    if !secondary.is_empty() {
        reader.read(word(secondary, index) as u64 * SECTOR, redundant)?;
        if table != redundant {
            return Err(invalid("grain table disagreement"));
        }
    }
    Ok(())
}
