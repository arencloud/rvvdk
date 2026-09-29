# ADR-0034: Resolve VMDK sources through owned, confined backing handles

Date: 2026-09-29. Status: Accepted for R4.2.

## Context

A descriptor proves layout arithmetic, not file safety, identity or live capacity.
Opening an untrusted name after canonicalization can race a rename/symlink change.
Using a plain file open before checking type can open a FIFO/device for I/O. Future
remote sources must not be forced into local pathname semantics.

## Decision

Add bounded owned descriptor acquisition and a portable caller-supplied
`BackingResolver` returning owned BlockDevice sources. Preflight reference/extent
counts before opening; deduplicate exact reference strings; retain sources and
endpoint observations; validate read access and maximum required physical end.
Expose revalidation and physical read-only methods, with no logical disk/native
RAW endpoint yet. Reuse existing local I/O admission through buffered file adoption.

On Linux, pin the caller-authorized descriptor directory. Reject ambiguous lexical
names, use openat2 beneath/no-symlinks/no-mount-crossing restrictions, obtain O_PATH
and inspect regular-file type, then reopen the pinned object through trusted
procfs and compare identity. Never fall back to unconstrained pathname opening.
See the [full contract](../vmdk-backing.md) for ownership, resource and race limits.

## Consequences

The portable layer can resolve memory/remote namespaces, including sources whose
identity is unknown. The local implementation requires Linux openat2 and procfs;
symlinks and nested mount points are deliberately unsupported. Hard links inside
the selected tree are not excluded. Retained handles resist pathname replacement
but do not freeze contents. Higher-level execution must revalidate and propagate
read failures, and custom resolvers remain responsible for their authorization.

Descriptor loading validates syntax and callers parse again to borrow metadata;
this explicit ownership avoids self-references. Bounds cap input bytes, entries
and source count, with normal vector/hash growth overhead. Actual logical mapping,
reference-tool comparisons and CLI integration remain subsequent work packages.

Tests include deterministic replacement between pin/reopen, physical truncation,
resource limits before resolver calls and failure cleanup. [Measured evidence](../benchmark-results/2026-09-29-r42/README.md)
tracks unchanged parser/RAW controls and establishes initial resolution/load costs.
