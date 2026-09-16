use rvvdk_platform::LinuxFdBackend;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoUringCompatibility {
    compatible: bool,
    direct_io: bool,
    alignment: usize,
}

impl IoUringCompatibility {
    pub const fn direct_io(&self) -> bool {
        self.direct_io
    }
    pub const fn compatible(&self) -> bool {
        self.compatible
    }

    pub const fn alignment(&self) -> usize {
        self.alignment
    }
}

pub fn evaluate_compatibility<S, D>(source: &S, destination: &D) -> IoUringCompatibility
where
    S: LinuxFdBackend,
    D: LinuxFdBackend,
{
    let source_capabilities = source.linux_fd_capabilities();

    let destination_capabilities = destination.linux_fd_capabilities();

    let alignment = source_capabilities
        .memory_alignment()
        .max(source_capabilities.offset_alignment())
        .max(destination_capabilities.memory_alignment())
        .max(destination_capabilities.offset_alignment());

    let direct_io = source_capabilities.direct_io() && destination_capabilities.direct_io();

    IoUringCompatibility {
        compatible: true,
        direct_io,
        alignment,
    }
}
