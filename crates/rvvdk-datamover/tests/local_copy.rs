use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::RawDisk;
use rvvdk_datamover::{CopyOptions, DataMover};
use rvvdk_local::LocalFileBlockDevice;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-{name}-{unique}.img"))
}

#[test]
fn copies_local_raw_disk() {
    let source_path = temporary_path("source");

    let destination_path = temporary_path("destination");

    let mut source_data = vec![0_u8; 4 * 1024 * 1024];

    source_data[1024..1029].copy_from_slice(b"rvvdk");

    source_data[2 * 1024 * 1024..2 * 1024 * 1024 + 9].copy_from_slice(b"datamover");

    fs::write(&source_path, &source_data).unwrap();

    fs::write(&destination_path, vec![0_u8; source_data.len()]).unwrap();

    let source_device = LocalFileBlockDevice::open_read_only(&source_path).unwrap();

    let destination_device = LocalFileBlockDevice::open_read_write(&destination_path).unwrap();

    let source = RawDisk::new(source_device);

    let destination = RawDisk::new(destination_device);

    let mover = DataMover::new(CopyOptions::default());

    let stats = mover.copy(&source, &destination).unwrap();

    assert_eq!(stats.bytes_written(), source_data.len() as u64,);

    let destination_data = fs::read(&destination_path).unwrap();

    assert_eq!(destination_data, source_data,);

    fs::remove_file(source_path).unwrap();
    fs::remove_file(destination_path).unwrap();
}
