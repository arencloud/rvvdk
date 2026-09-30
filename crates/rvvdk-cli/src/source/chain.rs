//! Explicit CLI policy: every parent descriptor is a basename in one pinned
//! directory; each backing resolves relative to that same descriptor directory.
use super::*;
use rvvdk_core::EndpointIdentity;
use rvvdk_vmdk::{
    ChainEntry, ParentResolver, SparseChain, SparseChainDisk, SparseChainLimits, SparseChainSource,
};
struct Policy {
    local: Arc<LocalResolver>,
    files: Arc<Mutex<Vec<Observed>>>,
}
struct Backings {
    local: Arc<LocalResolver>,
    files: Arc<Mutex<Vec<Observed>>>,
}
fn retain(
    file: File,
    files: &Mutex<Vec<Observed>>,
) -> std::result::Result<Arc<LocalFileBlockDevice>, BackingError> {
    let observed = Observed::new(file).map_err(observation_error)?;
    let device = LocalFileBlockDevice::from_buffered_file(
        observed.file.try_clone().map_err(observation_error)?,
    )
    .map_err(BackingError::Adopt)?;
    files.lock().unwrap().push(observed);
    Ok(Arc::new(device))
}
impl BackingResolver for Backings {
    fn resolve(&self, reference: &str) -> std::result::Result<Arc<dyn BlockDevice>, BackingError> {
        Ok(retain(self.local.open_regular(reference)?, &self.files)?)
    }
}
impl Policy {
    fn source(&self, file: File) -> std::result::Result<SparseChainSource, BackingError> {
        let descriptor = retain(file, &self.files)?;
        let mut magic = [0; 4];
        descriptor
            .read_exact_at(0, &mut magic)
            .map_err(BackingError::Adopt)?;
        Ok(SparseChainSource {
            descriptor,
            backings: Arc::new(Backings {
                local: self.local.clone(),
                files: self.files.clone(),
            }),
            entry: if magic == *b"KDMV" {
                ChainEntry::Embedded
            } else {
                ChainEntry::External
            },
        })
    }
}
impl ParentResolver for Policy {
    fn resolve_parent(
        &self,
        _: EndpointIdentity,
        hint: &str,
    ) -> std::result::Result<Option<SparseChainSource>, BackingError> {
        // The library calls only for admitted children. All layers live in the
        // same anchor, so parent lookup never depends on the current directory.
        if hint.contains('/') {
            return Err(BackingError::UnsafeReference);
        }
        let file = match self.local.open_regular(hint) {
            Ok(file) => file,
            Err(BackingError::Io { source, .. })
                if source.kind() == std::io::ErrorKind::NotFound =>
            {
                return Ok(None);
            }
            Err(e) => return Err(e),
        };
        self.source(file).map(Some)
    }
}
pub(super) fn open(path: &Path) -> Result<Source> {
    let (file, local) = LocalResolver::open_descriptor_file(path)?;
    let files = Arc::new(Mutex::new(Vec::new()));
    let policy = Policy {
        local: Arc::new(local),
        files: files.clone(),
    };
    let chain = SparseChain::load(policy.source(file)?, &policy, SparseChainLimits::default())?;
    drop(policy);
    let source = Source {
        disk: Disk::Chain(SparseChainDisk::new(chain)),
        files: std::mem::take(&mut *files.lock().unwrap()),
    };
    source.revalidate()?;
    Ok(source)
}
pub(super) fn identity_json(identity: Option<EndpointIdentity>) -> serde_json::Value {
    match identity {
        Some(EndpointIdentity::LocalFile { device, inode }) => {
            serde_json::json!({"device":device,"inode":inode})
        }
        _ => unreachable!("CLI chain sources are identified local regular files"),
    }
}
