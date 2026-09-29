# ADR-0031: Descriptor-bound RAW copy and private publication

Status: Accepted, 2026-09-29. Scope: R3.2 local Linux RAW workflows.

## Context

R3.1 previews express no-clobber creation or explicit in-place overwrite but are
not execution authorization artifacts. Execution must independently validate real
endpoints, budget simultaneous buffers, and identify partial effects on failure.

## Decision

Use an owned descriptor throughout copying and durability operations. Add a
buffered-file adoption API to the local backend; do not reopen an output pathname
for DataMover. Inspect existing leaves without following symlinks, compare the
subsequent read/write descriptor identity, reject aliases and insufficient capacity,
and preserve its inode and tail. Namespace stability remains a caller obligation.

For absent output, retain the parent descriptor and create an unnamed O_TMPFILE
in that directory. Copy and flush, optionally compare all logical bytes with a
bounded reusable Verifier, check source metadata, sync the file, publish with
no-replace linkat through procfs, then sync the directory and check name identity.
Fail if this platform/filesystem cannot support the protocol. No named fallback
or visible-output cleanup is implemented. A racing name is never replaced.

Reserve both verification buffers before copy mutation and subtract their actual
Vec capacities from the copy payload budget while they remain alive. Verification
is a portable DataMover-library facility; Linux CLI owns filesystem publication,
metadata stamps and reporting. Known endpoint aliases reject verification.

Errors retain phase and conservative destination state. Uncertain publication or
successful publication followed by sync failure does not imply rollback. In-place
failure leaves potential partial writes; report-output failure after success
retains the completed result. No lifecycle cancellation is introduced in this step.

## Consequences

Private output avoids exposing partial new files and avoids ownership ambiguities
when cleaning up named temporary files. O_TMPFILE, procfs and directory sync are
explicit requirements. Duplicate final file sync is conservative: the engine's
flush remains part of its contract and CLI sync covers final publication ordering.
Future removal requires a measured and justified durability contract change.

Full logical read-back adds two buffers and a complete source/destination read;
its performance is measured separately. Metadata stamps cannot provide snapshot
consistency against external writers. Bounded verification is not a persistent
checksum or physical-media proof. Cancellation and lifecycle reporting remain
R3.3; crash recovery/resume remains a later milestone.

[Transfer contract](../cli-transfer.md) and
[performance/validation evidence](../benchmark-results/2026-09-29-r32/README.md)
define exact fields, platform requirements and limits.
