# ADR-0027: Portable planning and explicit RAW execution adapters

## Status

Accepted for R1.1, 2026-09-28. Executor preparation and contextual errors remain
follow-up work. Supersedes the Linux RAW bounds of ADR-0019/0024; preserves the
validation contract of ADR-0025.

## Context

`copy` and source-only planning accepted logical disks, but destination-aware
planning, reports, and observed execution required Linux RAW backends. Memory
and translated format readers could not use the complete lifecycle. A physical
file descriptor cannot represent arbitrary logical offsets or parent resolution.

## Decision

The primary `plan`, `plan_with_destination`, `execute_plan`,
`execute_plan_with_observer`, `copy_with_report`, and `copy` methods accept
`VirtualDisk + ?Sized`. The observer also accepts `?Sized`. All payload operations
use logical disk methods. Threaded and Auto select threaded execution through
these entry points. An explicit IoUring request, or a native plan submitted to a
portable executor, returns `NativeExecutionUnsupported` before payload I/O or
observer notification. A type implementing native traits receives no automatic
FD specialization.

Linux RAW callers opt in through these adapters:

| Previous method | RAW replacement |
|---|---|
| `plan_with_destination` | `plan_raw_with_destination` |
| `execute_plan` | `execute_raw_plan` |
| `execute_plan_with_observer` | `execute_raw_plan_with_observer` |
| `copy_with_report` | `copy_raw_with_report` |

These retain `RawDisk<S>` with `S: BlockDevice + LinuxFdBackend` and the existing
Threaded/Auto/IoUring selection and FD-binding checks. Low-level `copy_native`
methods retain their existing signatures and caller-managed flush contract.
This is a deliberate pre-release API migration: existing Threaded callers may
keep the portable names; callers relying on Auto native selection must migrate.
`copy`/source-only `plan` formerly ignored an explicit native strategy; they now
reject it to make selection consistent across portable entry points.

Both families share structural validation and initial/final observation. Their
preflight policies remain distinct: portable checks use logical endpoint metadata;
RAW checks additionally inspect and bind descriptors. The portable path must not
query a container's FD. Endpoint implementations remain responsible for reporting
logical size, access, and appropriate identity.

`CopyPlan::new`, `NativeExtentPlan::new`, direct copying, and live plan validation
use one extent topology validator. A valid map covers the complete logical disk
without gaps, overlaps, or overflow. Validation is retained at trust boundaries.
Plans remain in-memory structural records, not serialized snapshot identities.

## Consequences and limits

- Memory, local RAW, and translated disks use the same portable lifecycle,
  including trait objects, single/multiple workers, and observers.
- Auto on a portable call intentionally does not choose native execution. The
  selected backend remains visible in the plan/report; structured fallback reasons
  and complete native runtime preparation are still pending.
- Logical Data/Zero/Hole semantics and counters are preserved. Existing DISCARD
  semantics are not strengthened here; the zero-read contract belongs to R2 and
  ADR-0026. R1.2 consolidates policy and sequential loops as recorded below.
- Initial progress follows endpoint and structural checks. Final progress follows
  successful flush. Concurrent/native execution still emits only initial/final
  progress; partial-error reporting and cancellation remain future work.
- Raw descriptor snapshot deduplication, total memory budgets, and contextual
  errors remain open. Point-in-time preflight cannot stabilize mappings or contents.

## Validation

Nine integration tests cover memory/local/translated disks, translated guard bytes,
all sparse capability combinations, dyn observers, one/four workers, stale maps,
configuration/capacity/alias rejection, flush failure, and explicit native rejection.
The translated fixture implements native traits that panic if consulted. A
constructor regression rejects invalid topology even when byte totals match.

The same external trait-object consumer fails to compile on R0.5 and executes on
R1.1. Core/datamover library compilation also passes for `wasm32-unknown-unknown`;
this is a compilation check, not a claim of WebAssembly runtime/thread support.
Performance evidence is in the [R1.1 report](../benchmark-results/2026-09-28-r11/README.md).

## R1.2 implementation follow-up — 2026-09-28

The private `policy::select` operation now serves sequential, worker, and
native destination-aware execution. It preserves capability precedence and
propagates operation errors without retry. Separate observed/unobserved sequential
loops were replaced by `sequential::execute` with statically dispatched progress
hooks. Native payload execution and worker scheduling remain separate mechanisms.

The existing progress cadence, whole-extent versus work-item operation granularity,
statistics, and flush ownership remain unchanged. Contract tests run on baseline
and candidate preserve these behaviors, including the pre-flush 100% byte-threshold
snapshot limitation. The [architecture contract](../architecture.md#shared-semantic-execution-r12)
and [benchmark record](../benchmark-results/2026-09-28-r12/README.md) describe scope
and remaining limits. R1.3 will address endpoint preparation/descriptor snapshots.
