#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NativeCopyStats {
    bytes_read: u64,
    bytes_written: u64,
    blocks_completed: u64,
}

impl NativeCopyStats {
    pub const fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    pub const fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    pub const fn blocks_completed(&self) -> u64 {
        self.blocks_completed
    }
}

#[cfg(target_os = "linux")]
impl From<crate::io_uring::IoUringCopyStats> for NativeCopyStats {
    fn from(stats: crate::io_uring::IoUringCopyStats) -> Self {
        Self {
            bytes_read: stats.bytes_read(),
            bytes_written: stats.bytes_written(),
            blocks_completed: stats.blocks_completed(),
        }
    }
}

use crate::ExecutionBackend;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeCopyReport {
    backend: ExecutionBackend,
    stats: NativeCopyStats,
}

impl NativeCopyReport {
    pub const fn backend(&self) -> ExecutionBackend {
        self.backend
    }

    pub const fn stats(&self) -> &NativeCopyStats {
        &self.stats
    }

    pub(crate) const fn new(backend: ExecutionBackend, stats: NativeCopyStats) -> Self {
        Self { backend, stats }
    }
}
