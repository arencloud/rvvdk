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

/// Why planning chose its backend. This is not a runtime readiness guarantee.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExecutionSelectionReason {
    RequestedThreaded,
    #[cfg(target_os = "linux")]
    RequestedNative,
    /// Auto through a portable API must preserve logical disk translation.
    #[cfg(target_os = "linux")]
    PortableApi,
    /// The RAW descriptor capability evaluator accepted the pair.
    #[cfg(target_os = "linux")]
    RawDescriptorsCompatible,
    #[cfg(target_os = "linux")]
    RawDescriptorsIncompatible,
    /// Auto selected the portable executor for the entire RAW plan.
    #[cfg(target_os = "linux")]
    RawRequestsIncompatible(rvvdk_core::NativeRequestIssue),
}

/// The planning-time request and decision; execution still performs live checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionSelection {
    requested: ExecutionStrategy,
    selected: ExecutionBackend,
    reason: ExecutionSelectionReason,
}

impl ExecutionSelection {
    #[cfg(target_os = "linux")]
    pub(crate) fn raw_requests(
        requested: ExecutionStrategy,
        compatible: bool,
        issue: Option<rvvdk_core::NativeRequestIssue>,
    ) -> rvvdk_core::Result<Self> {
        match (requested, issue) {
            (ExecutionStrategy::Auto(_), Some(issue)) => Ok(Self {
                requested,
                selected: ExecutionBackend::Threaded,
                reason: ExecutionSelectionReason::RawRequestsIncompatible(issue),
            }),
            (ExecutionStrategy::IoUring(_), Some(issue)) => Err(issue.into()),
            _ => Self::raw(requested, compatible),
        }
    }

    pub const fn requested(&self) -> ExecutionStrategy {
        self.requested
    }

    pub const fn selected(&self) -> ExecutionBackend {
        self.selected
    }

    pub const fn reason(&self) -> ExecutionSelectionReason {
        self.reason
    }

    pub(crate) fn portable(requested: ExecutionStrategy) -> rvvdk_core::Result<Self> {
        let reason = match requested {
            ExecutionStrategy::Threaded => ExecutionSelectionReason::RequestedThreaded,
            #[cfg(target_os = "linux")]
            ExecutionStrategy::Auto(_) => ExecutionSelectionReason::PortableApi,
            #[cfg(target_os = "linux")]
            ExecutionStrategy::IoUring(_) => {
                return Err(rvvdk_core::Error::NativeExecutionUnsupported);
            }
        };
        Ok(Self {
            requested,
            selected: ExecutionBackend::Threaded,
            reason,
        })
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn raw(requested: ExecutionStrategy, compatible: bool) -> rvvdk_core::Result<Self> {
        let (selected, reason) = match requested {
            ExecutionStrategy::Threaded => (
                ExecutionBackend::Threaded,
                ExecutionSelectionReason::RequestedThreaded,
            ),
            ExecutionStrategy::IoUring(_) if compatible => (
                ExecutionBackend::IoUring,
                ExecutionSelectionReason::RequestedNative,
            ),
            ExecutionStrategy::IoUring(_) => {
                return Err(rvvdk_core::Error::NativeExecutionUnsupported);
            }
            ExecutionStrategy::Auto(_) if compatible => (
                ExecutionBackend::IoUring,
                ExecutionSelectionReason::RawDescriptorsCompatible,
            ),
            ExecutionStrategy::Auto(_) => (
                ExecutionBackend::Threaded,
                ExecutionSelectionReason::RawDescriptorsIncompatible,
            ),
        };
        Ok(Self {
            requested,
            selected,
            reason,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn explicit_native_cannot_fallback_but_auto_records_incompatibility() {
        let options = IoUringExecutionOptions::new(8).unwrap();
        assert!(matches!(
            ExecutionSelection::raw(ExecutionStrategy::IoUring(options), false),
            Err(rvvdk_core::Error::NativeExecutionUnsupported)
        ));
        let selection = ExecutionSelection::raw(ExecutionStrategy::Auto(options), false).unwrap();
        assert_eq!(selection.selected(), ExecutionBackend::Threaded);
        assert_eq!(
            selection.reason(),
            ExecutionSelectionReason::RawDescriptorsIncompatible
        );
    }

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

/// A preparation-time fallback; CopyPlan still records its planning decision.
#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NativeRuntimeFallback {
    /// Ring construction was unavailable or denied. Resource exhaustion and
    /// invalid configuration are errors instead; no I/O was submitted.
    RingUnavailable { os_error: i32 },
}
