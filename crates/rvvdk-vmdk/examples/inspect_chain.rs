//! Local qualification helper, not public CLI parent support. All layers must
//! live in one explicitly selected directory; parent hints must be basenames.
#[path = "support/chain.rs"]
mod support;
#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 1 {
        return Err("usage: inspect_chain LEAF_DESCRIPTOR_OR_CONTAINER".into());
    }
    let chain = support::load(&args[0])?;
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
