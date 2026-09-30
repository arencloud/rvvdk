//! Fixture helper; not a production sparse disk reader.
#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rvvdk_core::EndpointIdentity;
    use rvvdk_vmdk::{
        Limits, LocalResolver, SparseDescriptor, SparseHeader, SparseLimits, SparseMetadata,
        SparseMetadataLimits,
    };
    use std::{
        fs::File,
        io::{Read, Seek, SeekFrom},
        os::unix::fs::MetadataExt,
        path::Path,
    };
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: inspect_sparse_metadata DESCRIPTOR INDEX|--embedded".into());
    }
    let path = Path::new(&args[0]);
    let mut file = File::open(path)?;
    let initial = file.metadata()?;
    if !initial.is_file() {
        return Err("expected regular source".into());
    }
    let embedded = args[1] == "--embedded";
    let mut text = Vec::new();
    let index = if embedded {
        let header = SparseHeader::read_from(&mut file, initial.len(), SparseLimits::default())?;
        let region = header.descriptor().ok_or("missing embedded descriptor")?;
        file.seek(SeekFrom::Start(region.offset()))?;
        file.by_ref().take(region.length()).read_to_end(&mut text)?;
        0
    } else {
        file.by_ref()
            .take(Limits::default().descriptor_bytes as u64 + 1)
            .read_to_end(&mut text)?;
        args[1].to_str().ok_or("invalid index")?.parse::<usize>()?
    };
    let descriptor = SparseDescriptor::parse(&text)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let resolver = LocalResolver::from_directory(File::open(parent)?)?;
    let m = SparseMetadata::load(
        &descriptor,
        index,
        &resolver,
        SparseMetadataLimits::default(),
    )?;
    if embedded
        && m.initial_endpoint().identity
            != Some(EndpointIdentity::LocalFile {
                device: initial.dev(),
                inode: initial.ino(),
            })
    {
        return Err("embedded descriptor resolved a different file".into());
    }
    println!(
        "{{\"capacity_bytes\":{},\"grain_bytes\":{},\"extent_index\":{},\"logical_offset\":{},\"reserved_memory_bytes\":{},\"metadata_read_bytes\":{},\"grain_sectors\":{:?}}}",
        m.header().capacity_bytes(),
        m.header().grain_bytes(),
        m.extent_index(),
        m.logical_offset(),
        m.reserved_memory_bytes(),
        m.metadata_read_bytes(),
        m.grain_sectors()
    );
    Ok(())
}
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("local fixture qualification requires Linux");
    std::process::exit(1);
}
