# ADR-0004: Linux Sparse-File Extent Discovery

## Status

Accepted

## Context

The extent-aware DataMover can avoid reading source regions represented
as holes.

RAW virtual disks backed by sparse Linux files need a mechanism for
discovering allocated and unallocated ranges.

Possible approaches include:

- scanning file contents for zeroes
- filesystem-specific APIs
- FIEMAP
- `SEEK_DATA` / `SEEK_HOLE`

Scanning for zeroes does not identify allocation state and would
require reading the data that rvvdk is attempting to avoid reading.

Filesystem-specific APIs would unnecessarily couple the local backend
to individual filesystems.

FIEMAP exposes richer physical extent metadata but is more complex than
required for the initial DataMover.

## Decision

The Linux local-file backend will use `SEEK_DATA` and `SEEK_HOLE` for
initial sparse extent discovery.

Extent discovery belongs to `BlockDevice` because physical allocation
is a property of the underlying storage representation.

`RawDisk` delegates extent discovery to its backing `BlockDevice` when
the backend advertises the `EXTENTS` capability.

## Consequences

### Positive

Sparse files avoid unnecessary source reads.

The implementation remains independent of individual Linux
filesystems.

RAW disk logic remains independent of Linux-specific syscalls.

The same `VirtualDisk` extent model can be consumed by the existing
DataMover.

### Negative

`