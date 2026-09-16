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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_capabilities() {
        let capabilities = LinuxFdCapabilities::new(true, 4096, 512);

        assert!(capabilities.direct_io());

        assert_eq!(capabilities.memory_alignment(), 4096,);

        assert_eq!(capabilities.offset_alignment(), 512,);
    }
}
