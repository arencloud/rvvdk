//! Metadata index qualification only. Never decompresses or returns guest bytes.
use rvvdk_vmdk::{StreamDirectory, StreamMap, StreamMapLimits};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args
        .next()
        .ok_or("usage: inspect_stream_map EXTENT [--records]")?;
    let records = match args.next() {
        None => false,
        Some(flag) if flag == "--records" => true,
        _ => return Err("expected --records or no option".into()),
    };
    if args.next().is_some() {
        return Err("too many arguments".into());
    }
    let file = std::fs::File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err("expected a regular file".into());
    }
    let map = StreamMap::read_from(file, metadata.len(), StreamMapLimits::default())?;
    let profile = if map.header().directory() == StreamDirectory::Footer {
        "footer"
    } else {
        "front"
    };
    let s = map.stats();
    print!(
        "{{\"scope\":\"grain_index_only\",\"profile\":\"{profile}\",\"capacity_bytes\":{},\"extent_bytes\":{},\"allocated_grains\":{},\"metadata_bytes_read\":{},\"table_entries_examined\":{},\"reserved_bytes\":{},\"payload_validated\":false,\"grain_map_validated\":true",
        map.header().capacity_bytes(),
        metadata.len(),
        s.allocated_grains,
        s.metadata_bytes_read,
        s.table_entries_examined,
        s.reserved_bytes
    );
    if records {
        print!(",\"records\":[");
        for (i, g) in map.grains().iter().enumerate() {
            if i > 0 {
                print!(",");
            }
            print!(
                "[{},{},{}]",
                g.index(),
                g.marker_offset() / 512,
                g.payload().length
            );
        }
        print!("]");
    }
    println!("}}");
    Ok(())
}
