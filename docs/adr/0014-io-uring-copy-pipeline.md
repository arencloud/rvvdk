# ADR-0014: io_uring Copy Pipeline

## Status

Accepted

## Context

The io_uring engine can safely own buffers while operations remain in
flight.

A data-copy pipeline must preserve that ownership while moving each
block from a source read into a destination write.

Copying data into a second userspace buffer would add unnecessary
memory bandwidth and allocation pressure.

## Decision

An io_uring block copy transfers one `BufferGuard` through the complete
operation lifecycle:

```text
BufferPool
    |
    v
source read
    |
    v
read completion
    |
    v
destination write
    |
    v
write completion
    |
    v
BufferPool
```
