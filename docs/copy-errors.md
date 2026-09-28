# Copy failures and partial progress

R1.5 preserves execution context in the existing core Result API. Execution
errors use Error::CopyExecution(Box<CopyFailure>); successful CopyStats and
CopyReport APIs are unchanged. Core owns the portable error data and has no
dependency on a data-mover executor or Linux platform crate.

## Inspecting a failure

```rust
fn explain(error: &rvvdk_core::Error) {
    if let Some(failure) = error.copy_failure() {
        eprintln!(
            "{} {:?}, attempted range {:?}, confirmed bytes written {}",
            failure.backend,
            failure.operation,
            failure.range,
            failure.progress.bytes_written,
        );
        eprintln!("underlying cause: {}", failure.cause);
    } else {
        eprintln!("planning or preflight rejection: {error}");
    }
}
```

copy_failure() also finds the execution context inside IoUringCleanup or
EndpointPreflight. The outer error still matters: IoUringCleanup retains its
unconfirmed operation count. The standard Error::source chain preserves the
original core error and OS error/errno; diagnostics need not parse strings.
Current executor labels are "threaded" and "io_uring".

This is a pre-release error-shape change. A caller that previously matched a
payload error directly as Error::Io or Error::Unsupported should inspect
failure.cause or traverse the source chain. Planning/preflight errors generally
keep their original outer variants.

R1.6 adds `MemoryBudgetExceeded { phase, required, budget }` and
`MemoryAccountingOverflow` as preparation rejections. They precede payload I/O,
flush, and observation and carry no partial execution counters. The
[memory contract](copy-memory.md) defines the accounted storage and exclusions.

## Operations and ranges

CopyOperation distinguishes Allocate, Read, Write, WriteZero, Discard, Flush,
Schedule, NativeSetup, NativeCompletion, and NativeShutdown.

A range describes the attempted operation, not the exact failing byte. Sequential
and worker failures identify the current block or accelerated sparse extent.
A failed write_all_at may have changed part of that block before returning an
error. Its original cause may contain a more precise offset.

Native submission and positive short-completion errors identify their request.
If the existing engine's completion API returns an error without a request
identity, the copy layer reports NativeCompletion and the attempted Data range.
It does not invent the unknown operation or failing request offset. Native
cleanup errors retain the original failure and its context. Whole-job allocation,
scheduling, and final flush use no range; native setup can name its requested
range.

## Partial counters

Every counter is a confirmed lower bound, not a durable result, rollback record,
or resume cursor. Counts can include operations that finish after another worker
reports an error. Concurrent/native completion order need not match disk offsets.

| Field | Meaning on failure |
|---|---|
| bytes_read | Fully completed backend read_exact_at calls, or observed positive native read CQE bytes |
| bytes_written | Fully completed backend write_all_at calls, or observed positive native write CQE bytes |
| bytes_zeroed | Successful accelerated zero operations |
| bytes_discarded | Logical bytes successfully processed by zero-guaranteed discard; not reclaimed physical space |
| blocks_completed | Successful complete Data transfers or fallback zero-write blocks |
| extents_completed | Completed sequential/native extents; None for worker failures whose work items do not identify completed extents |
| unconfirmed_io | Additional I/O may have occurred outside these counters |

The low-level native range API reports zero completed extents; it has no logical
extent map. Extent executors add preceding completed extents to a range failure.
Worker flush failure has a known completed extent count because all queued work
has finished. A positive short native write contributes confirmed written bytes
but does not count as a completed block.

For example, a backend copies one 4 KiB block, then writes 256 bytes of the next
block before failing. A threaded error reports 4 KiB written and sets
unconfirmed_io; the destination may contain 4 KiB + 256 bytes of new data. If the
second source read succeeded, bytes_read includes both 4 KiB reads.

A flush failure retains all confirmed payload counters. unconfirmed_io may be
false because byte accounting is complete; that says nothing about durability.
The copy still failed. Existing low-level native APIs remain caller-flushed.

## Collection and shutdown

Sequential execution captures counters at the failing operation. Worker execution
retains every worker's counters, including a worker that fails, then aggregates
after all scoped workers join. The first recorded error remains authoritative.
A later failed worker can still mark accounting uncertain when the first error
came from the producer. No per-block statistics atomics were introduced.

Native copying retains counters before exiting the pipeline, then performs the
existing shutdown. Completions consumed during shutdown are not added to the
copy counters; unconfirmed_io remains true. A shutdown that cannot establish
completion still retains operation owners for safety and reports its count
through IoUringCleanup. R1.5 does not change engine ownership/shutdown rules.

Backend operation errors are not retried through a different sparse capability.
Successful callbacks and final flush boundaries are unchanged. A failure returns
an error instead of the final success snapshot; earlier progress can already have
reported 100% of bytes before flush. There is still no terminal lifecycle tag.
Panics and process aborts are not converted into CopyFailure results.

## Validation categories

- InvalidCopyConfiguration: plan versus mover block size or alignment mismatch.
  Existing typed numeric/alignment/capacity errors retain their variants.
- StaleCopyPlan: source geometry or extent fingerprint changed since planning.
- EndpointChanged inside EndpointPreflight: current local source size differs
  from the geometry used for preflight, including during initial planning.
- CorruptMetadata: malformed extent topology/accounting or invalid internal
  metadata; it is no longer used for ordinary stale/configuration mismatches.

Ordinary planning/preflight rejection carries no execution-progress report.
Existing shared preparation checks still precede observers and mutation; complete
native runtime/request preparation remains R2. This contract does not add a
consistent source snapshot, cancellation, a contiguous checkpoint, or resume.

See [architecture](architecture.md#copy-failure-context-and-progress-r15),
[implementation log](implementation-log.md), and
[validation and benchmarks](benchmark-results/2026-09-29-r15/README.md).
