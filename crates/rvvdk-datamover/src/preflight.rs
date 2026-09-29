use rvvdk_core::{Capabilities, CopyEndpoint, Error, Result, VirtualDisk};

pub(crate) fn context(endpoint: &'static str, source: Error) -> Error {
    Error::EndpointPreflight {
        endpoint,
        source: Box::new(source),
    }
}

fn require(info: CopyEndpoint, flag: Capabilities, capability: &'static str) -> Result<()> {
    if !info.capabilities.contains(flag) {
        return Err(Error::MissingCapability { capability });
    }
    Ok(())
}

fn bounds(info: CopyEndpoint, offset: u64, length: u64) -> Result<()> {
    let end = offset
        .checked_add(length)
        .ok_or(Error::RangeOverflow { offset, length })?;
    if end > info.size {
        return Err(Error::OutOfBounds {
            offset,
            length,
            size: info.size,
        });
    }
    Ok(())
}

pub(crate) fn pair(
    source: CopyEndpoint,
    destination: CopyEndpoint,
    offset: u64,
    length: u64,
    flush: bool,
) -> Result<()> {
    (|| {
        require(source, Capabilities::READ, "read")?;
        bounds(source, offset, length)
    })()
    .map_err(|error| context("source", error))?;
    (|| {
        require(destination, Capabilities::WRITE, "write")?;
        if flush {
            require(destination, Capabilities::FLUSH, "flush")?;
        }
        bounds(destination, offset, length)
    })()
    .map_err(|error| context("destination", error))?;
    if source.identity.is_some() && source.identity == destination.identity {
        return Err(Error::AliasedEndpoints);
    }
    Ok(())
}

pub(crate) fn virtual_pair<S: VirtualDisk + ?Sized, D: VirtualDisk + ?Sized>(
    source: &S,
    destination: &D,
    length: u64,
) -> Result<(CopyEndpoint, CopyEndpoint)> {
    // Also detect the same non-ZST object when a custom backend has no identity.
    if std::mem::size_of_val(source) != 0
        && std::mem::size_of_val(destination) != 0
        && std::ptr::from_ref(source).cast::<()>() == std::ptr::from_ref(destination).cast::<()>()
    {
        return Err(Error::AliasedEndpoints);
    }
    let source_info = source.copy_endpoint().map_err(|e| context("source", e))?;
    source_size(source_info, length)?;
    let destination_info = destination
        .copy_endpoint()
        .map_err(|e| context("destination", e))?;
    pair(source_info, destination_info, 0, length, true)?;
    Ok((source_info, destination_info))
}

fn source_size(source_info: CopyEndpoint, length: u64) -> Result<()> {
    if source_info.size != length {
        return Err(context(
            "source",
            Error::EndpointChanged(format!(
                "source size changed: expected={length}, current={}",
                source_info.size
            )),
        ));
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub(crate) fn file(fd: std::os::fd::BorrowedFd<'_>, role: &'static str) -> Result<CopyEndpoint> {
    let state = rvvdk_platform::inspect_file(fd).map_err(|e| context(role, e.into()))?;
    file_state(state, role)
}

#[cfg(target_os = "linux")]
fn file_state(state: rvvdk_platform::FileState, role: &'static str) -> Result<CopyEndpoint> {
    (|| {
        if !state.regular {
            return Err(Error::InvalidEndpoint {
                reason: "native copy requires a regular file",
            });
        }
        if state.append {
            return Err(Error::InvalidEndpoint {
                reason: "append mode cannot preserve copy offsets",
            });
        }
        let mut capabilities = Capabilities::empty();
        if state.readable {
            capabilities |= Capabilities::READ;
        }
        if state.writable {
            capabilities |= Capabilities::WRITE | Capabilities::FLUSH;
        }
        Ok(CopyEndpoint {
            size: state.size,
            capabilities,
            identity: Some(rvvdk_core::EndpointIdentity::LocalFile {
                device: state.device,
                inode: state.inode,
            }),
        })
    })()
    .map_err(|e| context(role, e))
}

#[cfg(target_os = "linux")]
pub(crate) fn files(
    source: std::os::fd::BorrowedFd<'_>,
    destination: std::os::fd::BorrowedFd<'_>,
    offset: u64,
    length: u64,
) -> Result<()> {
    pair(
        file(source, "source")?,
        file(destination, "destination")?,
        offset,
        length,
        false,
    )
}

#[cfg(target_os = "linux")]
pub(crate) struct RawEndpoints {
    source: CopyEndpoint,
    destination: CopyEndpoint,
    source_fd: CopyEndpoint,
    destination_fd: CopyEndpoint,
    direct_modes: (bool, bool),
}

#[cfg(target_os = "linux")]
impl RawEndpoints {
    pub(crate) fn validate_io_modes(&self, declared: (bool, bool)) -> Result<()> {
        for (role, actual, expected) in [
            ("source", self.direct_modes.0, declared.0),
            ("destination", self.direct_modes.1, declared.1),
        ] {
            if actual != expected {
                return Err(context(
                    role,
                    Error::EndpointChanged(
                        "descriptor O_DIRECT disagrees with backend capabilities".into(),
                    ),
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn validate_native_binding(&self) -> Result<()> {
        for (role, backend, descriptor) in [
            ("source", self.source, self.source_fd),
            ("destination", self.destination, self.destination_fd),
        ] {
            if backend.identity.is_none() {
                return Err(context(
                    role,
                    Error::InvalidEndpoint {
                        reason: "native Data execution requires known backing identity",
                    },
                ));
            }
            if backend.identity != descriptor.identity {
                return Err(context(role, Error::EndpointMismatch));
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn raw_pair<S, D>(
    source: &rvvdk_core::RawDisk<S>,
    destination: &rvvdk_core::RawDisk<D>,
    length: u64,
) -> Result<RawEndpoints>
where
    S: rvvdk_core::BlockDevice + rvvdk_platform::LinuxFdBackend,
    D: rvvdk_core::BlockDevice + rvvdk_platform::LinuxFdBackend,
{
    if std::mem::size_of_val(source) != 0
        && std::mem::size_of_val(destination) != 0
        && std::ptr::from_ref(source).cast::<()>() == std::ptr::from_ref(destination).cast::<()>()
    {
        return Err(Error::AliasedEndpoints);
    }
    let (source_info, source_fd, source_direct) = raw_endpoint(source.device(), "source")?;
    source_size(source_info, length)?;
    let (destination_info, destination_fd, destination_direct) =
        raw_endpoint(destination.device(), "destination")?;
    pair(source_info, destination_info, 0, length, true)?;
    pair(source_fd, destination_fd, 0, length, false)?;
    Ok(RawEndpoints {
        source: source_info,
        destination: destination_info,
        source_fd,
        destination_fd,
        direct_modes: (source_direct, destination_direct),
    })
}

#[cfg(target_os = "linux")]
fn raw_endpoint<B>(backend: &B, role: &'static str) -> Result<(CopyEndpoint, CopyEndpoint, bool)>
where
    B: rvvdk_core::BlockDevice + rvvdk_platform::LinuxFdBackend,
{
    let inspection = rvvdk_platform::FileInspection::new(backend.as_fd())
        .map_err(|e| context(role, e.into()))?;
    let logical = backend
        .copy_endpoint_from_inspection(&inspection)
        .map_err(|e| context(role, e))?;
    let descriptor = file_state(inspection.state(), role)?;
    Ok((logical, descriptor, inspection.state().direct_io))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rvvdk_core::{DiskGeometry, Extent, MemoryBlockDevice, RawDisk};

    struct FailedMetadata;
    impl VirtualDisk for FailedMetadata {
        fn geometry(&self) -> DiskGeometry {
            DiskGeometry::new(4096, 512, 4096).unwrap()
        }
        fn capabilities(&self) -> Capabilities {
            Capabilities::READ
        }
        fn copy_endpoint(&self) -> Result<CopyEndpoint> {
            Err(Error::Io(std::io::Error::from_raw_os_error(13)))
        }
        fn read_at(&self, _: u64, _: &mut [u8]) -> Result<usize> {
            panic!("preflight must reject before reading")
        }
        fn write_at(&self, _: u64, _: &[u8]) -> Result<usize> {
            unreachable!()
        }
        fn write_zero_at(&self, _: u64, _: u64) -> Result<()> {
            unreachable!()
        }
        fn discard(&self, _: u64, _: u64) -> Result<()> {
            unreachable!()
        }
        fn flush(&self) -> Result<()> {
            unreachable!()
        }
        fn extents(&self, _: u64, _: u64) -> Result<Vec<Extent>> {
            panic!("preflight must precede extent query")
        }
    }

    #[test]
    fn copy_preserves_metadata_error_context_and_errno() {
        use std::error::Error as _;
        let destination = RawDisk::new(MemoryBlockDevice::new(4096).unwrap());
        let error = crate::DataMover::new(crate::CopyOptions::default())
            .copy(&FailedMetadata, &destination)
            .unwrap_err();
        assert!(error.source().unwrap().source().is_some());
        assert!(
            matches!(error, Error::EndpointPreflight { endpoint: "source", source } if matches!(*source, Error::Io(ref cause) if cause.raw_os_error() == Some(13)))
        );
    }
}
