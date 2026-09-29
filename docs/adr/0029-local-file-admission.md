# ADR-0029 — Cooperative admission for local regular-file aliases

Status: Accepted, 2026-09-29 (R2.6).

## Problem

The local backend can route unaligned I/O through a buffered alias of its direct
handle. Native SQEs bypass those methods, and independent opens/hard links can
reach the same inode. Backend-only locks would leave native and alias overlaps
uncoordinated; blocking inside enqueue could prevent completion draining.

## Decision

Place a cooperative identity registry and per-file range coordinator in
rvvdk-platform. Both local methods and regular-file IoUringFile requests join it.
Fail conflicting admission before I/O with a typed core error. Same-mode readers
and disjoint byte ranges can coexist; any overlapping writer conflicts. Mixed
buffered/direct payload modes conflict when their OS pages intersect. Sparse
operations take buffered-write intent, extent scans take inspection intent, and
flush takes a whole-file exclusive barrier.

An owned native guard follows the same confirmed-completion/quarantine lifetime
as its file and buffer. Mutexes protect metadata only, never I/O or callbacks.
There is no automatic retry or executor fallback on an admission conflict.

## Consequences

The policy changes overlapping operations from unsynchronized execution to an
explicit error and adds per-request bookkeeping. It preserves disjoint native
pipelines, including direct blocks smaller than a page. Range scans are linear
in active requests per file; registry access occurs at handle registration.
Vector growth can fail before submission. Native guards count in operation payload
budgets; shared backend bookkeeping remains outside the per-copy budget.

This is cooperative request admission, not an atomic copy, snapshot, kernel lock,
or coordination with external writers. Partial exact-read/all-write loops and
concurrent final flushes require caller scheduling. Unconfirmed native shutdown
can retain admission permanently. Caller-owned raw FDs and status changes remain
outside the contract. Full policy, migration guidance, and tests are documented
in [the concurrency contract](../local-file-concurrency.md).

[Matched measurements](../benchmark-results/2026-09-29-r26/README.md) retain raw
samples, adverse repeats, and SVG/PNG plots. This decision does not change tuning
defaults or close prior controlled-runner performance qualification.
