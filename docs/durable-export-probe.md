# Durable export lease probe (R6.1c.2a)

Linux `probe_owned_export(source, credentials, store, artifact, cancellation)`
connects an actual acquire-and-abort lease to the private ownership journal. It
consumes a locked `JobStore`, uses the explicit source's pinned endpoint, and
requires the same powered-off, single persistent unencrypted disk scope as
`export_selected_vm`. It never shuts down a VM, downloads payload, completes an
export, converts an image or publishes output.

This bounded package qualifies durable remote intent and local resource ownership
before adding transfer/completion. The existing capacity and explicit export
proofs keep their separate artifact lifecycle.

## Ordering and ownership

One blocking worker owns the store, fresh `Job`, private stage and all its handles.
A bounded one-slot channel carries serial commands with a reply after each durable
transition. No file writer or usable remote lease escapes into the worker's record.
The lease reference is used only in the live async operation; the temporary worker
command is zeroized after computing its journal fingerprint.

```mermaid
sequenceDiagram
    participant C as Async probe
    participant J as Journal worker
    participant E as Pinned endpoint
    C->>J: Create fresh Prepared record
    C->>E: Inventory and fresh explicit identity check
    C->>J: Prepare owned stage, persist AcquireIntent
    J-->>C: Durable acknowledgment
    C->>E: Recheck identity, then ExportVm
    E-->>C: Live lease reference
    C->>E: Bounded readiness and lease identity checks
    C->>J: Record LeaseHeld, then AbortIntent
    Note over C,J: Refresh ready lease while journal work runs
    J-->>C: Durable acknowledgment
    C->>E: Abort once
    E-->>C: Abort acknowledged
    C->>J: Persist AbortedLease
    C->>E: Logout
    C->>J: Close command channel and await worker
    J-->>C: Recovery assessment; all handles released
```

Initial local record creation precedes connection. Stage preparation follows the
first fresh source observation; a second fresh observation follows the durable
acquisition intent, so slow sync cannot authorize ExportVm using a stale check.
A missing, ambiguous or changed source never falls back to capacity. Fresh checks
remain observations and cannot exclude concurrent remote changes.

Ready-lease metadata uses the existing entity/capacity, timeout and data endpoint
admission rules, even though the probe does not issue a GET. If readiness fails but
a valid live lease reference was parsed, the probe still records ownership and an
abort intent before attempting abort. No readiness timeout is invented and no
heartbeat is attempted without an admitted ready-lease timeout.

## Waiting, cancellation and failure

For a ready lease, the async task runs the existing heartbeat loop while journal
commands perform blocking filesystem work. Cancellation requests cleanup; it does
not abandon a submitted write. If progress/deadline handling fails, the accepted
journal command is still awaited. A successfully persisted abort intent permits
one abort attempt using the separate cleanup deadline, even after progress failed.
No automatic acquisition or abort retry occurs.

A journal error closes the worker's command loop and prevents subsequent remote
mutation based on that command. Acquisition rejection, malformed/lost responses,
abort failure and journal acknowledgment failure retain conservative intent or
uncertain states. Even a reported acquisition rejection is not downgraded to
"nothing happened" by an unqualified new journal transition. If cancellation or
identity change happens after AcquireIntent but before RPC, that same conservative
state remains. Reopening never recreates a live capability or authorizes abort.

All accepted worker work is drained before the public future returns. No stage
file handles are handed to callers. Blocking filesystem calls cannot be forcibly
cancelled; their drain may outlast every network deadline. Dropping the public
future does not guarantee remote cleanup. Its worker still owns the store and
finishes the accepted command, then closes handles when its channel closes. Process
loss requires conservative assessment; remote cleanup is not promised.

## Result and local cleanup

`OwnedProbeReport` separates primary, journal and session/pagination cleanup errors
from the observed remote disposition and fresh recovery assessment. Success means
an acknowledged abort, durable AbortedLease, confirmed logout and no recorded
errors. It does not mean export or artifact success. Recovery report serialization
omits the operation identifier; Debug redacts it. No URL, lease reference, source
binding or private path is included in these reports.

The empty owned stage is retained. After the probe releases the store, callers may
reopen it and call `cleanup_local(artifact, recovery.operation, source)` only in an
eligible state, with the existing inode/marker and transaction checks. Cleanup is
explicit and preserves a terminal artifact-ID tombstone. Uncertain states and
pending transactions remain blocked. Initial failure can leave Prepared or
StageIntent; these are assessed rather than silently removed or reused.

The API expects a Tokio runtime with I/O and time enabled. Each active probe uses
one blocking worker for its lifetime and serializes its store through the existing
lock. This is a per-probe bound, not a scheduler or global concurrency limit.
The private-directory, trusted-ancestor, cooperating-writer and local-filesystem
requirements from [job ownership](durable-job-ownership.md) still apply.

## Continuation

R6.1c.2b must transfer through these owned resource handles and qualify writer drain,
TransferComplete/CompleteIntent ordering, lost completion responses and failure
cleanup. Completion uncertainty must not trigger the legacy proof's automatic
abort behavior. Verified container metadata, confined conversion and actual
journal-bound no-replace publication remain subsequent work. Nothing here upgrades
old manifests or qualifies full production recovery.

[ADR-0060](adr/0060-durable-export-lease-probe.md),
[failure tests and performance plots](benchmark-results/2026-10-02-r61c2a/README.md).
