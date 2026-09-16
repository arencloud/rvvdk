#![cfg(target_os = "linux")]

use std::fs::{self, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::AlignedBuffer;
use rvvdk_datamover::io_uring::IoUringEngine;

const ALIGNMENT: usize = 4096;
const TRANSFER_SIZE: usize = 4096;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-io-uring-{name}-{unique}.img"))
}

#[test]
fn positional_write_and_read_round_trip() {
    let path = temporary_path("round-trip");

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .unwrap();

    file.set_len(TRANSFER_SIZE as u64).unwrap();

    let mut engine = IoUringEngine::new(8).unwrap();

    let mut write_buffer = AlignedBuffer::new(TRANSFER_SIZE, ALIGNMENT).unwrap();

    write_buffer.fill(0x5a);

    let written = engine
        .write_at(file.as_raw_fd(), 0, write_buffer.as_slice())
        .unwrap();

    assert_eq!(written, TRANSFER_SIZE,);

    assert_eq!(engine.in_flight(), 0,);

    let mut read_buffer = AlignedBuffer::new(TRANSFER_SIZE, ALIGNMENT).unwrap();

    let read = engine
        .read_at(file.as_raw_fd(), 0, read_buffer.as_mut_slice())
        .unwrap();

    assert_eq!(read, TRANSFER_SIZE,);

    assert_eq!(engine.in_flight(), 0,);

    assert_eq!(read_buffer.as_slice(), write_buffer.as_slice(),);

    drop(file);

    fs::remove_file(path).unwrap();
}

#[test]
fn positional_io_respects_offsets() {
    const FILE_SIZE: usize = TRANSFER_SIZE * 4;

    let path = temporary_path("offsets");

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .unwrap();

    file.set_len(FILE_SIZE as u64).unwrap();

    let mut engine = IoUringEngine::new(8).unwrap();

    let mut first = AlignedBuffer::new(TRANSFER_SIZE, ALIGNMENT).unwrap();

    first.fill(0x11);

    let mut second = AlignedBuffer::new(TRANSFER_SIZE, ALIGNMENT).unwrap();

    second.fill(0x22);

    let first_offset = TRANSFER_SIZE as u64;

    let second_offset = (TRANSFER_SIZE * 3) as u64;

    let written = engine
        .write_at(file.as_raw_fd(), first_offset, first.as_slice())
        .unwrap();

    assert_eq!(written, TRANSFER_SIZE,);

    let written = engine
        .write_at(file.as_raw_fd(), second_offset, second.as_slice())
        .unwrap();

    assert_eq!(written, TRANSFER_SIZE,);

    let mut first_read = AlignedBuffer::new(TRANSFER_SIZE, ALIGNMENT).unwrap();

    let read = engine
        .read_at(file.as_raw_fd(), first_offset, first_read.as_mut_slice())
        .unwrap();

    assert_eq!(read, TRANSFER_SIZE,);

    assert_eq!(first_read.as_slice(), first.as_slice(),);

    let mut second_read = AlignedBuffer::new(TRANSFER_SIZE, ALIGNMENT).unwrap();

    let read = engine
        .read_at(file.as_raw_fd(), second_offset, second_read.as_mut_slice())
        .unwrap();

    assert_eq!(read, TRANSFER_SIZE,);

    assert_eq!(second_read.as_slice(), second.as_slice(),);

    assert_eq!(engine.in_flight(), 0,);

    drop(file);

    fs::remove_file(path).unwrap();
}

#[test]
fn positional_read_reports_end_of_file() {
    let path = temporary_path("eof");

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&path)
        .unwrap();

    file.set_len(TRANSFER_SIZE as u64).unwrap();

    let mut engine = IoUringEngine::new(8).unwrap();

    let mut buffer = AlignedBuffer::new(TRANSFER_SIZE, ALIGNMENT).unwrap();

    let read = engine
        .read_at(
            file.as_raw_fd(),
            TRANSFER_SIZE as u64,
            buffer.as_mut_slice(),
        )
        .unwrap();

    assert_eq!(read, 0,);

    assert_eq!(engine.in_flight(), 0,);

    drop(file);

    fs::remove_file(path).unwrap();
}
