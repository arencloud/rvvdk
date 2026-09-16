# ADR-0013: io_uring Buffer Ownership

## Status

Accepted

## Context

io_uring read and write SQEs contain userspace buffer pointers.

The kernel may access those buffers after submission and before the
corresponding completion is consumed.

A Rust slice borrow passed to a submission function does not naturally
remain active after that function returns.

A public API that accepts a borrowed slice and leaves the operation
outstanding would therefore permit safe caller code to invalidate the
buffer while the kernel still references it.

## Decision

Raw io_uring submission operations remain internal to the execution
engine.

The initial public API waits for completion before returning control of
the borrowed buffer to the caller.

Future multiple-in-flight APIs will transfer ownership of an
`AlignedBuffer` or `BufferGuard` into an in-flight request object.

The buffer will not return to the pool until its completion has been
consumed.

## Consequences

### Positive

Unsafe kernel buffer lifetimes are not exposed as a misleading safe
Rust API.

Buffer ownership remains explicit.

The design prepares naturally for the existing `BufferPool`.

Completion processing can eventually return completed buffers to the
pool automatically.

Multiple operations can later remain in flight without relying on
caller discipline to preserve buffer lifetime.

### Negative

The first public io_uring read/write API waits for each operation and
therefore does not yet exploit queue depth.

The execution engine requires an additional ownership layer before
multiple outstanding operations can be exposed safely.

Raw SQE submission remains an internal implementation detail.

## Safety invariant

For every submitted read or write:

```text
SQE submitted
     |
     v
userspace buffer remains alive
     |
     v
kernel accesses buffer
     |
     v
CQE consumed
     |
     v
buffer may be reused or released
```
