use io_uring::IoUring;

use rvvdk_core::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoUringCapabilities {
    queue_depth: u32,
    fast_poll: bool,
    nodrop: bool,
    submit_stable: bool,
}

impl IoUringCapabilities {
    pub const fn queue_depth(&self) -> u32 {
        self.queue_depth
    }

    pub const fn fast_poll(&self) -> bool {
        self.fast_poll
    }

    pub const fn nodrop(&self) -> bool {
        self.nodrop
    }

    pub const fn submit_stable(&self) -> bool {
        self.submit_stable
    }
}

pub fn probe_io_uring(queue_depth: u32) -> Result<IoUringCapabilities> {
    if queue_depth == 0 {
        return Err(Error::InvalidIoUringQueueDepth);
    }

    let ring = IoUring::new(queue_depth).map_err(Error::Io)?;

    let parameters = ring.params();

    Ok(IoUringCapabilities {
        queue_depth,
        fast_poll: parameters.is_feature_fast_poll(),
        nodrop: parameters.is_feature_nodrop(),
        submit_stable: parameters.is_feature_submit_stable(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_queue_depth() {
        let result = probe_io_uring(0);

        assert!(matches!(result, Err(Error::InvalidIoUringQueueDepth)));
    }

    #[test]
    fn creates_io_uring() {
        let capabilities = probe_io_uring(8).unwrap();

        assert_eq!(capabilities.queue_depth(), 8,);
    }

    #[test]
    fn supports_multiple_queue_depths() {
        for queue_depth in [1, 2, 4, 8, 16, 32, 64] {
            let capabilities = probe_io_uring(queue_depth).unwrap();

            assert_eq!(capabilities.queue_depth(), queue_depth,);
        }
    }
}
