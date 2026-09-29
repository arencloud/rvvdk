use rvvdk_core::Result;
use rvvdk_platform::{FileAccess, FileAccessGuard, FileAccessKind};
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::sync::{Arc, OnceLock};

#[derive(Debug)]
struct Access {
    file: Option<FileAccess>,
    direct: bool,
}
#[derive(Debug)]
struct FileInner {
    fd: OwnedFd,
    access: OnceLock<Access>,
}

/// An owned descriptor shared by queued operations, independently of the caller.
/// Regular-file requests join process-local alias admission. The caller must keep
/// descriptor status flags stable; raw syscalls and mmap do not participate.
#[derive(Clone, Debug)]
pub struct IoUringFile(Arc<FileInner>);
impl IoUringFile {
    pub fn new(fd: BorrowedFd<'_>) -> Result<Self> {
        let file = Self::from(fd.try_clone_to_owned()?);
        file.prepare_access()?;
        Ok(file)
    }
    fn prepare_access(&self) -> Result<&Access> {
        if let Some(access) = self.0.access.get() {
            return Ok(access);
        }
        let state = rvvdk_platform::inspect_file(self.as_fd())?;
        let access = Access {
            file: state
                .regular
                .then(|| FileAccess::for_identity(state.device, state.inode)),
            direct: state.direct_io,
        };
        // Concurrent initializers join the same identity; only one cache wins.
        let _ = self.0.access.set(access);
        Ok(self.0.access.get().expect("initialized file access"))
    }
    pub(crate) fn admit(
        &self,
        offset: u64,
        length: usize,
        write: bool,
    ) -> Result<Option<FileAccessGuard>> {
        let access = self.prepare_access()?;
        let kind = match (access.direct, write) {
            (false, false) => FileAccessKind::BufferedRead,
            (false, true) => FileAccessKind::BufferedWrite,
            (true, false) => FileAccessKind::DirectRead,
            (true, true) => FileAccessKind::DirectWrite,
        };
        match &access.file {
            Some(file) => file.try_acquire(offset, length as u64, kind),
            None => Ok(None),
        }
    }
}
impl From<OwnedFd> for IoUringFile {
    /// Ownership conversion remains infallible. Inspection/admission happens on
    /// the first enqueue; use new() for eager inspection during preparation.
    fn from(fd: OwnedFd) -> Self {
        Self(Arc::new(FileInner {
            fd,
            access: OnceLock::new(),
        }))
    }
}
impl AsFd for IoUringFile {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.fd.as_fd()
    }
}
