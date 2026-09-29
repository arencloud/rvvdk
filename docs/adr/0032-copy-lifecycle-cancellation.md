# ADR-0032: Coordinator lifecycle and cooperative cancellation

Status: Accepted, 2026-09-29. Scope: R3.3 local RAW workflows.

## Context

The legacy snapshot observer has sequential intermediate progress but mostly
initial/final concurrent/native snapshots. CLI copies must distinguish logical
processing from flush, verification and publication, and stop without losing
ownership of asynchronous operations or falsely promising rollback.

## Decision

Add explicit controlled execution APIs and CopyEvent/CopyPhase, with a cloneable
atomic CancellationToken and replaceable thread-safe cancellation predicate.
Callbacks stay on the calling coordinator. Sequential execution checks at block
and read/write boundaries; concurrent workers publish cumulative deltas to a fixed
mutex-protected aggregate and check cancellation without invoking observers;
the producer samples progress and releases its queue before joining all workers.
Native execution checks between completions/refills and uses existing owned-I/O
shutdown/quarantine. Generic no-op checkpoints keep observation out of old
unobserved paths. Legacy snapshot cadence remains compatible.

Emit preparation, started, transfer, flush and terminal lifecycle states. Progress
is confirmed lower-bound counters, not a durability or contiguous-resume claim.
Throttle intermediate callbacks by observed bytes/time. Blocking operations and
slow callbacks may delay cancellation; no deadline or syscall interruption is
promised. Library completion ends at flush; CLI completion additionally requires
configured verification, sync and publication.

CLI --progress writes human lines or JSON lines to stderr without changing success
stdout schemas. Binary-only SIGINT/SIGTERM handlers record an atomic request;
embeddable CLI APIs never install handlers. Use 130/143 for cooperative signal
cancellation, while operational/output/uncertain-cleanup errors remain exit 1.
After linking a new output, finish parent sync/name inspection before honoring
cancellation; report published state and never attempt rollback or unlink.

## Consequences

Controlled multi-worker copies pay aggregate synchronization costs and native
copies poll at completion boundaries. Measure both affected old controls and new
progress/cancellation workloads. Progress consumer allocation and fixed lifecycle
bookkeeping remain outside payload accounting. Failure context and native
quarantine remain authoritative; cancellation cannot make uncertain I/O safe.
Callbacks/predicates must not panic. No background heartbeat, forced shutdown,
snapshot, resumable checkpoint or signal escalation policy is introduced.

[Public contract](../cli-progress.md) and
[validation/performance report](../benchmark-results/2026-09-29-r33/README.md)
record the tested boundaries and remaining qualifications.
