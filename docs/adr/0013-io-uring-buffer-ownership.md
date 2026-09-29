# ADR-0013: io_uring Buffer and Descriptor Ownership

## Status

Accepted; revised for R0.3 on 2026-09-28. Supersedes the initial borrowed-slice
compatibility API and the assumption that ring destruction alone proves safety.

## Problem

The old `read_at`/`write_at` methods queued pointers into caller slices and could
return early on submit/wait errors. Later engine destruction could not extend a
borrow that had already ended. Owned operations retained buffers, but only raw FD
numbers, and their drop path released buffers after an unsuccessful drain.

A separate reproduction confirmed the descriptor problem: closing a queued source
and opening `/dev/zero` at the same descriptor number caused the original engine
to read the replacement. The borrowed-buffer problem was established by control
flow review; no use-after-free experiment was executed.

## Decision

The engine accepts only owned `BufferGuard` operations. Remove its borrowed
`read_at`/`write_at` methods instead of adding bounce-buffer copies or relying on
caller lifetime discipline. The native pipeline still transfers the same guard
from a completed read to a write, without a userspace data copy.

`IoUringFile` owns an `Arc<OwnedFd>`. Construct it by duplicating a `BorrowedFd`, or
transfer an existing `OwnedFd`. Each operation retains a cheap clone. The range
copy pipeline duplicates its two endpoints once per range, not per block. Public
range/extent functions accept `BorrowedFd`; `LinuxFdBackend` now requires `AsFd`.
A numeric descriptor alone is no longer accepted at native I/O entry points.

The engine installs both resource owners in its in-flight table **before**
publishing the SQE. Only a matching final CQE permits removal. Queue capacity is
also an explicit upper bound on tracked operations, including submitted ones.
Interruptions retry without releasing anything. Other enter/completion errors
stop new submissions; known remaining completions can still be drained.

`shutdown()` drains queued and submitted operations and then closes the ring.
Negative operation CQEs release only that confirmed operation and do not prevent
cleanup of others. A failed wait with no progress, or loss of completion identity
or finality, cannot justify releasing the remaining owners. In that case shutdown
permanently retains them, closes the ring, and returns
`IoUringShutdownUnconfirmed { operations }`. The count remains available through
`quarantined_operations()`, and repeated shutdown/drain calls cannot claim success.

Drop uses the same protocol but cannot return an error. Range-copy functions call
shutdown explicitly; if both copying and cleanup fail, `IoUringCleanup` preserves
the original cause and reports the number of unconfirmed operations.

## Safety argument

| Transition | Ownership and release rule |
|---|---|
| Validation rejects an operation | Nothing published; the passed guard can return to its pool |
| Build SQE and install table entry | Heap buffer address stays stable; shared owned FD cannot close |
| Publish SQE | Table already owns every referenced resource |
| SQ push fails | No entry published; remove only that operation |
| Submit succeeds partially, is interrupted, or fails | Submission counts do not release owners; table retains queued and accepted requests |
| Matching final read/write CQE | Kernel operation has completed; transfer buffer to `CompletedOperation` or drop it on a negative result |
| Completed read becomes write | Move the same guard to a new owned operation and retain the destination endpoint |
| Unknown/duplicate/nonfinal CQE | Do not guess its owner; stop accepting work and retain unconfirmed operations |
| Missing CQE or fatal cleanup wait | Permanently retain outstanding owners; closing the ring is not a substitute for observing completion |
| Panic after publication | Engine Drop attempts the same drain; no borrowed caller storage exists |
| Engine is deliberately forgotten | Owned resources are leaked with it, never returned to a caller/pool prematurely |

Assumptions: the kernel and pinned `io-uring` dependency satisfy their API
contracts; ordinary safe Rust callers do not forge `BorrowedFd` values or close
someone else's descriptor through unsafe code. The ring is private, uses default
setup, and submits only single-shot Read/Write with ordinary CQEs, no multishot,
CQE skipping, registered buffers, zero-copy networking, or user-defined SQEs.

The buffer-validity requirement and completion identity model come from the
[upstream io_uring manual](https://man7.org/linux/man-pages/man7/io_uring.7.html).
Ring teardown unmaps queue memory and closes the FD; some work is asynchronous.
The upstream cancellation guidance waits for request CQEs before freeing related
resources. This implementation uses that conservative completion boundary rather
than assuming close synchronously releases buffer references.
[Queue teardown](https://man7.org/linux/man-pages/man3/io_uring_queue_exit.3.html),
[cancellation and shutdown](https://man7.org/linux/man-pages/man7/io_uring_cancelation.7.html).

## Costs and limitations

Successful native copying adds two descriptor duplications per copied range,
shared-handle references per operation, and completion validation. No per-block
payload copies or descriptor duplications are introduced. Existing per-extent
ring/pool creation remains for a future performance step.

Unconfirmed cleanup deliberately leaks owned guards, their underlying pools,
and FD references until process exit. Its operation count is bounded by this
engine's queue depth, but retained guards may keep a larger caller-owned pool
alive. This is an exceptional safety fallback, not a recoverable cleanup success.
It can accumulate if callers repeatedly create engines after terminal failures.
A future proven cancellation/reaper protocol may reclaim such resources; this
step does not claim that guarantee.

Drain may submit previously queued writes and waits for active I/O. It does not
provide rollback, a deadline, or cancellation of stalled devices. Synchronous
cancellation is not used as a universal shortcut: it is kernel-version dependent
and does not cover SQEs not yet submitted. See the pinned dependency's
[submitter contract](https://docs.rs/io-uring/0.7.15/io_uring/struct.Submitter.html#method.register_sync_cancel).

Backend access/alignment preflight, same-device checks, logical offset translation,
and persistent snapshot identity remain separate work. An owned FD protects the
open file description's lifetime, not the consistency of its contents.

## API migration

- Implement `AsFd` when implementing `LinuxFdBackend`; `raw_fd()` remains a
  diagnostic compatibility accessor, not an I/O submission argument.
- Pass `source.as_fd()` and `destination.as_fd()` to native range/extent functions.
- Replace borrowed engine operations with `submit_owned_read`/`submit_owned_write`,
  an `IoUringFile` endpoint, and a `BufferPool` guard. Consume the resulting
  `CompletedOperation` before accessing/reusing its buffer.
- Call `shutdown()` when cleanup diagnostics matter. Public `DataMover` copy
  method signatures are unchanged; their platform trait bound now requires AsFd.

All repository call sites and tests are migrated. The 18 new lifetime cases use
real rings with test-only fault injection around submission/wait/CQE consumption;
potential hangs run in subprocesses with ten-second deadlines. They cover
interruptions, partial submission, fatal errors, malformed CQEs, unwinding,
descriptor reuse, capacity rejection, shutdown, and deliberate retention.
See the [R0.3 evidence](../benchmark-results/2026-09-28-r03/README.md) for correctness,
resource measurements, and performance disposition.

## R2.6 — Owned file admission, 2026-09-29

InFlightOperation additionally owns regular-file range admission before SQE
publication. Final confirmed completion releases the range even while the caller
retains CompletedOperation's buffer. All existing unconfirmed shutdown paths
quarantine the admission with file/buffer ownership. This avoids reusing a range
whose kernel access cannot be ruled out. See [ADR-0029](0029-local-file-admission.md).
