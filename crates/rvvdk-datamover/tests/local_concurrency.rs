#![cfg(target_os = "linux")]
use rvvdk_core::{AlignedBuffer, BlockDevice, BufferPool, Error};
use rvvdk_datamover::io_uring::{IoUringEngine, IoUringFile};
use rvvdk_local::LocalFileBlockDevice;
use rvvdk_platform::{FileAccess, FileAccessKind, inspect_file};
use std::{
    fs,
    os::fd::AsFd,
    sync::atomic::{AtomicUsize, Ordering},
};

const PAGE: usize = 4096;
fn page_size() -> usize {
    // SAFETY: sysconf has no pointer arguments.
    usize::try_from(unsafe { libc::sysconf(libc::_SC_PAGESIZE) }).unwrap()
}
fn fixture() -> (LocalFileBlockDevice, LocalFileBlockDevice, FileAccess) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::var_os("RVVDK_TEST_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(format!(
            "rvvdk-r26-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
    let link = path.with_extension("link");
    fs::write(&path, vec![0x5a; page_size().max(PAGE) * 4]).unwrap();
    fs::hard_link(&path, &link).unwrap();
    let direct = LocalFileBlockDevice::open_direct_read_write(&path).unwrap();
    let buffered = LocalFileBlockDevice::open_read_write(&link).unwrap();
    fs::remove_file(path).unwrap();
    fs::remove_file(link).unwrap();
    let state = inspect_file(direct.as_fd()).unwrap();
    let access = FileAccess::for_identity(state.device, state.inode);
    (direct, buffered, access)
}
fn conflict<T>(result: rvvdk_core::Result<T>) {
    assert!(matches!(result, Err(Error::ConcurrentFileAccess { .. })));
}

#[test]
fn queued_native_read_blocks_buffered_alias_and_unaligned_fallback_until_confirmed() {
    let (direct, buffered, _) = fixture();
    let file = IoUringFile::new(direct.as_fd()).unwrap();
    let pool = BufferPool::new(1, PAGE, PAGE).unwrap();
    let mut engine = IoUringEngine::new(1).unwrap();
    engine
        .submit_owned_read(&file, 0, PAGE, pool.acquire())
        .unwrap();
    conflict(buffered.read_at(1, &mut [0; 7]));
    conflict(direct.read_at(1, &mut [0; 7]));
    conflict(buffered.write_at(1, &[0; 7]));
    conflict(direct.write_zero_at(1, 7));
    conflict(buffered.discard(1, 7));
    conflict(direct.flush());
    // Extent inspection is compatible with readers, and disjoint pages stay available.
    buffered.extents(0, PAGE as u64).unwrap();
    buffered
        .write_at((page_size().max(PAGE) * 2) as u64, &[0xa5; 7])
        .unwrap();
    let done = engine.wait_owned_completion().unwrap();
    assert!(done.buffer().iter().all(|&v| v == 0x5a));
    // CompletedOperation still owns its buffer, but has released file admission.
    direct.write_at(1, &[0x33; 7]).unwrap();
    direct.flush().unwrap();
    drop(done);
    let mut bytes = [0; 9];
    buffered.read_exact_at(0, &mut bytes).unwrap();
    assert_eq!(
        bytes,
        [0x5a, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x33, 0x5a]
    );
}

#[test]
fn queued_native_writer_excludes_inspection_and_other_queues_without_losing_buffers() {
    let (_, buffered, _) = fixture();
    let file = IoUringFile::new(buffered.as_fd()).unwrap();
    let pool = BufferPool::new(2, PAGE, PAGE).unwrap();
    let mut first = IoUringEngine::new(1).unwrap();
    let mut second = IoUringEngine::new(1).unwrap();
    let mut bytes = pool.acquire();
    bytes.as_mut_slice().fill(0x66);
    first.submit_owned_write(&file, 0, PAGE, bytes).unwrap();
    conflict(second.submit_owned_read(&file, 0, PAGE, pool.acquire()));
    assert_eq!(second.in_flight(), 0);
    assert_eq!(pool.available(), 1);
    conflict(buffered.extents(0, PAGE as u64));
    conflict(buffered.write_zero_at(0, PAGE as u64));
    first.shutdown().unwrap();
    assert_eq!(pool.available(), 2);
    let mut bytes = [0; PAGE];
    buffered.read_exact_at(0, &mut bytes).unwrap();
    assert_eq!(bytes, [0x66; PAGE]);
    buffered.discard(1, 7).unwrap();
}

#[test]
fn local_aligned_unaligned_sparse_and_flush_paths_all_join_admission() {
    let (direct, buffered, access) = fixture();
    let mut bytes = AlignedBuffer::new(PAGE, PAGE).unwrap();
    bytes.fill(0x77);
    let guard = access
        .try_acquire(0, PAGE as u64, FileAccessKind::BufferedRead)
        .unwrap();
    conflict(direct.read_at(0, bytes.as_mut_slice()));
    conflict(direct.write_at(0, bytes.as_slice()));
    buffered.read_at(1, &mut [0; 7]).unwrap();
    conflict(buffered.write_at(1, &[1]));
    conflict(buffered.write_zero_at(1, 7));
    conflict(direct.discard(1, 7));
    drop(guard);
    direct.write_at(0, bytes.as_slice()).unwrap();
    let barrier = access.try_acquire(0, 0, FileAccessKind::Flush).unwrap();
    conflict(direct.read_at((page_size().max(PAGE) * 2) as u64, bytes.as_mut_slice()));
    conflict(buffered.extents((page_size().max(PAGE) * 2) as u64, PAGE as u64));
    conflict(buffered.flush());
    drop(barrier);
    direct.flush().unwrap();
    buffered.read_exact_at(0, bytes.as_mut_slice()).unwrap();
    assert!(bytes.as_slice().iter().all(|&v| v == 0x77));
}

#[test]
fn disjoint_direct_subpage_requests_remain_pipeline_compatible() {
    let (direct, _, _) = fixture();
    let alignment = direct.direct_io_alignment().unwrap();
    let block = alignment.offset_alignment();
    // Test at the actual discovered unit, including 512-byte units where supported.
    let pool = BufferPool::new(2, block, alignment.memory_alignment()).unwrap();
    let file = IoUringFile::new(direct.as_fd()).unwrap();
    let mut engine = IoUringEngine::new(2).unwrap();
    for i in 0..2 {
        let mut bytes = pool.acquire();
        bytes.as_mut_slice().fill(0x22 + i as u8);
        engine
            .submit_owned_write(&file, (i * block) as u64, block, bytes)
            .unwrap();
    }
    engine.shutdown().unwrap();
    assert_eq!(pool.available(), 2);
    let mut bytes = vec![0; block * 2];
    direct.read_exact_at(0, &mut bytes).unwrap();
    assert!(bytes[..block].iter().all(|&v| v == 0x22));
    assert!(bytes[block..].iter().all(|&v| v == 0x23));
}
