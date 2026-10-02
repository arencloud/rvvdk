# Journal-owned container transfer (R6.1c.2b)

Linux `transfer_owned_export(source, credentials, store, artifact, options)` now
transfers an export into the journal-owned payload and completes its live lease
under durable intent. The caller supplies explicit source identity and a locked
private store. The stage remains private and retained; this API does not publish,
convert, create R6 artifact metadata or restore a remote capability on reopening.

`OwnedTransferOptions` bounds encoded bytes (512 bytes through 1 TiB), total transfer
and verification time (positive, at most six hours), and cooperative cancellation.
The default limit is 40 GiB and one hour. Scope remains an already powered-off VM
with one persistent, unencrypted FlatVer2 disk, no parent or snapshots, and ExportVm
enabled. The endpoint and certificate pin come exclusively from the source identity.

## Transfer and writer lifetime

The R6.1c.2a worker still owns the store, Job and all file handles. The payload writer
opens only through the freshly checked owned stage, starts at length zero and is
never handed to another task. Network chunks are bounded to 1 MiB; the async receiver
assembles 1 MiB write batches and awaits each command. The one-slot worker channel
and serial producer prevent an unbounded payload queue. The network chunk, assembly
buffer and accepted write batch can coexist, each at most 1 MiB, excluding HTTP/TLS
internals and bounded inventory/metadata. Readback uses a separate 1 MiB buffer.

A cancelled download can abandon its reply waiter, but cannot abandon its accepted
write. The worker finishes that write and remains available for the subsequent
Abort command. Abort is ordered after accepted writes, closes the only writer and
persists AbortIntent before the RPC. Payload I/O failure closes the writer and
still permits this journaled abort when the store can persist it. A failed journal
transition stops the worker and cannot authorize the next remote mutation.

There is no buffered file destructor that silently flushes after an acknowledgment.
Writes track completed syscall bytes, including short writes; Interrupted retries
only the local write. The public future drains the worker before returning. An
uninterruptible filesystem call can outlast network deadlines. Dropping the public
future or killing its process still cannot promise remote cleanup.

## Byte verification and completion

The live operation checks explicit source identity in inventory, before staging,
after acquisition intent, before GET, after TransferComplete and after CompleteIntent.
Ready-lease source reads and manifest/completion RPCs have a request budget capped
at one third of the advertised lease timeout, ten seconds and the overall deadline.
Source reads refresh progress first. Observations do not exclude concurrent VM
changes between checks.

The completion sequence is:

1. Finish the bounded HTTP body; require a recognized sparse-container prefix and
   at least 512 bytes. Compute network SHA-256 and SHA-1, and verify the lease manifest
   against the observed encoded size, disk key and expected logical capacity.
2. Drain writes and sync the owned payload. Close its only writer. Open a fresh,
   identity-checked descriptor, verify length, and independently read every byte to
   compare both digests. Recheck stage/member identities and the ownership marker.
3. Only after successful readback, record TransferComplete. Preserve cancellation
   and source checks before crossing into completion.
4. Set the conservative completion guard **before submitting CompleteIntent**.
   Await its durable acknowledgment, then recheck cancellation and source identity.
5. Issue HttpNfcLeaseComplete once, record the remote acknowledgment separately,
   then persist CompletedLease. Logout and await worker shutdown before reporting.

Ready-lease heartbeats run during writes, sync/readback and journal waits. Cancellation
while a seal is accepted waits for its completion before deciding whether to abort.
Once CompleteIntent has been submitted, any persistence failure, cancellation, source
change, lost response or completion failure leaves the outcome for reconciliation.
There is **no automatic abort after this boundary**, even when the live caller knows
that a later source check prevented sending the completion RPC. This conservatism
avoids inventing an unsupported rollback transition or retrying uncertain requests.

A successful completion RPC with a failed journal acknowledgment is reported as a
remote completion with local uncertainty. It is not a successful workflow result.
Unknown intent states or surviving transactions continue to block local cleanup.

## What the report establishes

`OwnedTransferReport` separates accepted HTTP bytes, confirmed file-write bytes,
bytes covered by successful sync, manifest verification and independent container
readback. Written/durable counts may be lower bounds if the worker panics before
reporting. Payload errors and lifecycle/worker errors are retained with the fresh
recovery assessment. Reports omit source paths, endpoint details, lease references
and content hashes. Local worker command failures can also populate the lifecycle's
`journal_error` field; `payload_error` identifies payload-specific failures.

`is_success()` requires agreement of received/written/durable byte counts, manifest
and readback checks, acknowledged and durably recorded CompletedLease, confirmed
logout and no recorded errors. The stage still requires explicit checked cleanup or
future artifact admission. The fixed metadata file remains empty in this package.

**Container-byte verification is not VMDK structure validation or logical-disk
verification.** A matching header prefix/digest does not validate stream version,
grain maps, compressed payloads or the logical capacity encoded inside the image.
No LogicalReadbackVerified artifact claim is produced. The separate
[R6.1c.3 artifact API](owned-artifact-admission.md) now applies bounded native
structure/grain admission and persists private metadata before completion.
Retained artifact admission, conversion and publication remain later gates.

The existing private-store, trusted-ancestor and cooperating-writer constraints
remain in force. Reopening an assessment does not recreate Job or a live lease;
any future read-only artifact capability must be separately and freshly admitted.

[ADR-0061](adr/0061-owned-transfer-and-conservative-completion.md),
[failure tests and benchmark plots](benchmark-results/2026-10-02-r61c2b/README.md).
