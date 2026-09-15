use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::RawDisk;
use rvvdk_datamover::{CopyOptions, DataMover};
use rvvdk_local::LocalFileBlockDevice;

const MIB: usize = 1024 * 1024;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-tail-{name}-{unique}.img"))
}

#[test]
fn direct_copy_handles_unaligned_disk_size() {
    let size = 8 * MIB + 123;

    let source_path = temporary_path("source");

    let destination_path = temporary_path("destination");

    let mut source_data = vec![0x5a_u8; size];

    source_data[size - 123..size].fill(0xa5);

    fs::write(&source_path, &source_data).unwrap();

    fs::write(&destination_path, vec![0_u8; size]).unwrap();

    let source = RawDisk::new(LocalFileBlockDevice::open_direct_read_only(&source_path).unwrap());

    let destination =
        RawDisk::new(LocalFileBlockDevice::open_direct_read_write(&destination_path).unwrap());

    let options = CopyOptions::with_execution(MIB, 4096, 2, 4).unwrap();

    DataMover::new(options).copy(&source, &destination).unwrap();

    let destination_data = fs::read(&destination_path).unwrap();

    assert_eq!(destination_data, source_data,);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn direct_copy_handles_multiple_unaligned_sizes() {
    for tail in [1, 17, 511, 512, 513, 2047, 2048, 2049, 4095] {
        let size = 2 * MIB + tail;

        let source_path = temporary_path(&format!("source-tail-{tail}"));

        let destination_path = temporary_path(&format!("destination-tail-{tail}"));

        let mut source_data = vec![0x5a_u8; size];

        source_data[size - tail..size].fill(0xa5);

        fs::write(&source_path, &source_data).unwrap();

        fs::write(&destination_path, vec![0_u8; size]).unwrap();

        let source =
            RawDisk::new(LocalFileBlockDevice::open_direct_read_only(&source_path).unwrap());

        let destination =
            RawDisk::new(LocalFileBlockDevice::open_direct_read_write(&destination_path).unwrap());

        let options = CopyOptions::with_execution(MIB, 4096, 2, 4).unwrap();

        DataMover::new(options).copy(&source, &destination).unwrap();

        let actual = fs::read(&destination_path).unwrap();

        assert_eq!(actual, source_data, "copy mismatch with tail size {tail}",);

        fs::remove_file(source_path).unwrap();

        fs::remove_file(destination_path).unwrap();
    }
}

#[test]
fn direct_copy_handles_disk_smaller_than_alignment() {
    let source_path = temporary_path("tiny-source");

    let destination_path = temporary_path("tiny-destination");

    let source_data = b"rvvdk-direct-tail";

    fs::write(&source_path, source_data).unwrap();

    fs::write(&destination_path, vec![0_u8; source_data.len()]).unwrap();

    let source = RawDisk::new(LocalFileBlockDevice::open_direct_read_only(&source_path).unwrap());

    let destination =
        RawDisk::new(LocalFileBlockDevice::open_direct_read_write(&destination_path).unwrap());

    let options = CopyOptions::with_execution(MIB, 4096, 2, 4).unwrap();

    DataMover::new(options).copy(&source, &destination).unwrap();

    let actual = fs::read(&destination_path).unwrap();

    assert_eq!(actual, source_data,);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}
