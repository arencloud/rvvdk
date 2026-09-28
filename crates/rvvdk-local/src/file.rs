use crate::DirectIoAlignment;
use std::fs::File;
#[cfg(target_os = "linux")]
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::path::Path;

use rvvdk_core::{BlockDevice, Capabilities, DiskGeometry, Error, Extent, ExtentKind, Result};

#[cfg(target_os = "linux")]
use rvvdk_platform::{LinuxFdBackend, LinuxFdCapabilities};

pub struct LocalFileBlockDevice {
    file: File,
    buffered_file: Option<File>,
    geometry: DiskGeometry,
    capabilities: Capabilities,
    direct_io: bool,
    direct_io_alignment: Option<DirectIoAlignment>,
}

impl LocalFileBlockDevice {
    fn buffered_file(&self) -> &File {
        self.buffered_file.as_ref().unwrap_or(&self.file)
    }
    pub fn open_read_only(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::options().read(true).open(path)?;

        Self::from_file(
            file,
            None,
            Capabilities::READ | Capabilities::EXTENTS | Capabilities::SPARSE,
            false,
        )
    }

    pub fn open_read_write(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::options().read(true).write(true).open(path)?;

        Self::from_file(
            file,
            None,
            Capabilities::READ
                | Capabilities::WRITE
                | Capabilities::FLUSH
                | Capabilities::EXTENTS
                | Capabilities::SPARSE,
            false,
        )
    }

    pub fn open_direct_read_only(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let file = File::options()
            .read(true)
            .custom_flags(libc::O_DIRECT)
            .open(path)?;

        let buffered_file = File::options().read(true).open(path)?;

        Self::from_file(
            file,
            Some(buffered_file),
            Capabilities::READ
                | Capabilities::EXTENTS
                | Capabilities::SPARSE
                | Capabilities::DIRECT_IO,
            true,
        )
    }

    pub fn open_direct_read_write(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let file = File::options()
            .read(true)
            .write(true)
            .custom_flags(libc::O_DIRECT)
            .open(path)?;

        let buffered_file = File::options().read(true).write(true).open(path)?;

        Self::from_file(
            file,
            Some(buffered_file),
            Capabilities::READ
                | Capabilities::WRITE
                | Capabilities::FLUSH
                | Capabilities::EXTENTS
                | Capabilities::SPARSE
                | Capabilities::DIRECT_IO,
            true,
        )
    }

    pub const fn is_direct_io(&self) -> bool {
        self.direct_io
    }

    pub const fn direct_io_alignment(&self) -> Option<DirectIoAlignment> {
        self.direct_io_alignment
    }

    fn is_direct_io_compatible(&self, offset: u64, buffer: &[u8]) -> bool {
        if !self.direct_io {
            return false;
        }

        let Some(alignment) = self.direct_io_alignment else {
            return false;
        };

        let memory_alignment = alignment.memory_alignment();

        let offset_alignment = alignment.offset_alignment();

        let address = buffer.as_ptr() as usize;

        address.is_multiple_of(memory_alignment)
            && offset.is_multiple_of(offset_alignment as u64)
            && buffer.len().is_multiple_of(offset_alignment)
    }

    fn from_file(
        file: File,
        buffered_file: Option<File>,
        capabilities: Capabilities,
        direct_io: bool,
    ) -> Result<Self> {
        let metadata = file.metadata()?;

        if !metadata.is_file() {
            return Err(Error::Unsupported);
        }

        if let Some(buffered) = &buffered_file {
            use std::os::unix::fs::MetadataExt;
            let buffered_metadata = buffered.metadata()?;
            if (metadata.dev(), metadata.ino())
                != (buffered_metadata.dev(), buffered_metadata.ino())
            {
                return Err(Error::EndpointMismatch);
            }
        }

        let geometry = DiskGeometry::new(metadata.len(), 512, 4096)?;

        let direct_io_alignment = if direct_io {
            Some(crate::direct_io::discover_direct_io_alignment(&file)?)
        } else {
            None
        };

        Ok(Self {
            file,
            buffered_file,
            geometry,
            capabilities,
            direct_io,
            direct_io_alignment,
        })
    }
}

impl BlockDevice for LocalFileBlockDevice {
    fn geometry(&self) -> DiskGeometry {
        self.geometry
    }

    fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    #[cfg(target_os = "linux")]
    fn copy_endpoint(&self) -> Result<rvvdk_core::CopyEndpoint> {
        let state = rvvdk_platform::inspect_file(self.file.as_fd())?;
        if !state.regular {
            return Err(Error::InvalidEndpoint {
                reason: "a regular file is required",
            });
        }
        if state.append {
            return Err(Error::InvalidEndpoint {
                reason: "append mode cannot preserve copy offsets",
            });
        }
        let mut capabilities = self.capabilities;
        if !state.readable {
            capabilities.remove(Capabilities::READ);
        }
        if !state.writable {
            capabilities.remove(Capabilities::WRITE | Capabilities::FLUSH);
        }
        Ok(rvvdk_core::CopyEndpoint {
            size: state.size,
            capabilities,
            identity: Some(rvvdk_core::EndpointIdentity::LocalFile {
                device: state.device,
                inode: state.inode,
            }),
        })
    }

    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        if !self.capabilities.contains(Capabilities::READ) {
            return Err(Error::Unsupported);
        }

        self.validate_range(offset, buffer.len())?;

        if self.is_direct_io_compatible(offset, buffer) {
            return Ok(self.file.read_at(buffer, offset)?);
        }

        Ok(self.buffered_file().read_at(buffer, offset)?)
    }

    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize> {
        if !self.capabilities.contains(Capabilities::WRITE) {
            return Err(Error::Unsupported);
        }

        self.validate_range(offset, buffer.len())?;

        if self.is_direct_io_compatible(offset, buffer) {
            return Ok(self.file.write_at(buffer, offset)?);
        }

        Ok(self.buffered_file().write_at(buffer, offset)?)
    }

    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
        let length_usize =
            usize::try_from(length).map_err(|_| Error::RangeOverflow { offset, length })?;

        self.validate_range(offset, length_usize)?;

        if length == 0 {
            return Ok(Vec::new());
        }

        let range_end = offset
            .checked_add(length)
            .ok_or(Error::RangeOverflow { offset, length })?;

        let fd = self.file.as_raw_fd();

        let mut extents = Vec::new();
        let mut position = offset;

        while position < range_end {
            let data_offset = seek_extent(fd, position, libc::SEEK_DATA)?;

            let Some(data_offset) = data_offset else {
                extents.push(Extent::new(
                    position,
                    range_end - position,
                    ExtentKind::Hole,
                )?);

                break;
            };

            let data_offset = data_offset.min(range_end);

            if data_offset > position {
                extents.push(Extent::new(
                    position,
                    data_offset - position,
                    ExtentKind::Hole,
                )?);
            }

            if data_offset >= range_end {
                break;
            }

            let hole_offset = seek_extent(fd, data_offset, libc::SEEK_HOLE)?;

            let hole_offset = hole_offset.unwrap_or(range_end).min(range_end);

            if hole_offset <= data_offset {
                return Err(Error::CorruptMetadata(format!(
                    "invalid sparse extent map: \
                         data offset={data_offset}, \
                         hole offset={hole_offset}"
                )));
            }

            extents.push(Extent::new(
                data_offset,
                hole_offset - data_offset,
                ExtentKind::Data,
            )?);

            position = hole_offset;
        }

        Ok(extents)
    }

    fn flush(&self) -> Result<()> {
        if !self.capabilities.contains(Capabilities::FLUSH) {
            return Err(Error::Unsupported);
        }

        self.file.sync_data()?;

        if let Some(buffered_file) = &self.buffered_file {
            buffered_file.sync_data()?;
        }

        Ok(())
    }
}

fn seek_extent(fd: std::os::fd::RawFd, offset: u64, whence: libc::c_int) -> Result<Option<u64>> {
    let offset =
        libc::off_t::try_from(offset).map_err(|_| Error::RangeOverflow { offset, length: 0 })?;

    // SAFETY:
    // `fd` is obtained from a live `File`.
    // `lseek` does not dereference application pointers.
    // SEEK_DATA and SEEK_HOLE only query the file's
    // allocation map.
    let result = unsafe { libc::lseek(fd, offset, whence) };

    if result >= 0 {
        return Ok(Some(u64::try_from(result).map_err(|_| {
            Error::CorruptMetadata("lseek returned negative-compatible offset".into())
        })?));
    }

    let error = std::io::Error::last_os_error();

    match error.raw_os_error() {
        Some(libc::ENXIO) => Ok(None),

        _ => Err(Error::Io(error)),
    }
}

#[cfg(target_os = "linux")]
impl AsRawFd for LocalFileBlockDevice {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.file.as_raw_fd()
    }
}

#[cfg(target_os = "linux")]
impl AsFd for LocalFileBlockDevice {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.file.as_fd()
    }
}

#[cfg(target_os = "linux")]
impl LinuxFdBackend for LocalFileBlockDevice {
    fn raw_fd(&self) -> std::os::fd::RawFd {
        self.file.as_raw_fd()
    }

    fn linux_fd_capabilities(&self) -> LinuxFdCapabilities {
        match self.direct_io_alignment() {
            Some(alignment) => LinuxFdCapabilities::new(
                true,
                alignment.memory_alignment(),
                alignment.offset_alignment(),
            ),

            None => LinuxFdCapabilities::new(false, 1, 1),
        }
    }
}

#[cfg(test)]
mod preflight_tests {
    use super::*;

    #[test]
    fn rejects_mismatched_direct_and_buffered_handles() {
        let directory = std::env::temp_dir();
        let first = directory.join(format!("rvvdk-r05-pair-{}-a", std::process::id()));
        let second = directory.join(format!("rvvdk-r05-pair-{}-b", std::process::id()));
        let a = File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&first)
            .unwrap();
        let b = File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&second)
            .unwrap();
        std::fs::remove_file(first).unwrap();
        std::fs::remove_file(second).unwrap();
        a.set_len(4096).unwrap();
        b.set_len(4096).unwrap();
        assert!(matches!(
            LocalFileBlockDevice::from_file(a, Some(b), Capabilities::READ, true),
            Err(Error::EndpointMismatch)
        ));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn refreshes_append_flags_before_portable_copy() {
        let path = std::env::temp_dir().join(format!("rvvdk-r05-flags-{}", std::process::id()));
        std::fs::write(&path, [0xff; 4096]).unwrap();
        let device = LocalFileBlockDevice::open_read_write(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        // SAFETY: F_GETFL/F_SETFL operate on this live, uniquely used test FD.
        unsafe {
            let flags = libc::fcntl(device.as_raw_fd(), libc::F_GETFL);
            assert!(flags >= 0);
            assert_eq!(
                libc::fcntl(device.as_raw_fd(), libc::F_SETFL, flags | libc::O_APPEND),
                0
            );
        }
        assert!(matches!(
            device.copy_endpoint(),
            Err(Error::InvalidEndpoint { .. })
        ));
    }
}
