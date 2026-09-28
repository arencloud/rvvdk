use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::sync::Arc;

use rvvdk_core::Result;

/// An owned descriptor shared by queued operations, independently of the caller.
///
/// Duplicate once per endpoint, then cheaply clone the handle for each operation.
#[derive(Clone, Debug)]
pub struct IoUringFile(Arc<OwnedFd>);

impl IoUringFile {
    pub fn new(fd: BorrowedFd<'_>) -> Result<Self> {
        Ok(Self::from(fd.try_clone_to_owned()?))
    }
}

impl From<OwnedFd> for IoUringFile {
    fn from(fd: OwnedFd) -> Self {
        Self(Arc::new(fd))
    }
}

impl AsFd for IoUringFile {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}
