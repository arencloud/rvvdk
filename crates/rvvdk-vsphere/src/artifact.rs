//! Linux descriptor-bound, private staging and no-replace directory publication.
use crate::{Error, Result};
use std::{
    ffi::CString,
    fs::{File, OpenOptions},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{ffi::OsStrExt, fs::OpenOptionsExt},
    },
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub(crate) struct Artifact {
    parent: File,
    dir: File,
    temp: CString,
    final_name: CString,
    files: Vec<CString>,
    pub(crate) published: bool,
    removed: bool,
}
impl Artifact {
    pub(crate) fn create(path: &Path, max_bytes: u64) -> Result<Self> {
        let name = path.file_name().ok_or(Error::Artifact)?;
        let final_name = CString::new(name.as_bytes()).map_err(|_| Error::Artifact)?;
        if name == "." || name == ".." {
            return Err(Error::Artifact);
        }
        let parent = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(
                path.parent()
                    .filter(|p| !p.as_os_str().is_empty())
                    .unwrap_or(Path::new(".")),
            )
            .map_err(|_| Error::Artifact)?;
        // SAFETY: output pointers refer to initialized-sized writable storage.
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        let exists = unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                final_name.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if exists == 0 || std::io::Error::last_os_error().raw_os_error() != Some(libc::ENOENT) {
            return Err(Error::Artifact);
        }
        let mut space = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        // SAFETY: valid directory descriptor and output storage.
        if unsafe { libc::fstatvfs(parent.as_raw_fd(), space.as_mut_ptr()) } != 0 {
            return Err(Error::Artifact);
        }
        let space = unsafe { space.assume_init() };
        if (space.f_bavail as u128) * (space.f_frsize as u128) < max_bytes as u128 + 1024 * 1024 {
            return Err(Error::Artifact);
        }
        for _ in 0..128 {
            let temp = CString::new(format!(
                ".rvddk-export-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ))
            .map_err(|_| Error::Artifact)?;
            // SAFETY: valid directory descriptor and NUL-terminated single component.
            if unsafe { libc::mkdirat(parent.as_raw_fd(), temp.as_ptr(), 0o700) } != 0 {
                if std::io::Error::last_os_error().raw_os_error() == Some(libc::EEXIST) {
                    continue;
                }
                return Err(Error::Artifact);
            }
            let fd = unsafe {
                libc::openat(
                    parent.as_raw_fd(),
                    temp.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            };
            if fd < 0 {
                unsafe { libc::unlinkat(parent.as_raw_fd(), temp.as_ptr(), libc::AT_REMOVEDIR) };
                return Err(Error::Artifact);
            }
            let dir = unsafe { File::from_raw_fd(fd) };
            return Ok(Self {
                parent,
                dir,
                temp,
                final_name,
                files: Vec::new(),
                published: false,
                removed: false,
            });
        }
        Err(Error::Artifact)
    }
    pub(crate) fn file(&mut self, name: &str) -> Result<tokio::fs::File> {
        if name.contains('/') || name == "." || name == ".." {
            return Err(Error::Artifact);
        }
        let name = CString::new(name).map_err(|_| Error::Artifact)?;
        // SAFETY: valid pinned directory, exclusive creation, fixed private mode.
        let fd = unsafe {
            libc::openat(
                self.dir.as_raw_fd(),
                name.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        if fd < 0 {
            return Err(Error::Artifact);
        }
        self.files.push(name);
        Ok(tokio::fs::File::from_std(unsafe { File::from_raw_fd(fd) }))
    }
    pub(crate) fn publish(&mut self) -> Result<()> {
        self.dir.sync_all().map_err(|_| Error::Artifact)?;
        // SAFETY: both names are single components in the same pinned parent.
        if unsafe {
            libc::renameat2(
                self.parent.as_raw_fd(),
                self.temp.as_ptr(),
                self.parent.as_raw_fd(),
                self.final_name.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        } != 0
        {
            return Err(Error::Artifact);
        }
        self.published = true;
        self.parent.sync_all().map_err(|_| Error::Artifact)
    }
    pub(crate) fn discard(&mut self) -> Result<()> {
        if self.published || self.removed {
            return Ok(());
        }
        let mut failed = false;
        for name in &self.files {
            // Only files created by this instance, relative to its pinned directory.
            if unsafe { libc::unlinkat(self.dir.as_raw_fd(), name.as_ptr(), 0) } != 0
                && std::io::Error::last_os_error().raw_os_error() != Some(libc::ENOENT)
            {
                failed = true;
            }
        }
        if unsafe {
            libc::unlinkat(
                self.parent.as_raw_fd(),
                self.temp.as_ptr(),
                libc::AT_REMOVEDIR,
            )
        } != 0
        {
            failed = true;
        }
        self.removed = !failed;
        if failed { Err(Error::Artifact) } else { Ok(()) }
    }
}
impl Drop for Artifact {
    fn drop(&mut self) {
        let _ = self.discard();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn publication_race_preserves_existing_destination_and_private_modes() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!("rvddk-publish-test-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let output = root.join("artifact");
        let mut artifact = Artifact::create(&output, 1024).unwrap();
        assert_eq!(
            artifact.dir.metadata().unwrap().permissions().mode() & 0o777,
            0o700
        );
        std::fs::create_dir(&output).unwrap();
        std::fs::write(output.join("sentinel"), b"keep").unwrap();
        assert_eq!(artifact.publish(), Err(Error::Artifact));
        assert!(!artifact.published);
        artifact.discard().unwrap();
        assert_eq!(std::fs::read(output.join("sentinel")).unwrap(), b"keep");
        assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
        std::fs::remove_dir_all(root).unwrap();
    }
}
