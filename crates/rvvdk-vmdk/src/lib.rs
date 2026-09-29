//! Bounded VMDK metadata and read-only backing resolution.
//! Parsing alone opens no files; logical disk mapping is a separate layer.
//!
//! The supported subset and deliberately rejected syntax are documented in
//! `docs/vmdk-descriptor.md`. Parsed filenames remain untrusted resolver inputs.

mod descriptor;

pub use descriptor::{
    Access, CreateType, Descriptor, DescriptorError, ErrorKind, Extent, ExtentBacking, Limits,
    Metadata, SECTOR_BYTES,
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
