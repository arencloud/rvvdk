# ADR-0040: Read-only base sparse logical mapping

Status: Accepted. Date: 2026-09-30. Implements R5.3.

## Decision

Add SparseDisk as a separate read-only VirtualDisk over validated base sparse
metadata. Load all extents through SparseMetadata with per-extent and decreasing
aggregate memory/read allowances. Bound handle count and query output separately.
Retain sources/maps and revalidate all endpoints at the end of acquisition.

Map allocated grains to physical reads and unallocated **base** grains to zeros.
Combine physically adjacent grains for reads; combine logical kinds for extent
queries. Use an allocation-free run traversal shared by both paths, with a count
pass before query output allocation. Do not infer zero reads for parented images.

Expose no native RAW descriptor or write path. Copy/verify preflight revalidates
sources and rejects a destination alias to any backing, or any unknown identity.
Standalone readers must revalidate before sessions and keep sources quiescent;
endpoint observations do not prevent same-size content mutation.

## Consequences

Eager maps provide indexed lookup and allocation-free reads. Aggregate reservation
sums are conservative and can reject a job even when earlier temporary buffers
have already been released. Repeated references consume separate budget/handles.
Extent queries scan metadata twice to allocate only coalesced output, bounded by a
separate count limit. Callers budget simultaneous query outputs themselves.

The library is usable by portable DataMover/Verifier now. Public CLI integration,
parent chains and live VMware access remain separate milestones. The contract and
limits are in [the sparse disk guide](../vmdk-sparse-disk.md); validation and costs
are retained in [R5.3 evidence](../benchmark-results/2026-09-30-r53/README.md).
