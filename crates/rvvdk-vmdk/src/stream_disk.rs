//! Native bounded logical reads from an owned, quiescent streamOptimized source.
use crate::{StreamError, StreamHeader, StreamMap, StreamMapLimits};
use miniz_oxide::inflate::{
    TINFLStatus,
    core::{DecompressorOxide, decompress, inflate_flags},
};
use rvvdk_core::{
    BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, DiskRange, Error, Extent, ExtentKind,
    Result, VirtualDisk,
};
use std::{
    io::{self, Read, Seek, SeekFrom},
    sync::{Arc, Mutex},
};

const GRAIN: usize = 65536;
#[derive(Clone, Copy, Debug)]
pub struct StreamDiskLimits {
    pub metadata: StreamMapLimits,
    /// Decode state plus requested input/output buffer slots; excludes map,
    /// allocator/mutex overhead, transient stack, caller buffers and backend memory.
    pub decode_memory_bytes: u64,
    pub request_bytes: u64,
    /// Conservative compressed records (including prefixes) per read, even if cached.
    pub encoded_read_bytes: u64,
    pub decoded_grains: u64,
    pub output_extents: usize,
}
impl Default for StreamDiskLimits {
    fn default() -> Self {
        Self {
            metadata: StreamMapLimits::default(),
            decode_memory_bytes: 512 << 10,
            request_bytes: 64 << 20,
            encoded_read_bytes: 256 << 20,
            decoded_grains: 1025,
            output_extents: 65536,
        }
    }
}
#[derive(Debug, thiserror::Error)]
pub enum StreamDiskError {
    #[error("{0}")]
    Map(#[from] StreamError),
    #[error("{0}")]
    Source(#[from] Error),
    #[error("stream disk limit: {0}")]
    Limit(&'static str),
}
struct DecodeState {
    decoder: DecompressorOxide,
    input: Vec<u8>,
    output: Vec<u8>,
    cached: Option<u64>,
}
/// Owns the validated source handle. One shared decode slot serializes data reads;
/// memory does not grow with callers. External mutation still requires exclusion
/// by the caller. `revalidate` is not a snapshot or content fingerprint.
pub struct StreamDisk {
    source: Arc<dyn BlockDevice>,
    initial: CopyEndpoint,
    map: StreamMap,
    geometry: DiskGeometry,
    limits: StreamDiskLimits,
    decode_memory: u64,
    state: Mutex<DecodeState>,
}
fn invalid(reason: &'static str) -> Error {
    Error::CorruptMetadata(reason.into())
}
fn allocation(n: usize) -> Result<Vec<u8>> {
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| Error::BufferAllocation {
            size: n,
            alignment: 1,
        })?;
    v.resize(n, 0);
    Ok(v)
}
impl StreamDisk {
    pub fn load(
        source: Arc<dyn BlockDevice>,
        limits: StreamDiskLimits,
    ) -> std::result::Result<Self, StreamDiskError> {
        let initial = source.copy_endpoint()?;
        if !initial.capabilities.contains(Capabilities::READ) {
            return Err(Error::Unsupported.into());
        }
        let map = StreamMap::read_from(
            SourceCursor {
                source: source.as_ref(),
                position: 0,
            },
            initial.size,
            limits.metadata,
        )?;
        let input_bytes = map
            .grains()
            .iter()
            .map(|g| g.payload().length + 12)
            .max()
            .unwrap_or(0);
        let decode_memory =
            std::mem::size_of::<DecodeState>() as u64 + input_bytes + GRAIN as u64 + 1;
        if decode_memory > limits.decode_memory_bytes {
            return Err(StreamDiskError::Limit("decode memory bytes"));
        }
        let state = DecodeState {
            decoder: DecompressorOxide::new(),
            input: allocation(
                usize::try_from(input_bytes).map_err(|_| Error::MemoryAccountingOverflow)?,
            )?,
            output: allocation(GRAIN + 1)?,
            cached: None,
        };
        let disk = Self {
            geometry: DiskGeometry::new(map.header().capacity_bytes(), 512, 512)?,
            source,
            initial,
            map,
            limits,
            decode_memory,
            state: Mutex::new(state),
        };
        disk.revalidate()?;
        Ok(disk)
    }
    pub fn map(&self) -> &StreamMap {
        &self.map
    }
    pub fn decode_memory_bytes(&self) -> u64 {
        self.decode_memory
    }
    /// Invalidates cached content, refreshes endpoint facts and checks the semantic
    /// header. Call before read sessions; source quiescence is required throughout.
    pub fn revalidate(&self) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| invalid("decode lock poisoned"))?;
        state.cached = None;
        let now = self.source.copy_endpoint()?;
        if now.size != self.initial.size
            || now.identity != self.initial.identity
            || now.capabilities != self.initial.capabilities
        {
            return Err(Error::EndpointChanged(
                "stream backing facts changed".into(),
            ));
        }
        let mut raw = [0; 512];
        self.source.read_exact_at(0, &mut raw)?;
        let header = StreamHeader::parse(&raw, now.size, self.limits.metadata.envelope)
            .map_err(|e| Error::EndpointChanged(e.to_string()))?;
        if header != *self.map.header() {
            return Err(Error::EndpointChanged("stream header changed".into()));
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
    fn records(&self, range: DiskRange) -> &[crate::StreamGrain] {
        let grains = self.map.grains();
        let first = grains.partition_point(|g| g.logical_offset() + GRAIN as u64 <= range.offset());
        let last = grains.partition_point(|g| g.logical_offset() < range.end());
        &grains[first..last.max(first)]
    }
    fn read_record(&self, g: crate::StreamGrain, state: &mut DecodeState) -> Result<()> {
        if state.cached == Some(g.index()) {
            return Ok(());
        }
        // Failed replacement must never leave a previous cache key attached to
        // partially overwritten output, including I/O and checksum failures.
        state.cached = None;
        let length = (g.payload().length + 12) as usize;
        self.source
            .read_exact_at(g.marker_offset(), &mut state.input[..length])?;
        let lba = u64::from_le_bytes(state.input[..8].try_into().unwrap());
        let size = u32::from_le_bytes(state.input[8..12].try_into().unwrap());
        if lba != g.logical_offset() / 512 || size as u64 != g.payload().length {
            return Err(invalid("grain prefix changed"));
        }
        state.decoder.init();
        let flags = inflate_flags::TINFL_FLAG_PARSE_ZLIB_HEADER
            | inflate_flags::TINFL_FLAG_USING_NON_WRAPPING_OUTPUT_BUF;
        let (status, consumed, written) = decompress(
            &mut state.decoder,
            &state.input[12..length],
            &mut state.output,
            0,
            flags,
        );
        if status != TINFLStatus::Done || consumed != length - 12 || written != GRAIN {
            return Err(invalid(
                "grain must be one complete checksummed zlib stream of exactly 64 KiB",
            ));
        }
        state.cached = Some(g.index());
        Ok(())
    }
    /// Coalesced logical runs, O(present records), including very large holes.
    fn runs(
        &self,
        range: DiskRange,
        mut visit: impl FnMut(u64, u64, ExtentKind) -> Result<()>,
    ) -> Result<()> {
        if range.is_empty() {
            return Ok(());
        }
        let mut pos = range.offset();
        let mut data_start = None;
        for g in self.records(range) {
            let start = g.logical_offset().max(range.offset());
            let end = (g.logical_offset() + GRAIN as u64).min(range.end());
            if start > pos {
                if let Some(at) = data_start.take() {
                    visit(at, pos - at, ExtentKind::Data)?;
                }
                visit(pos, start - pos, ExtentKind::Zero)?;
            }
            data_start.get_or_insert(start);
            pos = end;
        }
        if let Some(at) = data_start {
            visit(at, pos - at, ExtentKind::Data)?;
        }
        if pos < range.end() {
            visit(pos, range.end() - pos, ExtentKind::Zero)?;
        }
        Ok(())
    }
}
impl VirtualDisk for StreamDisk {
    fn geometry(&self) -> DiskGeometry {
        self.geometry
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::READ | Capabilities::EXTENTS | Capabilities::SPARSE
    }
    fn copy_endpoint(&self) -> Result<CopyEndpoint> {
        self.revalidate()?;
        // Container identity cannot represent logical bytes; alias guard below
        // checks the retained physical source explicitly.
        Ok(CopyEndpoint {
            size: self.size(),
            capabilities: self.capabilities(),
            identity: None,
        })
    }
    fn validate_destination_identity(&self, destination: CopyEndpoint) -> Result<()> {
        self.revalidate()?;
        let source = self.initial.identity.ok_or(Error::InvalidEndpoint {
            reason: "stream source identity is unknown",
        })?;
        let destination = destination.identity.ok_or(Error::InvalidEndpoint {
            reason: "stream destination identity is unknown",
        })?;
        if source == destination {
            return Err(Error::AliasedEndpoints);
        }
        Ok(())
    }
    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        let range = self.range(offset, buffer.len() as u64)?;
        if range.is_empty() {
            return Ok(0);
        }
        if buffer.len() as u64 > self.limits.request_bytes {
            return Err(Error::InvalidEndpoint {
                reason: "stream read request limit",
            });
        }
        let records = self.records(range);
        if records.len() as u64 > self.limits.decoded_grains {
            return Err(Error::InvalidEndpoint {
                reason: "stream decoded grain limit",
            });
        }
        let encoded = records
            .iter()
            .try_fold(0u64, |total, g| total.checked_add(g.payload().length + 12))
            .ok_or(Error::MemoryAccountingOverflow)?;
        if encoded > self.limits.encoded_read_bytes {
            return Err(Error::InvalidEndpoint {
                reason: "stream encoded read limit",
            });
        }
        if records.is_empty() {
            buffer.fill(0);
            return Ok(buffer.len());
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| invalid("decode lock poisoned"))?;
        let mut pos = 0;
        for &g in records {
            let start = g.logical_offset().max(offset);
            let end = (g.logical_offset() + GRAIN as u64).min(range.end());
            let target = (start - offset) as usize;
            buffer[pos..target].fill(0);
            self.read_record(g, &mut state)?;
            let within = (start - g.logical_offset()) as usize;
            let length = (end - start) as usize;
            buffer[target..target + length].copy_from_slice(&state.output[within..within + length]);
            pos = target + length;
        }
        buffer[pos..].fill(0);
        Ok(buffer.len())
    }
    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
        let range = self.range(offset, length)?;
        let mut count = 0usize;
        self.runs(range, |_, _, _| {
            if count == self.limits.output_extents {
                return Err(Error::InvalidEndpoint {
                    reason: "stream output extent limit",
                });
            }
            count += 1;
            Ok(())
        })?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(count)
            .map_err(|_| Error::BufferAllocation {
                size: count.saturating_mul(std::mem::size_of::<Extent>()),
                alignment: std::mem::align_of::<Extent>(),
            })?;
        self.runs(range, |start, length, kind| {
            result.push(Extent::new(start, length, kind)?);
            Ok(())
        })?;
        Ok(result)
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
/// Metadata-only adapter retains positional reads and obtains fresh length facts.
struct SourceCursor<'a> {
    source: &'a dyn BlockDevice,
    position: u64,
}
impl Read for SourceCursor<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let read = self
            .source
            .read_at(self.position, buffer)
            .map_err(io::Error::other)?;
        if read > buffer.len() {
            return Err(io::Error::other("backend over-reported read"));
        }
        self.position = self
            .position
            .checked_add(read as u64)
            .ok_or_else(|| io::Error::other("source position overflow"))?;
        Ok(read)
    }
}
impl Seek for SourceCursor<'_> {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let position = match from {
            SeekFrom::Start(n) => n as i128,
            SeekFrom::Current(n) => self.position as i128 + n as i128,
            SeekFrom::End(n) => {
                self.source.copy_endpoint().map_err(io::Error::other)?.size as i128 + n as i128
            }
        };
        self.position =
            u64::try_from(position).map_err(|_| io::Error::other("invalid source seek"))?;
        Ok(self.position)
    }
}
