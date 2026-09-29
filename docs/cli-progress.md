# Copy lifecycle and cancellation (R3.3)

`copy` and `verify` now accept `--progress`. Success reports on stdout retain the
[R3.2 schema](cli-transfer.md); progress goes to stderr. Without the flag there
is no progress stream. Inspection and plan previews retain their earlier behavior.

```bash
rvddk copy source.raw new.raw --format raw --verify --progress
rvddk copy source.raw existing.raw --format raw --overwrite --backend auto --progress --json
rvddk verify source.raw new.raw --format raw --progress --json
```

## Progress stream

Human mode writes one concise line per event. With `--json --progress`, stderr
contains newline-delimited JSON progress objects, followed by the existing error
object if the command fails. Read stderr as a stream of objects, not one JSON
document. Stdout still contains only the ordinary success result. Each progress
object has `schema_version: 1`, `event: "progress"`, a monotonic `sequence`, `phase`,
`logical_bytes`, `logical_bytes_processed`, `bytes_verified`, optional-valued
`backend` and `destination_state`, `elapsed_seconds`, and `confirmed` I/O counters.
Unknown additive fields should be ignored. Backend is null outside engine events.

| Phase | Meaning |
|---|---|
| planning / preparing | CLI copy / standalone verify is opening and validating endpoints; total can still be unknown (zero) |
| copy_preparing | Engine revalidation, budgets and runtime preparation |
| copy_started | Actual backend is selected/prepared; payload processing has not started |
| copying | Confirmed logical processing; may be out of order and is not yet durable |
| copy_flushing | All logical work processed; engine flush has not finished |
| copy_flushed | Engine flush succeeded; CLI verification/publication are still outstanding |
| verification / verifying | Copy's optional verification boundary / active logical comparison |
| file_sync | Final CLI file sync boundary |
| publication | Last cancellable checkpoint before attempting no-replace linking |
| directory_sync | Linking succeeded; finish directory sync and identity check before honoring cancellation |
| completed | CLI operation and configured durability/verification steps succeeded |
| cancelled / failed | Terminal operational outcome; inspect the subsequent error object for details |

Logical processed bytes are confirmed writes + zeroed + discarded bytes. They
are not physical storage traffic, a contiguous prefix, or a resume checkpoint.
Verification separately counts a matching logical prefix. **100% processed is
not completion**: flush, verification or publication can still fail or be cancelled.
Empty images also follow lifecycle/durability rules. A failed success-report write
can follow a completed operation; its error retains `operation_completed: true`
and the completed result, as in R3.2.

Payload progress emits after the first observed advancement, then at observed
64 MiB or 100 ms thresholds. Verification also emits at its final block. These
are checkpoint thresholds, not timer-driven heartbeats or latency guarantees.
Concurrent progress is sampled by the scheduling coordinator; while it is blocked
on work admission, a backend call or worker join, no callback is guaranteed.
Callbacks and stderr writes are synchronous: slow output can slow a job. If
progress output fails, stop cooperatively and return an output error; stderr
itself may then be unavailable for reporting that error.

## Cancellation and signals

The standalone binary installs SIGINT/SIGTERM handlers for copy/verify. The
handler only records the first signal in a lock-free atomic. Cleanup, formatting,
file operations and native shutdown run through ordinary Rust control flow.
Repeated signals do not force resource destruction. Existing dispositions are
restored when the invocation exits. Read-only inspect/plan keep default signal
behavior. See Linux [sigaction](https://man7.org/linux/man-pages/man2/sigaction.2.html)
and [signal safety](https://man7.org/linux/man-pages/man7/signal-safety.7.html).

| Exit | Meaning |
|---|---|
| 0 | Operation and success report completed, or help/version |
| 1 | Operational, cleanup, verification or output failure |
| 2 | Invalid usage |
| 130 | Cooperative cancellation, including SIGINT |
| 143 | Binary cancellation initiated by SIGTERM |

Cancellation requests are sticky. Checkpoints occur before preparation/mutation,
between sequential blocks and read/write transitions, between concurrent work
items/read-write transitions, between native completions and refills, between
verification reads/blocks, and at CLI durability/publication boundaries. Already
running calls may finish; queued native I/O may have effects during cleanup.
A large zero/discard operation or blocking I/O/flush/output can delay observation.
There is no hard cancellation deadline or promise to interrupt a syscall.

Concurrent workers stop taking work, the producer unblocks when receivers exit,
and all workers are joined before returning. Native cancellation uses existing
owned-request shutdown: buffers, descriptors and admission guards remain owned
until completion or quarantine. If cleanup is unconfirmed, failure takes precedence
over a cancellation exit and counters remain conservative lower bounds.
The first concurrent error is retained, including cancellation; additional failed
worker effects can set `unconfirmed_io` without replacing that original cause.

Before linking, a cancelled new copy leaves no published name from this invocation.
After linking, the CLI completes directory sync/name inspection before checking
the request, then can return cancellation with `destination_state: published`.
It never unlinks that output. Explicit overwrite can leave a modified prefix and
preserves the existing tail. SIGKILL/process crashes are not cooperative cleanup
and provide no new crash-resume guarantee. A signal arriving after the final
cancellation check can race successful completion; completed operations are not
rolled back. Stable contents/namespaces remain caller obligations.

## Rust APIs

`CancellationToken` is cloneable and shares an atomic flag; `cancel()` requests
stopping. `Cancellation` accepts other cheap, non-panicking, thread-safe predicates;
`NoCancellation` never requests stopping. `DataMover::execute_plan_controlled` and
Linux `execute_raw_plan_controlled` take a token/predicate and `CopyObserver`.
Closures receiving `&CopyEvent` implement the observer. Callbacks run only on the
invoking coordinator and need not be Sync; they must not panic. Workers see only
the cancellation predicate and publish cumulative deltas to fixed aggregate state.

Library phases are Preparing, Started, Transferring, Flushing and exactly one
Completed/Cancelled/Failed on ordinary Result returns. Preparation rejection also
produces a terminal event. Started identifies the actual backend after Auto
fallback. Events include totals, elapsed time and confirmed `CopyProgress`.
Concurrent extent completion is unknown until all workers finish. The legacy
snapshot observer APIs retain their earlier cadence; use the controlled APIs for
full lifecycle and all-backend intermediate progress. Unobserved entry points
retain their no-observer behavior and compile out generic checkpoint work.

Library Completed refers to its engine flush only. CLI maps it to copy_flushed;
publication and optional verification are separate layers. `Error::Cancelled`
can be nested in contextual CopyExecution errors; `Error::is_cancelled()` treats
unconfirmed native cleanup as failure. `Verifier::verify_controlled` reports the
confirmed matching prefix at block boundaries. The original verify method uses
NoCancellation and no callback.

`rvvdk_cli::run_with_cancellation` is embeddable and installs **no signal handlers**;
its programmatic cancellation exit is 130. `run` supplies NoCancellation. Progress
output failure uses an internal cancellation flag without mutating the caller's
token. New fixed control state, mutexes and small reports are metadata outside the
existing [payload memory budget](copy-memory.md); no per-image event list is kept
by production code. Consumer-owned captures are outside that budget.

See [ADR-0032](adr/0032-copy-lifecycle-cancellation.md),
[tests and performance evidence](benchmark-results/2026-09-29-r33/README.md), and
[the implementation log](implementation-log.md).
