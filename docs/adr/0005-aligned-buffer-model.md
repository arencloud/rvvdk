# ADR-0005: Aligned I/O Buffer Model

## Status

Accepted

## Context

The initial DataMover used `Vec<u8>` as its reusable copy buffer.

Future high-performance I/O mechanisms may impose stronger
requirements on memory used for disk operations.

Examples include:

- direct I/O alignment
- `io_uring` registered buffers
- fixed-buffer I/O
- reusable worker buffer pools

Embedding `Vec<u8>` assumptions throughout the DataMover would make
those changes difficult.

## Decision

rvvdk will represent owned DataMover I/O memory using an explicit
aligned buffer abstraction.

`AlignedBuffer` owns a contiguous allocation with explicit size and
alignment.

The DataMover allocates the buffer once per copy operation and reuses
it.

Copy algorithms operate on ordinary byte slices rather than depending
directly on the concrete buffer implementation.

The initial default alignment is 4096 bytes.

## Consequences

### Positive

Buffer alignment becomes explicit.

The DataMover no longer depends on `Vec<u8>` allocation semantics.

The copy algorithm remains compatible with future buffer pools.

The design prepares for direct I/O and registered `io_uring` buffers.

### Negative

The aligned allocation requires a small amount of carefully contained
unsafe Rust.

The initial implementation does not itself improve ordinary buffered
file I/O performance.

Alignment requirements may differ between future backends and will
need capability or configuration handling.

## Safety

Unsafe code is restricted to allocation, slice construction, and
deallocation inside `AlignedBuffer`.

The allocation layout used for deallocation must exactly match the
layout used during allocation.

The buffer owns the allocation for its complete lifetime.

## Future work

Future milestones may introduce:

- reusable buffer pools
- per-worker buffers
- direct-I/O-specific alignment validation
- registered `io_uring` buffers
- NUMA-aware allocation