# ADR-0043: Resolve sparse parents without a second full-disk map

Date: 2026-09-30. Status: Accepted.

A child's unallocated grain must inherit the nearest allocated ancestor, and grain
sizes or extent boundaries can differ between layers. Materializing a combined map
would add a new capacity-dependent allocation and complicate admission budgets.

Wrap owned `SparseChain` metadata in `SparseChainDisk`. Walk layers iteratively,
clipping at every consulted grain boundary before choosing the nearest allocated
grain or fully resolved zero. Coalesce physically adjacent bytes only within one
retained backing map. Reads use constant auxiliary space and reuse caller buffers.
Logical extent queries count first, then allocate/fill under a separate output limit.

Revalidate all sources at copy preflight and compare destination identity against
every descriptor/backing, including hidden ancestors. Keep the source quiescent;
endpoint observations are not snapshots. No parent fallback after allocated I/O
failure is allowed. Per-read I/O validation is not added to the hot path.

This introduces portable library reads while retaining the metadata-only chain type
and current public CLI rejection. CLI parent acquisition/lifecycle is R5.8. See
[logical contract](../vmdk-chain-disk.md) and [evidence](../benchmark-results/2026-09-30-r57/README.md).
