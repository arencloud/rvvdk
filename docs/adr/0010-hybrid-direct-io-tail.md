# ADR-0010: Hybrid Direct and Buffered I/O

## Status

Accepted

## Context

Linux `O_DIRECT` imposes alignment constraints on the buffer address,
file offset, and request length.

Virtual disks are not guaranteed to have logical sizes aligned to
those constraints.

Rejecting unaligned virtual disks would unnecessarily restrict rvvdk.

Handling alignment in the DataMover would leak backend-specific
requirements into the generic copy engine.

## Decision

A direct local-file backend will maintain both:

- an `O_DIRECT` file descriptor
- a normal buffered file descriptor

Requests satisfying direct-I/O alignment requirements use the direct
descriptor.

Other requests transparently use the buffered descriptor.

The selection occurs inside `LocalFileBlockDevice`.

The DataMover remains independent of local Linux alignment semantics.

## Consequences

### Positive

Arbitrary logical disk sizes are supported.

Aligned bulk traffic continues to use direct I/O.

Tail handling does not complicate the DataMover.

The same mechanism handles other unaligned positional requests.

### Negative

Two descriptors reference the same inode.

Mixing buffered and direct I/O requires careful coherence handling.

Backends must avoid overlapping buffered and direct writes to the same
range when concurrent execution is active.

Flush operations must account for both descriptors.

## Current invariant

DataMover work items describe non-overlapping logical ranges.

The current implementation does not intentionally issue direct and
buffered writes to the same logical range concurrently.

## Future work

Future implementations may investigate:

- explicit cache invalidation
- alignment discovery
- fully aligned bounce-buffer strategies
- block-device-specific direct I/O
- `io_uring`