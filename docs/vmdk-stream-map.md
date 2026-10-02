# Bounded streamOptimized grain index — R5.11a

`StreamMap` validates directory/table pointers and physical record ownership for
the [R5.10 envelope subset](vmdk-stream-admission.md). It exposes immutable record
locations and lookup by logical grain index. It does **not** decompress payloads,
implement `BlockDevice`, or enable compressed disks in the public CLI.
[Decision](adr/0053-bounded-stream-grain-index.md),
[tests, reference checks and plots](benchmark-results/2026-10-02-r511a/README.md).

## Accepted map subset

Both observed QEMU front-directory images and VMware footer images use 64 KiB
grains, 512-entry tables, and 32-bit sector pointers. Zero directory/table entries
represent absent base grains; pointer value one is rejected. Present records must
appear in increasing logical-grain order, packed at sector-rounded physical
positions starting at the admitted overhead. Unused directory/table slots must
be zero. This is deliberately narrower than every potentially valid producer.

For front images, followed primary/redundant tables lie entirely before overhead.
Header, descriptor, directories and all followed tables must be disjoint. Optional
redundancy must agree on table presence and every table byte; the two copies must
occupy different regions. The final grain ends at the encoded extent length.

For footer images, each present table immediately follows its group's grains
with a one-sector GT marker. Groups may be absent. Followed GT markers/tables are
ordered, disjoint, after overhead and before the GD marker. The last group ends
at the GD marker. The existing envelope validates the GD/footer/EOS trailer.
Marker padding is ignored. Unreferenced pre-data overhead bytes are not scanned.

Each nonzero GTE must identify the next physical grain record. Its 12-byte prefix
must contain a bounded compressed length and the exact logical sector address
implied by its table slot. Sector-rounded record ownership rejects aliases,
gaps, orphan records, grain/metadata overlap and records crossing table/trailer
boundaries. Payload contents and grain-sector padding are **never read** by this
layer. An invalid compressed stream can therefore have a valid `StreamMap`.

## Acquisition and limits

`StreamMap::read_from(Read + Seek, encoded_length, StreamMapLimits)` reacquires an
envelope from the same source; callers cannot inject a previously validated map.
It reads directories once and validates all followed GT locations/overlaps before
table I/O. Two table passes first count populated grains, then fill an exactly
sized sparse index while checking each record prefix. Both passes compare any
redundant table. Allocation uses fallible reservation. Checked arithmetic and
explicit resource bounds precede dependent reads/allocation.

| Default limit | Value / meaning |
|---|---|
| Envelope | Existing `StreamLimits`, including 1 TiB virtual capacity, 2 TiB encoded extent, 128 KiB compressed grain length |
| Map memory | 128 MiB of requested directory, overlap-scratch and retained-index slot bytes |
| Metadata reads | 256 MiB requested across envelope, directories, both table passes, record/GT prefixes and final header recheck |
| Table work | 32 Mi primary GTE slots examined across both passes, including unused slots |
| Populated grains | 4 Mi records; 256 GiB fully populated logical data at 64 KiB/grain |

Limits apply together; the virtual-capacity maximum is not a promise that every
dense image of that capacity is admitted. The retained index stores 12 bytes per
populated grain, ordered by logical index, and uses binary search for lookup.
Directory storage grows with virtual capacity, while index storage grows with
populated grains. An empty 1 TiB footer image requests 131,136 map slot bytes and
zero table work. The map memory counter excludes separately bounded envelope
parsing, fixed stack buffers and allocator overhead; it is **not RSS**, allocator
capacity, total live process memory or device traffic. Read counters describe
requested bytes, irrespective of filesystem caching or read amplification.

The caller must keep the source quiescent during acquisition and subsequent use.
Final semantic-header/length rechecks and two-pass count checks catch some changes,
but do not provide a snapshot, locking, full metadata stability, cryptographic
identity or persistent source binding. Same-count changes may go undetected.
`StreamGrain` fields are private; a returned location is still meaningful only
for the exact quiescent source whose map was acquired.

## Validation and continuation

Fourteen tests cover sparse/dense/empty maps, multiple/absent tables, empty 1 TiB
capacity, optional redundancy and disagreement, aliases, reserved pointers, wrong LBAs, size
bounds, orphan/gap records, footer markers, unused slots, exact resource limits,
read traces, truncation and selected acquisition mutations. Synthetic payloads
are intentionally invalid compressed data, making the validation boundary explicit.
Six QEMU/authored images agree with independent table enumeration; QEMU decoding
still matches authored RAW. The retained ESXi export also passes map admission.
Only aggregate live counters enter Git; guest content and disk identities stay private.

[R5.11b native reads](vmdk-stream-reads.md) now retain source ownership, validate
exact checksummed zlib input/output and provide sparse-zero/cross-grain reads.
Complete QEMU and guest-oracle byte comparisons and repeated read/CPU/RSS
measurements pass within that documented scope. This map API itself still does
not validate payloads. R5.12 separately gates public CLI integration. Existing PERF.0 follow-ups remain open.

```sh
cargo test -p rvvdk-vmdk --test stream_map
cargo bench -p rvvdk-vmdk --bench stream_map
cargo build --release -p rvvdk-vmdk --example inspect_stream_map
# First generate a fresh synthetic corpus using the R5.10 reference command.
python3 scripts/vmdk/compare_stream_map.py \
  --inspect target/release/examples/inspect_stream_map \
  --directory target/stream-reference-new --report target/stream-map.json
```

The helper defaults to aggregate JSON. `--records` exposes grain locations and
is used only for synthetic fixture reports. Python orchestrates offline references
and plotting; VMware access and VMDK parsing remain native Rust.

Format reference: VMware's [VMDK 5.0 technical note](https://github.com/vmware/open-vmdk/blob/master/vmdk_50_technote.pdf),
sections on sparse directories/tables and streamOptimized records. No vendor SDK
or QEMU implementation code is imported.
