use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyStats {
    bytes_read: u64,
    bytes_written: u64,
    bytes_zeroed: u64,
    bytes_discarded: u64,
    blocks_copied: u64,
    extents_processed: u64,
    elapsed: Duration,
}

impl CopyStats {
    pub(crate) const fn new(
        bytes_read: u64,
        bytes_written: u64,
        bytes_zeroed: u64,
        bytes_discarded: u64,
        blocks_copied: u64,
        extents_processed: u64,
        elapsed: Duration,
    ) -> Self {
        Self {
            bytes_read,
            bytes_written,
            bytes_zeroed,
            bytes_discarded,
            blocks_copied,
            extents_processed,
            elapsed,
        }
    }

    pub const fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    pub const fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    pub const fn bytes_zeroed(&self) -> u64 {
        self.bytes_zeroed
    }

    pub const fn bytes_discarded(&self) -> u64 {
        self.bytes_discarded
    }

    pub const fn blocks_copied(&self) -> u64 {
        self.blocks_copied
    }

    pub const fn extents_processed(&self) -> u64 {
        self.extents_processed
    }

    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn throughput_bytes_per_second(&self) -> f64 {
        let seconds = self.elapsed.as_secs_f64();

        if seconds == 0.0 {
            return 0.0;
        }

        self.bytes_written as f64 / seconds
    }
}
