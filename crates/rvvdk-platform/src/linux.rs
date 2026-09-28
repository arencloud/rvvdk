use std::os::fd::{AsFd, AsRawFd, RawFd};

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

pub trait LinuxFdBackend: AsFd {
    fn raw_fd(&self) -> RawFd {
        self.as_fd().as_raw_fd()
    }

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

/// Facts read from the open descriptor, never from a path or cached geometry.
#[derive(Debug, Clone, Copy)]
pub struct FileState {
    pub regular: bool,
    pub size: u64,
    pub device: u64,
    pub inode: u64,
    pub readable: bool,
    pub writable: bool,
    pub append: bool,
}

pub fn inspect_file(fd: std::os::fd::BorrowedFd<'_>) -> std::io::Result<FileState> {
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    loop {
        // SAFETY: the borrowed FD stays alive and stat points to writable storage.
        if unsafe { libc::fstat(fd.as_raw_fd(), stat.as_mut_ptr()) } == 0 {
            break;
        }
        let error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
    // SAFETY: successful fstat initialized the complete structure.
    let stat = unsafe { stat.assume_init() };
    let flags = loop {
        // SAFETY: F_GETFL takes no third argument and does not modify the FD.
        let flags = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFL) };
        if flags >= 0 {
            break flags;
        }
        let error = std::io::Error::last_os_error();
        if error.kind() != std::io::ErrorKind::Interrupted {
            return Err(error);
        }
    };
    let path_only = flags & libc::O_PATH != 0;
    // stat field widths vary across Linux targets.
    #[allow(clippy::unnecessary_cast)]
    Ok(FileState {
        regular: stat.st_mode & libc::S_IFMT == libc::S_IFREG,
        size: stat.st_size.max(0) as u64,
        device: stat.st_dev as u64,
        inode: stat.st_ino as u64,
        readable: !path_only && matches!(flags & libc::O_ACCMODE, libc::O_RDONLY | libc::O_RDWR),
        writable: !path_only && matches!(flags & libc::O_ACCMODE, libc::O_WRONLY | libc::O_RDWR),
        append: flags & libc::O_APPEND != 0,
    })
}

#[cfg(test)]
mod access_tests {
    use super::*;
    use std::os::fd::{FromRawFd, OwnedFd};

    #[test]
    fn non_read_write_access_mode_is_not_advertised_as_data_access() {
        // SAFETY: the static path is NUL-terminated and O_CREAT is not requested.
        let fd = unsafe { libc::open(c"/dev/null".as_ptr(), libc::O_ACCMODE | libc::O_CLOEXEC) };
        assert!(fd >= 0, "{}", std::io::Error::last_os_error());
        // SAFETY: open returned a new descriptor whose ownership transfers here.
        let fd = unsafe { OwnedFd::from_raw_fd(fd) };
        let state = inspect_file(fd.as_fd()).unwrap();
        assert!(!state.readable);
        assert!(!state.writable);
    }
}
