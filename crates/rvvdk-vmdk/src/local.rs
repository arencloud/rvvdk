use crate::{BackingError, BackingResolver, DescriptorText, Limits};
use rvvdk_core::BlockDevice;
use rvvdk_local::LocalFileBlockDevice;
use std::{
    ffi::CString,
    fs::File,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::Path,
    sync::Arc,
};

/// Linux confined namespace rooted at a retained directory. Requires openat2 and
/// trusted procfs at /proc/self/fd; unsupported confinement never falls back.
pub struct LocalResolver {
    directory: File,
}
impl LocalResolver {
    /// The caller selects and authorizes this directory, including any hard links
    /// already present. Every subsequent reference is resolved beneath its FD.
    pub fn from_directory(directory: File) -> Result<Self, BackingError> {
        if !directory
            .metadata()
            .map_err(|e| BackingError::io("inspecting anchor", e))?
            .is_dir()
        {
            return Err(BackingError::NotDirectory);
        }
        Ok(Self { directory })
    }

    /// The path/parent are trusted caller input. The descriptor basename and its
    /// references use confined lookup; reference paths are descriptor-relative.
    pub fn open_descriptor(
        path: impl AsRef<Path>,
        limits: Limits,
    ) -> Result<(DescriptorText, Self), BackingError> {
        let (file, resolver) = Self::open_descriptor_file(path)?;
        let text = DescriptorText::read_from(file, limits)?;
        Ok((text, resolver))
    }

    /// Open a confined descriptor and retain its file for caller consistency checks.
    /// The caller must acquire/parse its text with bounded `DescriptorText` APIs.
    pub fn open_descriptor_file(path: impl AsRef<Path>) -> Result<(File, Self), BackingError> {
        let path = path.as_ref();
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or(BackingError::UnsafeReference)?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let directory = File::options()
            .read(true)
            .custom_flags(libc::O_PATH | libc::O_DIRECTORY | libc::O_CLOEXEC)
            .open(parent)
            .map_err(|e| BackingError::io("opening trusted descriptor directory", e))?;
        let resolver = Self::from_directory(directory)?;
        let file = resolver.open_regular(name)?;
        Ok((file, resolver))
    }

    /// Open one regular file through the same confined namespace as `resolve`.
    /// Useful for retaining descriptor-bound metadata observations; never reopen
    /// a reference by joining it to a pathname.
    pub fn open_regular(&self, reference: &str) -> Result<File, BackingError> {
        let name = local_name(reference)?;
        // O_PATH obtains an object identity without opening a FIFO/device for I/O.
        // SAFETY: open_how consists solely of integer fields; zero is valid.
        let mut how: libc::open_how = unsafe { std::mem::zeroed() };
        how.flags = (libc::O_PATH | libc::O_CLOEXEC) as u64;
        how.resolve = libc::RESOLVE_BENEATH | libc::RESOLVE_NO_SYMLINKS | libc::RESOLVE_NO_XDEV;
        // SAFETY: valid NUL-terminated name, live directory FD and fully initialized
        // open_how with its ABI size. A successful FD is immediately owned below.
        let fd = unsafe {
            libc::syscall(
                libc::SYS_openat2,
                self.directory.as_raw_fd(),
                name.as_ptr(),
                &how,
                std::mem::size_of::<libc::open_how>(),
            )
        };
        if fd < 0 {
            return Err(BackingError::io(
                "confined lookup (openat2 required)",
                std::io::Error::last_os_error(),
            ));
        }
        // SAFETY: syscall returned a new owned FD, not adopted anywhere else.
        let pinned = unsafe { File::from_raw_fd(fd as i32) };
        reopen_regular(pinned)
    }
}
impl BackingResolver for LocalResolver {
    fn resolve(&self, reference: &str) -> Result<Arc<dyn BlockDevice>, BackingError> {
        let file = self.open_regular(reference)?;
        let device = LocalFileBlockDevice::from_buffered_file(file).map_err(BackingError::Adopt)?;
        Ok(Arc::new(device))
    }
}
fn local_name(reference: &str) -> Result<CString, BackingError> {
    // Reject other-platform/URI syntax too, even where Unix would treat it literally.
    if reference.is_empty()
        || reference.len() > 4096
        || reference.chars().any(|c| c.is_control())
        || reference.contains(['\\', ':'])
        || reference
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err(BackingError::UnsafeReference);
    }
    CString::new(reference).map_err(|_| BackingError::UnsafeReference)
}
fn reopen_regular(pinned: File) -> Result<File, BackingError> {
    let before = pinned
        .metadata()
        .map_err(|e| BackingError::io("inspecting pinned backing", e))?;
    if !before.is_file() {
        return Err(BackingError::NotRegular);
    }
    // This is our live FD, not an untrusted descriptor pathname. Holding O_PATH
    // pins the inode across rename/unlink; the original name is never reopened.
    let file = File::options()
        .read(true)
        .custom_flags(libc::O_CLOEXEC)
        .open(format!("/proc/self/fd/{}", pinned.as_raw_fd()))
        .map_err(|e| BackingError::io("opening pinned regular backing through procfs", e))?;
    let after = file
        .metadata()
        .map_err(|e| BackingError::io("inspecting opened backing", e))?;
    if !after.is_file() || (before.dev(), before.ino()) != (after.dev(), after.ino()) {
        return Err(BackingError::LocalIdentityChanged);
    }
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::os::unix::fs::symlink;

    #[test]
    fn replacement_between_pin_and_reopen_cannot_redirect_reads() {
        let root = std::env::var_os("RVVDK_TEST_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let path = root.join(format!("rvvdk-pin-reopen-{}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        let original = path.join("original");
        std::fs::write(&original, b"pinned").unwrap();
        let pinned = File::options()
            .read(true)
            .custom_flags(libc::O_PATH | libc::O_CLOEXEC)
            .open(&original)
            .unwrap();
        std::fs::remove_file(&original).unwrap();
        symlink("/etc/passwd", &original).unwrap();
        let mut file = reopen_regular(pinned).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, b"pinned");
        // SAFETY: F_GETFL/F_GETFD inspect this live file descriptor.
        assert_eq!(
            unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) } & libc::O_ACCMODE,
            libc::O_RDONLY
        );
        // SAFETY: same live FD, read-only flag query.
        assert_ne!(
            unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFD) } & libc::FD_CLOEXEC,
            0
        );
        drop(file);
        std::fs::remove_dir_all(path).unwrap();
    }
}
