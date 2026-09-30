# Bounded sparse parent metadata (R5.6)

`SparseChain` validates and retains hosted-sparse metadata from leaf to base. It
has no logical read/write or extent-query interface and is not a `VirtualDisk`.
`SparseDescriptor`/`SparseDisk` and the default CLI path continue to reject parents.
[R5.7 SparseChainDisk](vmdk-chain-disk.md) now wraps it with logical parent fallback
and whole-chain alias checks; R5.8 adds [opt-in CLI acquisition](cli-vmdk-parents.md).
[Evidence and plots](benchmark-results/2026-09-30-r56/README.md) record qualification.

## Format and admission policy

The relationship follows VMware's [Virtual Disk Format 5.0, descriptor fields,
pp. 4–5](https://github.com/vmware/open-vmdk/blob/master/vmdk_50_technote.pdf): a child
records its parent's CID and a parent filename hint; `ffffffff` terminates the
chain. [Broadcom's CID mismatch explanation](https://knowledge.broadcom.com/external/article/345254/the-parent-virtual-disk-has-been-modifie.html)
confirms that a child's parentCID must match its parent's CID and describes
capacity mismatch failures. Only the published specification and vendor guidance
were consulted, not VMware SDK or third-party parser source. The specification
SHA-256 remains `88ce1615a703d1d4e3df3c227846bb9ecda4d929129c191ea4ffe7df3294ad59`.

The following are deliberately conservative rvddk policies:

- Each layer uses the existing clean, uncompressed version-1 hosted-sparse subset.
  Parents must also be sparse; FLAT/ZERO, VMFS, seSparse and compressed layers are
  separate work. Mixed monolithic/split layers are allowed.
- Every nonterminal parentCID requires one nonempty, quoted `parentFileNameHint`.
  A base must have no hint. Duplicate hints and unsupported syntax reject. The
  parser treats hints as bounded opaque strings, not authorized paths.
- Every adjacent pair has equal total logical capacity and matching parentCID/CID.
  Extent counts and grain geometry need not match. Equal CIDs alone do not imply
  a cycle; they are not unique object identities or cryptographic fingerprints.
- Every descriptor and backing must expose a readable, known endpoint identity.
  Repeated descriptor identities or cross-layer descriptor/backing aliases reject.
  Reusing a backing within or across layers also rejects. An embedded monolithic
  descriptor is allowed to share its own backing identity and must do so.
- Metadata for all extents is validated before returning the chain. Directory,
  redundant table, grain-range, duplicate-grain and source checks reuse the
  existing loader. Embedded text must agree on CID, layout, parentCID and hint.

## API and resolution boundary

`SparseLayerDescriptor` is an explicit parent-capable parser. Its fields borrow
bounded input; it cannot be passed to `SparseDisk::load`. Its private sparse view
is used only by the internal metadata loader, preventing a child from reaching
base logical-zero behavior through the public type interface.

`SparseChainSource` supplies an already opened descriptor/container, its layer's
`BackingResolver`, and explicit `ChainEntry::External` or `Embedded`. External
entry reads bounded observed descriptor length; embedded entry first validates
one header sector, then admits the advertised descriptor range before allocation.
No automatic file search or current-directory fallback exists in the library.

`ParentResolver::resolve_parent(child_identity, hint)` is the caller's opt-in
namespace and authorization boundary. `None` means missing; errors may deny
resolution. Each result brings its own backing namespace, allowing correct
layer-relative lookup. Resolver implementations must enforce path/confinement and
stable-handle semantics, and must not interpret a hint as authorization. Resolver
internal allocations, opens, blocking time and side effects are outside loader
budgets; it must implement its own bounds. Depth is checked before the next
parent callback. Backings resolve once per extent, then the loader receives the
same retained object. Identity/alias checks precede backing metadata reads.

The Linux `inspect_chain` example is a qualification helper with one explicitly
selected confined directory and basename-only parent hints. It rejects traversal,
subdirectory hints and symlinks through this policy and `LocalResolver`; there is
no public CLI policy. R5.8 supplies [public CLI parent access](cli-vmdk-parents.md)
separately. The helper’s four-byte entry probe per source is outside
`SparseChain` read counters. The library itself never probes entry type.

## Aggregate budgets and ownership

| Limit | Default | Charged work |
|---|---:|---|
| Layers | 16 | Leaf and terminal base, inclusive |
| Extents | 128 | All loaded extent handles across layers |
| Descriptor payload | 8 MiB | Acquired outer descriptor bytes, including padding |
| Memory reservation | 128 MiB | Layer slots + acquired text + metadata structs + all per-extent loader reservations |
| Read payload | 256 MiB | Outer descriptor/header acquisition + all metadata reads and header rechecks |

Existing per-descriptor/per-extent limits also apply, including 1 MiB descriptor
payload. Admission uses checked arithmetic and remaining aggregate budgets before
loader allocation/offset reads. Descriptor parsing is bounded by its own limits;
aggregate extent admission occurs after parsing and before backing resolution.
Metadata acquisition includes embedded descriptor reads a second time; those
reads and reservations are charged in the metadata loader, not deduplicated.

Memory is a conservative sum, not peak RSS: all permitted layer slots are reserved
up front, and released descriptor/scratch buffers remain charged. Parser vector
capacity, allocator overhead, resolver-owned state, source contents and fixed
stack buffers are excluded. Endpoint observations perform no charged payload reads.
Limits bound work/storage, not elapsed time. Caller changes to limits cannot bypass
checked arithmetic or format validation. A late error drops partial chain state.

Layers retain descriptor handles and immutable maps in leaf-to-base order. Sources
and resolvers can otherwise be dropped after successful loading. A final sweep
rechecks earlier descriptors/backings after later layers are acquired;
`revalidate()` repeats endpoint observations. It checks identity/readability/size,
not text, CIDs, timestamps or all data bytes. The caller must keep every source
quiescent. Same-size writes are outside this contract: no lock, snapshot, live-VM
consistency or immutable-content guarantee is provided.

## Qualification and next step

Sixteen new tests cover opt-in grammar, padding/bounds, both entry forms, retained
ownership, missing/denied parents, CID/capacity mismatch before parent backing reads,
self/long cycles, equal-CID distinct objects, shared backings, unknown/changed
identities, embedded provenance, differing extent geometry, failures, exact aggregate
boundaries and the inclusive 16-layer limit. Counters verify actual acquisition
payload. QEMU creates unmodified base, monolithic, split and mixed chains; the
helper's layer/CID/capacity observations match descriptors and QEMU's backing-chain
report. These are metadata checks, not decoded parent-byte qualification.

Six new benchmark cases establish child parsing, 1/4/16-layer embedded acquisition,
16-layer endpoint revalidation and 17-layer depth rejection. Fixture generation is
outside timing; acquisition includes parser/metadata validation, allocation and
drop. Memory fixtures are not filesystem or storage-throughput measurements.
Existing metadata/read/CLI controls retain paired comparisons and adverse repeats.

R5.7 now adds [read-only logical parent fallback](vmdk-chain-disk.md), range mapping, differing grain/
extent boundaries, whole-chain destination alias protection and independent byte
oracles. An unallocated child grain falls through until allocated data or the base
is reached. Only a fully resolved zero range can become logical Zero. R5.8 adds [public CLI
parent acquisition](cli-vmdk-parents.md). ESXi is still unnecessary for local work.
