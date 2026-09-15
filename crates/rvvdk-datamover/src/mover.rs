use std::time::Instant;

use rvvdk_core::{Error, Result, VirtualDisk};

use crate::{CopyOptions, CopyStats};

pub struct DataMover {
    options: CopyOptions,
}

impl DataMover {
    pub const fn new(options: CopyOptions) -> Self {
        Self { options }
    }

    pub fn copy<S, D>(&self, source: &S, destination: &D) -> Result<CopyStats>
    where
        S: VirtualDisk,
        D: VirtualDisk,
    {
        let source_size = source.size();
        let destination_size = destination.size();

        if destination_size < source_size {
            return Err(Error::OutOfBounds {
                offset: 0,
                length: source_size,
                size: destination_size,
            });
        }

        let mut buffer = vec![0_u8; self.options.block_size()];

        let mut offset = 0_u64;
        let mut bytes_read = 0_u64;
        let mut bytes_written = 0_u64;
        let mut blocks_copied = 0_u64;

        let started = Instant::now();

        while offset < source_size {
            let remaining = source_size - offset;

            let request_size = remaining.min(self.options.block_size() as u64);

            let request_size = usize::try_from(request_size).map_err(|_| Error::RangeOverflow {
                offset,
                length: request_size,
            })?;

            let current_buffer = &mut buffer[..request_size];

            let read = source.read_at(offset, current_buffer)?;

            if read == 0 {
                return Err(Error::CorruptMetadata(format!(
                    "unexpected end of source at offset {offset}"
                )));
            }

            write_all_at(destination, offset, &current_buffer[..read])?;

            let read_u64 = u64::try_from(read).map_err(|_| Error::RangeOverflow {
                offset,
                length: u64::MAX,
            })?;

            offset = offset.checked_add(read_u64).ok_or(Error::RangeOverflow {
                offset,
                length: read_u64,
            })?;

            bytes_read += read_u64;
            bytes_written += read_u64;
            blocks_copied += 1;
        }

        destination.flush()?;

        Ok(CopyStats::new(
            bytes_read,
            bytes_written,
            blocks_copied,
            started.elapsed(),
        ))
    }
}

fn write_all_at<D>(destination: &D, mut offset: u64, mut buffer: &[u8]) -> Result<()>
where
    D: VirtualDisk,
{
    while !buffer.is_empty() {
        let written = destination.write_at(offset, buffer)?;

        if written == 0 {
            return Err(Error::CorruptMetadata(format!(
                "destination made no write progress at offset {offset}"
            )));
        }

        let written_u64 = u64::try_from(written).map_err(|_| Error::RangeOverflow {
            offset,
            length: u64::MAX,
        })?;

        offset = offset
            .checked_add(written_u64)
            .ok_or(Error::RangeOverflow {
                offset,
                length: written_u64,
            })?;

        buffer = &buffer[written..];
    }

    Ok(())
}
