# ADR-0006: Bounded Buffer Pool Ownership

## Status

Accepted

## Context

The DataMover requires reusable aligned I/O buffers.

Future concurrent workers will require multiple buffers, but allocating
memory independently for every I/O request would create unnecessary
allocation overhead and could allow memory usage to grow with request
volume.

The architecture therefore requires explicit ownership and
backpressure semantics.

## Decision

rvvdk will use a bounded `BufferPool`.

The pool owns a fixed number of `AlignedBuffer` instances.

A caller acquires a buffer through a `BufferGuard`.

The guard owns exclusive access to one buffer until the guard is
dropped.

Dropping the guard automatically returns the buffer to the pool.

When all buffers are in use, acquisition waits until a buffer becomes
available.

The initial implementation uses standard-library synchronization
primitives and does not depend on an asynchronous runtime.

## Consequences

### Positive

I/O memory usage is bounded.

Buffers are reused instead of repeatedly allocated.

Exclusive buffer ownership is represented by Rust ownership.

Automatic return prevents callers from forgetting to release buffers.

Backpressure exists naturally when workers exceed pool capacity.

The design prepares for concurrent DataMover workers.

### Negative

Waiting acquisition currently blocks an operating-system thread.

The mutex and condition variable may become a scalability concern at
very high queue depths.

A future async or completion-oriented execution engine may require a
different waiting mechanism.

## Future work

Future milestones may add:

- configurable queue depth
- multiple DataMover workers
- per-NUMA-node pools
- `io_uring` registered-buffer pools
- fixed buffer identifiers
- lock-free free-buffer queues if benchmarks justify them