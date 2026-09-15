# ADR-0003: Extent-Aware Copy Semantics

## Status

Accepted

## Context

Virtual disks may contain regions that represent stored data, logical
zeroes, or unallocated holes.

Reading and transferring every logical byte wastes I/O and network
bandwidth.

However, simply skipping zero or hole regions is not always correct
because the destination may already contain non-zero data.

## Decision

The DataMover will interpret source extents according to their kind.

### Data

Data extents are read from the source and written to the destination.

### Zero

Zero extents are not read from the source.

The destination is made logically zero using:

1. `WRITE_ZERO` when available
2. explicit zero writes otherwise

### Hole

Hole extents are not read from the source.

The destination operation preference is:

1. `DISCARD`
2. `WRITE_ZERO`
3. explicit zero writes

The DataMover will not simply skip a hole unless a future destination
contract explicitly guarantees that the corresponding range is already
logically correct.

## Extent validation

For complete-disk copies, the source must return an ordered,
non-overlapping and gap-free extent map covering the complete logical
disk.

Malformed extent maps cause the copy to fail before data movement.

## Consequences

### Positive

Sparse and zero regions can avoid unnecessary source reads.

Backends capable of deallocation can preserve sparse storage.

Copies remain logically correct when the destination contains existing
data.

The same model can later consume VMware CBT or other block-change
information.

### Negative

Destinations without zero or discard capabilities must receive
explicit zero writes.

Extent metadata must be validated.

The current complete extent map is materialized as a `Vec<Extent>`,
which may need to become streaming for very large or highly fragmented
disks.

## Future work

Future implementations may add:

- streaming extent iteration
- sparse local-file discovery
- changed-block extent sources
- destination allocation awareness
- extent coalescing
- parallel extent scheduling