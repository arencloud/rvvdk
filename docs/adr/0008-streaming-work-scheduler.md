# ADR-0008: Bounded Streaming Work Scheduler

## Status

Accepted

## Context

The first concurrent DataMover converted the complete source extent map
into a `Vec<WorkItem>` before starting execution.

The number of work items grows approximately with:

```text
logical data size / block size
```

## Shutdown contract — R0.2, 2026-09-28

The bounded `sync_channel` is retained. Only worker closures own its receiver;
the coordinator drops its receiver reference before running the producer. When
workers exit, receiver destruction disconnects the channel and wakes a blocked
producer. Dropping the sender wakes idle workers. Scoped threads are joined
before returning, including on an error.

A shared failure state retains the first error recorded under a mutex, then
publishes an atomic stop flag. Workers check that flag before waiting for work
and after receiving an item. A producer error also records the failure before
dropping the sender. Successful production closes the sender without setting
the flag, so workers drain all remaining queued work normally.

The first *recorded* error wins. Concurrent backend failures have no guaranteed
wall-clock ordering. Queue disconnection caused by a worker failure cannot
replace its retained cause with `WorkQueueClosed`. Worker statistics are returned
only on success; this change does not introduce partial-result reporting.

Cancellation is cooperative between work items. Already-dispatched operations,
including work waiting for a pooled buffer, may finish. Synchronous backend calls
are not interruptible, so termination requires them to return. Backend deadlines,
public cancellation, panic-to-error conversion, and richer partial-copy errors
are outside this step. Rust scoped-thread panics continue to propagate after
joining; receiver release also prevents the all-workers-panicked queue deadlock.

The successful path adds two atomic loads per dequeued work item and no new
per-item mutex. Error storage is locked only on failure. Tests force a full queue
behind gated read/write/zero/discard failures, check buffer return and error
preservation, and isolate potential hangs in subprocesses with five-second
deadlines. See the [R0.2 benchmark report](../benchmark-results/2026-09-28-r02/README.md)
for successful-copy comparisons and failure-to-return measurements.
