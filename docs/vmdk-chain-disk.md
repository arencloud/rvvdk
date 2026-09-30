# Read-only sparse parent fallback (R5.7)

`SparseChainDisk` owns a validated `SparseChain` and implements `VirtualDisk` for
read-only logical access. Its constructor adds no mapping allocation or I/O; chain
admission remains the [R5.6 contract](vmdk-parent-chain.md). Public CLI parent
acquisition is defined by [R5.8](cli-vmdk-parents.md). [Evidence and plots](benchmark-results/2026-09-30-r57/README.md)
retain validation, independent reference bytes and first performance baselines.

## Resolution and zero semantics

For each requested position, consult layers from leaf toward base. Locate that
layer's extent by binary search, then index its immutable grain map. An allocated
grain provides the bytes, including any zero bytes actually stored there. An
unallocated grain falls through; it becomes logical Zero only if every consulted
layer through the base is unallocated. Backing errors never trigger fallback.

This follows the parent behavior for absent grains in VMware's
[Virtual Disk Format 5.0](https://github.com/vmware/open-vmdk/blob/master/vmdk_50_technote.pdf),
already recorded under the metadata contract. Special zero-grain markers, compressed
layouts and non-sparse parents remain outside admission. An allocated grain whose
contents happen to be zero remains logical Data; querying extents does not read data.

Clip each resolved segment at the earliest grain boundary encountered while walking
the layers. Each admitted extent is grain-aligned, so this also clips its extent end.
This prevents a large ancestor grain from obscuring an allocated child grain later
in the request. Layers can have different grain sizes and split boundaries.
Once a layer supplies data, deeper ancestors cannot affect that segment.

Adjacent segments coalesce for physical reads only when they use the same retained
metadata map and contiguous physical bytes. Fully resolved zero segments coalesce
and fill the caller's buffer without backing I/O. Physical requests never cross
backing objects, even when logical Data continues across them. No recursive walk,
per-read heap allocation or whole-disk resolved map is needed. The iterator keeps
one pending segment on the stack. Work grows with visited grain boundaries and
consulted depth, with a binary extent lookup per consulted layer.

## API, output bounds and preflight

`SparseChainDisk::new(chain)` uses the default 65,536 logical output limit.
`with_limits(chain, SparseChainDiskLimits { output_extents })` changes only that
limit. The metadata reservation/read/depth bounds remain those used to load the
chain. `chain()` exposes immutable metadata and acquisition counters.

`read_at` validates the complete range before touching backing bytes and returns
the requested length on success. Empty reads at EOF succeed; overflow and out-of-
range requests fail. Short backing reads are retried through the existing exact-read
helper; EOF/I/O errors propagate. A failed multi-run read can leave a completed
prefix in the caller's buffer; there is no rollback or substitution from an older
ancestor after an allocated-grain error.

`extents` resolves logical Data/Zero, coalescing equal kinds across layer and backing
changes. It counts output entries first, rejects over-limit output, then allocates
and fills exactly the requested vector payload in a second pass. Both passes use
immutable maps without backing reads. Empty queries produce no entries even at a
zero output limit. Small queries and direct reads can succeed where a full query
exceeds the limit. Output vectors and caller read buffers are outside the already
admitted chain metadata budget. There is no logical Hole without a byte guarantee.

Capabilities are READ, EXTENTS and SPARSE. Writes, zero writes, discard and flush
return Unsupported. There is no native RAW descriptor or single endpoint identity.
`copy_endpoint()` revalidates every retained source before returning the composite
endpoint. `validate_destination_identity()` also revalidates, rejects unknown
destination identity and compares against **every descriptor and backing** in all
layers, including ancestors currently hidden by child data. This protects portable
copy/verification preflight; direct read callers still own their buffers and I/O policy.

Standalone callers must revalidate before read sessions and keep all sources
quiescent. Per-read metadata replay/endpoint syscalls are intentionally absent.
Retained handles/maps support concurrent reads and survive resolver drops.
Observing identity, size and readability does not detect same-size content edits,
lock external writers, provide a snapshot or establish live-VM consistency.

## Qualification and next step

Twelve tests cover nine child/parent grain combinations × 38 ranges, independent
byte/kind oracles, differing split geometry, overrides, allocated zero bytes,
1/4/16-layer fallback, zero-only and leaf-only paths, physical coalescing, range
errors, exact output boundaries, all source aliases, changed endpoints, short reads,
EOF/errors, retained handles, read-only operations and four-worker copy/verify.
An error test checks the completed prefix and untouched suffix explicitly.

QEMU-generated monolithic, split and mixed three-layer chains include sub-grain
writes, grain-boundary writes, zero overrides and a two-file 2 GiB + 64 KiB case.
The production reader reads every byte in 65,537-byte chunks. Its output matches
both an independently edited RAW oracle and QEMU decoding/compare in the qualified
cases. The original large-case partial second-extent write failed the overlay oracle:
rvddk and QEMU agree on the produced bytes, which differ outside the intended write.
That failure and its hashes remain in the report. A separate large case fully
overwrites the affected grain; it does not qualify the original producer path. Source images
remain unchanged; default-mode public CLI rejects the parent leaf. Only authored fixture
code, the previously consulted specification and QEMU command output are used.

Ten new timings cover full 1 MiB reads and extent queries at depths 1/4/16, leaf
reads, alternating ownership, fragmented physical placement and all-zero reads.
Fixture creation, admission, buffers and byte assertions are outside timing.
These are memory-device mapping/copy costs, not physical storage throughput.

R5.8 now provides [explicit confined CLI parent acquisition](cli-vmdk-parents.md)
with full lifecycle integration. Next R5.9 adds bounded coverage-guided fuzzing.
