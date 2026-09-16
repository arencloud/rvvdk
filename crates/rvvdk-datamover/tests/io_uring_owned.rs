#![cfg(target_os = "linux")]

use std::fs::{self, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::BufferPool;
use rvvdk_datamover::io_uring::{IoUringEngine, IoUringOperationKind};

const ALIGNMENT: usize = 4096;
const BLOCK_SIZE: usize = 4096;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-io-uring-owned-{name}-{unique}.img"))
}

#[test]
fn owned_write_completes_and_returns_buffer() {
    let path = temporary_path("write");

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .unwrap();

    file.set_len(BLOCK_SIZE as u64).unwrap();

    let pool = BufferPool::new(1, BLOCK_SIZE, ALIGNMENT).unwrap();

    assert_eq!(pool.available(), 1,);

    let mut buffer = pool.acquire();

    assert_eq!(pool.available(), 0,);

    buffer.as_mut_slice().fill(0x5a);

    let mut engine = IoUringEngine::new(8).unwrap();

    let user_data = engine
        .submit_owned_write(file.as_raw_fd(), 0, BLOCK_SIZE, buffer)
        .unwrap();

    assert_eq!(engine.in_flight(), 1,);

    assert_eq!(pool.available(), 0,);

    engine.submit().unwrap();

    let completed = engine.wait_owned_completion().unwrap();

    assert_eq!(completed.user_data(), user_data,);

    assert_eq!(completed.kind(), IoUringOperationKind::Write,);

    assert_eq!(completed.offset(), 0,);

    assert_eq!(completed.length(), BLOCK_SIZE,);

    assert_eq!(completed.bytes_transferred(), BLOCK_SIZE,);

    assert_eq!(engine.in_flight(), 0,);

    /*
     * The CQE has been consumed, but CompletedOperation still owns
     * the BufferGuard. The buffer therefore must not yet be available
     * in the pool.
     */
    assert_eq!(pool.available(), 0,);

    drop(completed);

    /*
     * Dropping CompletedOperation drops its BufferGuard and returns
     * the buffer to the pool.
     */
    assert_eq!(pool.available(), 1,);

    let data = fs::read(&path).unwrap();

    assert!(data.iter().all(|value| { *value == 0x5a },));

    drop(file);

    fs::remove_file(path).unwrap();
}

#[test]
fn owned_read_returns_completed_buffer_with_data() {
    let path = temporary_path("read");

    fs::write(&path, vec![0xa5_u8; BLOCK_SIZE]).unwrap();

    let file = OpenOptions::new().read(true).open(&path).unwrap();

    let pool = BufferPool::new(1, BLOCK_SIZE, ALIGNMENT).unwrap();

    assert_eq!(pool.available(), 1,);

    let buffer = pool.acquire();

    assert_eq!(pool.available(), 0,);

    let mut engine = IoUringEngine::new(8).unwrap();

    let user_data = engine
        .submit_owned_read(file.as_raw_fd(), 0, BLOCK_SIZE, buffer)
        .unwrap();

    assert_eq!(engine.in_flight(), 1,);

    assert_eq!(pool.available(), 0,);

    engine.submit().unwrap();

    let completed = engine.wait_owned_completion().unwrap();

    assert_eq!(completed.user_data(), user_data,);

    assert_eq!(completed.kind(), IoUringOperationKind::Read,);

    assert_eq!(completed.offset(), 0,);

    assert_eq!(completed.length(), BLOCK_SIZE,);

    assert_eq!(completed.bytes_transferred(), BLOCK_SIZE,);

    assert_eq!(engine.in_flight(), 0,);

    assert!(completed.buffer().iter().all(|value| { *value == 0xa5 },));

    /*
     * CompletedOperation still owns the BufferGuard.
     */
    assert_eq!(pool.available(), 0,);

    drop(completed);

    assert_eq!(pool.available(), 1,);

    drop(file);

    fs::remove_file(path).unwrap();
}

#[test]
fn multiple_owned_reads_remain_in_flight() {
    const BLOCKS: usize = 4;

    const FILE_SIZE: usize = BLOCK_SIZE * BLOCKS;

    let path = temporary_path("multiple-reads");

    let mut data = vec![0_u8; FILE_SIZE];

    for block in 0..BLOCKS {
        let value = u8::try_from(block + 1).unwrap();

        let start = block * BLOCK_SIZE;

        let end = start + BLOCK_SIZE;

        data[start..end].fill(value);
    }

    fs::write(&path, &data).unwrap();

    let file = OpenOptions::new().read(true).open(&path).unwrap();

    let pool = BufferPool::new(BLOCKS, BLOCK_SIZE, ALIGNMENT).unwrap();

    assert_eq!(pool.available(), BLOCKS,);

    let mut engine = IoUringEngine::new(8).unwrap();

    let mut submitted = Vec::new();

    for block in 0..BLOCKS {
        let buffer = pool.acquire();

        let user_data = engine
            .submit_owned_read(
                file.as_raw_fd(),
                (block * BLOCK_SIZE) as u64,
                BLOCK_SIZE,
                buffer,
            )
            .unwrap();

        submitted.push(user_data);
    }

    /*
     * All four BufferGuards now belong to the engine's in-flight
     * operation table.
     */
    assert_eq!(engine.in_flight(), BLOCKS,);

    assert_eq!(pool.available(), 0,);

    engine.submit().unwrap();

    let mut completed = Vec::new();

    for _ in 0..BLOCKS {
        completed.push(engine.wait_owned_completion().unwrap());
    }

    assert_eq!(engine.in_flight(), 0,);

    /*
     * The operations have completed, but the CompletedOperation
     * objects still own all four BufferGuards.
     */
    assert_eq!(pool.available(), 0,);

    assert_eq!(completed.len(), BLOCKS,);

    /*
     * CQEs are not required to arrive in submission order.
     *
     * Sort by logical offset before checking the data.
     */
    completed.sort_by_key(|operation| operation.offset());

    for (block, operation) in completed.iter().enumerate() {
        assert_eq!(operation.kind(), IoUringOperationKind::Read,);

        assert_eq!(operation.offset(), (block * BLOCK_SIZE) as u64,);

        assert_eq!(operation.length(), BLOCK_SIZE,);

        assert_eq!(operation.bytes_transferred(), BLOCK_SIZE,);

        let expected = u8::try_from(block + 1).unwrap();

        assert!(
            operation
                .buffer()
                .iter()
                .all(|value| { *value == expected },),
            "block {block} contains incorrect data",
        );
    }

    /*
     * Verify that every submitted operation produced exactly one
     * completion.
     */
    let mut completed_ids = completed
        .iter()
        .map(|operation| operation.user_data())
        .collect::<Vec<_>>();

    submitted.sort_unstable();
    completed_ids.sort_unstable();

    assert_eq!(completed_ids, submitted,);

    drop(completed);

    /*
     * Dropping all CompletedOperation objects releases all guards.
     */
    assert_eq!(pool.available(), BLOCKS,);

    drop(file);

    fs::remove_file(path).unwrap();
}

#[test]
fn multiple_owned_writes_remain_in_flight() {
    const BLOCKS: usize = 4;

    const FILE_SIZE: usize = BLOCK_SIZE * BLOCKS;

    let path = temporary_path("multiple-writes");

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .unwrap();

    file.set_len(FILE_SIZE as u64).unwrap();

    let pool = BufferPool::new(BLOCKS, BLOCK_SIZE, ALIGNMENT).unwrap();

    let mut engine = IoUringEngine::new(8).unwrap();

    let mut submitted = Vec::new();

    for block in 0..BLOCKS {
        let mut buffer = pool.acquire();

        let value = u8::try_from(block + 1).unwrap();

        buffer.as_mut_slice().fill(value);

        let user_data = engine
            .submit_owned_write(
                file.as_raw_fd(),
                (block * BLOCK_SIZE) as u64,
                BLOCK_SIZE,
                buffer,
            )
            .unwrap();

        submitted.push(user_data);
    }

    assert_eq!(engine.in_flight(), BLOCKS,);

    assert_eq!(pool.available(), 0,);

    engine.submit().unwrap();

    let mut completed = Vec::new();

    for _ in 0..BLOCKS {
        completed.push(engine.wait_owned_completion().unwrap());
    }

    assert_eq!(engine.in_flight(), 0,);

    assert_eq!(pool.available(), 0,);

    for operation in &completed {
        assert_eq!(operation.kind(), IoUringOperationKind::Write,);

        assert_eq!(operation.length(), BLOCK_SIZE,);

        assert_eq!(operation.bytes_transferred(), BLOCK_SIZE,);
    }

    let mut completed_ids = completed
        .iter()
        .map(|operation| operation.user_data())
        .collect::<Vec<_>>();

    submitted.sort_unstable();
    completed_ids.sort_unstable();

    assert_eq!(completed_ids, submitted,);

    drop(completed);

    assert_eq!(pool.available(), BLOCKS,);

    let data = fs::read(&path).unwrap();

    for block in 0..BLOCKS {
        let expected = u8::try_from(block + 1).unwrap();

        let start = block * BLOCK_SIZE;

        let end = start + BLOCK_SIZE;

        assert!(
            data[start..end].iter().all(|value| { *value == expected },),
            "block {block} contains incorrect data",
        );
    }

    drop(file);

    fs::remove_file(path).unwrap();
}

#[test]
fn owned_submission_rejects_oversized_request() {
    let path = temporary_path("too-small");

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .unwrap();

    file.set_len((BLOCK_SIZE * 2) as u64).unwrap();

    let pool = BufferPool::new(1, BLOCK_SIZE, ALIGNMENT).unwrap();

    let buffer = pool.acquire();

    assert_eq!(pool.available(), 0,);

    let mut engine = IoUringEngine::new(8).unwrap();

    let result = engine.submit_owned_read(file.as_raw_fd(), 0, BLOCK_SIZE * 2, buffer);

    assert!(matches!(
        result,
        Err(rvvdk_core::Error::BufferTooSmall { .. })
    ));

    /*
     * The request never entered the ring.
     */
    assert_eq!(engine.in_flight(), 0,);

    /*
     * submit_owned_read() returned Err before taking ownership as an
     * in-flight operation. The local BufferGuard was therefore
     * dropped and returned to the pool.
     */
    assert_eq!(pool.available(), 1,);

    drop(file);

    fs::remove_file(path).unwrap();
}

#[test]
fn drain_returns_all_completed_operations() {
    const BLOCKS: usize = 4;

    const FILE_SIZE: usize = BLOCK_SIZE * BLOCKS;

    let path = temporary_path("drain");

    let mut data = vec![0_u8; FILE_SIZE];

    for block in 0..BLOCKS {
        let value = u8::try_from(block + 1).unwrap();

        let start = block * BLOCK_SIZE;

        let end = start + BLOCK_SIZE;

        data[start..end].fill(value);
    }

    fs::write(&path, &data).unwrap();

    let file = OpenOptions::new().read(true).open(&path).unwrap();

    let pool = BufferPool::new(BLOCKS, BLOCK_SIZE, ALIGNMENT).unwrap();

    let mut engine = IoUringEngine::new(8).unwrap();

    for block in 0..BLOCKS {
        let buffer = pool.acquire();

        engine
            .submit_owned_read(
                file.as_raw_fd(),
                (block * BLOCK_SIZE) as u64,
                BLOCK_SIZE,
                buffer,
            )
            .unwrap();
    }

    assert_eq!(engine.in_flight(), BLOCKS,);

    assert_eq!(pool.available(), 0,);

    engine.submit().unwrap();

    let completed = engine.drain().unwrap();

    assert_eq!(completed.len(), BLOCKS,);

    assert_eq!(engine.in_flight(), 0,);

    /*
     * drain() returns ownership of the completed operations to the
     * caller, so their buffers are not available yet.
     */
    assert_eq!(pool.available(), 0,);

    drop(completed);

    assert_eq!(pool.available(), BLOCKS,);

    drop(file);

    fs::remove_file(path).unwrap();
}

#[test]
fn dropping_engine_with_outstanding_operations_returns_buffers() {
    const BLOCKS: usize = 4;

    const FILE_SIZE: usize = BLOCK_SIZE * BLOCKS;

    let path = temporary_path("engine-drop");

    let mut data = vec![0x7c_u8; FILE_SIZE];

    /*
     * Make the blocks distinguishable. This is not required for the
     * ownership assertion, but ensures real I/O work is submitted.
     */
    for block in 0..BLOCKS {
        data[block * BLOCK_SIZE] = u8::try_from(block + 1).unwrap();
    }

    fs::write(&path, &data).unwrap();

    let file = OpenOptions::new().read(true).open(&path).unwrap();

    let pool = BufferPool::new(BLOCKS, BLOCK_SIZE, ALIGNMENT).unwrap();

    {
        let mut engine = IoUringEngine::new(8).unwrap();

        for block in 0..BLOCKS {
            let buffer = pool.acquire();

            engine
                .submit_owned_read(
                    file.as_raw_fd(),
                    (block * BLOCK_SIZE) as u64,
                    BLOCK_SIZE,
                    buffer,
                )
                .unwrap();
        }

        assert_eq!(engine.in_flight(), BLOCKS,);

        assert_eq!(pool.available(), 0,);

        engine.submit().unwrap();

        /*
         * Do not call wait_owned_completion() or drain().
         *
         * Leaving this scope invokes IoUringEngine::drop(), which must
         * keep the owned buffers alive while shutting down the ring and
         * must eventually return them to the pool.
         */
    }

    assert_eq!(pool.available(), BLOCKS,);

    drop(file);

    fs::remove_file(path).unwrap();
}
