# ADR-0055: Confined local CLI streamOptimized conversion

Status: accepted (R5.12). Extends [ADR-0054](0054-bounded-native-stream-reads.md).

## Context

The native reader is qualified against independent RAW, QEMU and guest bytes.
Users need local conversion without bypassing source confinement, alias checks,
request limits or the established output publication policy.

## Decision

Within explicit `--format vmdk`, binary version 3 dispatches to `StreamDisk`.
Dispatch is not admission: the complete bounded header, descriptor and grain map
must pass. Other versions continue through the existing hosted sparse parser.
The confined opened container is adopted directly and retained with its observation
handle. Embedded stream filenames are descriptive and never resolved or followed.
There is one physical backing even when its descriptor names another file.

All commands operate on logical bytes. Preview reports `stream_optimized`, map
and decode reservations, allocated grains and `payload_validation: on_read`.
Metadata admission does not claim payload checksum validation. Copy and verification
use portable execution; `auto` selects threaded and explicit `io-uring` rejects.
The existing `--allow-parents` policy stays specific to hosted sparse chains and
rejects stream input. No compressed disk exposes a native RAW descriptor.

A stream block size above 64 MiB rejects before destination preparation or verifier
allocation, even if a short source would use smaller requests. This conservative
policy fits the reader's default request, grain-work and encoded-input bounds.
Map/decode reservations remain separate from the copy payload memory budget.
One shared decode slot serializes allocated-grain reads across copy workers.

The existing source stamp checks, source/destination alias checks, cancellation,
private output, optional readback, file synchronization, no-replace publication
and directory synchronization remain in force. Corrupt payloads can fail after
partial in-place overwrite; new output is not published. External source quiescence
is still required. Metadata stamps do not provide an immutable snapshot.

## Validation and consequences

[CLI contract](../cli-vmdk.md), [tests and measured conversion results](../benchmark-results/2026-10-02-r512/README.md).
Synthetic front/footer conversions cover cross-grain boundaries and sparse holes.
Complete output comparison uses authored RAW and QEMU; the retained export also
has an independent guest oracle. Benchmarks separate complete CLI operations from
R5.11b map/decode measurements and preserve adverse controls.

This completes local conversion for the admitted subset. It does not add network
random access, parented stream disks, VMFS sparse, seSparse, live snapshots or
recoverable production export jobs. R6.1 must define source selection/trust,
artifact identity and durable ownership before connecting export to conversion.
PERF.0 retains controlled-host, cache/admission and bounded decode-scaling work.

The retained-export CLI attempt also exposed an allocation limitation: Zero work
uses the existing local `ZERO_RANGE` path and exhausted the small XFS runner root.
The private output was removed. Qualification continues on the development host
with sufficient space, preserving this failed attempt. R5.12p precedes R6.1 to
qualify space-efficient zero output without changing logical guarantees or
claiming that encoded size bounds required destination storage.
