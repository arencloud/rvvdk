# ADR-0052: Separate bounded streamOptimized envelope admission

Status: Accepted for metadata only, 2026-10-02. Performance follow-up open.

## Context

V0.3.2c qualified independent Rust export of a compressed version-3 disk, while
the native reader supports uncompressed hosted sparse disks. Reusing that reader
for compressed input would treat encoded records as logical blocks. The observed
QEMU front-directory profile also differs from the documented VMware footer
profile and the actual ESXi export.

## Decision

Add separate stream descriptor/header/marker/envelope types with explicit limits.
Admit the documented 64 KiB-grain footer subset and the observed QEMU front subset.
Validate bounded geometry, descriptor capacity and footer structure without
following grain pointers, decoding payloads or enabling CLI reads. Preserve old
parser behavior and the existing public format enum. Share the bounded descriptor
grammar through a compile-time mode, with the ESXi informational DDB key scoped
only to that mode. Embedded references are never opened.

Use authored adversarial fixtures, QEMU-produced front images and QEMU-decoded
authored footer layouts. Probe only metadata of the retained private export with
the Rust helper. Keep all guest content and image identities outside Git.

## Consequences and evidence

The [contract](../vmdk-stream-admission.md) states exactly which fields are checked
and which maps/records remain unvalidated. Existing public reads continue to
reject streamOptimized. Header/length rechecks require a quiescent source and are
not a snapshot. R5.11 must validate maps and record ownership before decoding;
R5.12 remains the public integration gate.

An initial design enlarged the existing public format enum. Review removed that
unnecessary change and retained only the stream descriptor's required fields.
Both measured designs, all adverse pairs and longer repeats are preserved in the
[evidence](../benchmark-results/2026-10-02-r510/README.md). Final small-descriptor
longer pairs remain +8.45%, +10.05% and +8.83%; this is unresolved, not a justified
correctness cost or a cleared gate. Investigate it in R5.10p before R5.11. The
shared host limits causal attribution; the revision alone does not prove a speedup.


## R5.10p follow-up

The [scoped comparison specialization](../benchmark-results/2026-10-02-r510p/README.md)
recovers the measured flat-descriptor cost without changing grammar or limits.
The parser gains 2926 bytes of generated code. Primary stream timing remains
mixed; separate-core identical-binary controls expose substantial measurement
variability but do not clear the primary observations. Accept the scoped
optimization with that code-size cost; retain PERF.0 controlled measurement and
layout investigation. No change to the metadata-only architectural boundary.
