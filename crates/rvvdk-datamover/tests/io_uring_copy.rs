#![cfg(target_os = "linux")]

use std::fs::{self, OpenOptions};
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_datamover::io_uring::copy_file_range;

use rvvdk_datamover::IoUringExecutionOptions;

use rvvdk_datamover::io_uring::copy_file_range_with_options;

const MIB: usize = 1024 * 1024;

const BLOCK_SIZE: usize = 64 * 1024;

const FILE_SIZE: usize = 8 * MIB;

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-io-uring-copy-{name}-{unique}.img"))
}

fn source_data() -> Vec<u8> {
    let mut data = vec![0_u8; FILE_SIZE];

    let mut state = 0x5256_5644_4b17_u64;

    for chunk in data.chunks_mut(8) {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;

        let bytes = state.to_le_bytes();

        let length = chunk.len();

        chunk.copy_from_slice(&bytes[..length]);
    }

    data
}

#[test]
fn copies_file_with_io_uring_pipeline() {
    let source_path = temporary_path("source");

    let destination_path = temporary_path("destination");

    let expected = source_data();

    fs::write(&source_path, &expected).unwrap();

    fs::write(&destination_path, vec![0_u8; FILE_SIZE]).unwrap();

    let source = OpenOptions::new().read(true).open(&source_path).unwrap();

    let destination = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&destination_path)
        .unwrap();

    let stats = copy_file_range(
        source.as_raw_fd(),
        destination.as_raw_fd(),
        0,
        FILE_SIZE as u64,
        BLOCK_SIZE,
        8,
        4096,
    )
    .unwrap();

    assert_eq!(stats.bytes_read(), FILE_SIZE as u64,);

    assert_eq!(stats.bytes_written(), FILE_SIZE as u64,);

    assert_eq!(stats.blocks_completed(), (FILE_SIZE / BLOCK_SIZE) as u64,);

    drop(destination);
    drop(source);

    let actual = fs::read(&destination_path).unwrap();

    assert_eq!(actual, expected,);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn copies_with_multiple_queue_depths() {
    for queue_depth in [1, 2, 4, 8, 16, 32] {
        let source_path = temporary_path(&format!("source-qd-{queue_depth}"));

        let destination_path = temporary_path(&format!("destination-qd-{queue_depth}"));

        let expected = source_data();

        fs::write(&source_path, &expected).unwrap();

        fs::write(&destination_path, vec![0_u8; FILE_SIZE]).unwrap();

        let source = OpenOptions::new().read(true).open(&source_path).unwrap();

        let destination = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&destination_path)
            .unwrap();

        let stats = copy_file_range(
            source.as_raw_fd(),
            destination.as_raw_fd(),
            0,
            FILE_SIZE as u64,
            BLOCK_SIZE,
            queue_depth,
            4096,
        )
        .unwrap();

        assert_eq!(
            stats.bytes_read(),
            FILE_SIZE as u64,
            "incorrect bytes_read at QD {queue_depth}",
        );

        assert_eq!(
            stats.bytes_written(),
            FILE_SIZE as u64,
            "incorrect bytes_written at QD {queue_depth}",
        );

        assert_eq!(
            stats.blocks_completed(),
            (FILE_SIZE / BLOCK_SIZE) as u64,
            "incorrect block count at QD {queue_depth}",
        );

        assert!(
            stats.peak_in_flight() <= queue_depth as usize,
            "pipeline exceeded QD {queue_depth}",
        );

        assert!(
            stats.peak_reads_in_flight() <= (queue_depth as usize).div_ceil(2).max(1),
            "read window exceeded at QD {queue_depth}",
        );

        drop(destination);
        drop(source);

        let actual = fs::read(&destination_path).unwrap();

        assert_eq!(actual, expected, "copy mismatch at QD {queue_depth}",);

        fs::remove_file(source_path).unwrap();

        fs::remove_file(destination_path).unwrap();
    }
}

#[test]
fn copies_partial_final_block() {
    const TAIL: usize = 123;

    let file_size = FILE_SIZE + TAIL;

    let source_path = temporary_path("partial-source");

    let destination_path = temporary_path("partial-destination");

    let mut expected = source_data();

    expected.extend((0..TAIL).map(|value| u8::try_from(value % 251).unwrap()));

    assert_eq!(expected.len(), file_size,);

    fs::write(&source_path, &expected).unwrap();

    fs::write(&destination_path, vec![0_u8; file_size]).unwrap();

    let source = OpenOptions::new().read(true).open(&source_path).unwrap();

    let destination = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&destination_path)
        .unwrap();

    let stats = copy_file_range(
        source.as_raw_fd(),
        destination.as_raw_fd(),
        0,
        file_size as u64,
        BLOCK_SIZE,
        8,
        4096,
    )
    .unwrap();

    assert_eq!(stats.bytes_read(), file_size as u64,);

    assert_eq!(stats.bytes_written(), file_size as u64,);

    assert_eq!(
        stats.blocks_completed(),
        file_size.div_ceil(BLOCK_SIZE) as u64,
    );

    drop(destination);
    drop(source);

    let actual = fs::read(&destination_path).unwrap();

    assert_eq!(actual, expected,);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn copies_non_zero_file_range() {
    const PREFIX: usize = 2 * BLOCK_SIZE;

    const COPY_LENGTH: usize = 4 * BLOCK_SIZE;

    const TOTAL_SIZE: usize = PREFIX + COPY_LENGTH + BLOCK_SIZE;

    let source_path = temporary_path("range-source");

    let destination_path = temporary_path("range-destination");

    let mut source_data = vec![0_u8; TOTAL_SIZE];

    for (index, value) in source_data.iter_mut().enumerate() {
        *value = u8::try_from(index % 251).unwrap();
    }

    let destination_data = vec![0xee_u8; TOTAL_SIZE];

    fs::write(&source_path, &source_data).unwrap();

    fs::write(&destination_path, &destination_data).unwrap();

    let source = OpenOptions::new().read(true).open(&source_path).unwrap();

    let destination = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&destination_path)
        .unwrap();

    let stats = copy_file_range(
        source.as_raw_fd(),
        destination.as_raw_fd(),
        PREFIX as u64,
        COPY_LENGTH as u64,
        BLOCK_SIZE,
        4,
        4096,
    )
    .unwrap();

    assert_eq!(stats.bytes_read(), COPY_LENGTH as u64,);

    assert_eq!(stats.bytes_written(), COPY_LENGTH as u64,);

    drop(destination);
    drop(source);

    let actual = fs::read(&destination_path).unwrap();

    /*
     * Prefix must remain untouched.
     */
    assert!(actual[..PREFIX].iter().all(|value| { *value == 0xee },));

    /*
     * Requested range must match the source.
     */
    assert_eq!(
        &actual[PREFIX..PREFIX + COPY_LENGTH],
        &source_data[PREFIX..PREFIX + COPY_LENGTH],
    );

    /*
     * Suffix must remain untouched.
     */
    assert!(
        actual[PREFIX + COPY_LENGTH..]
            .iter()
            .all(|value| { *value == 0xee },)
    );

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn zero_length_copy_does_nothing() {
    let source_path = temporary_path("zero-source");

    let destination_path = temporary_path("zero-destination");

    let source_data = vec![0x11_u8; BLOCK_SIZE];

    let destination_data = vec![0x22_u8; BLOCK_SIZE];

    fs::write(&source_path, &source_data).unwrap();

    fs::write(&destination_path, &destination_data).unwrap();

    let source = OpenOptions::new().read(true).open(&source_path).unwrap();

    let destination = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&destination_path)
        .unwrap();

    let stats = copy_file_range(
        source.as_raw_fd(),
        destination.as_raw_fd(),
        0,
        0,
        BLOCK_SIZE,
        8,
        4096,
    )
    .unwrap();

    assert_eq!(stats.bytes_read(), 0,);

    assert_eq!(stats.bytes_written(), 0,);

    assert_eq!(stats.blocks_completed(), 0,);

    drop(destination);
    drop(source);

    let actual = fs::read(&destination_path).unwrap();

    assert_eq!(actual, destination_data,);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn pipeline_uses_reads_and_writes_concurrently() {
    let source_path = temporary_path("balanced-source");

    let destination_path = temporary_path("balanced-destination");

    let expected = source_data();

    fs::write(&source_path, &expected).unwrap();

    fs::write(&destination_path, vec![0_u8; FILE_SIZE]).unwrap();

    let source = OpenOptions::new().read(true).open(&source_path).unwrap();

    let destination = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&destination_path)
        .unwrap();

    let stats = copy_file_range(
        source.as_raw_fd(),
        destination.as_raw_fd(),
        0,
        FILE_SIZE as u64,
        BLOCK_SIZE,
        8,
        4096,
    )
    .unwrap();

    assert!(stats.peak_reads_in_flight() >= 2,);

    assert!(stats.peak_writes_in_flight() >= 2,);

    assert!(stats.peak_in_flight() <= 8,);

    drop(destination);
    drop(source);

    let actual = fs::read(&destination_path).unwrap();

    assert_eq!(actual, expected,);

    assert!(
        stats.mixed_in_flight_observed(),
        "pipeline never had reads and writes in flight simultaneously",
    );

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}

#[test]
fn supports_custom_read_window() {
    let source_path = temporary_path("custom-window-source");

    let destination_path = temporary_path("custom-window-destination");

    let expected = source_data();

    fs::write(&source_path, &expected).unwrap();

    fs::write(&destination_path, vec![0_u8; FILE_SIZE]).unwrap();

    let source = OpenOptions::new().read(true).open(&source_path).unwrap();

    let destination = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&destination_path)
        .unwrap();

    let options = IoUringExecutionOptions::with_read_window(8, 3).unwrap();

    let stats = copy_file_range_with_options(
        source.as_raw_fd(),
        destination.as_raw_fd(),
        0,
        FILE_SIZE as u64,
        BLOCK_SIZE,
        4096,
        options,
    )
    .unwrap();

    assert_eq!(stats.bytes_written(), FILE_SIZE as u64,);

    assert!(stats.peak_reads_in_flight() <= 3,);

    assert!(stats.mixed_in_flight_observed(),);

    drop(destination);
    drop(source);

    let actual = fs::read(&destination_path).unwrap();

    assert_eq!(actual, expected,);

    fs::remove_file(source_path).unwrap();

    fs::remove_file(destination_path).unwrap();
}
