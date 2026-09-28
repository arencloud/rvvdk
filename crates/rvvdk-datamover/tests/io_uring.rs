#![cfg(target_os = "linux")]

use std::fs::{self, OpenOptions};
use std::os::fd::AsFd;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::BufferPool;
use rvvdk_datamover::io_uring::{IoUringEngine, IoUringFile};

const BLOCK: usize = 4096;

fn positional_round_trip(offsets: &[u64]) {
    let path = std::env::temp_dir().join(format!(
        "rvvdk-owned-positional-{}-{}.img",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    fs::remove_file(path).unwrap();
    file.set_len((4 * BLOCK) as u64).unwrap();
    let endpoint = IoUringFile::new(file.as_fd()).unwrap();
    let pool = BufferPool::new(1, BLOCK, BLOCK).unwrap();
    let mut engine = IoUringEngine::new(8).unwrap();
    for (index, offset) in offsets.iter().copied().enumerate() {
        let value = index as u8 + 1;
        let mut buffer = pool.acquire();
        buffer.as_mut_slice().fill(value);
        engine
            .submit_owned_write(&endpoint, offset, BLOCK, buffer)
            .unwrap();
        let completion = engine.wait_owned_completion().unwrap();
        assert_eq!(completion.bytes_transferred(), BLOCK);
        drop(completion);
        engine
            .submit_owned_read(&endpoint, offset, BLOCK, pool.acquire())
            .unwrap();
        let completion = engine.wait_owned_completion().unwrap();
        assert_eq!(completion.bytes_transferred(), BLOCK);
        assert!(completion.buffer().iter().all(|byte| *byte == value));
        drop(completion);
    }
    assert_eq!(engine.in_flight(), 0);
    assert_eq!(pool.available(), 1);
}

#[test]
fn positional_write_and_read_round_trip() {
    positional_round_trip(&[0]);
}

#[test]
fn positional_io_respects_offsets() {
    positional_round_trip(&[BLOCK as u64, (3 * BLOCK) as u64]);
}

#[test]
fn positional_read_reports_end_of_file() {
    let file = std::fs::File::open("/dev/null").unwrap();
    let endpoint = IoUringFile::new(file.as_fd()).unwrap();
    let pool = BufferPool::new(1, BLOCK, BLOCK).unwrap();
    let mut engine = IoUringEngine::new(8).unwrap();
    engine
        .submit_owned_read(&endpoint, BLOCK as u64, BLOCK, pool.acquire())
        .unwrap();
    let completed = engine.wait_owned_completion().unwrap();
    assert_eq!(completed.bytes_transferred(), 0);
    drop(completed);
    assert_eq!(engine.in_flight(), 0);
    assert_eq!(pool.available(), 1);
}
