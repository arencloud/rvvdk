# ADR-0038: Bounded hosted sparse header admission

Status: Accepted. Date: 2026-09-30. Implements R5.1.

## Context

Sparse extents add binary metadata offsets/counts that can drive allocation and
I/O. The existing FLAT/ZERO descriptor and logical reader must not accept those
files until their complete mapping is validated. Establish a separate header
contract first, with resource limits and explicit unsupported features.

## Decision

Add portable SparseHeader/SparseLimits/SparseRegion/SparseError APIs to rvvdk-vmdk.
Parse one exact sector with explicit little-endian reads and no packed casts,
unsafe code, metadata allocations or offset-following I/O. A bounded Read helper
uses a 512-byte stack buffer. Decode clean uncompressed version 1 only, with the
known newline and redundancy flags. Reject unsupported versions/flags, footer
sentinels, dirty state, compression and nonzero reserved bytes.

Check geometry, byte conversions, directory/table counts, resource ceilings,
observed physical length, minimum metadata storage and pairwise overlap of the
advertised descriptor/directories. Keep result fields private and expose checked
regions. Source ownership/consistency remains the caller's responsibility.

Do not change descriptor create/extent types, VmdkDisk or CLI format support.
Header admission says nothing about the actual contents of the advertised regions
or whether the disk can be read. R5.2 must validate directory/table locations,
redundancy and descriptor binding before logical sparse reads can be implemented.

## Consequences and validation

The policy deliberately rejects some producer outputs, including capacities not
aligned to a grain, even when another tool can open them. Limits are independent
of copy payload budgets. No caches, writes, native endpoints or parent handling
are introduced. Parsing work is constant with respect to virtual capacity.

Authored fixtures exercise boundaries, overflow, overlap, feature rejection,
short/interrupted reads and deterministic byte mutations. A separate generator
records QEMU-created headers, tool and file hashes, capacity comparisons and
explicit unsupported cases. An initial runner assumption about split descriptor
absence failed: QEMU advertises empty space there. This is retained as evidence
and deferred to descriptor binding; the parser required no compatibility change.

See the [contract](../vmdk-sparse-header.md) and
[benchmark/reference evidence](../benchmark-results/2026-09-30-r51/README.md).
