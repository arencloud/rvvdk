#![cfg(target_os = "linux")]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_datamover::io_uring::evaluate_compatibility;

use rvvdk_local::LocalFileBlockDevice;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-io-uring-compat-{name}-{unique}.img"))
}

#[test]
fn evaluates_buffered_pair() {
    let source_path = temporary_path("source");

    let destination_path = temporary_path("destination");

    fs::write(&source_path, vec![0_u8; 4096]).unwrap();

    fs::write(&destination_path, vec![0_u8; 4096]).unwrap();

    let source = LocalFileBlockDevice::open_read_only(&source_path).unwrap();

    let destination = LocalFileBlockDevice::open_read_write(&destination_path).unwrap();

    let compatibility = evaluate_compatibility(&source, &destination);

    assert!(compatibility.compatible());

    assert!(!compatibility.direct_io());

    assert_eq!(compatibility.alignment(), 1,);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn evaluates_direct_pair() {
    let source_path = temporary_path("direct-source");

    let destination_path = temporary_path("direct-destination");

    fs::write(&source_path, vec![0_u8; 4096]).unwrap();

    fs::write(&destination_path, vec![0_u8; 4096]).unwrap();

    let source = LocalFileBlockDevice::open_direct_read_only(&source_path).unwrap();

    let destination = LocalFileBlockDevice::open_direct_read_write(&destination_path).unwrap();

    let compatibility = evaluate_compatibility(&source, &destination);

    assert!(compatibility.compatible());

    assert!(compatibility.direct_io());

    assert!(compatibility.alignment() > 0);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn mixed_pair_is_not_fully_direct() {
    let source_path = temporary_path("mixed-source");

    let destination_path = temporary_path("mixed-destination");

    fs::write(&source_path, vec![0_u8; 4096]).unwrap();

    fs::write(&destination_path, vec![0_u8; 4096]).unwrap();

    let source = LocalFileBlockDevice::open_direct_read_only(&source_path).unwrap();

    let destination = LocalFileBlockDevice::open_read_write(&destination_path).unwrap();

    let compatibility = evaluate_compatibility(&source, &destination);

    assert!(compatibility.compatible());

    assert!(!compatibility.direct_io());

    assert!(compatibility.alignment() > 0);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}
