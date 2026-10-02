# ADR-0060: Qualify durable acquire/abort before payload integration

Status: Accepted for R6.1c.2a; transfer/completion integration remains open.

## Context

The R6.1b Job borrows its store and performs synchronous durable writes. R6.1c.1
adds explicit selection to an async export proof, whose artifact writer has no
journal ownership. A dropped blocking task or an acknowledgment recorded after a
remote action cannot establish correct ownership or intent ordering.

## Decision

Add a separate Linux acquire/abort API. A blocking worker owns the store and Job
for its entire lifetime. Serial commands cross one bounded channel; accepted writes
are drained before the async API returns. The worker issues no remote calls and
exports no stage handles. There is no self-referential borrowed structure moved
between async tasks.

Persist acquisition intent, revalidate the source, then issue ExportVm once.
Keep the parsed lease capability process-local. Record its fingerprint and durable
abort intent before issuing one abort. Record the response independently from its
journal acknowledgment. Keep uncertain records when either observation fails.

Ready-lease heartbeats continue during journal waits. Cancellation and heartbeat
failure do not discard an accepted command; cleanup may proceed only after the
required durable acknowledgment. After worker shutdown, explicit checked local
cleanup remains available for eligible states. Do not infer remote ownership from
records, auto-retry uncertain requests, or promote the legacy proof writer.

## Consequences

This qualifies real intent/RPC ordering and ownership of an empty stage without
pretending that payload validation, completion or publication is integrated. One
blocking thread is occupied per active probe. Blocking filesystem operations can
outlast network deadlines, and callers must await completion for cleanup reporting.
Whole-process termination can leave an intent and remote uncertainty; reopening
only assesses it.

Seven journal commits plus stage durability have a substantial measured fixed
Btrfs cost. Keep barriers and report CPU/wall/allocation separately from payload
throughput. tmpfs comparisons measure overhead on volatile storage, not durability.

Next R6.1c.2b integrates transfer and completion, with outstanding-writer coordination
and no abort after an uncertain completion. Subsequent packages cover private
verified metadata, conversion and actual journal-bound publication.

[Contract](../durable-export-probe.md),
[evidence](../benchmark-results/2026-10-02-r61c2a/README.md).
