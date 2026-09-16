use std::ffi::CString;
use std::os::fd::{AsRawFd, RawFd};

use rvvdk_core::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectIoAlignmentSource {
    Statx,
    Fallback,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectIoAlignment {
    memory_alignment: usize,
    offset_alignment: usize,
    source: DirectIoAlignmentSource,
}

impl DirectIoAlignment {
    pub const fn new(
        memory_alignment: usize,
        offset_alignment: usize,
        source: DirectIoAlignmentSource,
    ) -> Self {
        Self {
            memory_alignment,
            offset_alignment,
            source,
        }
    }

    pub const fn source(&self) -> DirectIoAlignmentSource {
        self.source
    }

    pub const fn memory_alignment(&self) -> usize {
        self.memory_alignment
    }

    pub const fn offset_alignment(&self) -> usize {
        self.offset_alignment
    }
}

pub(crate) fn discover_direct_io_alignment(file: &std::fs::File) -> Result<DirectIoAlignment> {
    match statx_direct_io_alignment(file.as_raw_fd())? {
        Some(alignment) => Ok(alignment),

        None => Ok(fallback_alignment()),
    }
}

fn statx_direct_io_alignment(fd: RawFd) -> Result<Option<DirectIoAlignment>> {
    let empty_path = CString::new("").expect("empty CString must be valid");

    // SAFETY:
    //
    // `statx_buffer` is initialized before use.
    // `fd` references a live File owned by the caller.
    // `empty_path` is a valid NUL-terminated string.
    // AT_EMPTY_PATH requests metadata for `fd`.
    let mut statx_buffer: libc::statx = unsafe { std::mem::zeroed() };

    let result = unsafe {
        libc::statx(
            fd,
            empty_path.as_ptr(),
            libc::AT_EMPTY_PATH,
            libc::STATX_DIOALIGN,
            &mut statx_buffer,
        )
    };

    if result != 0 {
        let error = std::io::Error::last_os_error();

        return match error.raw_os_error() {
            Some(libc::EINVAL) | Some(libc::ENOSYS) | Some(libc::EOPNOTSUPP) => Ok(None),

            _ => Err(Error::Io(error)),
        };
    }

    if statx_buffer.stx_mask & libc::STATX_DIOALIGN == 0 {
        return Ok(None);
    }

    let memory_alignment = usize::try_from(statx_buffer.stx_dio_mem_align).map_err(|_| {
        Error::CorruptMetadata("direct I/O memory alignment does not fit usize".into())
    })?;

    let offset_alignment = usize::try_from(statx_buffer.stx_dio_offset_align).map_err(|_| {
        Error::CorruptMetadata("direct I/O offset alignment does not fit usize".into())
    })?;

    if memory_alignment == 0 || offset_alignment == 0 {
        return Ok(None);
    }

    Ok(Some(DirectIoAlignment::new(
        memory_alignment,
        offset_alignment,
        DirectIoAlignmentSource::Statx,
    )))
}

fn fallback_alignment() -> DirectIoAlignment {
    DirectIoAlignment::new(4096, 4096, DirectIoAlignmentSource::Fallback)
}
