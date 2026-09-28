# ADR-0027: Portable planning and explicit RAW execution adapters

## Status

Accepted for R1.1, extended through R1.5 on 2026-09-29. Full native runtime
preparation remains follow-up work. Supersedes the Linux RAW bounds of ADR-0019/0024; preserves the
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
  selected backend remains visible in the plan/report. R1.4 adds planning selection
  reasons; complete native runtime preparation is still pending.
- Logical Data/Zero/Hole semantics and counters are preserved. Existing DISCARD
  semantics are not strengthened here; the zero-read contract belongs to R2 and
  ADR-0026. R1.2 consolidates policy and sequential loops as recorded below.
- Initial progress follows endpoint and structural checks. Final progress follows
  successful flush. Concurrent/native execution still emits only initial/final
  progress. R1.5 adds partial-error counters; cancellation remains future work.
- R1.3 shares local descriptor inspections and R1.5 adds contextual errors. Total
  memory budgets remain open. Point-in-time preflight cannot stabilize mappings or contents.

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
and remaining limits. R1.3 shared endpoint descriptor inspections as recorded in
[ADR-0028](0028-endpoint-inspection.md).


## R1.4 — Logical intent and invocation preparation, 2026-09-28

CopyPlan now contains a private LogicalCopyPlan (extent map, accounting, and
fingerprint) separately from execution selection and block/alignment settings.
Logical intent has no executor or descriptor fields. Existing public plan getters
remain; `execution_selection()` adds a read-only planning decision containing the
requested strategy/options, selected backend, and a non-exhaustive reason enum.

| Planning request and API | Selection reason | Backend |
|---|---|---|
| Explicit Threaded, either API | RequestedThreaded | Threaded |
| Auto, portable API | PortableApi | Threaded |
| Explicit IoUring, accepted RAW descriptors | RequestedNative | IoUring |
| Auto, accepted RAW descriptors | RawDescriptorsCompatible | IoUring |
| Auto, rejected RAW descriptors | RawDescriptorsIncompatible | Threaded |

Explicit IoUring on a portable API or rejected RAW descriptor pair still errors.
The current descriptor evaluator accepts pairs and combines their reported
alignment; it does not test kernel availability or complete request alignment.
The incompatible reason represents the selection policy, covered with a unit
test, rather than a newly implemented runtime fallback. Runtime readiness and
unaligned native policy remain R2.

Both plan execution families build a private PreparedExecution for each
invocation. It borrows the validated plan/source/destination and contains checked
dispatch state. Preparation performs existing live endpoint and structural checks;
native preparation also resolves the executing mover's native options, rechecks
descriptor capability alignment, and constructs the NativeExtentPlan. Dispatch
uses that checked configuration. Initial observation follows this preparation.

A native plan executed by a Threaded mover, changed native buffer alignment, or
changed backend alignment now fails before any observer callback. These failures
previously happened after the initial callback. Successful callback cadence,
flush boundaries, and the native payload timer boundary are preserved.

Planning selection is historical metadata, not the executing mover's settings.
Existing behavior remains: a compatible Threaded plan stays Threaded when used
with another mover; a native plan requires a mover requesting native execution
and uses that mover's current native queue/read-window settings. Reports still
identify the backend that actually executed. CopyPlan equality includes its
planning selection; equivalent logical maps may have different plan provenance.

PreparedExecution is private, short-lived, and not saved in CopyPlan. It does not
open a ring, reserve buffers, stabilize endpoint state, or promise that later
allocation/kernel/request checks cannot fail. Low-level native checks remain
active. Runtime failures can still occur after initial observation; final
observation still requires successful flush. The direct portable `copy` path
retains its existing single-pass validation/execution flow without creating a
persistent plan.

Seven new tests cover selection/provenance and preparation boundaries. Three
behavioral regressions fail on the baseline solely because it emits one callback
before rejection, and pass on the candidate. See the
[R1.4 evidence](../benchmark-results/2026-09-28-r14/README.md).


## R1.5 — Execution failures, 2026-09-29

Accept boxed CopyFailure in the existing core Error/Result contract, rather than
introducing a parallel set of copy entry points. It preserves executor,
operation/range, original cause, and confirmed lower-bound counters. Aggregate
worker progress only after joining; retain the first error. Native cleanup may
leave additional uncounted I/O and retains its existing ownership guarantees.

Success signatures remain unchanged; callers matching raw payload errors must
inspect the underlying cause inside execution context. Validation errors now
separate stale/configuration/endpoint changes from malformed metadata.
The [error contract](../copy-errors.md) defines the accepted scope. Failure
counters do not imply durability, cancellation, resumability, or atomic rollback.

## R1.6 — Copy payload admission, 2026-09-29

Accept a configurable per-invocation budget, default 256 MiB, for accounted
buffers, queue/worker entries, and extent Vec capacity. Check both retained and
live revalidation maps, then release the latter before executor allocation.
Borrow extents for concurrent scheduling; include the native preparation clone
in execution accounting. Use the executing mover's settings and expose the
execution breakdown through `DataMover::execution_memory`.

Keep the Vec extent API for this phase. A backend allocates before returning
metadata, so admission cannot prevent an oversized query's transient allocation.
Opaque allocator/container overhead, stacks, backend/observer resources, kernel
memory, and earlier native quarantines remain external. This is not a process
RSS cap, a resource reservation, or a claim of fragmentation-independent memory.
Zero is a valid budget; overflow and excess are typed pre-execution errors.
The [accounting contract](../copy-memory.md) records phases and exclusions.
