# Confined CLI sparse parent chains (R5.8)

All four Linux commands accept `--format vmdk --allow-parents` for read-only
hosted sparse chains. Destinations remain RAW. Without `--allow-parents`, the
[existing base-only CLI contract](cli-vmdk.md) is unchanged.

```bash
rvddk inspect leaf.vmdk --format vmdk --allow-parents --extents --json
rvddk plan leaf.vmdk output.raw --format vmdk --allow-parents --backend auto --json
rvddk copy leaf.vmdk output.raw --format vmdk --allow-parents --backend auto --workers 4 --verify --progress
rvddk verify leaf.vmdk output.raw --format vmdk --allow-parents --json
```

## Supported entries

| Entry | With `--allow-parents` |
|---|---|
| Clean version-1 `monolithicSparse` container | Accepted; embedded extent must identify that same container |
| External `twoGbMaxExtentSparse` / `2GbMaxExtentSparse` descriptor | Accepted, including multiple extent files |
| External `monolithicSparse` descriptor mirror | Accepted only with the library's embedded descriptor binding checks |
| Mixed monolithic/split sparse ancestry | Accepted with matching CID and logical capacity |
| Single sparse base | Accepted as a one-layer chain |
| RAW / FLAT / ZERO / managed or compressed variants | Rejected; omit the flag for existing RAW or FLAT/ZERO support |

Every layer uses [bounded chain admission](vmdk-parent-chain.md), then
[SparseChainDisk](vmdk-chain-disk.md) resolves the nearest allocated ancestor.
Allocated zero bytes mask parent data. Only ranges unallocated throughout the
chain become logical Zero. CID agreement is structural validation, not a content
hash or snapshot guarantee.

## Filesystem policy and observations

The explicitly supplied source selects one pinned directory. Every parent hint
must be a **basename** within that directory. Absolute paths, slashes, traversal,
backslashes, URI-like names, symlinks and mount crossings reject. There is no
current-directory lookup, ancestor search, external path fallback or prompt.
All layers therefore share the same descriptor directory. Backing references
use the existing confined resolver, which permits safe relative subdirectories.
The user-supplied source's parent path remains trusted; a trusted `/proc/self/fd`
and Linux `openat2` confinement remain required.

Each admitted descriptor and backing retains an observation FD bound to the
actual opened object. Device/inode, length, mtime and ctime (including nanoseconds)
are captured before reads/adoption and rechecked after acquisition and at the
existing command lifecycle boundaries. Changes fail with `source_changed`.
Every ancestor descriptor/backing participates in destination alias rejection,
including hard links and ancestors whose data is fully hidden by children.
Embedded identity binding rejects redirection to an identical second inode;
a hard link to the same container is allowed. Retained handles prevent pathname
replacement from redirecting reads.

Keep all sources quiescent. Timestamp checks detect changes at boundaries; they
do not provide writer exclusion or atomic snapshots. Existing overwrite behavior
may leave partial changes on failure. New outputs retain private creation,
verification, sync, no-replace publication and cooperative cancellation rules.

## Budgets, execution and reports

Default limits are 16 layers, 128 total loaded extents, 8 MiB acquired descriptors,
128 MiB reserved metadata and 256 MiB metadata reads, in addition to per-extent
limits and the 1 MiB per-descriptor input bound. Queries admit at most 65,536
coalesced entries. The CLI performs one extra four-byte entry probe per admitted
layer, at most 64 bytes for a successful default-depth chain. Descriptor and
backing observation FDs and JSON reports are additional bounded resources.
`--memory-budget` remains a copy/verification payload budget, not total RSS.

`threaded` and `auto` use portable logical copying; `io-uring` rejects before
opening the destination. Plan is read-only; copy and verify compare logical bytes
and retain existing destination tail, progress and cancellation behavior.

Preview schema remains version 1. `source.vmdk` adds:

- `layout: "hosted_sparse_chain"`, `parent_policy: "same_directory_basename"`,
  `layer_count`, leaf `cid` and leaf `descriptor_identity`.
- Leaf-to-base `layers`, each with CID, optional parent CID, descriptor identity
  and extent count. Base `parent_cid` is null.
- Whole-chain `backing_file_count`, `descriptor_extent_count`, `backing_identities`,
  `metadata_memory_reservation_bytes`, `metadata_read_bytes`,
  `chain_descriptor_bytes` and separate `entry_probe_bytes`.

The composite source has no single storage identity. Copy/verify keep their
existing report shape. Malformed or disallowed chains report `vmdk`; a RAW request
with the flag reports `arguments`.

## Qualification and next work

[Tests, benchmark evidence and reproducible SVG/PNG plots](benchmark-results/2026-09-30-r58/README.md)
cover acquisition, logical copying, lifecycle protections and QEMU-generated
three-layer monolithic, split, mixed and multi-file split chains. The prior
[QEMU partial second-extent write discrepancy](benchmark-results/2026-09-30-r57/README.md#full-logical-byte-reference-qualification)
remains unqualified; the large reference case fully overwrites the affected grain.
Local format support makes no live VMware compatibility claim. ESXi is unnecessary.

R5.9 now adds [bounded coverage-guided admission fuzzing](../fuzz/README.md),
with retained seeds, resource limits and reproducible evidence. Next V0.1 prepares
the independent-access workflow and lab acceptance plan.
Broader parent path policies and formats require separate qualification.
