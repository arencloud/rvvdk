//! Bounded VMDK metadata and read-only backing resolution.
//! Parsing alone opens no files; VmdkDisk maps retained sources into logical reads.
//!
//! The supported subset and deliberately rejected syntax are documented in
//! `docs/vmdk-descriptor.md`. Parsed filenames remain untrusted resolver inputs.

mod descriptor;

pub use descriptor::{
    Access, CreateType, Descriptor, DescriptorError, ErrorKind, Extent, ExtentBacking, Limits,
    Metadata, ParentReference, SECTOR_BYTES, SparseDescriptor, SparseLayerDescriptor,
    StreamDescriptor,
};

mod backing;
mod load;
#[cfg(target_os = "linux")]
mod local;

pub use backing::{
    BackingError, BackingResolver, ResolutionLimits, ResolvedBacking, ResolvedDescriptor,
    ResolvedExtent, ResolvedExtentBacking,
};
pub use load::DescriptorText;
#[cfg(target_os = "linux")]
pub use local::LocalResolver;

mod disk;
pub use disk::VmdkDisk;

mod sparse;
pub use sparse::{SPARSE_HEADER_BYTES, SparseError, SparseHeader, SparseLimits, SparseRegion};

mod sparse_metadata;
pub use sparse_metadata::{SparseMetadata, SparseMetadataError, SparseMetadataLimits};

mod sparse_disk;
pub use sparse_disk::{SparseDisk, SparseDiskLimits};

mod chain;
pub use chain::{
    ChainEntry, ParentResolver, SparseChain, SparseChainError, SparseChainLayer, SparseChainLimits,
    SparseChainSource,
};

mod chain_disk;
pub use chain_disk::{SparseChainDisk, SparseChainDiskLimits};

mod stream;
pub use stream::{
    StreamDirectory, StreamEnvelope, StreamError, StreamHeader, StreamLimits, StreamMarker,
    StreamRegion,
};
