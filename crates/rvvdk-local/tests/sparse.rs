use std::fs::{self, File};
use std::io::{Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::{BlockDevice, ExtentKind};
use rvvdk_local::LocalFileBlockDevice;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-{name}-{unique}.img"))
}

#[test]
fn discovers_sparse_file_extents() {
    let path = temporary_path("sparse");

    let mut file = File::create(&path).unwrap();

    file.set_len(16 * 1024 * 1024).unwrap();

    file.seek(SeekFrom::Start(1024 * 1024)).unwrap();

    file.write_all(&vec![0xaa; 4096]).unwrap();

    file.seek(SeekFrom::Start(8 * 1024 * 1024)).unwrap();

    file.write_all(&vec![0xbb; 4096]).unwrap();

    file.sync_all().unwrap();

    drop(file);

    let device = LocalFileBlockDevice::open_read_only(&path).unwrap();

    let extents = device.extents(0, device.size()).unwrap();

    assert!(
        extents
            .iter()
            .any(|extent| { extent.kind() == ExtentKind::Data })
    );

    assert!(
        extents
            .iter()
            .any(|extent| { extent.kind() == ExtentKind::Hole })
    );

    assert_eq!(extents.first().unwrap().offset(), 0,);

    assert_eq!(extents.last().unwrap().end(), device.size(),);

    for pair in extents.windows(2) {
        assert_eq!(pair[0].end(), pair[1].offset(),);
    }

    fs::remove_file(path).unwrap();
}

#[test]
fn raw_disk_exposes_sparse_extents() {
    let path = temporary_path("raw-sparse");

    let mut file = File::create(&path).unwrap();

    file.set_len(8 * 1024 * 1024).unwrap();

    file.seek(SeekFrom::Start(2 * 1024 * 1024)).unwrap();

    file.write_all(&vec![0xcc; 4096]).unwrap();

    file.sync_all().unwrap();

    drop(file);

    let device = LocalFileBlockDevice::open_read_only(&path).unwrap();

    let disk = rvvdk_core::RawDisk::new(device);

    let extents =
        rvvdk_core::VirtualDisk::extents(&disk, 0, rvvdk_core::VirtualDisk::size(&disk)).unwrap();

    assert!(
        extents
            .iter()
            .any(|extent| { extent.kind() == ExtentKind::Hole })
    );

    assert!(
        extents
            .iter()
            .any(|extent| { extent.kind() == ExtentKind::Data })
    );

    fs::remove_file(path).unwrap();
}
