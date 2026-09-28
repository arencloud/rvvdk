# ADR-0025: Shared Structural Plan Validation

## Status

Accepted for R0.1, 2026-09-28. This decision covers structural validation and
dispatch. Cancellation, terminal lifecycle events, and executor consolidation
remain separate work packages.

## Context

`execute_plan` validated source size, destination capacity, block size, source
extent structure, and threaded buffer alignment. The initial single-worker
`execute_plan_with_observer` path bypassed these checks and called its copy loop
directly. This allowed stale plans to change the destination and allowed an
execution block size larger than the planned buffer to panic.

The other observer paths called `execute_plan` after emitting an initial
snapshot, so even structurally invalid plans produced an execution notification.

## Decision

Both public entry points call one private `validate_plan` operation before
structural plan execution or observer notification. It checks:

- source logical size;
- destination capacity;
- configured versus planned block size;
- current source extent validity and structural fingerprint;
- configured versus planned alignment for threaded execution.

Ordinary execution dispatches through `execute_validated_plan`. Observed
concurrent/native execution uses the same private dispatch after validation,
avoiding another source extent scan through the public entry point. The observed
single-worker executor retains intermediate reporting after the shared check.

Native compatibility and runtime alignment are still checked inside native
execution before native I/O. This change does not claim complete native preflight
or guarantee that every later preparation/I/O error occurs before an initial
notification. That broader preparation contract belongs to R2.

## Consequences

- Structurally invalid plans return the same error through both entry points,
  leave destination contents untouched, and emit no progress snapshots.
- Valid plans retain their existing copy semantics, statistics, flush boundary,
  and observer cadence.
- Validation adds one required extent scan to the formerly unchecked sequential
  observer path. It does not add another scan to ordinary/concurrent/native
  dispatch. Benchmark both cost and correctness rather than treating the old
  unchecked operation as a safe performance target.
- Structural fingerprints still do not establish source snapshot consistency.
  Callers must keep the source stable for the copy.
- Separate observed/unobserved sequential loops remain. R1 will consolidate their
  semantics; a broad loop rewrite is not required to close the validation bypass.

## Validation

Regression tests cover smaller/larger execution blocks, alignment changes,
source-size changes, insufficient destination capacity, stale Hole-to-Data maps,
and pre-notification rejection in concurrent/native observer dispatch.

Existing successful-copy and progress tests preserve output and callback behavior.
The `progress` benchmark compares planning and valid observed/unobserved copies
with equivalent destination reset, flush, and read-back verification. Results are
recorded with R0.1 in the implementation log.


## R1.4/R1.5 follow-up

R1.4 moved live validation and native strategy/alignment preparation ahead of
initial observation. R1.5 distinguishes InvalidCopyConfiguration, StaleCopyPlan,
and live EndpointChanged preflight errors from malformed CorruptMetadata.
Configuration/capacity rejection still precedes payload I/O through the shared
checks. Execution failures carry context and confirmed partial counters under
the [copy error contract](../copy-errors.md). Full native runtime preparation,
cancellation, and terminal lifecycle tags remain separate work.
