# ADR-0035: Map retained FLAT/ZERO sources through VirtualDisk

Date: 2026-09-29. Status: Accepted for R4.3.

## Context

The parser and resolver establish metadata and retained physical sources. A
logical reader must translate offsets, preserve ZERO semantics, handle cross-extent
reads and avoid bypassing those rules through native RAW execution. Composite
sources also have multiple physical identities; the existing single-identity
preflight cannot exclude destination aliases to every backing.

## Decision

Add read-only VmdkDisk over an owned ResolvedDescriptor, validated at construction
and during endpoint observations. Use binary search for the first extent, then
sequential traversal with direct caller-buffer reads/zero fills. Reject complete
invalid ranges before changing the buffer. Propagate short-read EOF and I/O errors;
errors may leave a partial buffer. Report clipped/coalesced Data/Zero extents without
assuming physical holes. Keep fixed logical 512-byte geometry and no native FD API.

Add VirtualDisk::validate_destination_identity with a no-op default, called after
fresh endpoint/access/range checks by portable copy and verification. VmdkDisk
checks every known backing; unknown identity fails closed for those operations.
Wrappers must forward the hook. Ordinary RAW identity and native binding checks
remain intact. This extends the portable contract without changing CopyEndpoint's
single-object meaning or allocating a list of identities on every RAW preflight.

## Consequences

Logical reads allocate no per-request mapping state and can run concurrently.
Extent discovery allocates a bounded map. Revalidation is outside the per-read hot
path; callers still need stable contents and must handle failures during execution.
Custom sources without identity can be read but cannot be copied/verified safely
through the supplied workflow, unless the VMDK contains only ZERO extents.

[The contract](../vmdk-logical.md) distinguishes oracle tests from external decoder
qualification. QEMU agrees on the supported hosted layouts. Its generated padded
headers need fixture-only normalization under the current text policy, and it
rejects custom createType. These are retained limitations, not silently accepted
variants. CLI VMDK integration follows in R4.4. [Performance evidence](../benchmark-results/2026-09-29-r43/README.md)
records matched controls, new read/copy costs and any adverse repeats.
