# ADR-0036: Explicit logical VMDK sources in the CLI

Status: Accepted. Date: 2026-09-29. Implements R4.4.

## Context

The CLI assumes one RAW file for source capacity, identity, timestamp observations
and native execution. A VMDK descriptor instead names a logical mapping over
several files; treating its FD as disk bytes would silently copy the wrong data.
Overwriting any backing or the descriptor must reject before mutation.

## Decision

Add explicit `--format vmdk` to inspect/plan/copy/verify with RAW destinations.
An owned source enum preserves the typed RAW native adapter and uses VmdkDisk's
portable logical APIs for VMDK. Threaded and Auto are supported; explicit io-uring
rejects before destination effects. Do not implement a native FD interface for
logical VMDK disks or bypass their composite destination identity hook.

Expose `LocalResolver::open_descriptor_file` and confined `open_regular` so the
CLI can retain metadata observations of the same objects supplied to the reader.
The existing `open_descriptor` convenience API delegates to the new file opener.
A bounded observing resolver deduplicates with ResolvedDescriptor, retains each
opened file before adoption, and records identity/size/mtime/ctime. Descriptor
text remains bounded and validated by DescriptorText. No untrusted path reopen
is introduced. The existing local BlockDevice admission registry is reused.

Target opening uses logical size and checks all source identities, including the
descriptor, before overwrite. Metadata-only previews apply the same alias policy;
verify also rejects descriptor/backing aliases. Acquisition and final observations
detect changes but cannot establish a coherent snapshot against external writers.

Keep publication, durability, progress, cancellation and verification ownership in
the existing CLI lifecycle. Reports keep schema version 1 with explicit format and
optional VMDK details; geometry/extents/counters describe logical disk bytes.

## Consequences and validation

One additional observation FD per distinct backing remains live until command
completion, plus the descriptor FD. Existing parser/resolver bounds apply; these
resources are separate from the execution payload budget. No read-loop stat scan
is added. Shared CLI RAW paths receive matched before/after benchmarks.

Integration tests exercise logical previews, mixed-offset/repeated-source copies,
Zero regions, single/four-worker behavior through existing engine tests, odd block
boundaries, overwrite tails, mismatch offsets, every backing/descriptor alias,
strict parsing/confinement, early native rejection, source mutation, cancellation
and publication collisions. See the [CLI contract](../cli-vmdk.md) and
[performance evidence](../benchmark-results/2026-09-29-r44/README.md).

Padded descriptors and independent custom-layout decoder qualification remain
explicit follow-ups. Local format support implies no VMware transport support.
