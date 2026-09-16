#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExecutionStrategy {
    #[default]
    Threaded,

    #[cfg(target_os = "linux")]
    IoUring(IoUringExecutionOptions),

    #[cfg(target_os = "linux")]
    Auto(IoUringExecutionOptions),
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoUringExecutionOptions {
    queue_depth: u32,
    read_window: usize,
}

#[cfg(target_os = "linux")]
impl IoUringExecutionOptions {
    pub fn new(queue_depth: u32) -> Option<Self> {
        if queue_depth == 0 {
            return None;
        }

        let read_window = (queue_depth as usize).div_ceil(2).max(1);

        Some(Self {
            queue_depth,
            read_window,
        })
    }

    pub fn with_read_window(queue_depth: u32, read_window: usize) -> Option<Self> {
        if queue_depth == 0 || read_window == 0 || read_window > queue_depth as usize {
            return None;
        }

        Some(Self {
            queue_depth,
            read_window,
        })
    }

    pub const fn queue_depth(&self) -> u32 {
        self.queue_depth
    }

    pub const fn read_window(&self) -> usize {
        self.read_window
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionBackend {
    Threaded,

    #[cfg(target_os = "linux")]
    IoUring,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threaded_is_default() {
        assert_eq!(ExecutionStrategy::default(), ExecutionStrategy::Threaded,);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn derives_read_window() {
        let options = IoUringExecutionOptions::new(8).unwrap();

        assert_eq!(options.queue_depth(), 8,);

        assert_eq!(options.read_window(), 4,);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn derives_odd_read_window() {
        let options = IoUringExecutionOptions::new(7).unwrap();

        assert_eq!(options.read_window(), 4,);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn rejects_zero_queue_depth() {
        assert!(IoUringExecutionOptions::new(0).is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn supports_custom_read_window() {
        let options = IoUringExecutionOptions::with_read_window(8, 3).unwrap();

        assert_eq!(options.queue_depth(), 8,);

        assert_eq!(options.read_window(), 3,);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn rejects_read_window_above_queue_depth() {
        assert!(IoUringExecutionOptions::with_read_window(8, 9,).is_none());
    }
}
