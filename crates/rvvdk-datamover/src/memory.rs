use std::mem::size_of;

use rvvdk_core::{AlignedBuffer, Error, Extent, Result};

use crate::{CopyPlan, DataMover, ExecutionBackend};

/// Per-copy payload storage charged to the memory budget, in bytes.
///
/// Counts buffers, pool descriptors, queue/worker entries, and extent Vec
/// capacity. Excludes allocator/container overhead, thread stacks, kernel ring
/// mappings, backend/observer allocations, and previously quarantined native
/// resources. This is not an RSS limit. Extent queries allocate inside backends
/// before their returned capacity can be checked. See docs/copy-memory.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyMemoryUsage {
    extent_bytes: usize,
    buffer_bytes: usize,
    queue_bytes: usize,
    total_bytes: usize,
}

impl CopyMemoryUsage {
    fn new(extent_slots: usize, buffer_bytes: usize, queue_bytes: usize) -> Result<Self> {
        let extent_bytes = mul(extent_slots, size_of::<Extent>())?;
        let total_bytes = add(add(extent_bytes, buffer_bytes)?, queue_bytes)?;
        Ok(Self {
            extent_bytes,
            buffer_bytes,
            queue_bytes,
            total_bytes,
        })
    }

    pub const fn extent_bytes(&self) -> usize {
        self.extent_bytes
    }
    /// Includes payload bytes and the pool's AlignedBuffer descriptors.
    pub const fn buffer_bytes(&self) -> usize {
        self.buffer_bytes
    }
    pub const fn queue_bytes(&self) -> usize {
        self.queue_bytes
    }
    pub const fn total_bytes(&self) -> usize {
        self.total_bytes
    }

    pub(crate) fn check(self, budget: usize, phase: &'static str) -> Result<()> {
        if self.total_bytes > budget {
            return Err(Error::MemoryBudgetExceeded {
                phase,
                required: self.total_bytes,
                budget,
            });
        }
        Ok(())
    }
}

fn add(a: usize, b: usize) -> Result<usize> {
    a.checked_add(b).ok_or(Error::MemoryAccountingOverflow)
}
fn mul(a: usize, b: usize) -> Result<usize> {
    a.checked_mul(b).ok_or(Error::MemoryAccountingOverflow)
}

impl DataMover {
    /// Execution payload requirement using this mover's current settings.
    /// Does not reserve memory or validate endpoints. Live revalidation can need
    /// more extent storage and is checked separately when executing the plan.
    /// Native sparse extents reserve a fallback buffer even if accelerated.
    pub fn execution_memory(&self, plan: &CopyPlan) -> Result<CopyMemoryUsage> {
        match plan.backend() {
            ExecutionBackend::Threaded => self.threaded_memory(plan.extent_capacity()),
            #[cfg(target_os = "linux")]
            ExecutionBackend::IoUring => {
                let options = match self.execution_strategy() {
                    crate::ExecutionStrategy::IoUring(options)
                    | crate::ExecutionStrategy::Auto(options) => options,
                    crate::ExecutionStrategy::Threaded => {
                        return Err(Error::NativeExecutionNotSelected);
                    }
                };
                self.native_memory(
                    plan.extent_capacity(),
                    plan.extent_count(),
                    plan.data_bytes() != 0,
                    plan.extent_count() != 0,
                    options,
                )
            }
        }
    }

    pub(crate) fn check_extent_memory(
        &self,
        resident: usize,
        incoming: usize,
        phase: &'static str,
    ) -> Result<()> {
        CopyMemoryUsage::new(add(resident, incoming)?, 0, 0)?
            .check(self.options.memory_budget(), phase)
    }

    pub(crate) fn threaded_memory(&self, extent_capacity: usize) -> Result<CopyMemoryUsage> {
        let buffers = mul(
            self.options.buffer_count(),
            add(self.options.block_size(), size_of::<AlignedBuffer>())?,
        )?;
        let workers = self.options.concurrency();
        let queue = if workers > 1 {
            // Bounded channel, one active item per worker, and one blocked producer item.
            let entries = add(add(self.options.queue_capacity(), workers)?, 1)?;
            add(
                mul(entries, size_of::<crate::work::WorkItem>())?,
                mul(workers, size_of::<crate::concurrent::WorkerStats>())?,
            )?
        } else {
            0
        };
        CopyMemoryUsage::new(extent_capacity, buffers, queue)
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn native_memory(
        &self,
        resident: usize,
        cloned: usize,
        data: bool,
        nonempty: bool,
        options: crate::IoUringExecutionOptions,
    ) -> Result<CopyMemoryUsage> {
        let depth = options.queue_depth() as usize;
        let (buffers, queue) = if data {
            (
                mul(
                    depth,
                    add(self.options.block_size(), size_of::<AlignedBuffer>())?,
                )?,
                add(
                    mul(depth, crate::io_uring::OPERATION_BYTES)?,
                    size_of::<crate::io_uring::CompletedOperation>(),
                )?,
            )
        } else if nonempty {
            (self.options.block_size(), 0)
        } else {
            (0, 0)
        };
        // Data jobs reuse a pool buffer for zero fallback; sparse-only jobs
        // reserve one scratch block. No second payload buffer overlaps the pool.
        CopyMemoryUsage::new(add(resident, cloned)?, buffers, queue)
    }
}
