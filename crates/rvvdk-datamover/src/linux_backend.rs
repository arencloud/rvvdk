#![cfg(target_os = "linux")]

use std::os::fd::RawFd;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinuxFdCapabilities {
    direct_io: bool,
    memory_alignment: usize,
    offset_alignment: usize,
}

impl LinuxFdCapabilities {
    pub const fn new(direct_io: bool, memory_alignment: usize, offset_alignment: usize) -> Self {
        Self {
            direct_io,
            memory_alignment,
            offset_alignment,
        }
    }

    pub const fn direct_io(&self) -> bool {
        self.direct_io
    }

    pub const fn memory_alignment(&self) -> usize {
        self.memory_alignment
    }

    pub const fn offset_alignment(&self) -> usize {
        self.offset_alignment
    }
}

pub trait LinuxFdBackend {
    fn raw_fd(&self) -> RawFd;

    fn linux_fd_capabilities(&self) -> LinuxFdCapabilities;
}
