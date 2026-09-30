//! Local fixture qualification through SparseDisk; not public CLI publication.
#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rvvdk_core::{EndpointIdentity, VirtualDisk};
    use rvvdk_vmdk::{
        Limits, LocalResolver, SparseDescriptor, SparseDisk, SparseDiskLimits, SparseHeader,
        SparseLimits,
    };
    use std::{
        fs::File,
        io::{Read, Seek, SeekFrom, Write},
        os::unix::fs::MetadataExt,
        path::Path,
    };
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 3 {
        return Err("usage: dump_sparse DESCRIPTOR --embedded|--external NEW_RAW_OUTPUT".into());
    }
    let path = Path::new(&args[0]);
    let mut source = File::open(path)?;
    let initial = source.metadata()?;
    if !initial.is_file() {
        return Err("expected regular source".into());
    }
    let embedded = match args[1].to_str() {
        Some("--embedded") => true,
        Some("--external") => false,
        _ => return Err("invalid descriptor mode".into()),
    };
    let mut text = Vec::new();
    if embedded {
        let h = SparseHeader::read_from(&mut source, initial.len(), SparseLimits::default())?;
        let r = h.descriptor().ok_or("missing embedded descriptor")?;
        source.seek(SeekFrom::Start(r.offset()))?;
        Read::by_ref(&mut source)
            .take(r.length())
            .read_to_end(&mut text)?;
    } else {
        Read::by_ref(&mut source)
            .take(Limits::default().descriptor_bytes as u64 + 1)
            .read_to_end(&mut text)?;
    }
    let descriptor = SparseDescriptor::parse(&text)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let resolver = LocalResolver::from_directory(File::open(parent)?)?;
    let disk = SparseDisk::load(&descriptor, &resolver, SparseDiskLimits::default())?;
    if embedded
        && disk.metadata()[0].initial_endpoint().identity
            != Some(EndpointIdentity::LocalFile {
                device: initial.dev(),
                inode: initial.ino(),
            })
    {
        return Err("embedded descriptor resolved another file".into());
    }
    disk.revalidate()?;
    let mut output = File::options()
        .write(true)
        .create_new(true)
        .open(&args[2])?;
    output.set_len(disk.size())?;
    let mut buffer = vec![0; 65537];
    let mut offset = 0;
    while offset < disk.size() {
        let n = (disk.size() - offset).min(buffer.len() as u64) as usize;
        disk.read_exact_at(offset, &mut buffer[..n])?;
        // Every logical byte is read. Keep known-zero output chunks sparse.
        if buffer[..n].iter().any(|&b| b != 0) {
            output.seek(SeekFrom::Start(offset))?;
            output.write_all(&buffer[..n])?;
        }
        offset += n as u64;
    }
    disk.revalidate()?;
    output.sync_all()?;
    let extents = disk.extents(0, disk.size())?;
    println!(
        "{{\"size_bytes\":{},\"backings\":{},\"logical_extents\":{},\"reserved_memory_bytes\":{},\"metadata_read_bytes\":{}}}",
        disk.size(),
        disk.metadata().len(),
        extents.len(),
        disk.reserved_memory_bytes(),
        disk.metadata_read_bytes()
    );
    Ok(())
}
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("local fixture qualification requires Linux");
    std::process::exit(1);
}
