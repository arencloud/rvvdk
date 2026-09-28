use rvvdk_core::{CopyFailure, CopyOperation, CopyProgress, DiskRange, Error};

#[cold]
pub(crate) fn execution(
    backend: &'static str,
    operation: CopyOperation,
    range: Option<(u64, u64)>,
    progress: CopyProgress,
    cause: Error,
) -> Error {
    Error::CopyExecution(Box::new(CopyFailure {
        backend,
        operation,
        range: range.and_then(|(offset, length)| DiskRange::new(offset, length).ok()),
        progress,
        cause,
    }))
}

/// Replace local worker counters after every scoped worker has joined.
#[cold]
pub(crate) fn worker_total(error: Error, mut total: CopyProgress) -> Error {
    match error {
        Error::CopyExecution(mut failure) => {
            total.unconfirmed_io |= failure.progress.unconfirmed_io;
            failure.progress = total;
            Error::CopyExecution(failure)
        }
        error => execution("threaded", CopyOperation::Schedule, None, total, error),
    }
}

/// Add already completed extents to the failing native range's lower bounds.
#[cfg(target_os = "linux")]
#[cold]
pub(crate) fn native_prior(error: Error, prior: CopyProgress) -> Error {
    match error {
        Error::CopyExecution(mut failure) => {
            failure.progress.bytes_read += prior.bytes_read;
            failure.progress.bytes_written += prior.bytes_written;
            failure.progress.bytes_zeroed += prior.bytes_zeroed;
            failure.progress.bytes_discarded += prior.bytes_discarded;
            failure.progress.blocks_completed += prior.blocks_completed;
            failure.progress.extents_completed = prior.extents_completed;
            Error::CopyExecution(failure)
        }
        Error::IoUringCleanup {
            original,
            operations,
        } => Error::IoUringCleanup {
            original: Box::new(native_prior(*original, prior)),
            operations,
        },
        error => execution("io_uring", CopyOperation::NativeSetup, None, prior, error),
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;

    #[test]
    fn native_cleanup_keeps_original_cause_and_prior_extent_progress() {
        let original = execution(
            "io_uring",
            CopyOperation::Write,
            Some((4096, 4096)),
            CopyProgress {
                bytes_read: 4096,
                bytes_written: 256,
                unconfirmed_io: true,
                ..CopyProgress::default()
            },
            Error::Io(std::io::Error::from_raw_os_error(5)),
        );
        let error = native_prior(
            Error::IoUringCleanup {
                original: Box::new(original),
                operations: 2,
            },
            CopyProgress {
                bytes_read: 4096,
                bytes_written: 4096,
                bytes_zeroed: 1024,
                extents_completed: Some(2),
                ..CopyProgress::default()
            },
        );
        assert!(matches!(error, Error::IoUringCleanup { operations: 2, .. }));
        let f = error.copy_failure().unwrap();
        assert_eq!(
            (
                f.progress.bytes_read,
                f.progress.bytes_written,
                f.progress.bytes_zeroed
            ),
            (8192, 4352, 1024)
        );
        assert_eq!(f.progress.extents_completed, Some(2));
        assert!(f.progress.unconfirmed_io);
        assert_eq!(f.range.unwrap().offset(), 4096);
        assert!(matches!(&f.cause, Error::Io(e) if e.raw_os_error() == Some(5)));
    }
}
