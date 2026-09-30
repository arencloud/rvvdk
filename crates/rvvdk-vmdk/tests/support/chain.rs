//! Authored chain fixtures. No producer implementation or SDK code.
#[path = "sparse_metadata.rs"]
mod sparse;
use rvvdk_core::{
    BlockDevice, Capabilities, CopyEndpoint, DiskGeometry, EndpointIdentity, MemoryBlockDevice,
    Result,
};
use rvvdk_vmdk::{BackingError, BackingResolver, ChainEntry, ParentResolver, SparseChainSource};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
pub struct Tracked {
    pub device: Arc<MemoryBlockDevice>,
    pub calls: AtomicU64,
    pub bytes: AtomicU64,
    pub unknown: AtomicBool,
    pub changed: AtomicBool,
    pub fail: AtomicBool,
    pub track: bool,
}
impl Tracked {
    pub fn new(bytes: &[u8], track: bool) -> Arc<Self> {
        Arc::new(Self {
            device: sparse::device(bytes),
            calls: AtomicU64::new(0),
            bytes: AtomicU64::new(0),
            unknown: AtomicBool::new(false),
            changed: AtomicBool::new(false),
            fail: AtomicBool::new(false),
            track,
        })
    }
}
impl BlockDevice for Tracked {
    fn geometry(&self) -> DiskGeometry {
        self.device.geometry()
    }
    fn capabilities(&self) -> Capabilities {
        Capabilities::READ
    }
    fn copy_endpoint(&self) -> Result<CopyEndpoint> {
        let mut e = self.device.copy_endpoint()?;
        e.capabilities = self.capabilities();
        if self.unknown.load(Ordering::Relaxed) {
            e.identity = None;
        }
        if self.changed.load(Ordering::Relaxed) {
            e.size += 512;
        }
        Ok(e)
    }
    fn read_at(&self, offset: u64, b: &mut [u8]) -> Result<usize> {
        if self.track {
            self.calls.fetch_add(1, Ordering::Relaxed);
            self.bytes.fetch_add(b.len() as u64, Ordering::Relaxed);
        }
        if self.fail.load(Ordering::Relaxed) {
            return Err(rvvdk_core::Error::Unsupported);
        }
        self.device.read_at(offset, b)
    }
    fn write_at(&self, _: u64, _: &[u8]) -> Result<usize> {
        panic!("source write")
    }
    fn flush(&self) -> Result<()> {
        panic!("source flush")
    }
}
pub struct Backings(pub Arc<Tracked>);
impl BackingResolver for Backings {
    fn resolve(&self, name: &str) -> std::result::Result<Arc<dyn BlockDevice>, BackingError> {
        assert!(name == "disk.vmdk" || name == "second.vmdk");
        Ok(self.0.clone())
    }
}
pub fn text(cid: u32, parent: Option<(u32, &str)>, split: bool) -> String {
    let kind = if split {
        "twoGbMaxExtentSparse"
    } else {
        "monolithicSparse"
    };
    let mut t = sparse::descriptor(kind, "RW 2048 SPARSE \"disk.vmdk\"")
        .replace("CID=12345678", &format!("CID={cid:08x}"));
    if let Some((cid, name)) = parent {
        t = t.replace(
            "parentCID=ffffffff",
            &format!("parentCID={cid:08x}\nparentFileNameHint=\"{name}\""),
        );
    }
    t
}
pub struct Node {
    pub source: SparseChainSource,
    pub descriptor: Arc<Tracked>,
    pub backing: Arc<Tracked>,
}
pub fn node(text: &str, embedded: bool, track: bool) -> Node {
    let backing = Tracked::new(&sparse::bytes(Some(text)), track);
    let descriptor = if embedded {
        backing.clone()
    } else {
        Tracked::new(text.as_bytes(), track)
    };
    Node {
        source: SparseChainSource {
            descriptor: descriptor.clone(),
            backings: Arc::new(Backings(backing.clone())),
            entry: if embedded {
                ChainEntry::Embedded
            } else {
                ChainEntry::External
            },
        },
        descriptor,
        backing,
    }
}
pub struct Graph {
    pub nodes: Vec<Node>,
    pub calls: AtomicU64,
}
impl Graph {
    pub fn new(depth: usize, embedded: bool, track: bool) -> Self {
        Self {
            nodes: (0..depth)
                .map(|i| {
                    node(
                        &text(
                            i as u32 + 1,
                            (i + 1 < depth).then_some((i as u32 + 2, "parent.vmdk")),
                            false,
                        ),
                        embedded,
                        track,
                    )
                })
                .collect(),
            calls: AtomicU64::new(0),
        }
    }
    pub fn root(&self) -> SparseChainSource {
        self.nodes[0].source.clone()
    }
}
impl ParentResolver for Graph {
    fn resolve_parent(
        &self,
        child: EndpointIdentity,
        hint: &str,
    ) -> std::result::Result<Option<SparseChainSource>, BackingError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        if hint != "parent.vmdk" {
            return Err(BackingError::UnsafeReference);
        }
        let index = self
            .nodes
            .iter()
            .position(|n| n.descriptor.copy_endpoint().unwrap().identity == Some(child))
            .unwrap();
        Ok(self.nodes.get(index + 1).map(|n| n.source.clone()))
    }
}
