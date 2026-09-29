use std::time::Duration;

use crate::{CopyPlan, ExecutionBackend};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressTotals {
    logical_bytes: u64,
    data_bytes: u64,
    zero_bytes: u64,
    hole_bytes: u64,
    extent_count: u64,
}

impl ProgressTotals {
    pub const fn new(
        logical_bytes: u64,
        data_bytes: u64,
        zero_bytes: u64,
        hole_bytes: u64,
        extent_count: u64,
    ) -> Self {
        Self {
            logical_bytes,
            data_bytes,
            zero_bytes,
            hole_bytes,
            extent_count,
        }
    }

    pub(crate) fn from_plan(plan: &CopyPlan) -> Self {
        Self {
            logical_bytes: plan.logical_bytes(),
            data_bytes: plan.data_bytes(),
            zero_bytes: plan.zero_bytes(),
            hole_bytes: plan.hole_bytes(),
            extent_count: plan.extent_count() as u64,
        }
    }

    pub const fn logical_bytes(&self) -> u64 {
        self.logical_bytes
    }
    pub const fn data_bytes(&self) -> u64 {
        self.data_bytes
    }
    pub const fn zero_bytes(&self) -> u64 {
        self.zero_bytes
    }
    pub const fn hole_bytes(&self) -> u64 {
        self.hole_bytes
    }
    pub const fn extent_count(&self) -> u64 {
        self.extent_count
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProgressCompleted {
    logical_bytes_completed: u64,
    bytes_read: u64,
    bytes_written: u64,
    bytes_zeroed: u64,
    bytes_discarded: u64,
    extents_processed: u64,
}

impl ProgressCompleted {
    pub const fn new(
        logical_bytes_completed: u64,
        bytes_read: u64,
        bytes_written: u64,
        bytes_zeroed: u64,
        bytes_discarded: u64,
        extents_processed: u64,
    ) -> Self {
        Self {
            logical_bytes_completed,
            bytes_read,
            bytes_written,
            bytes_zeroed,
            bytes_discarded,
            extents_processed,
        }
    }

    pub const fn logical_bytes_completed(&self) -> u64 {
        self.logical_bytes_completed
    }
    pub const fn bytes_read(&self) -> u64 {
        self.bytes_read
    }
    pub const fn bytes_written(&self) -> u64 {
        self.bytes_written
    }
    pub const fn bytes_zeroed(&self) -> u64 {
        self.bytes_zeroed
    }
    pub const fn bytes_discarded(&self) -> u64 {
        self.bytes_discarded
    }
    pub const fn extents_processed(&self) -> u64 {
        self.extents_processed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressSnapshot {
    backend: ExecutionBackend,
    totals: ProgressTotals,
    completed: ProgressCompleted,
    elapsed: Duration,
}

impl ProgressSnapshot {
    pub(crate) const fn new(
        backend: ExecutionBackend,
        totals: ProgressTotals,
        completed: ProgressCompleted,
        elapsed: Duration,
    ) -> Self {
        Self {
            backend,
            totals,
            completed,
            elapsed,
        }
    }

    pub(crate) fn initial(plan: &CopyPlan, backend: ExecutionBackend) -> Self {
        Self::new(
            backend,
            ProgressTotals::from_plan(plan),
            ProgressCompleted::default(),
            Duration::ZERO,
        )
    }

    pub const fn backend(&self) -> ExecutionBackend {
        self.backend
    }
    pub const fn totals(&self) -> &ProgressTotals {
        &self.totals
    }
    pub const fn completed(&self) -> &ProgressCompleted {
        &self.completed
    }
    pub const fn logical_bytes(&self) -> u64 {
        self.totals.logical_bytes()
    }
    pub const fn data_bytes(&self) -> u64 {
        self.totals.data_bytes()
    }
    pub const fn zero_bytes(&self) -> u64 {
        self.totals.zero_bytes()
    }
    pub const fn hole_bytes(&self) -> u64 {
        self.totals.hole_bytes()
    }
    pub const fn extent_count(&self) -> u64 {
        self.totals.extent_count()
    }
    pub const fn logical_bytes_completed(&self) -> u64 {
        self.completed.logical_bytes_completed()
    }
    pub const fn bytes_read(&self) -> u64 {
        self.completed.bytes_read()
    }
    pub const fn bytes_written(&self) -> u64 {
        self.completed.bytes_written()
    }
    pub const fn bytes_zeroed(&self) -> u64 {
        self.completed.bytes_zeroed()
    }
    pub const fn bytes_discarded(&self) -> u64 {
        self.completed.bytes_discarded()
    }
    pub const fn extents_processed(&self) -> u64 {
        self.completed.extents_processed()
    }
    pub const fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn logical_completion(&self) -> f64 {
        if self.logical_bytes() == 0 {
            return 1.0;
        }

        (self.logical_bytes_completed() as f64 / self.logical_bytes() as f64).clamp(0.0, 1.0)
    }

    pub fn logical_completion_percent(&self) -> f64 {
        self.logical_completion() * 100.0
    }

    pub fn logical_throughput_bytes_per_second(&self) -> f64 {
        let seconds = self.elapsed.as_secs_f64();

        if seconds == 0.0 {
            return 0.0;
        }

        self.logical_bytes_completed() as f64 / seconds
    }

    pub fn write_throughput_bytes_per_second(&self) -> f64 {
        let seconds = self.elapsed.as_secs_f64();

        if seconds == 0.0 {
            return 0.0;
        }

        self.bytes_written() as f64 / seconds
    }
}

#[derive(Debug)]
pub(crate) struct ProgressState {
    totals: ProgressTotals,
    completed: ProgressCompleted,
}

impl ProgressState {
    pub(crate) fn from_plan(plan: &CopyPlan) -> Self {
        Self {
            totals: ProgressTotals::from_plan(plan),
            completed: ProgressCompleted::default(),
        }
    }

    pub(crate) const fn totals(&self) -> ProgressTotals {
        self.totals
    }

    pub(crate) const fn completed(&self) -> ProgressCompleted {
        self.completed
    }

    pub(crate) fn complete_data(&mut self, length: u64) {
        self.completed.logical_bytes_completed += length;

        self.completed.bytes_read += length;

        self.completed.bytes_written += length;
    }

    pub(crate) fn complete_zero(&mut self, length: u64) {
        self.completed.logical_bytes_completed += length;

        self.completed.bytes_zeroed += length;
    }

    pub(crate) fn complete_discard(&mut self, length: u64) {
        self.completed.logical_bytes_completed += length;

        self.completed.bytes_discarded += length;
    }

    pub(crate) fn complete_fallback_write(&mut self, length: u64) {
        self.completed.logical_bytes_completed += length;

        self.completed.bytes_written += length;
    }

    pub(crate) fn complete_extent(&mut self) {
        self.completed.extents_processed += 1;
    }

    pub(crate) fn snapshot(
        &self,
        backend: ExecutionBackend,
        elapsed: Duration,
    ) -> ProgressSnapshot {
        ProgressSnapshot::new(backend, self.totals, self.completed, elapsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_logical_completion() {
        let snapshot = ProgressSnapshot::new(
            ExecutionBackend::Threaded,
            ProgressTotals::new(1_000, 500, 250, 250, 4),
            ProgressCompleted::new(250, 250, 250, 0, 0, 1),
            Duration::from_secs(1),
        );

        assert_eq!(snapshot.logical_completion(), 0.25);
        assert_eq!(snapshot.logical_completion_percent(), 25.0);
    }

    #[test]
    fn empty_copy_is_complete() {
        let snapshot = ProgressSnapshot::new(
            ExecutionBackend::Threaded,
            ProgressTotals::new(0, 0, 0, 0, 0),
            ProgressCompleted::default(),
            Duration::ZERO,
        );

        assert_eq!(snapshot.logical_completion(), 1.0);
        assert_eq!(snapshot.logical_completion_percent(), 100.0);
    }

    #[test]
    fn logical_completion_is_independent_of_write_accounting() {
        let snapshot = ProgressSnapshot::new(
            ExecutionBackend::Threaded,
            ProgressTotals::new(1_000, 0, 500, 500, 2),
            ProgressCompleted::new(500, 0, 500, 0, 0, 1),
            Duration::from_secs(1),
        );

        assert_eq!(snapshot.logical_completion(), 0.5);
        assert_eq!(snapshot.bytes_written(), 500);
        assert_eq!(snapshot.bytes_zeroed(), 0);
        assert_eq!(snapshot.bytes_discarded(), 0);
    }

    #[test]
    fn completion_is_clamped_to_one() {
        let snapshot = ProgressSnapshot::new(
            ExecutionBackend::Threaded,
            ProgressTotals::new(1_000, 1_000, 0, 0, 1),
            ProgressCompleted::new(1_500, 1_500, 1_500, 0, 0, 1),
            Duration::from_secs(1),
        );

        assert_eq!(snapshot.logical_completion(), 1.0);
    }

    #[test]
    fn calculates_logical_throughput() {
        let snapshot = ProgressSnapshot::new(
            ExecutionBackend::Threaded,
            ProgressTotals::new(4_000, 4_000, 0, 0, 1),
            ProgressCompleted::new(2_000, 2_000, 2_000, 0, 0, 1),
            Duration::from_secs(2),
        );

        assert_eq!(snapshot.logical_throughput_bytes_per_second(), 1_000.0);
        assert_eq!(snapshot.write_throughput_bytes_per_second(), 1_000.0);
    }

    #[test]
    fn zero_elapsed_has_zero_throughput() {
        let snapshot = ProgressSnapshot::new(
            ExecutionBackend::Threaded,
            ProgressTotals::new(1_000, 1_000, 0, 0, 1),
            ProgressCompleted::new(500, 500, 500, 0, 0, 1),
            Duration::ZERO,
        );

        assert_eq!(snapshot.logical_throughput_bytes_per_second(), 0.0);
        assert_eq!(snapshot.write_throughput_bytes_per_second(), 0.0);
    }
}
