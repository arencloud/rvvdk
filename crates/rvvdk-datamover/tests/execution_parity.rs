#![cfg(target_os = "linux")]

mod support;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use rvvdk_core::{Capabilities, Extent, ExtentKind, RawDisk};

use rvvdk_datamover::{
    CopyOptions, DataMover, ExecutionBackend, ExecutionStrategy, IoUringExecutionOptions,
};

use support::test_block_device::TestExtentBlockDevice;

const BLOCK_SIZE: usize = 64 * 1024;

const EXTENT_SIZE: usize = 2 * 1024 * 1024;

const FILE_SIZE: usize = 4 * EXTENT_SIZE;

#[derive(Debug, Clone, Copy)]
struct ParityProfile {
    name: &'static str,
    capabilities: Capabilities,
    expected_bytes_written: u64,
    expected_bytes_zeroed: u64,
    expected_bytes_discarded: u64,
    expected_blocks_copied: u64,
}

fn temporary_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("rvvdk-parity-{name}-{unique}.img"))
}

fn mixed_extent_map() -> Vec<Extent> {
    vec![
        Extent::new(0, EXTENT_SIZE as u64, ExtentKind::Data).unwrap(),
        Extent::new(EXTENT_SIZE as u64, EXTENT_SIZE as u64, ExtentKind::Zero).unwrap(),
        Extent::new(
            (2 * EXTENT_SIZE) as u64,
            EXTENT_SIZE as u64,
            ExtentKind::Hole,
        )
        .unwrap(),
        Extent::new(
            (3 * EXTENT_SIZE) as u64,
            EXTENT_SIZE as u64,
            ExtentKind::Data,
        )
        .unwrap(),
    ]
}

fn source_contents() -> Vec<u8> {
    let mut data = vec![0_u8; FILE_SIZE];

    /*
     * First Data extent.
     */
    for (index, value) in data[0..EXTENT_SIZE].iter_mut().enumerate() {
        *value = u8::try_from(index % 251).unwrap();
    }

    /*
     * Zero and Hole extents remain logically zero.
     */

    /*
     * Final Data extent.
     */
    for (index, value) in data[(3 * EXTENT_SIZE)..FILE_SIZE].iter_mut().enumerate() {
        *value = u8::try_from((index + 97) % 251).unwrap();
    }

    data
}

fn source_capabilities() -> Capabilities {
    Capabilities::READ | Capabilities::EXTENTS
}

fn base_destination_capabilities() -> Capabilities {
    Capabilities::READ | Capabilities::WRITE | Capabilities::FLUSH
}

fn parity_profiles() -> [ParityProfile; 3] {
    let base = base_destination_capabilities();

    let data_blocks = ((2 * EXTENT_SIZE) / BLOCK_SIZE) as u64;

    let all_blocks = (FILE_SIZE / BLOCK_SIZE) as u64;

    [
        /*
         * Profile 1
         *
         * Zero -> WRITE_ZERO
         * Hole -> DISCARD
         */
        ParityProfile {
            name: "write-zero-and-discard",

            capabilities: base | Capabilities::WRITE_ZERO | Capabilities::DISCARD,

            /*
             * Only the two Data extents are ordinary writes.
             */
            expected_bytes_written: (2 * EXTENT_SIZE) as u64,

            /*
             * Zero uses WRITE_ZERO.
             */
            expected_bytes_zeroed: EXTENT_SIZE as u64,

            /*
             * Hole uses DISCARD.
             */
            expected_bytes_discarded: EXTENT_SIZE as u64,

            /*
             * Only Data contributes copied blocks.
             */
            expected_blocks_copied: data_blocks,
        },
        /*
         * Profile 2
         *
         * Zero -> WRITE_ZERO
         * Hole -> WRITE_ZERO
         */
        ParityProfile {
            name: "write-zero-only",

            capabilities: base | Capabilities::WRITE_ZERO,

            /*
             * Only the two Data extents are ordinary writes.
             */
            expected_bytes_written: (2 * EXTENT_SIZE) as u64,

            /*
             * Both Zero and Hole use WRITE_ZERO.
             */
            expected_bytes_zeroed: (2 * EXTENT_SIZE) as u64,

            expected_bytes_discarded: 0,

            /*
             * Only Data contributes copied blocks.
             */
            expected_blocks_copied: data_blocks,
        },
        /*
         * Profile 3
         *
         * Zero -> ordinary zero-filled writes
         * Hole -> ordinary zero-filled writes
         */
        ParityProfile {
            name: "zero-write-fallback",

            capabilities: base,

            /*
             * Data + Zero fallback + Hole fallback all use ordinary
             * writes.
             *
             * The entire logical 8 MiB destination is therefore
             * written.
             */
            expected_bytes_written: FILE_SIZE as u64,

            /*
             * IMPORTANT:
             *
             * bytes_zeroed tracks execution through WRITE_ZERO.
             *
             * Neither WRITE_ZERO nor DISCARD is available in this
             * profile, so Zero and Hole are materialized using
             * ordinary zero-filled writes.
             *
             * Those bytes therefore contribute to bytes_written,
             * not bytes_zeroed.
             */
            expected_bytes_zeroed: 0,

            expected_bytes_discarded: 0,

            /*
             * All 8 MiB are written using 64 KiB work units.
             */
            expected_blocks_copied: all_blocks,
        },
    ]
}

fn create_source(path: &PathBuf) -> RawDisk<TestExtentBlockDevice> {
    fs::write(path, source_contents()).unwrap();

    RawDisk::new(
        TestExtentBlockDevice::open_read_only(
            path,
            FILE_SIZE as u64,
            source_capabilities(),
            mixed_extent_map(),
        )
        .unwrap(),
    )
}

fn open_source(path: &PathBuf) -> RawDisk<TestExtentBlockDevice> {
    RawDisk::new(
        TestExtentBlockDevice::open_read_only(
            path,
            FILE_SIZE as u64,
            source_capabilities(),
            mixed_extent_map(),
        )
        .unwrap(),
    )
}

fn create_destination(
    path: &PathBuf,
    capabilities: Capabilities,
) -> RawDisk<TestExtentBlockDevice> {
    let device =
        TestExtentBlockDevice::create(path, FILE_SIZE as u64, capabilities, Vec::new()).unwrap();

    /*
     * Start with non-zero contents.
     *
     * This ensures that Zero/Hole handling has observable work to do.
     */
    fs::write(path, vec![0xff_u8; FILE_SIZE]).unwrap();

    RawDisk::new(device)
}

fn cleanup(paths: &[PathBuf]) {
    for path in paths {
        fs::remove_file(path).unwrap();
    }
}

fn run_parity_profile(profile: ParityProfile) {
    let source_path = temporary_path(&format!("{}-source", profile.name,));

    let threaded_path = temporary_path(&format!("{}-threaded", profile.name,));

    let native_path = temporary_path(&format!("{}-native", profile.name,));

    let expected = source_contents();

    /*
     * Both execution engines use independent descriptors over the same
     * deterministic source file and receive the same extent map.
     */
    let threaded_source = create_source(&source_path);

    let native_source = open_source(&source_path);

    let threaded_destination = create_destination(&threaded_path, profile.capabilities);

    let native_destination = create_destination(&native_path, profile.capabilities);

    /*
     * Threaded execution.
     */
    let threaded_mover = DataMover::new(CopyOptions::new(BLOCK_SIZE).unwrap());

    let threaded_report = threaded_mover
        .copy_with_report(&threaded_source, &threaded_destination)
        .unwrap();

    assert_eq!(
        threaded_report.backend(),
        ExecutionBackend::Threaded,
        "{}",
        profile.name,
    );

    /*
     * Native io_uring execution.
     */
    let native_mover = DataMover::with_execution_strategy(
        CopyOptions::new(BLOCK_SIZE).unwrap(),
        ExecutionStrategy::IoUring(IoUringExecutionOptions::new(8).unwrap()),
    );

    let native_report = native_mover
        .copy_with_report(&native_source, &native_destination)
        .unwrap();

    assert_eq!(
        native_report.backend(),
        ExecutionBackend::IoUring,
        "{}",
        profile.name,
    );

    /*
     * Copy deterministic statistics before dropping the disks.
     *
     * elapsed is intentionally excluded from parity checks.
     */
    let threaded_stats = *threaded_report.stats();

    let native_stats = *native_report.stats();

    drop(threaded_destination);
    drop(threaded_source);

    drop(native_destination);
    drop(native_source);

    let threaded_contents = fs::read(&threaded_path).unwrap();

    let native_contents = fs::read(&native_path).unwrap();

    /*
     * Logical destination parity.
     */
    assert_eq!(threaded_contents, expected, "{}", profile.name,);

    assert_eq!(native_contents, expected, "{}", profile.name,);

    assert_eq!(threaded_contents, native_contents, "{}", profile.name,);

    /*
     * Threaded/native semantic statistics parity.
     */
    assert_eq!(
        threaded_stats.bytes_read(),
        native_stats.bytes_read(),
        "{}",
        profile.name,
    );

    assert_eq!(
        threaded_stats.bytes_written(),
        native_stats.bytes_written(),
        "{}",
        profile.name,
    );

    assert_eq!(
        threaded_stats.bytes_zeroed(),
        native_stats.bytes_zeroed(),
        "{}",
        profile.name,
    );

    assert_eq!(
        threaded_stats.bytes_discarded(),
        native_stats.bytes_discarded(),
        "{}",
        profile.name,
    );

    assert_eq!(
        threaded_stats.blocks_copied(),
        native_stats.blocks_copied(),
        "{}",
        profile.name,
    );

    assert_eq!(
        threaded_stats.extents_processed(),
        native_stats.extents_processed(),
        "{}",
        profile.name,
    );

    /*
     * Independent expected accounting.
     *
     * Two Data extents:
     *
     *     2 x 2 MiB = 4 MiB
     */
    assert_eq!(
        native_stats.bytes_read(),
        (2 * EXTENT_SIZE) as u64,
        "{}",
        profile.name,
    );

    assert_eq!(
        native_stats.bytes_written(),
        profile.expected_bytes_written,
        "{}",
        profile.name,
    );

    assert_eq!(
        native_stats.bytes_zeroed(),
        profile.expected_bytes_zeroed,
        "{}",
        profile.name,
    );

    assert_eq!(
        native_stats.bytes_discarded(),
        profile.expected_bytes_discarded,
        "{}",
        profile.name,
    );

    assert_eq!(
        native_stats.blocks_copied(),
        profile.expected_blocks_copied,
        "{}",
        profile.name,
    );

    assert_eq!(native_stats.extents_processed(), 4, "{}", profile.name,);

    cleanup(&[source_path, threaded_path, native_path]);
}

#[test]
fn parity_with_write_zero_and_discard() {
    run_parity_profile(parity_profiles()[0]);
}

#[test]
fn parity_with_write_zero_only() {
    run_parity_profile(parity_profiles()[1]);
}

#[test]
fn parity_with_zero_write_fallback() {
    run_parity_profile(parity_profiles()[2]);
}
