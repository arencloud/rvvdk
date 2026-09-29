use std::os::fd::{AsFd, AsRawFd};
use std::os::unix::fs::FileExt;
use std::sync::atomic::Ordering;

use rvvdk_core::{BlockDevice, Capabilities, Error, Result};

use super::LocalFileBlockDevice;

// Shared read-only storage; fallback does not allocate per operation or worker.
static ZEROES: [u8; 64 * 1024] = [0; 64 * 1024];

#[derive(Clone, Copy)]
pub(super) enum Operation {
    Zero,
    Punch,
}
impl Operation {
    fn mode(self) -> i32 {
        libc::FALLOC_FL_KEEP_SIZE
            | match self {
                Self::Zero => libc::FALLOC_FL_ZERO_RANGE,
                Self::Punch => libc::FALLOC_FL_PUNCH_HOLE,
            }
    }
    fn bit(self) -> u8 {
        match self {
            Self::Zero => 1,
            Self::Punch => 2,
        }
    }
}

impl LocalFileBlockDevice {
    pub(super) fn sparse_operation(
        &self,
        operation: Operation,
        offset: u64,
        length: u64,
    ) -> Result<()> {
        self.sparse_with(operation, offset, length, |mode, offset, length| {
            // SAFETY: the live File owns this FD; checked positive offsets and
            // lengths fit off_t. fallocate takes no application pointers. Only
            // ZERO_RANGE/PUNCH_HOLE with KEEP_SIZE are ever passed.
            if unsafe { libc::fallocate(self.buffered_file().as_raw_fd(), mode, offset, length) }
                == 0
            {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        })
    }

    fn sparse_with(
        &self,
        operation: Operation,
        offset: u64,
        length: u64,
        mut allocate: impl FnMut(i32, libc::off_t, libc::off_t) -> std::io::Result<()>,
    ) -> Result<()> {
        if !self.capabilities.contains(Capabilities::WRITE) {
            return Err(Error::Unsupported);
        }
        let end = offset
            .checked_add(length)
            .ok_or(Error::RangeOverflow { offset, length })?;
        if end > self.geometry.size() {
            return Err(Error::OutOfBounds {
                offset,
                length,
                size: self.geometry.size(),
            });
        }
        let _access = self.access.try_acquire(
            offset,
            length,
            rvvdk_platform::FileAccessKind::BufferedWrite,
        )?;
        // Range/access checks apply even to empty requests. Neither KEEP_SIZE nor
        // cached geometry alone protects against a file truncated since open.
        let endpoint = self.copy_endpoint()?;
        if !endpoint.capabilities.contains(Capabilities::WRITE) {
            return Err(Error::Unsupported);
        }
        if end > endpoint.size {
            return Err(Error::OutOfBounds {
                offset,
                length,
                size: endpoint.size,
            });
        }
        if let Some(buffered) = &self.buffered_file {
            let inspection = rvvdk_platform::FileInspection::new(buffered.as_fd())?;
            let state = inspection.state();
            if !state.regular || state.append || !state.writable {
                return Err(Error::InvalidEndpoint {
                    reason: "sparse operations require a writable regular non-append buffered descriptor",
                });
            }
        }
        let native_offset =
            libc::off_t::try_from(offset).map_err(|_| Error::RangeOverflow { offset, length })?;
        let native_length =
            libc::off_t::try_from(length).map_err(|_| Error::RangeOverflow { offset, length })?;
        // Check the endpoint as well as individual syscall arguments.
        libc::off_t::try_from(end).map_err(|_| Error::RangeOverflow { offset, length })?;
        if length == 0 {
            return Ok(());
        }
        if self.sparse_unsupported.load(Ordering::Relaxed) & operation.bit() == 0 {
            loop {
                match allocate(operation.mode(), native_offset, native_length) {
                    Ok(()) => return Ok(()),
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(error)
                        if matches!(
                            error.raw_os_error(),
                            Some(libc::EOPNOTSUPP | libc::ENOSYS)
                        ) =>
                    {
                        // Cache per operation on this open file. Logical support
                        // remains available through bounded writes.
                        self.sparse_unsupported
                            .fetch_or(operation.bit(), Ordering::Relaxed);
                        break;
                    }
                    Err(error) => return Err(Error::Io(error)),
                }
            }
        }
        let mut current = offset;
        while current < end {
            let count = (end - current).min(ZEROES.len() as u64) as usize;
            self.buffered_file()
                .write_all_at(&ZEROES[..count], current)?;
            current += count as u64;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn fixture() -> LocalFileBlockDevice {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "rvvdk-r22-fault-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, vec![0xa5; 200_003]).unwrap();
        let disk = LocalFileBlockDevice::open_read_write(&path).unwrap();
        std::fs::remove_file(path).unwrap();
        disk
    }
    fn check(disk: &LocalFileBlockDevice, start: usize, end: usize) {
        let mut actual = vec![0; 200_003];
        disk.read_exact_at(0, &mut actual).unwrap();
        assert!(actual[..start].iter().all(|&v| v == 0xa5));
        assert!(actual[start..end].iter().all(|&v| v == 0));
        assert!(actual[end..].iter().all(|&v| v == 0xa5));
        assert_eq!(disk.file.metadata().unwrap().len(), 200_003);
    }

    #[test]
    fn unsupported_modes_fall_back_in_bounded_chunks_and_cache_independently() {
        for errno in [libc::EOPNOTSUPP, libc::ENOSYS] {
            let disk = fixture();
            let mut calls = 0;
            for op in [Operation::Zero, Operation::Punch] {
                disk.sparse_with(op, 3, 199_997, |mode, offset, length| {
                    calls += 1;
                    assert_eq!(mode, op.mode());
                    assert_eq!(offset, 3);
                    assert_eq!(length, 199_997);
                    Err(std::io::Error::from_raw_os_error(errno))
                })
                .unwrap();
                disk.sparse_with(op, 3, 199_997, |_, _, _| panic!("unsupported mode retried"))
                    .unwrap();
            }
            assert_eq!(calls, 2);
            check(&disk, 3, 200_000);
        }
    }

    #[test]
    fn interrupted_calls_retry_but_real_errors_are_not_hidden_or_cached() {
        for errno in [
            libc::EIO,
            libc::ENOSPC,
            libc::EPERM,
            libc::EINVAL,
            libc::EFBIG,
        ] {
            let disk = fixture();
            let mut calls = 0;
            let error = disk
                .sparse_with(Operation::Punch, 3, 1000, |_, _, _| {
                    calls += 1;
                    if calls == 1 {
                        return Err(std::io::Error::from_raw_os_error(libc::EINTR));
                    }
                    // A failed operation may already have changed some bytes.
                    disk.buffered_file().write_all_at(&[0], 3).unwrap();
                    Err(std::io::Error::from_raw_os_error(errno))
                })
                .unwrap_err();
            assert!(matches!(error, Error::Io(e) if e.raw_os_error() == Some(errno)));
            assert_eq!(calls, 2);
            assert_eq!(disk.sparse_unsupported.load(Ordering::Relaxed), 0);
            check(&disk, 3, 4);
        }
    }

    #[test]
    fn sparse_guard_covers_syscall_fallback_and_error_unwinding() {
        for outcome in [libc::EOPNOTSUPP, libc::EIO] {
            let disk = fixture();
            let result = disk.sparse_with(Operation::Zero, 3, 199_997, |_, _, _| {
                assert!(matches!(
                    disk.read_at(3, &mut [0; 1]),
                    Err(Error::ConcurrentFileAccess { .. })
                ));
                assert!(matches!(
                    disk.flush(),
                    Err(Error::ConcurrentFileAccess { .. })
                ));
                Err(std::io::Error::from_raw_os_error(outcome))
            });
            assert_eq!(result.is_ok(), outcome == libc::EOPNOTSUPP);
            disk.flush().unwrap();
            if outcome == libc::EOPNOTSUPP {
                check(&disk, 3, 200_000);
            }
        }
        let disk = fixture();
        assert!(
            std::panic::catch_unwind(|| {
                let _ =
                    disk.sparse_with(Operation::Punch, 0, 4096, |_, _, _| panic!("syscall panic"));
            })
            .is_err()
        );
        disk.flush().unwrap();
    }

    #[test]
    fn empty_and_invalid_ranges_never_reach_the_kernel_operation() {
        let disk = fixture();
        disk.sparse_with(Operation::Zero, 200_003, 0, |_, _, _| {
            panic!("empty syscall")
        })
        .unwrap();
        for (offset, length) in [(200_003, 1), (200_004, 0), (u64::MAX, 2)] {
            assert!(
                disk.sparse_with(Operation::Punch, offset, length, |_, _, _| panic!(
                    "invalid syscall"
                ))
                .is_err()
            );
        }
        disk.file.set_len(100).unwrap();
        assert!(matches!(
            disk.sparse_with(Operation::Zero, 99, 2, |_, _, _| panic!("stale size")),
            Err(Error::OutOfBounds { size: 100, .. })
        ));
        assert_eq!(disk.file.metadata().unwrap().len(), 100);
    }

    #[test]
    fn fallback_rechecks_append_flags_even_after_mode_is_cached() {
        let disk = fixture();
        disk.sparse_with(Operation::Zero, 3, 1000, |_, _, _| {
            Err(std::io::Error::from_raw_os_error(libc::EOPNOTSUPP))
        })
        .unwrap();
        // SAFETY: F_GETFL/F_SETFL operate on the live test-owned descriptor.
        unsafe {
            let flags = libc::fcntl(disk.file.as_raw_fd(), libc::F_GETFL);
            assert!(flags >= 0);
            assert_eq!(
                libc::fcntl(disk.file.as_raw_fd(), libc::F_SETFL, flags | libc::O_APPEND),
                0
            );
        }
        assert!(matches!(
            disk.write_zero_at(1003, 10),
            Err(Error::InvalidEndpoint { .. })
        ));
        check(&disk, 3, 1003);
    }
}

#[cfg(test)]
mod alias_tests {
    use super::*;
    #[test]
    fn independent_buffered_alias_append_flag_is_rejected() {
        let path = std::env::temp_dir().join(format!("rvvdk-r22-alias-{}", std::process::id()));
        std::fs::write(&path, [0xa5; 4096]).unwrap();
        let mut disk = LocalFileBlockDevice::open_read_write(&path).unwrap();
        disk.buffered_file = Some(
            std::fs::OpenOptions::new()
                .read(true)
                .append(true)
                .open(&path)
                .unwrap(),
        );
        std::fs::remove_file(&path).unwrap();
        assert!(disk.copy_endpoint().is_ok());
        assert!(matches!(
            disk.sparse_with(Operation::Punch, 0, 100, |_, _, _| panic!(
                "unsafe alias used"
            )),
            Err(Error::InvalidEndpoint { .. })
        ));
        let mut actual = [0; 4096];
        disk.file.read_exact_at(&mut actual, 0).unwrap();
        assert_eq!(actual, [0xa5; 4096]);
    }
}
