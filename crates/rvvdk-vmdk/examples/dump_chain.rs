//! Full logical-byte reference helper; public CLI parent support is separate.
#[path = "support/chain.rs"]
mod support;
#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use rvvdk_core::VirtualDisk;
    use rvvdk_vmdk::SparseChainDisk;
    use std::io::{Seek, SeekFrom, Write};
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: dump_chain LEAF_DESCRIPTOR_OR_CONTAINER NEW_RAW_OUTPUT".into());
    }
    let disk = SparseChainDisk::new(support::load(&args[0])?);
    disk.revalidate()?;
    let mut output = std::fs::File::options()
        .write(true)
        .create_new(true)
        .open(&args[1])?;
    output.set_len(disk.size())?;
    let mut bytes = vec![0; 65537];
    let mut offset = 0;
    while offset < disk.size() {
        let n = (disk.size() - offset).min(bytes.len() as u64) as usize;
        disk.read_exact_at(offset, &mut bytes[..n])?;
        if bytes[..n].iter().any(|&b| b != 0) {
            output.seek(SeekFrom::Start(offset))?;
            output.write_all(&bytes[..n])?;
        }
        offset += n as u64;
    }
    disk.revalidate()?;
    output.sync_all()?;
    println!(
        "{{\"size_bytes\":{},\"layers\":{},\"logical_extents\":{}}}",
        disk.size(),
        disk.chain().layers().len(),
        disk.extents(0, disk.size())?.len()
    );
    Ok(())
}
#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("local chain qualification requires Linux");
    std::process::exit(1);
}
