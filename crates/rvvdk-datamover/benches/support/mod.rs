#![allow(dead_code)]

use std::fs::{self, File};
use std::io::{BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const MIB: usize = 1024 * 1024;

pub fn benchmark_directory() -> PathBuf {
    match std::env::var_os("RVVDK_BENCH_DIR") {
        Some(path) => PathBuf::from(path),

        None => std::env::temp_dir(),
    }
}

pub fn benchmark_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock must be after UNIX epoch")
        .as_nanos();

    benchmark_directory().join(format!("rvvdk-bench-{name}-{unique}.img"))
}

pub fn ensure_benchmark_directory() {
    let directory = benchmark_directory();

    fs::create_dir_all(&directory).expect("failed to create benchmark directory");
}

pub fn create_incompressible_file(path: &Path, size: usize, seed: u64) {
    let file = File::create(path).expect("failed to create benchmark file");

    let mut writer = BufWriter::with_capacity(MIB, file);

    let mut generator = XorShift64::new(seed);

    let mut buffer = vec![0_u8; MIB];

    let mut remaining = size;

    while remaining > 0 {
        let length = remaining.min(buffer.len());

        generator.fill(&mut buffer[..length]);

        writer
            .write_all(&buffer[..length])
            .expect("failed to write benchmark data");

        remaining -= length;
    }

    writer.flush().expect("failed to flush benchmark writer");

    let file = writer
        .into_inner()
        .expect("failed to recover benchmark file");

    file.sync_all()
        .expect("failed to synchronize benchmark file");
}

pub fn create_zero_file(path: &Path, size: usize) {
    let file = File::create(path).expect("failed to create destination file");

    file.set_len(size as u64)
        .expect("failed to size destination file");

    file.sync_all()
        .expect("failed to synchronize destination file");
}

pub fn create_sparse_incompressible_file(
    path: &Path,
    size: usize,
    extent_size: usize,
    data_extents_per_group: usize,
    extents_per_group: usize,
    seed: u64,
) {
    assert!(extent_size > 0, "extent size must be non-zero");

    assert!(extents_per_group > 0, "extent group size must be non-zero");

    assert!(
        data_extents_per_group <= extents_per_group,
        "data extents cannot exceed group size"
    );

    let mut file = File::create(path).expect("failed to create sparse benchmark file");

    /*
     * Establish the complete logical disk size without materializing
     * all blocks.
     *
     * Ranges that are never written remain filesystem holes.
     */
    file.set_len(size as u64)
        .expect("failed to size sparse benchmark file");

    let mut generator = XorShift64::new(seed);

    let mut buffer = vec![0_u8; extent_size];

    let extent_count = size.div_ceil(extent_size);

    for extent_index in 0..extent_count {
        let group_index = extent_index % extents_per_group;

        /*
         * Extents outside the Data portion of the pattern remain
         * sparse holes.
         */
        if group_index >= data_extents_per_group {
            continue;
        }

        let offset = extent_index
            .checked_mul(extent_size)
            .expect("benchmark extent offset overflow");

        let remaining = size - offset;

        let length = remaining.min(extent_size);

        generator.fill(&mut buffer[..length]);

        file.seek(SeekFrom::Start(offset as u64))
            .expect("failed to seek sparse benchmark source");

        file.write_all(&buffer[..length])
            .expect("failed to write sparse benchmark data");
    }

    file.sync_all()
        .expect("failed to synchronize sparse benchmark source");
}

pub fn materialize_file(path: &Path, size: usize, value: u8) {
    let file = File::create(path).expect("failed to create materialized benchmark file");

    let mut writer = BufWriter::with_capacity(MIB, file);

    let buffer = vec![value; MIB];

    let mut remaining = size;

    while remaining > 0 {
        let length = remaining.min(buffer.len());

        writer
            .write_all(&buffer[..length])
            .expect("failed to materialize benchmark file");

        remaining -= length;
    }

    writer
        .flush()
        .expect("failed to flush materialized benchmark file");

    let file = writer
        .into_inner()
        .expect("failed to recover materialized benchmark file");

    file.sync_all()
        .expect("failed to synchronize materialized benchmark file");
}

pub fn remove_file(path: impl AsRef<Path>) {
    fs::remove_file(path).expect("failed to remove benchmark file");
}

struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    const fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x9e37_79b9_7f4a_7c15
            } else {
                seed
            },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;

        value ^= value << 13;

        value ^= value >> 7;

        value ^= value << 17;

        self.state = value;

        value
    }

    fn fill(&mut self, buffer: &mut [u8]) {
        for chunk in buffer.chunks_mut(8) {
            let bytes = self.next_u64().to_le_bytes();

            let length = chunk.len();

            chunk.copy_from_slice(&bytes[..length]);
        }
    }
}

pub fn incompressible_buffer(size: usize, seed: u64) -> Vec<u8> {
    let mut buffer = vec![0_u8; size];

    let mut generator = XorShift64::new(seed);

    generator.fill(&mut buffer);

    buffer
}
