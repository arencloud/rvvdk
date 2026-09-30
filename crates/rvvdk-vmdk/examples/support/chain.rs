//! Confined, same-directory fixture acquisition; not public CLI policy.
#[cfg(target_os = "linux")]
pub fn load(
    path: impl AsRef<std::path::Path>,
) -> Result<rvvdk_vmdk::SparseChain, Box<dyn std::error::Error>> {
    use rvvdk_core::{BlockDevice, EndpointIdentity};
    use rvvdk_local::LocalFileBlockDevice;
    use rvvdk_vmdk::{
        BackingError, ChainEntry, LocalResolver, ParentResolver, SparseChain, SparseChainLimits,
        SparseChainSource,
    };
    use std::{fs::File, path::Path, sync::Arc};
    struct Policy(Arc<LocalResolver>);
    impl Policy {
        fn source(&self, file: File) -> Result<SparseChainSource, BackingError> {
            let descriptor = Arc::new(
                LocalFileBlockDevice::from_buffered_file(file).map_err(BackingError::Adopt)?,
            );
            let mut magic = [0; 4];
            descriptor
                .read_exact_at(0, &mut magic)
                .map_err(BackingError::Adopt)?;
            Ok(SparseChainSource {
                descriptor,
                backings: self.0.clone(),
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
        ) -> Result<Option<SparseChainSource>, BackingError> {
            if hint.contains('/') {
                return Err(BackingError::UnsafeReference);
            }
            let file = match self.0.open_regular(hint) {
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
    let (file, resolver) = LocalResolver::open_descriptor_file(Path::new(path.as_ref()))?;
    let policy = Policy(Arc::new(resolver));
    Ok(SparseChain::load(
        policy.source(file)?,
        &policy,
        SparseChainLimits::default(),
    )?)
}
