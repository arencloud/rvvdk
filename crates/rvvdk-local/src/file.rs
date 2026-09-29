use crate::DirectIoAlignment;
mod discovery;
#[cfg(target_os = "linux")]
mod sparse;
use std::fs::File;
#[cfg(target_os = "linux")]
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::os::unix::fs::{FileExt, OpenOptionsExt};
use std::path::Path;

use rvvdk_core::{BlockDevice, Capabilities, DiskGeometry, Error, Extent, Result};

#[cfg(target_os = "linux")]
use rvvdk_platform::{LinuxFdBackend, LinuxFdCapabilities};

pub struct LocalFileBlockDevice {
    access: rvvdk_platform::FileAccess,
    file: File,
    buffered_file: Option<File>,
    geometry: DiskGeometry,
    capabilities: Capabilities,
    direct_io: bool,
    direct_io_alignment: Option<DirectIoAlignment>,
    #[cfg(target_os = "linux")]
    sparse_unsupported: std::sync::atomic::AtomicU8,
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

        #[cfg(target_os = "linux")]
        let capabilities = if capabilities.contains(Capabilities::WRITE) {
            capabilities
                | Capabilities::WRITE_ZERO
                | Capabilities::DISCARD
                | Capabilities::DISCARD_ZEROES
        } else {
            capabilities
        };

        use std::os::unix::fs::MetadataExt;
        Ok(Self {
            access: rvvdk_platform::FileAccess::for_identity(metadata.dev(), metadata.ino()),
            #[cfg(target_os = "linux")]
            sparse_unsupported: std::sync::atomic::AtomicU8::new(0),
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
        let inspection = rvvdk_platform::FileInspection::new(self.file.as_fd())?;
        self.copy_endpoint_from_inspection(&inspection)
    }

    fn read_at(&self, offset: u64, buffer: &mut [u8]) -> Result<usize> {
        if !self.capabilities.contains(Capabilities::READ) {
            return Err(Error::Unsupported);
        }

        self.validate_range(offset, buffer.len())?;

        let direct = self.is_direct_io_compatible(offset, buffer);
        let kind = if direct {
            rvvdk_platform::FileAccessKind::DirectRead
        } else {
            rvvdk_platform::FileAccessKind::BufferedRead
        };
        let _access = self.access.try_acquire(offset, buffer.len() as u64, kind)?;
        if direct {
            return Ok(self.file.read_at(buffer, offset)?);
        }
        Ok(self.buffered_file().read_at(buffer, offset)?)
    }

    fn write_at(&self, offset: u64, buffer: &[u8]) -> Result<usize> {
        if !self.capabilities.contains(Capabilities::WRITE) {
            return Err(Error::Unsupported);
        }

        self.validate_range(offset, buffer.len())?;

        let direct = self.is_direct_io_compatible(offset, buffer);
        let kind = if direct {
            rvvdk_platform::FileAccessKind::DirectWrite
        } else {
            rvvdk_platform::FileAccessKind::BufferedWrite
        };
        let _access = self.access.try_acquire(offset, buffer.len() as u64, kind)?;
        if direct {
            return Ok(self.file.write_at(buffer, offset)?);
        }
        Ok(self.buffered_file().write_at(buffer, offset)?)
    }

    #[cfg(target_os = "linux")]
    fn write_zero_at(&self, offset: u64, length: u64) -> Result<()> {
        self.sparse_operation(sparse::Operation::Zero, offset, length)
    }

    #[cfg(target_os = "linux")]
    fn discard(&self, offset: u64, length: u64) -> Result<()> {
        self.sparse_operation(sparse::Operation::Punch, offset, length)
    }

    fn extents(&self, offset: u64, length: u64) -> Result<Vec<Extent>> {
        self.discover_extents(offset, length, |position, whence| {
            discovery::seek(self.file.as_raw_fd(), position, whence)
        })
    }

    fn flush(&self) -> Result<()> {
        if !self.capabilities.contains(Capabilities::FLUSH) {
            return Err(Error::Unsupported);
        }

        let _access = self
            .access
            .try_acquire(0, 0, rvvdk_platform::FileAccessKind::Flush)?;
        self.file.sync_data()?;

        if let Some(buffered_file) = &self.buffered_file {
            buffered_file.sync_data()?;
        }

        Ok(())
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
    fn copy_endpoint_from_inspection(
        &self,
        inspection: &rvvdk_platform::FileInspection<'_>,
    ) -> Result<rvvdk_core::CopyEndpoint> {
        if !inspection.is_for(self.file.as_fd()) {
            return Err(Error::EndpointMismatch);
        }
        let state = inspection.state();
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
            capabilities.remove(
                Capabilities::WRITE
                    | Capabilities::FLUSH
                    | Capabilities::WRITE_ZERO
                    | Capabilities::DISCARD
                    | Capabilities::DISCARD_ZEROES,
            );
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
    #[cfg(target_os = "linux")]
    fn inspection_rejects_other_descriptor_even_for_same_inode() {
        let path = std::env::temp_dir().join(format!("rvvdk-r13-binding-{}", std::process::id()));
        std::fs::write(&path, [0x5a; 4096]).unwrap();
        let device = LocalFileBlockDevice::open_read_write(&path).unwrap();
        let alias = File::open(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        let inspection = rvvdk_platform::FileInspection::new(alias.as_fd()).unwrap();
        assert!(matches!(
            device.copy_endpoint_from_inspection(&inspection),
            Err(Error::EndpointMismatch)
        ));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn inspection_preserves_logical_restrictions_and_current_capacity() {
        let path = std::env::temp_dir().join(format!("rvvdk-r13-access-{}", std::process::id()));
        std::fs::write(&path, [0x5a; 4096]).unwrap();
        let file = File::options().read(true).write(true).open(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        let device =
            LocalFileBlockDevice::from_file(file, None, Capabilities::READ, false).unwrap();
        device.file.set_len(8192).unwrap();
        let inspection = rvvdk_platform::FileInspection::new(device.as_fd()).unwrap();
        assert!(inspection.state().writable);
        let endpoint = device.copy_endpoint_from_inspection(&inspection).unwrap();
        assert_eq!(endpoint.capabilities, Capabilities::READ);
        assert_eq!(endpoint.size, 8192);
        assert_eq!(device.geometry().size(), 4096);
    }

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
        let before = rvvdk_platform::FileInspection::new(device.as_fd()).unwrap();
        assert!(!before.state().append);
        // SAFETY: F_GETFL/F_SETFL operate on this live, uniquely used test FD.
        unsafe {
            let flags = libc::fcntl(device.as_raw_fd(), libc::F_GETFL);
            assert!(flags >= 0);
            assert_eq!(
                libc::fcntl(device.as_raw_fd(), libc::F_SETFL, flags | libc::O_APPEND),
                0
            );
        }
        let after = rvvdk_platform::FileInspection::new(device.as_fd()).unwrap();
        assert!(after.state().append);
        assert!(matches!(
            device.copy_endpoint_from_inspection(&after),
            Err(Error::InvalidEndpoint { .. })
        ));
        assert!(matches!(
            device.copy_endpoint(),
            Err(Error::InvalidEndpoint { .. })
        ));
    }
}
