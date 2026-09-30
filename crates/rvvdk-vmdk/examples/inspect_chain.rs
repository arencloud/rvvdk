//! Local qualification helper, not public CLI parent support. All layers must
//! live in one explicitly selected directory; parent hints must be basenames.
#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 {
        return Err("usage: inspect_chain LEAF_DESCRIPTOR_OR_CONTAINER".into());
    }
    let (file, resolver) = LocalResolver::open_descriptor_file(Path::new(&args[0]))?;
    let policy = Policy(Arc::new(resolver));
    let chain = SparseChain::load(policy.source(file)?, &policy, SparseChainLimits::default())?;
    println!("{{\"layers\":[");
    for (i, l) in chain.layers().iter().enumerate() {
        if i != 0 {
            println!(",");
        }
        print!(
            "{{\"cid\":{},\"parent_cid\":{},\"capacity_bytes\":{},\"extents\":{}}}",
            l.cid(),
            l.parent_cid().map_or("null".into(), |v| v.to_string()),
            l.size_bytes(),
            l.metadata().len()
        );
    }
    println!(
        "],\"reserved_memory_bytes\":{},\"metadata_read_bytes\":{},\"descriptor_bytes\":{}}}",
        chain.reserved_memory_bytes(),
        chain.metadata_read_bytes(),
        chain.descriptor_bytes()
    );
    Ok(())
}
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("local chain qualification requires Linux");
    std::process::exit(1);
}
