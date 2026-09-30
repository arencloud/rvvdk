//! Header qualification helper, not sparse disk reads or the public CLI.
use rvvdk_vmdk::{SparseHeader, SparseLimits, SparseRegion};
fn region(value: Option<SparseRegion>) -> String {
    value.map_or_else(
        || "null".to_owned(),
        |r| format!("{{\"offset\":{},\"length\":{}}}", r.offset(), r.length()),
    )
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 {
        return Err("usage: inspect_sparse_header EXTENT".into());
    }
    let mut file = std::fs::File::open(&args[0])?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err("expected a regular extent file".into());
    }
    let h = SparseHeader::read_from(&mut file, metadata.len(), SparseLimits::default())?;
    println!(
        "{{\"flags\":{},\"capacity_bytes\":{},\"grain_bytes\":{},\"directory_entries\":{},\"grain_table_bytes\":{},\"overhead_bytes\":{},\"minimum_metadata_bytes\":{},\"descriptor\":{},\"primary_directory\":{},\"redundant_directory\":{}}}",
        h.flags(),
        h.capacity_bytes(),
        h.grain_bytes(),
        h.directory_entries(),
        h.grain_table_bytes(),
        h.overhead_bytes(),
        h.minimum_metadata_bytes(),
        region(h.descriptor()),
        region(Some(h.primary_directory())),
        region(h.redundant_directory())
    );
    Ok(())
}
