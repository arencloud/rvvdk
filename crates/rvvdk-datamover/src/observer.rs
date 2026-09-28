use crate::ProgressSnapshot;

pub trait ProgressObserver {
    fn on_progress(&self, snapshot: &ProgressSnapshot);
}

impl<F> ProgressObserver for F
where
    F: Fn(&ProgressSnapshot),
{
    fn on_progress(&self, snapshot: &ProgressSnapshot) {
        self(snapshot);
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct NoopProgressObserver;

impl ProgressObserver for NoopProgressObserver {
    fn on_progress(&self, _snapshot: &ProgressSnapshot) {}
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::time::Duration;

    use crate::{ExecutionBackend, ProgressCompleted, ProgressSnapshot, ProgressTotals};

    use super::*;

    #[test]
    fn closure_can_observe_progress() {
        let calls = Cell::new(0_u64);

        let observer = |snapshot: &ProgressSnapshot| {
            assert_eq!(snapshot.backend(), ExecutionBackend::Threaded);
            calls.set(calls.get() + 1);
        };

        let snapshot = ProgressSnapshot::new(
            ExecutionBackend::Threaded,
            ProgressTotals::new(1_000, 1_000, 0, 0, 1),
            ProgressCompleted::new(500, 500, 500, 0, 0, 1),
            Duration::from_secs(1),
        );

        observer.on_progress(&snapshot);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn noop_observer_accepts_snapshot() {
        let observer = NoopProgressObserver;

        let snapshot = ProgressSnapshot::new(
            ExecutionBackend::Threaded,
            ProgressTotals::new(1_000, 1_000, 0, 0, 1),
            ProgressCompleted::default(),
            Duration::ZERO,
        );

        observer.on_progress(&snapshot);
    }
}
