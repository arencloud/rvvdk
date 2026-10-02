# ADR-0064: Own and verify private RAW output before publication

Status: Accepted for R6.1c.5a; publication and output recovery actions remain open.

## Context

A caller-owned destination from R6.1c.4 has no durable owner. The next publication
step needs explicit output identity, format and verification evidence without
reconstructing Job or a remote lease from retained records. Exported container
identity and logical RAW identity are different contracts.

## Decision

Consume RetainedArtifact into a synchronous private RAW operation, retaining its
store lock through all writes, readback and acknowledgment. Add a separate version-1
output journal bound to store, artifact, source, canonical container metadata,
output ID, operation ID and logical capacity. Leave the source journal unchanged.
Use seven transitions: Prepared, StageIntent, Staged, ConvertIntent, Converted,
VerifyIntent, Verified. Intent precedes effects; file/directory sync precedes the
relevant acknowledgment. Unknown versions and fields fail closed.

Create fixed members under a generated private stage using descriptor-relative,
no-follow exclusive opens and recorded identities. Reuse retained conversion checks
and DataMover; close writers before Converted. Compare all logical source and RAW
bytes with bounded buffers, hash the RAW, recheck source/stage and persist/read back
bounded private metadata before Verified. Native decoder reuse is explicit; an
independent authored oracle qualifies behavior in tests.

Keep uncertain transactions and partial stages. Read-only assessment provides no
handle or restart, publication, cleanup or remote authority. Do not reconstruct a
mutable owner from a record. A process can die after record rename before directory
sync; observed state and operation success therefore remain separate concepts.

## Consequences

The source capability and store lock drain before the public call returns. No output
writer escapes. Verified output remains private and consumes disk space until a
future explicit checked operation. An interrupted StageIntent may lack member
identity stamps; future cleanup must not guess ownership from a generated name.

Full logical readback adds O(logical capacity) work even for sparse outputs. It uses
2 MiB of buffers separate from copy payload and native-map budgets. Seven journal
commits and additional source hashing add latency. Measure the complete operation,
retain CPU/RSS/allocation and plots, repeat adverse results, and keep PERF.0 open.
No durability or admission barrier is removed for benchmark appearance.

Split R6.1c.5 into this output prerequisite, R6.1c.5b publication/recovery actions,
and R6.1c.5c composed live qualification. The umbrella remains incomplete. The
current step needs no ESXi access; VMware/data-path code remains Rust.

[Contract](../owned-raw-output.md),
[tests and measurements](../benchmark-results/2026-10-02-r61c5a/README.md).
