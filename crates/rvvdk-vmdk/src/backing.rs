use crate::{Access, CreateType, Descriptor, DescriptorError, ExtentBacking};
use rvvdk_core::{BlockDevice, Capabilities, CopyEndpoint};
use std::{collections::HashMap, sync::Arc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BackingError {
    #[error("{0}")]
    Descriptor(#[from] DescriptorError),
    #[error("backing resource limit exceeded: {0}")]
    Limit(&'static str),
    #[error("unsafe or unsupported local reference")]
    UnsafeReference,
    #[error("local backing is not a regular file")]
    NotRegular,
    #[error("resolver anchor is not a directory")]
    NotDirectory,
    #[error("{operation}: {source}")]
    Io {
        operation: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("adopting local backing: {0}")]
    Adopt(#[source] rvvdk_core::Error),
    #[error("backing {index}: {source}")]
    Device {
        index: usize,
        #[source]
        source: rvvdk_core::Error,
    },
    #[error("resolving backing {index}: {source}")]
    Resolve {
        index: usize,
        #[source]
        source: Box<BackingError>,
    },
    #[error("backing {index} is not readable")]
    Unreadable { index: usize },
    #[error("backing {index} is truncated: need {required} bytes, have {actual}")]
    Truncated {
        index: usize,
        required: u64,
        actual: u64,
    },
    #[error("backing {index} identity changed or became unknown")]
    Changed { index: usize },
    #[error("opened local object does not match pinned identity")]
    LocalIdentityChanged,
}
impl BackingError {
    pub(crate) fn io(operation: &'static str, source: std::io::Error) -> Self {
        Self::Io { operation, source }
    }
}

/// Caller-provided namespace and authorization policy. Implementations return a
/// stable owned source; endpoint size/access must be refreshed by copy_endpoint.
/// References are opaque here. Local path policy belongs to LocalResolver.
pub trait BackingResolver: Send + Sync {
    fn resolve(&self, reference: &str) -> Result<Arc<dyn BlockDevice>, BackingError>;
}

#[derive(Clone, Copy, Debug)]
pub struct ResolutionLimits {
    pub extents: usize,
    pub backing_files: usize,
}
impl Default for ResolutionLimits {
    fn default() -> Self {
        Self {
            extents: 1024,
            backing_files: 128,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolvedExtentBacking {
    Flat {
        backing_index: usize,
        offset_bytes: u64,
    },
    Zero,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedExtent {
    access: Access,
    logical_offset: u64,
    size_bytes: u64,
    backing: ResolvedExtentBacking,
}
impl ResolvedExtent {
    pub fn access(&self) -> Access {
        self.access
    }
    pub fn logical_offset(&self) -> u64 {
        self.logical_offset
    }
    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }
    pub fn backing(&self) -> ResolvedExtentBacking {
        self.backing
    }
}

/// Owns the resolved object, exposing only reads and endpoint observations.
/// Validation is not a snapshot or a lease against other writers.
pub struct ResolvedBacking {
    device: Arc<dyn BlockDevice>,
    initial: CopyEndpoint,
    required_len: u64,
}
impl ResolvedBacking {
    pub fn initial_endpoint(&self) -> CopyEndpoint {
        self.initial
    }
    pub fn required_len(&self) -> u64 {
        self.required_len
    }
    pub fn copy_endpoint(&self) -> rvvdk_core::Result<CopyEndpoint> {
        self.device.copy_endpoint()
    }
    /// Physical I/O only. The future logical reader must translate its own ranges.
    pub fn read_exact_at(&self, offset: u64, buffer: &mut [u8]) -> rvvdk_core::Result<()> {
        self.device.read_exact_at(offset, buffer)
    }
}

/// Owned validated metadata and sources; input text and resolver may be dropped.
/// No logical disk read/write API or native RAW endpoint is implemented here.
pub struct ResolvedDescriptor {
    cid: u32,
    create_type: CreateType,
    size_bytes: u64,
    extents: Vec<ResolvedExtent>,
    backings: Vec<ResolvedBacking>,
}
impl ResolvedDescriptor {
    pub fn cid(&self) -> u32 {
        self.cid
    }
    pub fn create_type(&self) -> CreateType {
        self.create_type
    }
    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }
    pub fn extents(&self) -> &[ResolvedExtent] {
        &self.extents
    }
    pub fn backings(&self) -> &[ResolvedBacking] {
        &self.backings
    }

    pub fn resolve(
        descriptor: &Descriptor<'_>,
        resolver: &dyn BackingResolver,
        limits: ResolutionLimits,
    ) -> Result<Self, BackingError> {
        if descriptor.extents().len() > limits.extents {
            return Err(BackingError::Limit("resolved extents"));
        }
        let mut names = HashMap::new();
        let mut pending: Vec<(&str, u64)> = Vec::new();
        let mut extents = Vec::new();
        // Count and compute every physical requirement before invoking a resolver.
        for extent in descriptor.extents() {
            let backing = match extent.backing() {
                ExtentBacking::Zero => ResolvedExtentBacking::Zero,
                ExtentBacking::Flat {
                    file_name,
                    offset_bytes,
                } => {
                    // Parser already checked this sum; fields cannot be mutated.
                    let required = offset_bytes + extent.size_bytes();
                    let index = if let Some(&index) = names.get(file_name) {
                        index
                    } else {
                        if pending.len() >= limits.backing_files {
                            return Err(BackingError::Limit("backing files"));
                        }
                        let index = pending.len();
                        pending.push((file_name, required));
                        names.insert(file_name, index);
                        index
                    };
                    pending[index].1 = pending[index].1.max(required);
                    ResolvedExtentBacking::Flat {
                        backing_index: index,
                        offset_bytes,
                    }
                }
            };
            extents.push(ResolvedExtent {
                access: extent.access(),
                logical_offset: extent.logical_offset(),
                size_bytes: extent.size_bytes(),
                backing,
            });
        }
        let mut backings = Vec::new();
        for (index, (reference, required_len)) in pending.into_iter().enumerate() {
            let device = resolver
                .resolve(reference)
                .map_err(|source| BackingError::Resolve {
                    index,
                    source: Box::new(source),
                })?;
            let initial = device
                .copy_endpoint()
                .map_err(|source| BackingError::Device { index, source })?;
            validate(index, initial, required_len)?;
            backings.push(ResolvedBacking {
                device,
                initial,
                required_len,
            });
        }
        Ok(Self {
            cid: descriptor.cid(),
            create_type: descriptor.create_type(),
            size_bytes: descriptor.size_bytes(),
            extents,
            backings,
        })
    }

    /// Reinspect retained objects, never reopen names. Unknown initial identities
    /// stay explicitly unqualified; known identities must remain known and equal.
    pub fn revalidate(&self) -> Result<(), BackingError> {
        for (index, backing) in self.backings.iter().enumerate() {
            let current = backing
                .copy_endpoint()
                .map_err(|source| BackingError::Device { index, source })?;
            if backing.initial.identity.is_some() && current.identity != backing.initial.identity {
                return Err(BackingError::Changed { index });
            }
            validate(index, current, backing.required_len)?;
        }
        Ok(())
    }
}
fn validate(index: usize, endpoint: CopyEndpoint, required: u64) -> Result<(), BackingError> {
    if !endpoint.capabilities.contains(Capabilities::READ) {
        return Err(BackingError::Unreadable { index });
    }
    if endpoint.size < required {
        return Err(BackingError::Truncated {
            index,
            required,
            actual: endpoint.size,
        });
    }
    Ok(())
}
