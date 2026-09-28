#![cfg(target_os = "linux")]
use rvvdk_core::{BlockDevice, Capabilities, Error};
use rvvdk_local::LocalFileBlockDevice;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

fn path() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::var_os("RVVDK_TEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    fs::create_dir_all(&directory).unwrap();
    directory.join(format!(
        "rvvdk-r22-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ))
}
fn contents(size: usize) -> Vec<u8> {
    let mut state = 0x483bc95a670f28d1u64;
    (0..size)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state as u8
        })
        .collect()
}

#[test]
fn zero_and_punch_preserve_unaligned_edges_size_and_readback_for_both_open_modes() {
    let size = 256 * 1024 + 17;
    for direct in [false, true] {
        for punch in [false, true] {
            for (offset, length) in [
                (0, 1),
                (7, 1),
                (3, 8193),
                (4096, 65536),
                (5, size - 5),
                (size - 1, 1),
                (size, 0),
            ] {
                let path = path();
                let mut expected = contents(size);
                fs::write(&path, &expected).unwrap();
                let disk = if direct {
                    LocalFileBlockDevice::open_direct_read_write(&path)
                } else {
                    LocalFileBlockDevice::open_read_write(&path)
                }
                .unwrap();
                assert!(disk.capabilities().contains(
                    Capabilities::WRITE_ZERO | Capabilities::DISCARD | Capabilities::DISCARD_ZEROES
                ));
                if punch {
                    disk.discard(offset as u64, length as u64)
                } else {
                    disk.write_zero_at(offset as u64, length as u64)
                }
                .unwrap();
                disk.flush().unwrap();
                expected[offset..offset + length].fill(0);
                let mut actual = vec![0; size];
                disk.read_exact_at(0, &mut actual).unwrap();
                assert_eq!(actual, expected);
                assert_eq!(fs::metadata(&path).unwrap().len(), size as u64);
                drop(disk);
                assert_eq!(fs::read(&path).unwrap(), expected);
                fs::remove_file(path).unwrap();
            }
        }
    }
}

#[test]
fn sparse_writes_reject_read_only_invalid_and_truncated_ranges_without_growth() {
    for direct in [false, true] {
        let path = path();
        let expected = contents(8192);
        fs::write(&path, &expected).unwrap();
        let read_only = if direct {
            LocalFileBlockDevice::open_direct_read_only(&path)
        } else {
            LocalFileBlockDevice::open_read_only(&path)
        }
        .unwrap();
        assert!(!read_only.capabilities().intersects(
            Capabilities::WRITE_ZERO | Capabilities::DISCARD | Capabilities::DISCARD_ZEROES
        ));
        for length in [0, 4096] {
            assert!(matches!(
                read_only.write_zero_at(0, length),
                Err(Error::Unsupported)
            ));
            assert!(matches!(
                read_only.discard(0, length),
                Err(Error::Unsupported)
            ));
        }
        let writable = if direct {
            LocalFileBlockDevice::open_direct_read_write(&path)
        } else {
            LocalFileBlockDevice::open_read_write(&path)
        }
        .unwrap();
        for (offset, length) in [(8192, 1), (8193, 0), (u64::MAX, 2)] {
            assert!(writable.write_zero_at(offset, length).is_err());
            assert!(writable.discard(offset, length).is_err());
        }
        assert_eq!(fs::read(&path).unwrap(), expected);
        fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_len(4096)
            .unwrap();
        for punch in [false, true] {
            let result = if punch {
                writable.discard(4095, 2)
            } else {
                writable.write_zero_at(4095, 2)
            };
            assert!(matches!(result, Err(Error::OutOfBounds { size: 4096, .. })));
        }
        assert_eq!(fs::read(&path).unwrap(), expected[..4096]);
        fs::remove_file(path).unwrap();
    }
}

#[test]
#[ignore = "requires RVVDK_TEST_DIR on a storage filesystem supporting hole punching"]
fn storage_hole_punch_reclaims_blocks_and_preserves_data() {
    use std::os::unix::fs::MetadataExt;
    assert!(std::env::var_os("RVVDK_TEST_DIR").is_some());
    let path = path();
    let mut expected = contents(8 * 1024 * 1024);
    fs::write(&path, &expected).unwrap();
    let disk = LocalFileBlockDevice::open_read_write(&path).unwrap();
    disk.flush().unwrap();
    let before = fs::metadata(&path).unwrap().blocks();
    disk.discard(1024 * 1024, 6 * 1024 * 1024).unwrap();
    disk.flush().unwrap();
    let after = fs::metadata(&path).unwrap().blocks();
    println!("allocated 512-byte blocks before={before}, after={after}");
    assert!(
        after < before / 2,
        "filesystem did not demonstrate substantial reclamation"
    );
    expected[1024 * 1024..7 * 1024 * 1024].fill(0);
    assert_eq!(fs::read(&path).unwrap(), expected);
    fs::remove_file(path).unwrap();
}
