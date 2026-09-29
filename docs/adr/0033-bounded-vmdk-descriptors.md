# ADR-0033: Validate a bounded hosted descriptor subset before backing I/O

Date: 2026-09-29. Status: Accepted for R4.1.

## Context

The next disk format must not blur textual declarations, physical backing access,
and logical disk reads. Untrusted descriptors can contain overflowing sizes,
unsupported layouts, parent references, and filenames outside a descriptor's
location. The RAW CLI and its native FD assumptions cannot be reused directly.

## Decision

Add a standalone portable `rvvdk-vmdk` parser with no filesystem dependency or
`VirtualDisk` implementation yet. Borrow strings, bound text/lines/extents/metadata,
validate integer arithmetic and emit typed line-specific errors. Restrict support
to version-1 hosted base layouts: monolithicFlat, split flat, and custom FLAT/ZERO.
Reject unknown fields and unsupported storage features explicitly. Preserve
informational metadata through a small allowlist. Source access flags never grant
write support. See the [precise contract](../vmdk-descriptor.md).

Keep parsed structures immutable through getters. Parsing proves structural byte
ranges, not backing-file safety. R4.2 supplies resolution/confinement/live-size and
identity validation; R4.3 supplies read-only logical mapping. No inferred VMware
remote-access compatibility follows from local format work.

## Consequences and evidence

The initial compatibility surface is intentionally small and testable. Extra DDB
keys, escaped paths and managed images can be valid VMDKs but are rejected until
explicitly supported. Borrowed descriptors require the input buffer to outlive
metadata. Entry bounds allow vector growth overhead; future file acquisition has
its own bound. No proprietary SDK or third-party implementation is required.

Synthetic fixtures document provenance. Boundary/adversarial tests and portable
compilation accompany the parser. [R4.1 performance evidence](../benchmark-results/2026-09-29-r41/README.md)
measures ordinary and maximal extent parsing and early/late limit rejection;
matched RAW controls track existing behavior. There is no prior parser baseline,
so parser results establish costs without a speedup claim. ESXi remains unnecessary.
