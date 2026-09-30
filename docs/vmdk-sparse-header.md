# Hosted sparse header admission (R5.1)

`SparseHeader` parses the first 512 bytes of a hosted sparse extent. It validates
header geometry and advertised metadata ranges without allocating, opening
references or reading directory/table entries. It is **not a sparse disk reader**:
VmdkDisk and the public CLI still accept only the documented FLAT/ZERO subset.

The field layout and units were checked against VMware's
[Virtual Disk Format 5.0, pp. 6–9](https://github.com/vmware/open-vmdk/blob/master/vmdk_50_technote.pdf).
This implementation was written from the specification, without reading or copying
another parser implementation. The inspected PDF SHA-256 is
`88ce1615a703d1d4e3df3c227846bb9ecda4d929129c191ea4ffe7df3294ad59`.
The restrictions below are the rvvdk admission policy, not a claim to cover every
valid hosted sparse file.

## Supported header subset

| Property | Admission policy |
|---|---|
| Input | Exactly one 512-byte sector, packed little-endian fields decoded by fixed offsets |
| Magic / version | Hosted sparse magic, version 1 only |
| Flags | Only newline-test and redundant-metadata bits; unknown/zero-grain/compression/marker flags reject |
| Grain geometry | Power of two, greater than 8 sectors; nonzero capacity must be grain-aligned |
| Grain table shape | 512 entries, 4 bytes each |
| State | Clean shutdown byte only; compression algorithm zero; reserved padding zero |
| Newline bytes | Checked when their flag is present; otherwise ignored |
| Descriptor fields | Both offset and size zero, or both nonzero; nonzero pair advertises a bounded region |
| Directories | Nonzero primary offset; redundant offset required with its flag and zero without it |
| Footer directory | Sentinel offset rejects; stream/marker layouts are not supported |
| Overhead | Nonzero, grain-aligned, within the observed extent file and metadata limit |

Sector arithmetic, range ends, directory rounding and full table-storage counts
use checked arithmetic. Directory entry count is the ceiling of grain count / 512.
Directory storage rounds its four-byte entries up to a sector. Descriptor and
active directory regions start after the header sector, fit within overhead, and
do not overlap each other. Overhead must accommodate the header, advertised
descriptor, active rounded directories and all full-sized grain tables implied by
the directory count. This checks available space, not actual table placement.

## Limits and API

| SparseLimits field | Default |
|---|---:|
| capacity_bytes | 1 TiB |
| grain_bytes | 16 MiB |
| descriptor_bytes | 1 MiB |
| directory_entries | 1,048,576 |
| metadata_bytes | 256 MiB |

The metadata ceiling applies independently to advertised overhead and computed
minimum full metadata storage, including redundancy. Satisfying the capacity
ceiling alone does not ensure another limit admits the geometry. Custom limits
may tighten or relax resource policy but never bypass checked arithmetic or the
supported syntax. These bounds are not a cache, allocation or RSS promise.

`SparseHeader::parse(bytes, extent_bytes)` applies defaults;
`parse_with_limits` accepts custom limits. The file length is an observation of
the container, not virtual capacity. `read_from(reader, extent_bytes, limits)`
reads exactly one sector from the reader's current position into a stack buffer,
retries Interrupted reads, and propagates short reads/I/O errors. It never seeks,
follows offsets or reads an extra byte. The caller must supply the first sector
and bind/revalidate the owned source; these APIs do not provide a snapshot.

The immutable result exposes flags, capacity/grain sizes, grain/directory counts,
fixed table size, optional descriptor/redundant regions, primary directory,
overhead and minimum metadata storage. `SparseRegion` exposes checked offset,
length and end; callers cannot construct unchecked regions through public fields.
Errors distinguish input length/magic, unsupported features, invalid relationships,
arithmetic overflow, policy limits, physical bounds and acquisition I/O.

## What header admission does not prove

Directory/GTE contents, table alignment/overlap, data-grain locations, redundancy
agreement, descriptor syntax/capacity/type agreement, source consistency and
parent identity remain unvalidated. No unallocated grain is treated as logical
zero here. No native RAW endpoint, writes, decompression or parent resolution is
introduced. Version 2, dirty recovery, streamOptimized, VMFS sparse and seSparse
need separate contracts and qualification.

The [reference evidence](benchmark-results/2026-09-30-r51/README.md) admits three
QEMU-generated headers and deliberately rejects unaligned capacity and stream
variants. It verifies capacity against QEMU info and records original header bytes
and hashes; it does not compare decoded sparse bytes. QEMU's split fixture has an
advertised descriptor region containing only zeros. R5.2 must bind the external
descriptor explicitly instead of assuming that an advertised region holds text.

Next is R5.2: bounded grain-directory/table acquisition and validation, including
metadata placement and redundancy policy, before logical sparse reads.
