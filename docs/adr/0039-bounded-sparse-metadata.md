# ADR-0039: Bounded sparse metadata and descriptor binding

Status: Accepted. Date: 2026-09-30. Implements R5.2.

## Decision

Introduce an explicit SparseDescriptor parser mode for hosted base sparse text;
share common syntax with the existing parser without widening default Descriptor
or CLI support. Sparse CIDs accept one to eight hex digits (32-bit values),
including unpadded producer output; the default parser retains its eight-digit rule.
Reject mixed layouts and parents. Add an eager per-extent
SparseMetadata loader over BackingResolver and retained BlockDevice ownership.

Bind selected extent capacity to the header and any embedded text to the supplied
CID/type/ordered mapping. Monolithic text is required; split reserved descriptor
space may be absent or all zero. Do not reopen names from embedded text.

Admit computed memory/read work before variable buffers or offset reads. Validate
all table placements before table reads, require primary/redundant equality,
validate complete grain ranges and reject duplicate physical grains. Preserve
logical map order and unallocated entries. No recovery fallback, logical reads,
writes, native FD interface or parent-zero inference is introduced.

Retain the source, map and observed endpoint. Compare endpoint state and header
again after acquisition. Same-size concurrent table/content mutation is outside
this observation contract: callers must maintain source quiescence. Per-job totals
and logical alias protection belong to the next mapping layer.

## Consequences

The eager map and temporary sorting storage trade bounded opening cost for direct
future lookup. Admission is per extent, separate from copy payload budgets and
resolver/parser overhead. Strict unused-entry/padding/duplicate policies may reject
images outside the qualified subset. Unknown identities remain unqualified.

Tests cover binding, budgets, placement, redundancy, grain bounds, endpoint changes
and short/error I/O. Reference fixtures reconstruct known bytes from admitted maps,
without claiming a production reader. See the [contract](../vmdk-sparse-metadata.md)
and [evidence](../benchmark-results/2026-09-30-r52/README.md).
