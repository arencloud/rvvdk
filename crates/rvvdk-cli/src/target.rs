use crate::error::Failure;
use std::{
    ffi::CString,
    fs::File,
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt},
        },
    },
    path::Path,
};

type Result<T> = std::result::Result<T, Failure>;
fn cstring(path: &Path) -> Result<CString> {
    CString::new(path.as_os_str().as_bytes())
        .map_err(|_| Failure::new("invalid_destination", "path contains a NUL byte"))
}
fn openat(directory: &File, name: &CString, flags: i32) -> std::io::Result<File> {
    // SAFETY: directory and NUL-terminated name are live; mode is supplied for O_TMPFILE.
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            flags | libc::O_CLOEXEC,
            0o600 as libc::mode_t,
        )
    };
    if fd < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(unsafe { File::from_raw_fd(fd) })
    } // SAFETY: newly returned owned descriptor.
}
pub(crate) fn source(path: &Path) -> Result<File> {
    let file = File::options()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
        .map_err(|e| Failure::io("open source", e))?;
    if !file
        .metadata()
        .map_err(|e| Failure::io("inspect source", e))?
        .is_file()
    {
        return Err(Failure::new(
            "not_regular_file",
            "source must be a regular file",
        ));
    }
    Ok(file)
}
pub(crate) fn verify_destination(path: &Path) -> Result<File> {
    let file = File::options()
        .read(true)
        .custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW)
        .open(path)
        .map_err(|e| Failure::io("open verification destination", e))?;
    if !file
        .metadata()
        .map_err(|e| Failure::io("inspect destination", e))?
        .is_file()
    {
        return Err(Failure::new(
            "invalid_destination",
            "verification destination must be a regular file",
        ));
    }
    Ok(file)
}
pub(crate) struct Target {
    pub file: File,
    pub existing: bool,
    pub published: bool,
    pub mutation_started: bool,
    publication_uncertain: bool,
    directory: File,
    name: CString,
}
impl Target {
    pub fn open(path: &Path, overwrite: bool, source: &crate::source::Source) -> Result<Self> {
        let bytes = path.as_os_str().as_bytes();
        let leaf = path
            .file_name()
            .filter(|_| !bytes.ends_with(b"/") && !bytes.ends_with(b"/."))
            .ok_or_else(|| Failure::new("invalid_destination", "destination must name a file"))?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let directory = File::options()
            .read(true)
            .custom_flags(libc::O_DIRECTORY)
            .open(parent)
            .map_err(|e| Failure::io("open destination directory", e))?;
        let name = cstring(Path::new(leaf))?;
        let size = source.logical().size();
        let (file, existing) = match openat(&directory, &name, libc::O_PATH | libc::O_NOFOLLOW) {
            Ok(observed) => {
                let observed = observed
                    .metadata()
                    .map_err(|e| Failure::io("inspect destination", e))?;
                if !observed.is_file() {
                    return Err(Failure::new(
                        "invalid_destination",
                        "destination must be a regular file, not a symlink or special file",
                    ));
                }
                source.validate_destination(&observed)?;
                if !overwrite {
                    return Err(Failure::new(
                        "destination_exists",
                        "destination exists; --overwrite is required",
                    ));
                }
                let file = openat(
                    &directory,
                    &name,
                    libc::O_RDWR | libc::O_NOFOLLOW | libc::O_NONBLOCK,
                )
                .map_err(|e| Failure::io("open overwrite destination", e))?;
                let live = file
                    .metadata()
                    .map_err(|e| Failure::io("inspect overwrite descriptor", e))?;
                if !live.is_file() || (live.dev(), live.ino()) != (observed.dev(), observed.ino()) {
                    return Err(Failure::new(
                        "destination_changed",
                        "destination changed while opening",
                    ));
                }
                if live.len() < size {
                    return Err(Failure::new(
                        "destination_too_small",
                        "overwrite destination is smaller than source",
                    ));
                }
                (file, true)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let file = openat(
                    &directory,
                    &CString::new(".").unwrap(),
                    libc::O_RDWR | libc::O_TMPFILE,
                )
                .map_err(|e| {
                    Failure::io(
                        "create anonymous output (filesystem must support O_TMPFILE)",
                        e,
                    )
                })?;
                file.set_len(size)
                    .map_err(|e| Failure::io("size anonymous output", e))?;
                (file, false)
            }
            Err(e) => return Err(Failure::io("inspect destination name", e)),
        };
        Ok(Self {
            file,
            existing,
            published: false,
            mutation_started: false,
            publication_uncertain: false,
            directory,
            name,
        })
    }
    pub fn failure_state(&self) -> &'static str {
        if self.existing {
            if self.mutation_started {
                "existing_may_be_modified"
            } else {
                "existing_unchanged"
            }
        } else if self.published {
            "published"
        } else if self.publication_uncertain {
            "publication_unconfirmed"
        } else {
            "private_unpublished"
        }
    }
    pub fn publish(&mut self) -> Result<()> {
        // Following this owned FD through procfs avoids AT_EMPTY_PATH's capability
        // requirement. The target name is relative to the pinned parent FD.
        let fd_path = CString::new(format!("/proc/self/fd/{}", self.file.as_raw_fd())).unwrap();
        // SAFETY: both strings and the directory descriptor remain live. linkat
        // creates a name only if absent, including when an existing name is a symlink.
        if unsafe {
            libc::linkat(
                libc::AT_FDCWD,
                fd_path.as_ptr(),
                self.directory.as_raw_fd(),
                self.name.as_ptr(),
                libc::AT_SYMLINK_FOLLOW,
            )
        } != 0
        {
            let error = std::io::Error::last_os_error();
            // Filesystem errors can leave namespace effects uncertain. Do not
            // remove a possibly published name or claim rollback.
            self.publication_uncertain = error.raw_os_error() != Some(libc::EEXIST);
            return Err(Failure::io("publish without replacement", error));
        }
        self.published = true;
        Ok(())
    }
    pub fn sync_parent(&self) -> Result<()> {
        self.directory
            .sync_all()
            .map_err(|e| Failure::io("sync destination directory", e))
    }
    pub fn check_name(&self) -> Result<()> {
        let named = openat(&self.directory, &self.name, libc::O_PATH | libc::O_NOFOLLOW)
            .map_err(|e| Failure::io("recheck destination name", e))?;
        let a = named
            .metadata()
            .map_err(|e| Failure::io("inspect destination name", e))?;
        let b = self
            .file
            .metadata()
            .map_err(|e| Failure::io("inspect destination descriptor", e))?;
        if (a.dev(), a.ino()) != (b.dev(), b.ino()) {
            return Err(Failure::new(
                "destination_changed",
                "destination name no longer identifies the opened file",
            ));
        }
        Ok(())
    }
}
