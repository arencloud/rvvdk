//! Owned logical source and descriptor-bound observations. No backing pathname
//! is reopened for metadata or identity checks.
use crate::{error::Failure, target};
use rvvdk_core::{BlockDevice, RawDisk, VirtualDisk};
use rvvdk_local::LocalFileBlockDevice;
use rvvdk_vmdk::{
    BackingError, BackingResolver, CreateType, Descriptor, Limits, LocalResolver, ResolutionLimits,
    ResolvedDescriptor, SparseDescriptor, SparseDisk, SparseDiskLimits, SparseHeader, SparseLimits,
    VmdkDisk,
};
use std::{
    fs::{File, Metadata},
    io::Read,
    os::unix::fs::{FileExt, MetadataExt},
    path::Path,
    sync::{Arc, Mutex},
};

mod chain;

type Result<T> = std::result::Result<T, Failure>;
type Stamp = (u64, u64, u64, i64, i64, i64, i64);
pub(crate) fn stamp(file: &File) -> std::io::Result<Stamp> {
    let m = file.metadata()?;
    Ok((
        m.dev(),
        m.ino(),
        m.len(),
        m.mtime(),
        m.mtime_nsec(),
        m.ctime(),
        m.ctime_nsec(),
    ))
}
struct Observed {
    file: File,
    initial: Stamp,
}
impl Observed {
    fn new(file: File) -> std::io::Result<Self> {
        let initial = stamp(&file)?;
        Ok(Self { file, initial })
    }
}
pub(crate) enum Disk {
    Raw(RawDisk<LocalFileBlockDevice>),
    Vmdk(VmdkDisk),
    Sparse(SparseDisk),
    Chain(rvvdk_vmdk::SparseChainDisk),
}
pub(crate) struct Source {
    pub disk: Disk,
    files: Vec<Observed>,
}
struct ObservingResolver {
    local: LocalResolver,
    embedded_identity: Option<(u64, u64)>,
    files: Mutex<Vec<Observed>>,
}
fn observation_error(source: std::io::Error) -> BackingError {
    BackingError::Io {
        operation: "retain source observation",
        source,
    }
}
impl BackingResolver for ObservingResolver {
    fn resolve(&self, reference: &str) -> std::result::Result<Arc<dyn BlockDevice>, BackingError> {
        let file = self.local.open_regular(reference)?;
        let observed = Observed::new(file).map_err(observation_error)?;
        if self
            .embedded_identity
            .is_some_and(|identity| identity != (observed.initial.0, observed.initial.1))
        {
            return Err(BackingError::LocalIdentityChanged);
        }
        let device = LocalFileBlockDevice::from_buffered_file(
            observed.file.try_clone().map_err(observation_error)?,
        )
        .map_err(BackingError::Adopt)?;
        self.files.lock().unwrap().push(observed);
        Ok(Arc::new(device))
    }
}
impl Source {
    pub fn open(path: &Path, format: &str, allow_parents: bool) -> Result<Self> {
        if allow_parents {
            if format != "vmdk" {
                return Err(Failure::new(
                    "arguments",
                    "--allow-parents requires --format vmdk",
                ));
            }
            return chain::open(path);
        }
        let source = if format == "raw" {
            let file = Observed::new(target::source(path)?)
                .map_err(|e| Failure::io("inspect source", e))?;
            let disk = RawDisk::new(LocalFileBlockDevice::from_buffered_file(
                file.file
                    .try_clone()
                    .map_err(|e| Failure::io("retain source", e))?,
            )?);
            Self {
                disk: Disk::Raw(disk),
                files: vec![file],
            }
        } else {
            let (file, local) = LocalResolver::open_descriptor_file(path)?;
            let mut descriptor =
                Observed::new(file).map_err(|e| Failure::io("inspect descriptor", e))?;
            let (bytes, embedded) = acquire(&mut descriptor)?;
            let resolver = ObservingResolver {
                local,
                embedded_identity: embedded.then_some((descriptor.initial.0, descriptor.initial.1)),
                files: Mutex::new(vec![descriptor]),
            };
            let end = bytes.iter().rposition(|&b| b != 0).map_or(0, |i| i + 1);
            let disk = if !embedded {
                match Descriptor::parse(&bytes[..end]) {
                    Ok(parsed) => Disk::Vmdk(VmdkDisk::new(ResolvedDescriptor::resolve(
                        &parsed,
                        &resolver,
                        ResolutionLimits::default(),
                    )?)?),
                    Err(flat_error) => {
                        let parsed = SparseDescriptor::parse(&bytes)
                            .map_err(|_| Failure::from(flat_error))?;
                        if parsed.create_type() != CreateType::TwoGbMaxExtentSparse {
                            return Err(Failure::new(
                                "vmdk",
                                "monolithic sparse input must be its container file",
                            ));
                        }
                        Disk::Sparse(SparseDisk::load(
                            &parsed,
                            &resolver,
                            SparseDiskLimits::default(),
                        )?)
                    }
                }
            } else {
                let parsed = SparseDescriptor::parse(&bytes)?;
                if parsed.create_type() != CreateType::MonolithicSparse {
                    return Err(Failure::new(
                        "vmdk",
                        "embedded input requires a monolithic sparse descriptor",
                    ));
                }
                Disk::Sparse(SparseDisk::load(
                    &parsed,
                    &resolver,
                    SparseDiskLimits::default(),
                )?)
            };
            Self {
                disk,
                files: resolver.files.into_inner().unwrap(),
            }
        };
        source.revalidate()?;
        Ok(source)
    }
    pub fn logical(&self) -> &dyn VirtualDisk {
        match &self.disk {
            Disk::Raw(d) => d,
            Disk::Vmdk(d) => d,
            Disk::Sparse(d) => d,
            Disk::Chain(d) => d,
        }
    }
    pub fn format(&self) -> &'static str {
        match self.disk {
            Disk::Raw(_) => "raw",
            Disk::Vmdk(_) | Disk::Sparse(_) | Disk::Chain(_) => "vmdk",
        }
    }
    pub fn validate_backend(&self, requested: &str) -> Result<()> {
        if matches!(self.disk, Disk::Vmdk(_) | Disk::Sparse(_) | Disk::Chain(_))
            && requested == "io-uring"
        {
            return Err(Failure::new(
                "unsupported_backend",
                "VMDK logical sources require threaded or auto execution",
            ));
        }
        Ok(())
    }
    pub fn revalidate(&self) -> Result<()> {
        for file in &self.files {
            if stamp(&file.file).map_err(|e| Failure::io("recheck source", e))? != file.initial {
                return Err(Failure::new(
                    "source_changed",
                    "opened source metadata changed during operation",
                ));
            }
        }
        Ok(())
    }
    pub fn validate_destination(&self, metadata: &Metadata) -> Result<()> {
        if self
            .files
            .iter()
            .any(|f| (f.initial.0, f.initial.1) == (metadata.dev(), metadata.ino()))
        {
            return Err(Failure::new(
                "same_file",
                "destination aliases the source descriptor or a backing file",
            ));
        }
        Ok(())
    }
    pub fn identity(&self) -> (u64, u64) {
        (self.files[0].initial.0, self.files[0].initial.1)
    }
    pub fn vmdk_report(&self) -> Option<serde_json::Value> {
        if let Disk::Chain(disk) = &self.disk {
            let chain = disk.chain();
            let layers = chain.layers();
            let backings = layers.iter().flat_map(|l| l.metadata()).collect::<Vec<_>>();
            return Some(serde_json::json!({
                "descriptor_identity":{"device":self.identity().0,"inode":self.identity().1},
                "layout":"hosted_sparse_chain", "parent_policy":"same_directory_basename",
                "layer_count":layers.len(), "cid":format!("{:08x}",layers[0].cid()),
                "backing_file_count":backings.len(), "descriptor_extent_count":backings.len(),
                "metadata_memory_reservation_bytes":chain.reserved_memory_bytes(),
                "metadata_read_bytes":chain.metadata_read_bytes(),
                "chain_descriptor_bytes":chain.descriptor_bytes(), "entry_probe_bytes":4*layers.len(),
                "layers":layers.iter().map(|l| serde_json::json!({
                    "cid":format!("{:08x}",l.cid()), "parent_cid":l.parent_cid().map(|v|format!("{v:08x}")),
                    "descriptor_identity":chain::identity_json(l.descriptor_endpoint().identity),
                    "descriptor_extent_count":l.metadata().len()
                })).collect::<Vec<_>>(),
                "backing_identities":backings.iter().map(|m|chain::identity_json(m.initial_endpoint().identity)).collect::<Vec<_>>()
            }));
        }
        if let Disk::Sparse(disk) = &self.disk {
            return Some(serde_json::json!({
                "descriptor_identity": {"device":self.identity().0,"inode":self.identity().1},
                "layout":"hosted_sparse",
                "backing_file_count":disk.metadata().len(),
                "descriptor_extent_count":disk.metadata().len(),
                "cid":format!("{:08x}",disk.cid()),
                "metadata_memory_reservation_bytes":disk.reserved_memory_bytes(),
                "metadata_read_bytes":disk.metadata_read_bytes(),
                "backing_identities":self.files.iter().skip(1).map(|f| serde_json::json!({"device":f.initial.0,"inode":f.initial.1})).collect::<Vec<_>>()
            }));
        }
        let Disk::Vmdk(disk) = &self.disk else {
            return None;
        };
        Some(serde_json::json!({
            "descriptor_identity": {"device":self.identity().0,"inode":self.identity().1},
            "backing_file_count":disk.resolved().backings().len(),
            "descriptor_extent_count":disk.resolved().extents().len(),
            "cid":format!("{:08x}", disk.resolved().cid()),
            "backing_identities":self.files.iter().skip(1).map(|f| serde_json::json!({"device":f.initial.0,"inode":f.initial.1})).collect::<Vec<_>>()
        }))
    }
}
impl From<BackingError> for Failure {
    fn from(error: BackingError) -> Self {
        Self::new("vmdk", error.to_string())
    }
}
impl From<rvvdk_vmdk::DescriptorError> for Failure {
    fn from(error: rvvdk_vmdk::DescriptorError) -> Self {
        Self::new("vmdk", error.to_string())
    }
}

impl From<rvvdk_vmdk::SparseMetadataError> for Failure {
    fn from(error: rvvdk_vmdk::SparseMetadataError) -> Self {
        Self::new("vmdk", error.to_string())
    }
}
impl From<rvvdk_vmdk::SparseError> for Failure {
    fn from(error: rvvdk_vmdk::SparseError) -> Self {
        Self::new("vmdk", error.to_string())
    }
}
/// The first bounded chunk is also descriptor input, so text sources need no
/// extra probe syscall. Format dispatch stays inside explicit --format vmdk.
fn acquire(source: &mut Observed) -> Result<(Vec<u8>, bool)> {
    let mut chunk = [0; 4096];
    let mut n = 0;
    while n < 4 {
        match source.file.read(&mut chunk[n..]) {
            Ok(0) => break,
            Ok(count) => n += count,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(observation_error(e).into()),
        }
    }
    if chunk[..n].starts_with(b"KDMV") {
        let mut raw = [0; 512];
        source
            .file
            .read_exact_at(&mut raw, 0)
            .map_err(observation_error)?;
        let header =
            SparseHeader::parse_with_limits(&raw, source.initial.2, SparseLimits::default())?;
        let region = header
            .descriptor()
            .ok_or_else(|| Failure::new("vmdk", "missing embedded descriptor"))?;
        let mut text = vec![0; region.length() as usize]; // admitted by the 1 MiB header limit
        source
            .file
            .read_exact_at(&mut text, region.offset())
            .map_err(observation_error)?;
        return Ok((text, true));
    }
    let limit = Limits::default().descriptor_bytes;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&chunk[..n]);
    loop {
        let remaining = limit - bytes.len();
        let count = if remaining == 0 {
            1
        } else {
            remaining.min(chunk.len())
        };
        let n = match source.file.read(&mut chunk[..count]) {
            Ok(n) => n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(observation_error(e).into()),
        };
        if n == 0 {
            break;
        }
        if remaining == 0 {
            return Err(BackingError::Limit("descriptor acquisition bytes").into());
        }
        bytes.extend_from_slice(&chunk[..n]);
    }
    Ok((bytes, false))
}

impl From<rvvdk_vmdk::SparseChainError> for Failure {
    fn from(error: rvvdk_vmdk::SparseChainError) -> Self {
        Self::new("vmdk", error.to_string())
    }
}
