# ADR-0007: Synchronous Concurrent DataMover

## Status

Accepted

## Context

The sequential DataMover issues one positional I/O operation at a time.

Modern storage and network transports generally require multiple
outstanding operations to achieve high throughput.

rvvdk already provides:

- positional I/O
- extent-aware work
- aligned buffers
- bounded buffer pooling

The next execution model must introduce concurrency without coupling
the core architecture to an asynchronous runtime.

## Decision

The initial concurrent DataMover will use scoped operating-system
threads and synchronous positional I/O.

Source extents are split into independent block-sized work items.

Workers consume work items and acquire reusable buffers from the
bounded BufferPool.

The initial concurrency parameter determines:

- worker count
- buffer count
- maximum simultaneously active block operations

Concurrency of one continues to use the existing sequential execution
path.

## Consequences

### Positive

The existing synchronous BlockDevice and VirtualDisk APIs remain
unchanged.

Multiple positional operations can execute concurrently.

Memory usage remains bounded.

The sequential baseline remains directly comparable.

No asynchronous runtime dependency is introduced.

### Negative

Operating-system threads are heavier than completion-based I/O.

A shared work queue introduces synchronization.

Worker count and true device queue depth are not independent.

Very high concurrency is not expected to scale efficiently with this
model.

## Future work

The synchronous worker engine is an intermediate execution model.

A future completion-oriented engine may use Linux `io_uring` with:

- independent submission queue depth
- completion-driven scheduling
- registered buffers
- fixed files
- reduced thread count

The synchronous implementation remains useful as a portable baseline
and fallback.