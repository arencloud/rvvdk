//! Metadata-only parent admission. No logical-zero or read semantics are exposed.
use crate::{
    BackingError, BackingResolver, CreateType, SparseHeader, SparseLayerDescriptor, SparseMetadata,
    SparseMetadataError, SparseMetadataLimits,
};
use rvvdk_core::{BlockDevice, Capabilities, CopyEndpoint, EndpointIdentity};
use std::sync::Arc;
use thiserror::Error;

/// Explicit entry form; the caller authorizes and opens each descriptor/container.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChainEntry {
    External,
    Embedded,
}
#[derive(Clone)]
pub struct SparseChainSource {
    pub descriptor: Arc<dyn BlockDevice>,
    /// Namespace anchored to this layer, not to the original child.
    pub backings: Arc<dyn BackingResolver>,
    pub entry: ChainEntry,
}
/// Opt-in policy boundary. The hint is untrusted and must not authorize arbitrary
/// paths. Return None for a missing parent; use an error for denied resolution.
/// The child's retained object identity provides namespace context without a path.
pub trait ParentResolver: Send + Sync {
    fn resolve_parent(
        &self,
        child: EndpointIdentity,
        hint: &str,
    ) -> std::result::Result<Option<SparseChainSource>, BackingError>;
}
#[derive(Clone, Copy, Debug)]
pub struct SparseChainLimits {
    pub metadata: SparseMetadataLimits,
    /// Includes the leaf and terminal base; checked before following another hint.
    pub layers: usize,
    /// Total extent handles, including repeated references (which are rejected).
    pub extents: usize,
    /// Sum of acquired descriptor payload, including terminal padding.
    pub descriptor_bytes: u64,
    /// Conservative reservations: layer slots, text, map structs and loader budgets.
    pub memory_bytes: u64,
    /// Descriptor/header acquisition plus all backing metadata read payload.
    pub read_bytes: u64,
}
impl Default for SparseChainLimits {
    fn default() -> Self {
        Self {
            metadata: SparseMetadataLimits::default(),
            layers: 16,
            extents: 128,
            descriptor_bytes: 8 << 20,
            memory_bytes: 128 << 20,
            read_bytes: 256 << 20,
        }
    }
}
#[derive(Debug, Error)]
pub enum SparseChainError {
    #[error("{0}")]
    Metadata(#[from] SparseMetadataError),
    #[error("{0}")]
    Resolve(#[from] BackingError),
    #[error("chain descriptor I/O: {0}")]
    Device(#[from] rvvdk_core::Error),
    #[error("chain resource limit: {0}")]
    Limit(&'static str),
    #[error("chain parent missing")]
    MissingParent,
    #[error("chain CID mismatch")]
    CidMismatch,
    #[error("chain capacity mismatch")]
    CapacityMismatch,
    #[error("chain object cycle or shared backing")]
    RepeatedIdentity,
    #[error("chain source identity unknown or changed")]
    Identity,
    #[error("invalid chain entry: {0}")]
    Invalid(&'static str),
    #[error("chain allocation failed")]
    Allocation,
}
type Result<T> = std::result::Result<T, SparseChainError>;
fn charge(used: &mut u64, amount: u64, ceiling: u64, label: &'static str) -> Result<()> {
    let total = used
        .checked_add(amount)
        .filter(|v| *v <= ceiling)
        .ok_or(SparseChainError::Limit(label))?;
    *used = total;
    Ok(())
}
fn slots<T>(count: usize) -> Result<u64> {
    (count as u64)
        .checked_mul(std::mem::size_of::<T>() as u64)
        .ok_or(SparseChainError::Limit("allocation arithmetic"))
}
fn identity(endpoint: CopyEndpoint) -> Result<EndpointIdentity> {
    if !endpoint.capabilities.contains(Capabilities::READ) {
        return Err(SparseChainError::Identity);
    }
    endpoint.identity.ok_or(SparseChainError::Identity)
}
fn observe(source: &dyn BlockDevice, initial: CopyEndpoint) -> Result<()> {
    let live = source.copy_endpoint()?;
    if identity(live)? != identity(initial)? || live.size != initial.size {
        return Err(SparseChainError::Identity);
    }
    Ok(())
}

/// One retained layer. Zero grain entries remain unallocated, not logical zero.
pub struct SparseChainLayer {
    descriptor: Arc<dyn BlockDevice>,
    initial: CopyEndpoint,
    cid: u32,
    parent_cid: Option<u32>,
    size: u64,
    metadata: Vec<SparseMetadata>,
}
impl SparseChainLayer {
    pub fn cid(&self) -> u32 {
        self.cid
    }
    pub fn parent_cid(&self) -> Option<u32> {
        self.parent_cid
    }
    pub fn size_bytes(&self) -> u64 {
        self.size
    }
    pub fn descriptor_endpoint(&self) -> CopyEndpoint {
        self.initial
    }
    pub fn metadata(&self) -> &[SparseMetadata] {
        &self.metadata
    }
    fn contains(&self, id: EndpointIdentity) -> bool {
        self.initial.identity == Some(id)
            || self
                .metadata
                .iter()
                .any(|m| m.initial_endpoint().identity == Some(id))
    }
}
/// Validated leaf-to-base metadata, intentionally not a VirtualDisk.
/// Caller must keep all sources quiescent; CIDs/identities are not content hashes.
pub struct SparseChain {
    layers: Vec<SparseChainLayer>,
    reserved_memory: u64,
    read_bytes: u64,
    descriptor_bytes: u64,
}
impl SparseChain {
    pub fn load(
        mut source: SparseChainSource,
        parents: &dyn ParentResolver,
        limits: SparseChainLimits,
    ) -> Result<Self> {
        if limits.layers == 0 {
            return Err(SparseChainError::Limit("layers"));
        }
        let mut chain = Self {
            layers: Vec::new(),
            reserved_memory: 0,
            read_bytes: 0,
            descriptor_bytes: 0,
        };
        charge(
            &mut chain.reserved_memory,
            slots::<SparseChainLayer>(limits.layers)?,
            limits.memory_bytes,
            "memory bytes",
        )?;
        chain
            .layers
            .try_reserve_exact(limits.layers)
            .map_err(|_| SparseChainError::Allocation)?;
        let mut total_extents = 0usize;
        loop {
            let initial = source.descriptor.copy_endpoint()?;
            let id = identity(initial)?;
            if chain.layers.iter().any(|l| l.contains(id)) {
                return Err(SparseChainError::RepeatedIdentity);
            }
            let (offset, length) = match source.entry {
                ChainEntry::External => (0, initial.size),
                ChainEntry::Embedded => {
                    charge(&mut chain.read_bytes, 512, limits.read_bytes, "read bytes")?;
                    let mut bytes = [0; 512];
                    source.descriptor.read_exact_at(0, &mut bytes)?;
                    let header = SparseHeader::parse_with_limits(
                        &bytes,
                        initial.size,
                        limits.metadata.header,
                    )
                    .map_err(SparseMetadataError::from)?;
                    let region = header
                        .descriptor()
                        .ok_or(SparseChainError::Invalid("embedded descriptor missing"))?;
                    (region.offset(), region.length())
                }
            };
            if length > limits.metadata.descriptor.descriptor_bytes as u64 {
                return Err(SparseChainError::Limit("per-layer descriptor bytes"));
            }
            charge(
                &mut chain.descriptor_bytes,
                length,
                limits.descriptor_bytes,
                "descriptor bytes",
            )?;
            charge(
                &mut chain.reserved_memory,
                length,
                limits.memory_bytes,
                "memory bytes",
            )?;
            charge(
                &mut chain.read_bytes,
                length,
                limits.read_bytes,
                "read bytes",
            )?;
            let length =
                usize::try_from(length).map_err(|_| SparseChainError::Limit("address space"))?;
            let mut text = Vec::new();
            text.try_reserve_exact(length)
                .map_err(|_| SparseChainError::Allocation)?;
            text.resize(length, 0);
            source.descriptor.read_exact_at(offset, &mut text)?;
            observe(source.descriptor.as_ref(), initial)?;
            let descriptor =
                SparseLayerDescriptor::parse_with_limits(&text, limits.metadata.descriptor)
                    .map_err(SparseMetadataError::from)?;
            if source.entry == ChainEntry::Embedded
                && descriptor.create_type() != CreateType::MonolithicSparse
            {
                return Err(SparseChainError::Invalid(
                    "embedded entry must be monolithic",
                ));
            }
            if let Some(child) = chain.layers.last() {
                if child.parent_cid != Some(descriptor.cid()) {
                    return Err(SparseChainError::CidMismatch);
                }
                if child.size != descriptor.size_bytes() {
                    return Err(SparseChainError::CapacityMismatch);
                }
            }
            let count = descriptor.extents().len();
            total_extents = total_extents
                .checked_add(count)
                .filter(|&n| n <= limits.extents)
                .ok_or(SparseChainError::Limit("extents"))?;
            charge(
                &mut chain.reserved_memory,
                slots::<SparseMetadata>(count)?,
                limits.memory_bytes,
                "memory bytes",
            )?;
            let mut metadata: Vec<SparseMetadata> = Vec::new();
            metadata
                .try_reserve_exact(count)
                .map_err(|_| SparseChainError::Allocation)?;
            for (index, extent) in descriptor.extents().iter().enumerate() {
                let mut admission = limits.metadata;
                admission.memory_bytes = admission
                    .memory_bytes
                    .min(limits.memory_bytes - chain.reserved_memory);
                admission.read_bytes = admission
                    .read_bytes
                    .min(limits.read_bytes - chain.read_bytes);
                if admission.read_bytes < 512 {
                    return Err(SparseChainError::Limit("read bytes"));
                }
                let crate::ExtentBacking::Sparse { file_name } = extent.backing() else {
                    unreachable!()
                };
                let backing = source.backings.resolve(file_name)?;
                let endpoint = backing.copy_endpoint()?;
                let backing_id = identity(endpoint)?;
                if chain.layers.iter().any(|l| l.contains(backing_id))
                    || metadata
                        .iter()
                        .any(|m| m.initial_endpoint().identity == Some(backing_id))
                {
                    return Err(SparseChainError::RepeatedIdentity);
                }
                if source.entry == ChainEntry::Embedded {
                    if backing_id != id {
                        return Err(SparseChainError::Identity);
                    }
                } else if backing_id == id {
                    return Err(SparseChainError::RepeatedIdentity);
                }
                // Resolve once; the metadata loader receives the same retained object.
                let pinned = Pinned(backing.clone());
                let m = SparseMetadata::load_layer(&descriptor, index, &pinned, admission)?;
                observe(backing.as_ref(), endpoint)?;
                if m.initial_endpoint().identity != Some(backing_id) {
                    return Err(SparseChainError::Identity);
                }
                chain.reserved_memory += m.reserved_memory_bytes();
                chain.read_bytes += m.metadata_read_bytes();
                metadata.push(m);
            }
            let parent = descriptor.parent();
            chain.layers.push(SparseChainLayer {
                descriptor: source.descriptor.clone(),
                initial,
                cid: descriptor.cid(),
                parent_cid: parent.map(|p| p.cid),
                size: descriptor.size_bytes(),
                metadata,
            });
            let Some(parent) = parent else {
                break;
            };
            if chain.layers.len() == limits.layers {
                return Err(SparseChainError::Limit("layers"));
            }
            source = parents
                .resolve_parent(id, parent.file_name_hint)?
                .ok_or(SparseChainError::MissingParent)?;
        }
        chain.revalidate()?;
        Ok(chain)
    }
    pub fn layers(&self) -> &[SparseChainLayer] {
        &self.layers
    }
    pub fn reserved_memory_bytes(&self) -> u64 {
        self.reserved_memory
    }
    pub fn metadata_read_bytes(&self) -> u64 {
        self.read_bytes
    }
    pub fn descriptor_bytes(&self) -> u64 {
        self.descriptor_bytes
    }
    /// Endpoint observations only; no content replay or snapshot guarantee.
    pub fn revalidate(&self) -> Result<()> {
        for layer in &self.layers {
            observe(layer.descriptor.as_ref(), layer.initial)?;
            for metadata in &layer.metadata {
                metadata.revalidate()?;
            }
        }
        Ok(())
    }
}
struct Pinned(Arc<dyn BlockDevice>);
impl BackingResolver for Pinned {
    fn resolve(&self, _: &str) -> std::result::Result<Arc<dyn BlockDevice>, BackingError> {
        Ok(self.0.clone())
    }
}
