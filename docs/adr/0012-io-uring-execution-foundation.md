# ADR-0012: io_uring Execution Foundation

## Status

Accepted

## Context

rvvdk currently provides sequential and synchronous multi-threaded
data-movement execution.

Direct-I/O benchmarking shows that additional operating-system threads
do not necessarily increase physical-storage throughput.

Linux `io_uring` provides a completion-oriented I/O interface capable
of maintaining multiple outstanding operations without requiring one
blocking worker thread per operation.

rvvdk should evaluate `io_uring` without replacing its existing
portable execution models.

## Decision

rvvdk will introduce `io_uring` as an additional Linux-specific
execution engine.

The integration will be incremental.

The first stage provides:

- Linux-only dependency isolation
- ring creation
- configurable queue depth probing
- runtime feature discovery

The existing sequential and threaded engines remain unchanged.

The generic `BlockDevice` and `VirtualDisk` abstractions do not depend
on `io_uring`.

## Consequences

### Positive

`io_uring` can be evaluated independently against existing engines.

Linux-specific implementation details remain isolated.

The project retains portable fallback execution models.

Queue-depth behavior can eventually be controlled independently of
thread count.

The existing aligned-buffer and buffer-pool architecture can be reused.

### Negative

The DataMover now has a Linux-specific optional execution path.

Multiple execution engines increase implementation and testing
complexity.

Not every storage backend will necessarily benefit from `io_uring`.

## Future work

The implementation will proceed incrementally:

1. runtime capability probe
2. engine abstraction
3. asynchronous positional read/write
4. completion processing
5. queue-depth management
6. registered aligned buffers
7. fixed files where useful
8. direct comparison against the synchronous engine