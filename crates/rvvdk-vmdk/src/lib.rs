//! Bounded, read-only VMDK descriptor parsing. No backing files are opened.
//!
//! The supported subset and deliberately rejected syntax are documented in
//! `docs/vmdk-descriptor.md`. Parsed filenames remain untrusted resolver inputs.

mod descriptor;

pub use descriptor::{
    Access, CreateType, Descriptor, DescriptorError, ErrorKind, Extent, ExtentBacking, Limits,
    Metadata, SECTOR_BYTES,
};
