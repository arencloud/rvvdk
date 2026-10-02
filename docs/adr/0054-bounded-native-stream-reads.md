# ADR-0054: Owned stream reader with fixed decode storage

Status: Accepted for the bounded native-read subset, 2026-10-02.

## Context

R5.11a validates compressed record ownership but exposes no logical bytes. Native
reads need source binding, exact decompression boundaries, sparse-zero semantics
and predictable memory under concurrent calls. The technical note names DEFLATE;
independent producer fixtures and the retained export establish the admitted
zlib wrapper profile.

## Decision

Add `StreamDisk` as a read-only `VirtualDisk` retaining the same physical
`Arc<dyn BlockDevice>` used to acquire its map. Require one checksummed zlib stream
per allocated grain, exact compressed-input consumption and exactly 64 KiB output.
Use `miniz_oxide` core with default features disabled, fixed caller-owned decode
state and a 64 KiB + 1 output buffer. No vendor SDK, C zlib, external runtime decoder
or unbounded expansion buffer is used.

Use one shared decode slot and last-grain cache. Serialize data reads to bound
memory independently of caller count. Invalidate cache keys before replacements
and before source revalidation. Bound request bytes, encoded input, decoded grain
work and extent output. Coalesce logical extents from sparse records without
scanning all virtual grains. Reject writes and unknown/aliased destination
identities at copy preflight. Preserve caller-enforced source quiescence.

## Consequences

[The contract](../vmdk-stream-reads.md) specifies partial-buffer errors, lazy
payload validation, accounting exclusions and revalidation limitations. Cache
hits are fast; random partial-grain misses still decode a complete grain.
Concurrent correctness is supported, but parallel decode scaling is deferred
until a separately bounded pool is justified by measurements. A generic caller
must respect per-request limits and the backend must support buffered byte reads.

Independent RAW/QEMU images and the retained export's full logical image agree.
The separate guest oracle also agrees. [Evidence and plots](../benchmark-results/2026-10-02-r511b/README.md)
record in-memory costs and repeated live CPU/RSS/read throughput with their cache
and shared-host conditions. No new export or VM state change was required.

The public CLI continues to reject compressed disks. R5.12 owns confined source
acquisition and full CLI conversion/verification qualification. Existing PERF.0
stream timing concerns and R6 artifact/recovery requirements remain open.
