# Bounded sparse metadata validation (R5.2)

`SparseMetadata::load` validates and retains one version-1 base sparse extent's
metadata. It is not a VirtualDisk or CLI sparse reader. Existing `Descriptor`,
DescriptorText, FLAT/ZERO resolution and CLI source support stay explicit and
continue to reject sparse inputs.

## Descriptor binding

`SparseDescriptor::parse` / `parse_with_limits` are a separate opt-in subset:
monolithicSparse (exactly one extent) or twoGbMaxExtentSparse/2GbMaxExtentSparse
(each extent at most 2 GiB), containing only SPARSE filename references. Existing
version/access/DDB rules and limits apply. The opt-in sparse parser accepts one
to eight hexadecimal CID digits as a 32-bit value (including QEMU’s unpadded
output); the existing FLAT/ZERO parser still requires exactly eight. Parent CIDs/hints, mixed extent
kinds, unknown keys and unsupported variants reject. Terminal NUL padding is
accepted within the total input bound. Common syntax is shared through a constant
parser mode; the default Descriptor parser is not widened. CreateType and
ExtentBacking gain explicit sparse variants; the flat resolver defensively rejects
them before opening sources.

The caller supplies a parsed base descriptor, extent index, BackingResolver and
SparseMetadataLimits. The loader resolves exactly that selected filename. A
LocalResolver provides the existing confinement policy. Embedded filenames never
cause another open. The header's capacity must equal the selected descriptor
extent, including when it is one part of a split disk.

A monolithic source must contain valid embedded sparse descriptor text. Split
sources may have no advertised descriptor region or an all-zero reserved region,
as observed in QEMU output. Any nonempty region must parse and agree with the
supplied CID, create type and complete ordered extent mapping (names, access,
capacities). DDB is validated but informational and excluded from mapping equality.
These checks bind metadata to the caller-selected source; they do not authenticate
CID or prove the caller supplied an authoritative descriptor.

## Metadata and redundancy policy

Before following directory offsets, the loader checks its buffer and complete-read
payload budgets. It reads the advertised descriptor and active directories, then
validates **all table ranges before reading any table**. Version-1 directories must
have a nonzero entry for every required grain table. Directory sector padding must
be zero. Table ranges must fit within overhead and must not overlap the header,
descriptor reservation, directories or any other table, including redundant copies.
All positions are sector units; no extra 2 KiB table alignment is inferred.

When redundant metadata is advertised, each redundant table must exactly equal
its corresponding primary table. Disagreement fails; there is no recovery choice
or silent fallback. A header without redundancy uses only its primary directory.
Unused entries beyond logical capacity in the final table must be zero.

Every nonzero GTE must describe a complete grain aligned to grain size, starting
at/after overhead and ending within the observed file. Duplicate physical grains
are rejected. The fixed grain size and alignment make distinct admitted ranges
disjoint. GTE value 1 therefore rejects in this version-1 subset. Zero entries are
retained as **unallocated metadata**, not exposed as a logical-zero guarantee.

Metadata placement checks sort ranges; duplicate grain detection sorts a separate
copy of the map so logical order remains intact. The result keeps the primary map,
source Arc, initial endpoint and binding fields. No grain payload is read, and no
writes, flushes or native RAW endpoint are introduced.

## Bounds and consistency

SparseMetadataLimits combines header and descriptor limits with default **128 MiB
loader buffer payload** and **256 MiB complete-read payload**, per extent. Admission
uses checked arithmetic before metadata-sized allocation or offset-following I/O.
Allocation uses try_reserve_exact and reports failure. A failed budget may already
have resolved the source and read its 512-byte header; a read budget below 512
rejects before resolve. Index errors also precede resolve.

The reserved buffer payload conservatively covers the advertised descriptor,
active rounded directories, four bytes per logical grain for both the map and sort
scratch, and 16 bytes per table-range slot plus four fixed region slots. Fixed
stack buffers add 5 KiB. Parser-owned collections, supplied descriptor storage,
resolver resources and allocator overhead are separately bounded by their APIs,
not included in this payload budget. It is not an RSS limit. Multiple loaded
extents need caller/job-level aggregate admission, provided separately by [SparseDisk in R5.3](vmdk-sparse-disk.md).

Complete-read payload counts the initial and final header, descriptor region,
active directories and full tables including redundancy. Short/error reads can
require additional backend calls; this is not a syscall-count or blocking-time
limit. No metadata cache eviction or incremental lazy table loading is provided.

Acquisition checks READ capability and observes source size/identity before and
after loading; known identity must remain known and equal. It rereads the header
and rejects observed changes. `revalidate()` checks endpoint size/access/known
identity only. Unknown identity remains explicitly unqualified. There is no
snapshot, timestamp/content lease, or revalidation of cached tables against later
same-size changes. Callers must keep sources quiescent throughout validation and
future use. A future logical reader must preserve this consistency contract.

## Evidence and next step

The [R5.2 report](benchmark-results/2026-09-30-r52/README.md) records tests, bounds,
reference maps and measurements. Generated monolithic and split fixtures include
allocated and unallocated grains; a test utility reconstructs their bytes from the
validated map and compares with the original RAW oracle and QEMU. This qualifies
those maps, not a production sparse reader or arbitrary VMware images.

The separate [R5.3 SparseDisk layer](vmdk-sparse-disk.md) now provides read-only base
logical mapping, aggregate admission, composite alias protection and retained-source
consistency. SparseMetadata itself still exposes no logical reads. R5.4 now adds [CLI integration](cli-vmdk.md); parent chains, version 2,
streamOptimized, managed variants and writes remain separate.
