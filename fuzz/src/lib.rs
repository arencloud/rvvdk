//! Bounded, memory-only qualification harnesses. No production cfg overrides.
use rvvdk_core::{BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, EndpointIdentity};
use rvvdk_vmdk::*;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
#[path = "../../crates/rvvdk-vmdk/tests/support/sparse_metadata.rs"]
#[allow(dead_code)]
mod authored;
pub const INPUT_LIMIT: usize = 65536;
pub const CHAIN_INPUT_LIMIT: usize = 520;
const MEMORY: u64 = 4 << 20;
const READS: u64 = 4 << 20;
fn text_limits() -> Limits {
    Limits {
        descriptor_bytes: INPUT_LIMIT,
        line_bytes: 4096,
        extents: 32,
        metadata_entries: 16,
        filename_bytes: 1024,
    }
}
fn header_limits() -> SparseLimits {
    SparseLimits {
        capacity_bytes: 16 << 20,
        grain_bytes: 1 << 20,
        descriptor_bytes: INPUT_LIMIT as u64,
        directory_entries: 64,
        metadata_bytes: 1 << 20,
    }
}
fn metadata_limits() -> SparseMetadataLimits {
    SparseMetadataLimits {
        header: header_limits(),
        descriptor: text_limits(),
        memory_bytes: MEMORY,
        read_bytes: READS,
    }
}
fn extent_invariants(extents: &[Extent<'_>], size: u64) {
    assert!(extents.len() <= 32);
    let mut end = 0;
    for e in extents {
        assert_eq!(e.logical_offset(), end);
        assert!(e.size_bytes() > 0);
        end = end.checked_add(e.size_bytes()).unwrap();
    }
    assert_eq!(end, size);
}
pub fn descriptor(data: &[u8]) -> bool {
    if data.len() > INPUT_LIMIT {
        return false;
    }
    let mut accepted = false;
    if let Ok(d) = Descriptor::parse_with_limits(data, text_limits()) {
        extent_invariants(d.extents(), d.size_bytes());
        assert!(d.metadata().len() <= 16);
        accepted = true;
    }
    if let Ok(d) = SparseDescriptor::parse_with_limits(data, text_limits()) {
        extent_invariants(d.extents(), d.size_bytes());
        assert!(d.metadata().len() <= 16);
        accepted = true;
    }
    if let Ok(d) = SparseLayerDescriptor::parse_with_limits(data, text_limits()) {
        extent_invariants(d.extents(), d.size_bytes());
        if d.parent().is_none() {
            assert!(SparseDescriptor::parse_with_limits(data, text_limits()).is_ok());
        }
        accepted = true;
    }
    accepted
}
fn extent_size(data: &[u8]) -> Option<(u64, &[u8])> {
    Some((
        u64::from_le_bytes(data.get(..8)?.try_into().unwrap()),
        &data[8..],
    ))
}
pub fn header(data: &[u8]) -> bool {
    if data.len() > 520 {
        return false;
    }
    let Some((size, bytes)) = extent_size(data) else {
        return false;
    };
    if let Ok(h) = SparseHeader::parse_with_limits(bytes, size, header_limits()) {
        assert!(h.capacity_bytes() <= header_limits().capacity_bytes);
        assert!(h.grain_bytes() <= header_limits().grain_bytes);
        assert!(h.primary_directory().end() <= size);
        if let Some(r) = h.redundant_directory() {
            assert!(r.end() <= size);
        }
        if let Some(r) = h.descriptor() {
            assert!(r.end() <= size);
            assert!(r.length() <= INPUT_LIMIT as u64);
        }
        return true;
    }
    false
}
#[derive(Default)]
struct Meter {
    bytes: AtomicU64,
    calls: AtomicU64,
}
impl Meter {
    fn check(&self, limit: u64) {
        assert!(self.bytes.load(Ordering::Relaxed) <= limit);
        assert!(self.calls.load(Ordering::Relaxed) <= limit + 32);
    }
}
struct Device {
    prefix: Vec<u8>,
    size: u64,
    id: Option<usize>,
    meter: Arc<Meter>,
}
impl BlockDevice for Device {
    fn geometry(&self) -> DiskGeometry {
        DiskGeometry::new(self.size, 512, 512).unwrap()
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::READ
    }
    fn copy_endpoint(&self) -> rvvdk_core::Result<CopyEndpoint> {
        Ok(CopyEndpoint {
            size: self.size,
            capabilities: self.capabilities(),
            identity: self.id.map(|address| EndpointIdentity::Memory { address }),
        })
    }
    fn read_at(&self, offset: u64, b: &mut [u8]) -> rvvdk_core::Result<usize> {
        self.meter.calls.fetch_add(1, Ordering::Relaxed);
        if offset >= self.size {
            return Ok(0);
        }
        let n = (self.size - offset).min(b.len() as u64) as usize;
        b[..n].fill(0);
        if offset < (self.prefix.len() as u64) {
            let offset = offset as usize;
            let copied = n.min(self.prefix.len() - offset);
            b[..copied].copy_from_slice(&self.prefix[offset..offset + copied]);
        }
        self.meter.bytes.fetch_add(n as u64, Ordering::Relaxed);
        Ok(n)
    }
    fn write_at(&self, _: u64, _: &[u8]) -> rvvdk_core::Result<usize> {
        panic!("source mutation")
    }
    fn flush(&self) -> rvvdk_core::Result<()> {
        panic!("source flush")
    }
}
struct Backing(Arc<Device>);
impl BackingResolver for Backing {
    fn resolve(&self, name: &str) -> Result<Arc<dyn BlockDevice>, BackingError> {
        if name == "disk.vmdk" {
            Ok(self.0.clone())
        } else {
            Err(BackingError::UnsafeReference)
        }
    }
}
fn base_text() -> String {
    authored::descriptor("monolithicSparse", "RW 2048 SPARSE \"disk.vmdk\"")
}
pub fn metadata(data: &[u8]) -> bool {
    if data.len() > INPUT_LIMIT {
        return false;
    }
    let Some((size, prefix)) = extent_size(data) else {
        return false;
    };
    let meter = Arc::new(Meter::default());
    let backing = Backing(Arc::new(Device {
        prefix: prefix.to_vec(),
        size,
        id: Some(1),
        meter: meter.clone(),
    }));
    let text = base_text();
    let descriptor = SparseDescriptor::parse(text.as_bytes()).unwrap();
    let result = SparseMetadata::load(&descriptor, 0, &backing, metadata_limits());
    meter.check(READS);
    if let Ok(m) = result {
        assert!(m.reserved_memory_bytes() <= MEMORY);
        assert!(m.metadata_read_bytes() <= READS);
        assert_eq!(m.grain_sectors().len() as u64, m.header().grain_count());
        assert_eq!(m.header().capacity_bytes(), 1048576);
        assert!(m.revalidate().is_ok());
        return true;
    }
    false
}
struct Graph {
    sources: Vec<SparseChainSource>,
    mode: u8,
    calls: AtomicU64,
}
impl ParentResolver for Graph {
    fn resolve_parent(
        &self,
        child: EndpointIdentity,
        hint: &str,
    ) -> Result<Option<SparseChainSource>, BackingError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        if hint != "parent.vmdk" {
            return Err(BackingError::UnsafeReference);
        }
        if self.mode == 1 {
            return Ok(None);
        }
        let Some(index) = self
            .sources
            .iter()
            .position(|s| s.descriptor.copy_endpoint().unwrap().identity == Some(child))
        else {
            return Err(BackingError::UnsafeReference);
        };
        Ok(self
            .sources
            .get(if self.mode == 2 { 0 } else { index + 1 })
            .cloned())
    }
}
/// Structured mutations preserve valid scaffolding to reach deep admission.
/// Header: depth-1, embedded mask, resolver mode, limit mode, four reserved bytes.
/// Up to 64 records: layer, region, little-endian offset, four replacement bytes.
pub fn chain(data: &[u8]) -> bool {
    if data.len() < 8 || data.len() > CHAIN_INPUT_LIMIT {
        return false;
    }
    let depth = (data[0] % 4 + 1) as usize;
    let mode = data[2] % 5;
    let meter = Arc::new(Meter::default());
    let mut sources = Vec::new();
    for i in 0..depth {
        let embedded = data[1] & (1 << i) != 0;
        let mut text = base_text().replace("CID=12345678", &format!("CID={:08x}", i + 1));
        if i + 1 < depth {
            text = text.replace(
                "parentCID=ffffffff",
                &format!(
                    "parentCID={:08x}\nparentFileNameHint=\"parent.vmdk\"",
                    i + 2
                ),
            );
        }
        let mut descriptor = text.into_bytes();
        let mut prefix = authored::bytes(Some(std::str::from_utf8(&descriptor).unwrap()));
        prefix.truncate(16384);
        for record in data[8..].chunks_exact(8) {
            if record[0] as usize % depth != i {
                continue;
            }
            let offset = u16::from_le_bytes([record[2], record[3]]) as usize;
            let offsets = match record[1] % 6 {
                0 => vec![offset % 509],
                1 => vec![512 + offset % 10237],
                2 => vec![21 * 512 + offset % 509, 26 * 512 + offset % 509],
                3 => vec![27 * 512 + offset % 2045],
                4 => vec![22 * 512 + offset % 2045, 27 * 512 + offset % 2045],
                _ => {
                    let at = offset % descriptor.len();
                    let n = 4.min(descriptor.len() - at);
                    descriptor[at..at + n].copy_from_slice(&record[4..4 + n]);
                    continue;
                }
            };
            for at in offsets {
                prefix[at..at + 4].copy_from_slice(&record[4..8]);
            }
        }
        let backing = Arc::new(Device {
            prefix,
            size: 196608,
            id: if mode == 4 {
                None
            } else {
                Some(if mode == 3 { 1 } else { 2 * i + 1 })
            },
            meter: meter.clone(),
        });
        let descriptor: Arc<dyn BlockDevice> = if embedded {
            backing.clone()
        } else {
            Arc::new(Device {
                size: descriptor.len() as u64,
                prefix: descriptor,
                id: Some(2 * i + 2),
                meter: meter.clone(),
            })
        };
        sources.push(SparseChainSource {
            descriptor,
            backings: Arc::new(Backing(backing)),
            entry: if embedded {
                ChainEntry::Embedded
            } else {
                ChainEntry::External
            },
        });
    }
    let mut limits = SparseChainLimits {
        metadata: metadata_limits(),
        layers: 4,
        extents: 8,
        descriptor_bytes: 4 * INPUT_LIMIT as u64,
        memory_bytes: MEMORY,
        read_bytes: READS,
    };
    match data[3] % 4 {
        1 => limits.memory_bytes = 1024,
        2 => limits.read_bytes = 512,
        3 => limits.layers = 1,
        _ => {}
    }
    let graph = Graph {
        sources,
        mode,
        calls: AtomicU64::new(0),
    };
    let result = SparseChain::load(graph.sources[0].clone(), &graph, limits);
    meter.check(limits.read_bytes);
    assert!(graph.calls.load(Ordering::Relaxed) < limits.layers as u64);
    if let Ok(c) = result {
        assert!(c.layers().len() <= limits.layers);
        assert!(c.layers().iter().map(|l| l.metadata().len()).sum::<usize>() <= limits.extents);
        assert!(c.reserved_memory_bytes() <= limits.memory_bytes);
        assert!(c.metadata_read_bytes() <= limits.read_bytes);
        assert!(c.descriptor_bytes() <= limits.descriptor_bytes);
        assert!(c.layers().last().unwrap().parent_cid().is_none());
        for pair in c.layers().windows(2) {
            assert_eq!(pair[0].parent_cid(), Some(pair[1].cid()));
            assert_eq!(pair[0].size_bytes(), pair[1].size_bytes());
        }
        assert!(c.revalidate().is_ok());
        return true;
    }
    false
}
pub fn seeds() -> Vec<(&'static str, &'static str, Vec<u8>)> {
    let base = base_text();
    let bytes = authored::bytes(Some(&base));
    let mut header = 196608u64.to_le_bytes().to_vec();
    header.extend(&bytes[..512]);
    let mut metadata = 196608u64.to_le_bytes().to_vec();
    metadata.extend(&bytes[..16384]);
    let child = base.replace(
        "parentCID=ffffffff",
        "parentCID=12345679\nparentFileNameHint=\"parent.vmdk\"",
    );
    vec![
        ("descriptor", "base", base.into_bytes()),
        ("descriptor", "child", child.into_bytes()),
        (
            "descriptor",
            "flat",
            authored::descriptor("monolithicFlat", "RW 2048 FLAT \"disk.vmdk\" 0").into_bytes(),
        ),
        ("descriptor", "invalid", vec![0xff, 0]),
        ("header", "valid", header),
        ("header", "short", vec![0; 8]),
        ("metadata", "valid", metadata),
        ("metadata", "short", vec![0; 8]),
        ("chain", "embedded", vec![2, 7, 0, 0, 0, 0, 0, 0]),
        ("chain", "external", vec![2, 0, 0, 0, 0, 0, 0, 0]),
        ("chain", "mixed", vec![3, 5, 0, 0, 0, 0, 0, 0]),
        ("chain", "cycle", vec![2, 7, 2, 0, 0, 0, 0, 0]),
        ("chain", "missing", vec![2, 7, 1, 0, 0, 0, 0, 0]),
        ("chain", "depth", vec![2, 7, 0, 3, 0, 0, 0, 0]),
    ]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_seeds_reach_success_and_rejection() {
        for (target, name, bytes) in seeds() {
            let accepted = match target {
                "descriptor" => descriptor(&bytes),
                "header" => header(&bytes),
                "metadata" => metadata(&bytes),
                "chain" => chain(&bytes),
                _ => unreachable!(),
            };
            assert_eq!(
                accepted,
                !matches!(name, "invalid" | "short" | "cycle" | "missing" | "depth"),
                "{target}/{name}"
            );
        }
    }
    #[test]
    fn size_gates_reject_before_scaffolding() {
        let bytes = vec![0; INPUT_LIMIT + 1];
        assert!(!descriptor(&bytes));
        assert!(!header(&bytes));
        assert!(!metadata(&bytes));
        assert!(!chain(&bytes));
    }
}
