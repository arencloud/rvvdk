# Read-only base sparse logical mapping (R5.3)

`SparseDisk::load` creates an owned `VirtualDisk` from an explicit SparseDescriptor,
BackingResolver and SparseDiskLimits. It admits only the clean, uncompressed
version-1 base hosted subset defined by [header admission](vmdk-sparse-header.md)
and [metadata validation](vmdk-sparse-metadata.md). Supported descriptor types are
monolithicSparse and twoGbMaxExtentSparse/2GbMaxExtentSparse. Parents, compressed
streams, dirty recovery, version 2 and managed variants remain unsupported.

This is a library API. [R5.4 CLI integration](cli-vmdk.md) now accepts its supported
subset through explicit VMDK mode. Existing Descriptor/VmdkDisk paths remain
FLAT/ZERO-only.
No VMware SDK or third-party implementation source is used.

## Ownership and admission

The loader validates every selected extent with SparseMetadata, retains each
source and map, and drops all acquired handles/maps if any later step fails. Input
text and the resolver can be dropped afterward. Repeated references are resolved
and charged separately; there is no deduplication or assumption that names uniquely
identify objects. Final acquisition revalidates all endpoints, including earlier
sources that could have changed while later ones loaded.

| SparseDiskLimits field | Default | Boundary |
|---|---:|---|
| metadata | SparseMetadataLimits defaults | Independent per-extent limits |
| extents | 128 | Loaded extents/handles; checked before resolver calls |
| memory_bytes | 128 MiB | Sum of loader buffer reservations plus metadata structs |
| read_bytes | 256 MiB | Sum of complete metadata read payloads |
| output_extents | 65,536 | Coalesced entries in one logical extent query |

The memory reservation includes `count * size_of::<SparseMetadata>()` for the
owning vector plus each loader's reservation, including its temporary scratch.
Summing reservations is deliberately conservative: released scratch is not reused
as admission credit. Each next loader receives the smaller of its per-extent limit
and the remaining aggregate. No later metadata offset read or variable metadata
buffer allocation can exceed that admission. A rejected loader may already have
read its header; a remaining read budget below 512 rejects before resolving it.
Arithmetic and fallible vector allocation are checked.

These are payload limits, not RSS or time limits. Supplied descriptors, parser-owned
collections, resolver/Arc resources, allocator overhead and fixed stack buffers
retain their R5.2 exclusions. Logical read buffers belong to the caller. Extent
query output is separate: a first allocation-free pass counts coalesced kinds and
rejects the output limit, then an exact-size vector reservation precedes a second
pass. No query allocation grows with uncoalesced grain count. Concurrent callers
must budget their own combined query outputs and read buffers.

## Reads and logical extents

Reads validate the complete range before changing the buffer. Empty reads at EOF
succeed; overflow and out-of-bounds ranges reject. A binary search selects the first
backing extent, then direct grain indexing maps the start. Reads cross grain and
split boundaries, including different supported grain sizes in different extents.
Consecutive physical grains within one backing are combined into one exact-read
request; fragmented grains remain separate requests. Short backend reads complete
through the existing exact-read helper. EOF and I/O errors propagate, never turning
missing allocated data into zeros. An I/O failure can leave an already-read buffer
prefix changed; this is not transactional I/O.

Unallocated grains are zero-filled because the descriptor subset requires a base
disk without a parent. No payload read is issued for those ranges. This guarantee
belongs to SparseDisk, not to a bare SparseMetadata map. Allocated grains are Data
even when their actual payload happens to contain zeros. Extent queries clip to
the request and coalesce adjacent logical Data/Zero kinds across physical fragments
and backing boundaries. They never report Hole or expose physical file offsets.

Reads allocate no internal buffers and do not reopen paths. Extent queries perform
no source I/O. Metadata is immutable and supports concurrent readers. Opening cost
includes eager map acquisition; subsequent reads do not reparse descriptors/tables.

## Identity, consistency and execution

Capabilities are READ, EXTENTS and SPARSE with fixed 512-byte logical geometry.
Writes, zero writes, discard and flush return Unsupported. There is no native RAW
endpoint. The existing portable single/concurrent DataMover and Verifier can consume
this VirtualDisk through their normal planning and preflight contracts.

The composite endpoint has no single identity. After endpoint revalidation,
`validate_destination_identity` compares the destination with **every** initial
backing identity, including metadata-only/unallocated backing files. A match rejects
as AliasedEndpoints. Unknown source or destination identity fails closed for copy
and verification; standalone reads with unknown identities remain unqualified.
This also catches hard links. Path replacement does not redirect retained handles.

Standalone callers must call `revalidate()` before read sessions and maintain source
quiescence throughout use. DataMover/Verifier preflight calls `copy_endpoint()`, which
revalidates all backing sizes, read access and known identities. These observations
are not a snapshot, lock, content lease or metadata replay. Same-size table/payload
mutation is outside the contract, and mutations racing after preflight remain a
caller consistency responsibility. The library does not claim live-VM consistency.

## Evidence and next step

[R5.3 evidence](benchmark-results/2026-09-30-r53/README.md) records range/zero/fragment
and split tests, aggregate rejection, partial/error reads, concurrent copy/verify,
unknown/aliased/changed identities and retained-inode behavior. QEMU-generated
monolithic and split fixtures compare full production logical reads with original
patterned RAW oracles and QEMU. A multi-file fixture crosses the 2 GiB boundary.
The reference helper reads every byte in 65,537-byte chunks; it leaves zero-only
output chunks sparse without substituting a second mapping implementation.

R5.4 now provides [sparse CLI acquisition and integration](cli-vmdk.md), preserving
descriptor provenance, publication, cancellation and alias checks. Next is R5.5:
adversarial validation and capacity/fragmentation benchmarks. Parent chains remain
separate. ESXi is not required for this local work.
