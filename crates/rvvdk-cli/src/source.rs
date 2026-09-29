//! Owned logical source and descriptor-bound observations. No backing pathname
//! is reopened for metadata or identity checks.
use crate::{error::Failure, target};
use rvvdk_core::{BlockDevice, RawDisk, VirtualDisk};
use rvvdk_local::LocalFileBlockDevice;
use rvvdk_vmdk::{
    BackingError, BackingResolver, DescriptorText, Limits, LocalResolver, ResolutionLimits,
    ResolvedDescriptor, VmdkDisk,
};
use std::{
    fs::{File, Metadata},
    os::unix::fs::MetadataExt,
    path::Path,
    sync::{Arc, Mutex},
};

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
}
pub(crate) struct Source {
    pub disk: Disk,
    files: Vec<Observed>,
}
struct ObservingResolver {
    local: LocalResolver,
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
        let device = LocalFileBlockDevice::from_buffered_file(
            observed.file.try_clone().map_err(observation_error)?,
        )
        .map_err(BackingError::Adopt)?;
        self.files.lock().unwrap().push(observed);
        Ok(Arc::new(device))
    }
}
impl Source {
    pub fn open(path: &Path, format: &str) -> Result<Self> {
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
            let text = DescriptorText::read_from(&mut descriptor.file, Limits::default())?;
            let resolver = ObservingResolver {
                local,
                files: Mutex::new(vec![descriptor]),
            };
            let resolved = ResolvedDescriptor::resolve(
                &text.parse()?,
                &resolver,
                ResolutionLimits::default(),
            )?;
            let disk = VmdkDisk::new(resolved)?;
            Self {
                disk: Disk::Vmdk(disk),
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
        }
    }
    pub fn format(&self) -> &'static str {
        match self.disk {
            Disk::Raw(_) => "raw",
            Disk::Vmdk(_) => "vmdk",
        }
    }
    pub fn validate_backend(&self, requested: &str) -> Result<()> {
        if matches!(self.disk, Disk::Vmdk(_)) && requested == "io-uring" {
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
