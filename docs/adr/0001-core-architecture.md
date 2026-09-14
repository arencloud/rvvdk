# ADR-0001: Separate Core Disk, Format, Transport, and Data-Movement Layers

## Status

Accepted

## Context

rvvdk is intended to provide high-performance access to virtual disk
data.

The initial use case focuses on VMware workloads, but coupling the
core architecture directly to VMware VDDK concepts would make the
implementation difficult to reuse with other virtualization and
storage platforms.

Virtual disk systems contain several distinct concerns:

- logical disk representation
- disk format interpretation
- physical or remote storage access
- changed-block discovery
- data movement

These concerns should not require knowledge of one another's concrete
implementations.

## Decision

rvvdk will use separate architectural layers for:

```text
VirtualDisk
Disk Format
BlockDevice
Transport
DataMover
ChangeTracker
```