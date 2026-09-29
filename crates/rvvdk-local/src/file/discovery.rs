use super::LocalFileBlockDevice;
use rvvdk_core::{Error, Extent, ExtentKind, Result};
use std::io;
use std::os::fd::RawFd;

impl LocalFileBlockDevice {
    pub(super) fn discover_extents(
        &self,
        offset: u64,
        length: u64,
        mut seek: impl FnMut(u64, libc::c_int) -> io::Result<u64>,
    ) -> Result<Vec<Extent>> {
        let end = offset
            .checked_add(length)
            .ok_or(Error::RangeOverflow { offset, length })?;
        check_size(offset, length, end, self.geometry.size())?;
        // Validate the complete range before interpreting EINVAL as unsupported.
        libc::off_t::try_from(end).map_err(|_| Error::RangeOverflow { offset, length })?;
        let _access =
            self.access
                .try_acquire(offset, length, rvvdk_platform::FileAccessKind::Inspect)?;
        let size = self.file.metadata()?.len();
        check_size(offset, length, end, size)?;
        if length == 0 {
            return Ok(Vec::new());
        }

        let extents = scan(offset, end, size, &mut seek)?;
        // In particular, ENXIO after truncation must not create a false Hole.
        // This detects observed shrinkage, not concurrent truncate-and-regrow.
        check_size(offset, length, end, self.file.metadata()?.len())?;
        Ok(extents)
    }
}

fn check_size(offset: u64, length: u64, end: u64, size: u64) -> Result<()> {
    if end > size {
        return Err(Error::OutOfBounds {
            offset,
            length,
            size,
        });
    }
    Ok(())
}

enum Seek {
    Offset(u64),
    End,
    Unsupported,
}

fn query(
    seek: &mut impl FnMut(u64, libc::c_int) -> io::Result<u64>,
    offset: u64,
    whence: libc::c_int,
) -> Result<Seek> {
    loop {
        match seek(offset, whence) {
            Ok(position) => return Ok(Seek::Offset(position)),
            Err(error) => match error.raw_os_error() {
                Some(libc::EINTR) => continue,
                Some(libc::EINVAL | libc::EOPNOTSUPP | libc::ENOSYS) => {
                    return Ok(Seek::Unsupported);
                }
                Some(libc::ENXIO) if whence == libc::SEEK_DATA => return Ok(Seek::End),
                _ => return Err(Error::Io(error)),
            },
        }
    }
}

fn scan(
    offset: u64,
    end: u64,
    size: u64,
    seek: &mut impl FnMut(u64, libc::c_int) -> io::Result<u64>,
) -> Result<Vec<Extent>> {
    let mut extents = Vec::new();
    let mut position = offset;
    while position < end {
        let data = match query(seek, position, libc::SEEK_DATA)? {
            Seek::Unsupported => return dense(offset, end),
            Seek::End => {
                extents.push(Extent::new(position, end - position, ExtentKind::Hole)?);
                break;
            }
            Seek::Offset(data) if data >= position && data < size => data,
            Seek::Offset(data) => {
                return Err(Error::CorruptMetadata(format!(
                    "invalid sparse data offset={data}, position={position}, size={size}"
                )));
            }
        };
        if data > position {
            extents.push(Extent::new(
                position,
                data.min(end) - position,
                ExtentKind::Hole,
            )?);
        }
        if data >= end {
            break;
        }
        let hole = match query(seek, data, libc::SEEK_HOLE)? {
            // Throw away even a successfully discovered prefix: unknown allocation
            // is Data for the entire query, never an inferred Hole.
            Seek::Unsupported => return dense(offset, end),
            Seek::Offset(hole) if hole > data && hole <= size => hole.min(end),
            Seek::Offset(hole) => {
                return Err(Error::CorruptMetadata(format!(
                    "invalid sparse hole offset={hole}, data offset={data}, size={size}"
                )));
            }
            Seek::End => unreachable!("only SEEK_DATA maps ENXIO to End"),
        };
        extents.push(Extent::new(data, hole - data, ExtentKind::Data)?);
        position = hole;
    }
    Ok(extents)
}

fn dense(offset: u64, end: u64) -> Result<Vec<Extent>> {
    Ok(vec![Extent::new(offset, end - offset, ExtentKind::Data)?])
}

pub(super) fn seek(fd: RawFd, offset: u64, whence: libc::c_int) -> io::Result<u64> {
    let offset =
        libc::off_t::try_from(offset).map_err(|_| io::Error::from_raw_os_error(libc::EOVERFLOW))?;
    // SAFETY: fd belongs to a live File and lseek takes no application pointers.
    // It changes the shared file cursor; all local payload I/O is positional.
    let result = unsafe { libc::lseek(fd, offset, whence) };
    if result < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(result as u64)
    }
}

#[cfg(test)]
mod tests;
