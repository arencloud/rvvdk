use rvvdk_core::NativeRequestIssue;
use rvvdk_platform::{LinuxFdBackend, LinuxFdCapabilities};

/// Descriptor declarations only; this does not establish request or runtime readiness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoUringCompatibility {
    compatible: bool,
    direct_io: bool,
    alignment: usize,
    source: LinuxFdCapabilities,
    destination: LinuxFdCapabilities,
}

impl IoUringCompatibility {
    pub(crate) fn direct_modes(&self) -> (bool, bool) {
        (self.source.direct_io(), self.destination.direct_io())
    }

    /// Check the complete native intent without allocating or touching payloads.
    /// Only Data extents become native requests; Zero/Hole use backend operations.
    pub(crate) fn request_issue(
        &self,
        offset: u64,
        length: u64,
        block_size: usize,
        buffer_alignment: usize,
        data: impl IntoIterator<Item = (u64, u64)>,
    ) -> Option<NativeRequestIssue> {
        if block_size == 0 || u32::try_from(block_size).is_err() {
            return Some(NativeRequestIssue::BlockSize { block_size });
        }
        if offset
            .checked_add(length)
            .is_none_or(|end| end > i64::MAX as u64)
        {
            return Some(NativeRequestIssue::Range { offset, length });
        }
        let endpoints = [("source", self.source), ("destination", self.destination)];
        for (endpoint, caps) in endpoints {
            if !valid_alignment(caps) {
                return Some(NativeRequestIssue::DescriptorAlignment {
                    endpoint,
                    memory: caps.memory_alignment(),
                    offset: caps.offset_alignment(),
                });
            }
        }
        let alignment = self.alignment.max(buffer_alignment);
        if std::alloc::Layout::from_size_align(block_size, alignment).is_err() {
            return Some(NativeRequestIssue::BufferLayout {
                block_size,
                alignment,
            });
        }
        // Buffered descriptors impose no direct-I/O offset/tail restrictions.
        if !self.source.direct_io() && !self.destination.direct_io() {
            return None;
        }
        for (offset, length) in data {
            for (endpoint, caps) in endpoints {
                if !caps.direct_io() {
                    continue;
                }
                let alignment = caps.offset_alignment();
                if !offset.is_multiple_of(alignment as u64)
                    || !length.is_multiple_of(alignment as u64)
                {
                    return Some(NativeRequestIssue::DirectRange {
                        endpoint,
                        offset,
                        length,
                        alignment,
                    });
                }
                // A smaller extent uses one request, so an unused block size
                // need not be aligned. Otherwise every split must stay aligned.
                if length > block_size as u64 && !block_size.is_multiple_of(alignment) {
                    return Some(NativeRequestIssue::DirectBlockSize {
                        endpoint,
                        block_size,
                        alignment,
                    });
                }
            }
        }
        None
    }

    /// True only when both endpoints declare direct I/O. Mixed pairs still
    /// require the direct endpoint's alignment for each native request.
    pub const fn direct_io(&self) -> bool {
        self.direct_io
    }
    pub const fn compatible(&self) -> bool {
        self.compatible
    }

    pub const fn alignment(&self) -> usize {
        self.alignment
    }
}

pub fn evaluate_compatibility<S, D>(source: &S, destination: &D) -> IoUringCompatibility
where
    S: LinuxFdBackend,
    D: LinuxFdBackend,
{
    let source_capabilities = source.linux_fd_capabilities();

    let destination_capabilities = destination.linux_fd_capabilities();

    let alignment = source_capabilities
        .memory_alignment()
        .max(source_capabilities.offset_alignment())
        .max(destination_capabilities.memory_alignment())
        .max(destination_capabilities.offset_alignment());

    let direct_io = source_capabilities.direct_io() && destination_capabilities.direct_io();

    IoUringCompatibility {
        compatible: valid_alignment(source_capabilities)
            && valid_alignment(destination_capabilities),
        direct_io,
        alignment,
        source: source_capabilities,
        destination: destination_capabilities,
    }
}

fn valid_alignment(capabilities: LinuxFdCapabilities) -> bool {
    capabilities.memory_alignment().is_power_of_two()
        && capabilities.offset_alignment().is_power_of_two()
}

#[cfg(test)]
mod request_tests {
    use super::*;
    fn pair(source: LinuxFdCapabilities, destination: LinuxFdCapabilities) -> IoUringCompatibility {
        IoUringCompatibility {
            compatible: true,
            direct_io: source.direct_io() && destination.direct_io(),
            alignment: source
                .memory_alignment()
                .max(source.offset_alignment())
                .max(destination.memory_alignment())
                .max(destination.offset_alignment()),
            source,
            destination,
        }
    }
    fn buffered() -> LinuxFdCapabilities {
        LinuxFdCapabilities::new(false, 1, 1)
    }
    fn direct() -> LinuxFdCapabilities {
        LinuxFdCapabilities::new(true, 4096, 512)
    }

    #[test]
    fn mixed_pairs_enforce_each_direct_endpoint_and_distinguish_memory_from_offset_alignment() {
        for (source, destination, role) in [
            (direct(), buffered(), "source"),
            (buffered(), direct(), "destination"),
        ] {
            let pair = pair(source, destination);
            assert!(
                pair.request_issue(0, 8192, 512, 4096, [(512, 512)])
                    .is_none()
            );
            for (offset, length) in [(1, 512), (512, 513)] {
                assert_eq!(
                    pair.request_issue(0, 8192, 512, 4096, [(offset, length)]),
                    Some(NativeRequestIssue::DirectRange {
                        endpoint: role,
                        offset,
                        length,
                        alignment: 512
                    })
                );
            }
        }
    }

    #[test]
    fn validates_splits_but_not_an_unused_block_boundary() {
        let pair = pair(direct(), direct());
        assert!(
            pair.request_issue(0, 4096, 4097, 4096, [(0, 4096)])
                .is_none()
        );
        assert_eq!(
            pair.request_issue(0, 8192, 4097, 4096, [(0, 8192)]),
            Some(NativeRequestIssue::DirectBlockSize {
                endpoint: "source",
                block_size: 4097,
                alignment: 512
            })
        );
    }

    #[test]
    fn buffered_odd_ranges_and_backend_only_sparse_extents_need_no_direct_alignment() {
        assert!(
            pair(buffered(), buffered())
                .request_issue(0, 8193, 4097, 4096, [(1, 8191)])
                .is_none()
        );
        assert!(
            pair(direct(), direct())
                .request_issue(0, 8193, 4097, 4096, [])
                .is_none()
        );
    }

    #[test]
    fn rejects_invalid_declarations_and_unallocatable_layouts() {
        for value in [0, 3] {
            for caps in [
                LinuxFdCapabilities::new(true, value, 512),
                LinuxFdCapabilities::new(true, 4096, value),
            ] {
                assert!(matches!(
                    pair(caps, buffered()).request_issue(0, 0, 4096, 4096, []),
                    Some(NativeRequestIssue::DescriptorAlignment {
                        endpoint: "source",
                        ..
                    })
                ));
            }
        }
        let huge = 1usize << (usize::BITS - 1);
        assert!(matches!(
            pair(LinuxFdCapabilities::new(true, huge, 512), buffered()).request_issue(
                0,
                0,
                4096,
                4096,
                []
            ),
            Some(NativeRequestIssue::BufferLayout { .. })
        ));
    }

    #[test]
    fn validates_signed_ranges_and_sqe_lengths_even_without_payload() {
        let pair = pair(buffered(), buffered());
        for (offset, length) in [(u64::MAX, 1), (i64::MAX as u64, 1), (u64::MAX, 0)] {
            assert_eq!(
                pair.request_issue(offset, length, 4096, 4096, []),
                Some(NativeRequestIssue::Range { offset, length })
            );
        }
        assert!(
            pair.request_issue(i64::MAX as u64, 0, 4096, 4096, [])
                .is_none()
        );
        assert_eq!(
            pair.request_issue(0, 0, 0, 4096, []),
            Some(NativeRequestIssue::BlockSize { block_size: 0 })
        );
        if let Ok(block_size) = usize::try_from(u64::from(u32::MAX) + 1) {
            assert_eq!(
                pair.request_issue(0, 0, block_size, 4096, []),
                Some(NativeRequestIssue::BlockSize { block_size })
            );
        }
    }
}
