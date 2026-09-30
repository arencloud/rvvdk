//! Authored layered byte oracle, independent of production map traversal.
#[path = "sparse_scale.rs"]
mod layout;
use rvvdk_core::{
    BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, EndpointIdentity, MemoryBlockDevice,
    Result,
};
use rvvdk_vmdk::{
    BackingError, BackingResolver, ChainEntry, ParentResolver, SparseChain, SparseChainDisk,
    SparseChainLimits, SparseChainSource,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
pub struct Device {
    pub inner: MemoryBlockDevice,
    pub calls: AtomicUsize,
    pub mode: AtomicUsize,
    pub track: bool,
}
impl Device {
    pub fn new(bytes: &[u8], track: bool) -> Arc<Self> {
        let inner = MemoryBlockDevice::new(bytes.len()).unwrap();
        inner.write_all_at(0, bytes).unwrap();
        Arc::new(Self {
            inner,
            calls: AtomicUsize::new(0),
            mode: AtomicUsize::new(0),
            track,
        })
    }
}
impl BlockDevice for Device {
    fn geometry(&self) -> DiskGeometry {
        self.inner.geometry()
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::READ
    }
    fn copy_endpoint(&self) -> Result<CopyEndpoint> {
        let mut e = self.inner.copy_endpoint()?;
        e.capabilities = self.capabilities();
        let m = self.mode.load(Ordering::Relaxed);
        if m & 8 != 0 {
            e.identity = None;
        }
        if m & 16 != 0 {
            e.size += 512;
        }
        Ok(e)
    }
    fn read_at(&self, o: u64, b: &mut [u8]) -> Result<usize> {
        if self.track {
            self.calls.fetch_add(1, Ordering::Relaxed);
        }
        let mode = self.mode.load(Ordering::Relaxed);
        if mode & 1 != 0 {
            return Ok(0);
        }
        if mode & 2 != 0 {
            return Err(rvvdk_core::Error::Unsupported);
        }
        let n = if mode & 4 != 0 {
            b.len().min(7)
        } else {
            b.len()
        };
        self.inner.read_at(o, &mut b[..n])
    }
    fn write_at(&self, _: u64, _: &[u8]) -> Result<usize> {
        panic!("source write")
    }
    fn flush(&self) -> Result<()> {
        panic!("source flush")
    }
}
#[derive(Clone)]
pub struct Spec {
    pub lengths: Vec<u64>,
    pub grain: u64,
    pub allocated: Vec<bool>,
    pub reverse: bool,
    pub salt: u8,
}
impl Spec {
    pub fn new(lengths: Vec<u64>, grain: u64, step: usize, phase: usize, salt: u8) -> Self {
        let n = lengths.iter().sum::<u64>() / grain;
        Self {
            lengths,
            grain,
            allocated: (0..n as usize)
                .map(|i| step != 0 && i % step == phase)
                .collect(),
            reverse: false,
            salt,
        }
    }
    pub fn byte(&self, pos: u64) -> u8 {
        self.salt.wrapping_add((pos % 251) as u8)
    }
}
struct Backings(Vec<Arc<Device>>);
impl BackingResolver for Backings {
    fn resolve(&self, n: &str) -> std::result::Result<Arc<dyn BlockDevice>, BackingError> {
        let i = n
            .strip_prefix("extent")
            .unwrap()
            .strip_suffix(".vmdk")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        Ok(self.0[i].clone())
    }
}
pub struct Layer {
    pub source: SparseChainSource,
    pub descriptor: Arc<Device>,
    pub backings: Vec<Arc<Device>>,
}
pub struct Fixture {
    pub specs: Vec<Spec>,
    pub layers: Vec<Layer>,
}
impl Fixture {
    pub fn new(specs: Vec<Spec>, track: bool) -> Self {
        let layers = specs
            .iter()
            .enumerate()
            .map(|(depth, spec)| {
                let parent = if depth + 1 == specs.len() {
                    "parentCID=ffffffff\n".to_owned()
                } else {
                    format!(
                        "parentCID={:08x}\nparentFileNameHint=\"parent.vmdk\"\n",
                        depth + 2
                    )
                };
                let mut text = format!(
                    "version=1\nCID={:08x}\n{parent}createType=\"twoGbMaxExtentSparse\"\n",
                    depth + 1
                );
                for (i, len) in spec.lengths.iter().enumerate() {
                    text += &format!("RW {} SPARSE \"extent{i}.vmdk\"\n", len / 512);
                }
                let mut logical = 0;
                let backings = spec
                    .lengths
                    .iter()
                    .map(|&length| {
                        let l = layout::Layout::new(length, spec.grain);
                        let mut bytes = l.metadata(layout::Pattern::Zero);
                        bytes.resize((l.overhead + length) as usize, 0);
                        // Split sparse permits an empty embedded descriptor region.
                        bytes[512..21 * 512].fill(0);
                        for i in 0..l.grains {
                            if spec.allocated[((logical + i * spec.grain) / spec.grain) as usize] {
                                let slot = if spec.reverse { l.grains - i - 1 } else { i };
                                let physical = l.overhead + slot * spec.grain;
                                for start in [l.gt, l.rgt] {
                                    layout::put32(
                                        &mut bytes,
                                        start + i as usize * 4,
                                        (physical / 512) as u32,
                                    );
                                }
                                for j in 0..spec.grain {
                                    bytes[(physical + j) as usize] =
                                        spec.byte(logical + i * spec.grain + j);
                                }
                            }
                        }
                        logical += length;
                        Device::new(&bytes, track)
                    })
                    .collect::<Vec<_>>();
                let descriptor = Device::new(text.as_bytes(), track);
                Layer {
                    source: SparseChainSource {
                        descriptor: descriptor.clone(),
                        backings: Arc::new(Backings(backings.clone())),
                        entry: ChainEntry::External,
                    },
                    descriptor,
                    backings,
                }
            })
            .collect();
        Self { specs, layers }
    }
    pub fn chain(&self) -> SparseChain {
        SparseChain::load(
            self.layers[0].source.clone(),
            self,
            SparseChainLimits::default(),
        )
        .unwrap()
    }
    pub fn disk(&self) -> SparseChainDisk {
        SparseChainDisk::new(self.chain())
    }
    pub fn oracle(&self, offset: u64, len: usize) -> Vec<u8> {
        (offset..offset + len as u64)
            .map(|pos| {
                self.specs
                    .iter()
                    .find(|s| s.allocated[(pos / s.grain) as usize])
                    .map_or(0, |s| s.byte(pos))
            })
            .collect()
    }
    pub fn reset(&self) {
        for l in &self.layers {
            l.descriptor.calls.store(0, Ordering::Relaxed);
            for b in &l.backings {
                b.calls.store(0, Ordering::Relaxed);
            }
        }
    }
    pub fn reads(&self) -> usize {
        self.layers
            .iter()
            .flat_map(|l| &l.backings)
            .map(|d| d.calls.load(Ordering::Relaxed))
            .sum()
    }
}
impl ParentResolver for Fixture {
    fn resolve_parent(
        &self,
        c: EndpointIdentity,
        h: &str,
    ) -> std::result::Result<Option<SparseChainSource>, BackingError> {
        assert_eq!(h, "parent.vmdk");
        let i = self
            .layers
            .iter()
            .position(|l| l.descriptor.copy_endpoint().unwrap().identity == Some(c))
            .unwrap();
        Ok(self.layers.get(i + 1).map(|l| l.source.clone()))
    }
}
