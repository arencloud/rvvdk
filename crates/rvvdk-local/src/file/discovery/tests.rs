use super::*;
use rvvdk_core::BlockDevice;
use std::fs::File;
use std::os::unix::fs::FileExt;
use std::sync::atomic::{AtomicUsize, Ordering};

fn device() -> LocalFileBlockDevice {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "rvvdk-r23-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let file = File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    std::fs::remove_file(path).unwrap();
    file.set_len(12288).unwrap();
    LocalFileBlockDevice::from_file(file, None, rvvdk_core::Capabilities::READ, false).unwrap()
}

fn errno(code: i32) -> io::Result<u64> {
    Err(io::Error::from_raw_os_error(code))
}

#[test]
fn unsupported_at_either_seek_or_after_a_partial_map_replaces_the_whole_query() {
    let device = device();
    for code in [libc::EINVAL, libc::EOPNOTSUPP, libc::ENOSYS] {
        for fail_at in 0..4 {
            let mut calls = 0;
            let map = device
                .discover_extents(17, 8175, |offset, whence| {
                    let expected = [
                        (17, libc::SEEK_DATA, 1024),
                        (1024, libc::SEEK_HOLE, 2048),
                        (2048, libc::SEEK_DATA, 4096),
                        (4096, libc::SEEK_HOLE, 8192),
                    ][calls];
                    assert_eq!((offset, whence), (expected.0, expected.1));
                    calls += 1;
                    if calls - 1 == fail_at {
                        errno(code)
                    } else {
                        Ok(expected.2)
                    }
                })
                .unwrap();
            assert_eq!(map, vec![Extent::new(17, 8175, ExtentKind::Data).unwrap()]);
            assert_eq!(calls, fail_at + 1);
        }
    }
}

#[test]
fn dense_fallback_reads_all_logical_bytes_including_real_holes_and_nonzero_tail() {
    let device = device();
    let mut expected = vec![0; 12288];
    expected[..4096].fill(0x5a);
    expected[8192..].fill(0xc3);
    device.file.write_all_at(&expected[..4096], 0).unwrap();
    device.file.write_all_at(&expected[8192..], 8192).unwrap();
    for (offset, length) in [(0, 12288), (7, 12274), (4097, 8191)] {
        let map = device
            .discover_extents(offset, length, |_, _| errno(libc::EINVAL))
            .unwrap();
        let mut actual = vec![0xa5; length as usize];
        for extent in map {
            assert_eq!(extent.kind(), ExtentKind::Data);
            let begin = (extent.offset() - offset) as usize;
            device
                .read_exact_at(
                    extent.offset(),
                    &mut actual[begin..begin + extent.length() as usize],
                )
                .unwrap();
        }
        assert_eq!(
            actual,
            expected[offset as usize..(offset + length) as usize]
        );
    }
}

#[test]
fn real_errors_propagate_even_after_a_successful_prefix() {
    let device = device();
    for code in [
        libc::EIO,
        libc::EBADF,
        libc::EPERM,
        libc::EACCES,
        libc::ENOSPC,
        libc::EOVERFLOW,
        libc::ESPIPE,
    ] {
        for fail_at in 0..4 {
            let mut calls = 0;
            let result = device.discover_extents(0, 8192, |_, _| {
                let index = calls;
                calls += 1;
                if index == fail_at {
                    errno(code)
                } else {
                    Ok([1024, 2048, 4096, 8192][index])
                }
            });
            assert!(matches!(result, Err(Error::Io(error)) if error.raw_os_error() == Some(code)));
            assert_eq!(calls, fail_at + 1);
        }
    }
}

#[test]
fn interrupted_queries_retry_the_same_offset_and_whence() {
    let device = device();
    let mut calls = 0;
    let map = device
        .discover_extents(7, 8185, |offset, whence| {
            let expected = [(7, libc::SEEK_DATA, 4096), (4096, libc::SEEK_HOLE, 12288)][calls / 2];
            assert_eq!((offset, whence), (expected.0, expected.1));
            calls += 1;
            if calls % 2 == 1 {
                errno(libc::EINTR)
            } else {
                Ok(expected.2)
            }
        })
        .unwrap();
    assert_eq!(calls, 4);
    assert_eq!(
        map,
        vec![
            Extent::new(7, 4089, ExtentKind::Hole).unwrap(),
            Extent::new(4096, 4096, ExtentKind::Data).unwrap(),
        ]
    );
}

#[test]
fn trailing_holes_and_data_beyond_the_query_are_clipped() {
    let device = device();
    for next_data in [None, Some(8192)] {
        let map = device
            .discover_extents(7, 4096, |offset, whence| {
                assert_eq!((offset, whence), (7, libc::SEEK_DATA));
                next_data.map_or_else(|| errno(libc::ENXIO), Ok)
            })
            .unwrap();
        assert_eq!(map, vec![Extent::new(7, 4096, ExtentKind::Hole).unwrap()]);
    }
}

#[test]
fn backwards_nonprogressing_and_past_eof_offsets_are_rejected() {
    let device = device();
    for replies in [
        vec![6],
        vec![12288],
        vec![12289],
        vec![7, 7],
        vec![7, 6],
        vec![7, 12289],
        vec![7, 4096, 2048],
    ] {
        let mut replies = replies.into_iter();
        let result = device.discover_extents(7, 8185, |_, _| Ok(replies.next().unwrap()));
        assert!(matches!(result, Err(Error::CorruptMetadata(_))));
    }
}

#[test]
fn missing_hole_after_known_data_is_an_error() {
    let device = device();
    let result = device.discover_extents(0, 8192, |_, whence| {
        if whence == libc::SEEK_DATA {
            Ok(0)
        } else {
            errno(libc::ENXIO)
        }
    });
    assert!(matches!(result, Err(Error::Io(error)) if error.raw_os_error() == Some(libc::ENXIO)));
}

#[test]
fn invalid_and_empty_ranges_do_not_seek_and_observed_truncation_is_rejected() {
    let device = device();
    let no_seek = |_, _| panic!("invalid or empty request must not seek");
    assert!(matches!(
        device.discover_extents(u64::MAX, 1, no_seek),
        Err(Error::RangeOverflow { .. })
    ));
    for (offset, length) in [(12289, 0), (12288, 1), (7, 12288)] {
        assert!(matches!(
            device.discover_extents(offset, length, no_seek),
            Err(Error::OutOfBounds { .. })
        ));
    }
    assert!(
        device
            .discover_extents(12288, 0, no_seek)
            .unwrap()
            .is_empty()
    );
    device.file.set_len(4096).unwrap();
    for (offset, length) in [(0, 8192), (12288, 0)] {
        assert!(matches!(
            device.discover_extents(offset, length, no_seek),
            Err(Error::OutOfBounds { size: 4096, .. })
        ));
    }
    // Growth does not change this handle's logical geometry.
    device.file.set_len(16384).unwrap();
    assert!(matches!(
        device.discover_extents(12288, 1, no_seek),
        Err(Error::OutOfBounds { size: 12288, .. })
    ));
}

#[test]
fn shrink_during_discovery_cannot_return_a_hole_or_dense_map() {
    for code in [libc::ENXIO, libc::EINVAL, libc::EOPNOTSUPP, libc::ENOSYS] {
        let device = device();
        let result = device.discover_extents(0, 8192, |_, _| {
            device.file.set_len(4096).unwrap();
            errno(code)
        });
        assert!(matches!(result, Err(Error::OutOfBounds { size: 4096, .. })));
    }
}

#[test]
fn unsupported_discovery_does_not_hide_later_errors_or_map_changes() {
    let device = device();
    device
        .discover_extents(0, 8192, |_, _| errno(libc::EINVAL))
        .unwrap();
    let result = device.discover_extents(0, 8192, |_, _| errno(libc::EIO));
    assert!(matches!(result, Err(Error::Io(error)) if error.raw_os_error() == Some(libc::EIO)));
    let map = device
        .discover_extents(0, 8192, |_, _| errno(libc::ENXIO))
        .unwrap();
    assert_eq!(map, vec![Extent::new(0, 8192, ExtentKind::Hole).unwrap()]);
}
