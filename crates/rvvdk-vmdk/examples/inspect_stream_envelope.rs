//! Metadata-only qualification helper. No decompression, map validation or public CLI admission.
use rvvdk_vmdk::{StreamDirectory, StreamEnvelope, StreamLimits};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("usage: inspect_stream_envelope EXTENT")?;
    if args.next().is_some() {
        return Err("expected one extent".into());
    }
    let file = std::fs::File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Err("expected a regular file".into());
    }
    let e = StreamEnvelope::read_from(file, metadata.len(), StreamLimits::default())?;
    let profile = if e.header.directory() == StreamDirectory::Footer {
        "footer"
    } else {
        "front"
    };
    println!(
        "{{\"scope\":\"metadata_envelope_only\",\"profile\":\"{profile}\",\"capacity_bytes\":{},\"extent_bytes\":{},\"grain_bytes\":{},\"descriptor_bytes\":{},\"directory_bytes\":{},\"metadata_bytes_read\":{},\"payload_validated\":false,\"grain_map_validated\":false}}",
        e.header.capacity_bytes(),
        e.header.extent_bytes(),
        e.header.grain_bytes(),
        e.header.descriptor().length,
        e.primary_directory.length,
        e.metadata_bytes_read
    );
    Ok(())
}
