use std::fs::{self, File};
use std::io::{BufWriter, Write};
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

#[allow(dead_code)]
pub fn incompressible_buffer(size: usize, seed: u64) -> Vec<u8> {
    let mut buffer = vec![0_u8; size];

    let mut generator = XorShift64::new(seed);

    generator.fill(&mut buffer);

    buffer
}
