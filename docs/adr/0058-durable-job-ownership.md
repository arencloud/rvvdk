# ADR-0058: Private durable ownership journal and conservative recovery

Status: accepted for R6.1b's Linux local foundation; production workflow integration remains R6.1c.

## Context

R6.1a artifacts describe untrusted source/content claims. They do not establish
who acquired a lease, created a staging directory, or may remove resources after
a process dies. A request may reach the server before its response or local
acknowledgment is recorded. Retrying an export or aborting a lease from a copied
manifest is therefore unsafe.

## Decision

Use a separate private `JobStore` bound to a caller-provisioned local directory.
Hold an exclusive nonblocking advisory lock on its inode. Generate an operation
ID with Linux `getrandom`, bind it to the artifact ID, expected source binding and
store identity, and retain terminal records so artifact IDs cannot be reused in
the store. Records contain no credentials, cookies, endpoint URLs or usable lease
references. A fingerprint records a live caller's lease observation only.

Each transition exclusively creates a bounded temporary record, writes and syncs
it, atomically renames it, then syncs the parent directory. Only after that succeeds
may the caller issue the corresponding external action. A persistence failure
poisons the writer; leftover transactions are preserved and block further mutation.
Initial reservation uses no-replace rename. A SHA-256 checksum detects accidental
record corruption, not an attacker authorized to rewrite the private store.

Preparation first persists intent, then creates a private stage with three fixed
members: `disk-1.vmdk`, `manifest.json`, and an operation marker. Persist their
device/inode identities only after syncing the files, stage and parent. Never
recover an ownership claim from a generated name when preparation was interrupted.

Reopening yields a nonmutating recovery report, never a live `Job`. Unknown lease
acquisition, transfer, completion/abort and publication outcomes remain unresolved.
There is no automatic network retry, lease abort, resume or publication operation.
Explicit local cleanup is available only before acquisition or after an acknowledged
remote ending, with fresh source/operation, private-mode, inode and marker checks.
Persist cleanup intent before unlinking only the recorded members; never recurse.
Repeated cleanup after a process loss tolerates missing owned members only in the
durable cleanup-intent state. Unknown entries prevent directory removal.

## Boundaries and consequences

The store trusts its caller-controlled directory and ancestors and cooperating
writers. It is not a defense against malicious same-UID writers, offline rollback,
hostile filesystems, inherited-store use after fork, or restoring a store onto a
different device/inode identity. Input is bounded to 8 KiB with fixed record shape.
All Debug/errors redact operational identifiers. Open handles and dropped jobs do
not trigger deletion; releasing the store explicitly releases its lock.

This package implements local journal persistence, owned staging and conservative
assessment/cleanup. It does not wire the legacy exporter into these operations or
implement automatic remote reconciliation. R6.1c must enforce fresh source checks,
own the actual lease capability, finish/verify/sync data, and bind real publication
to the journal's intent/acknowledgment boundaries. Ambiguous outcomes stay pending
until independently qualified evidence exists; a parsed artifact is insufficient.

[Detailed contract](../durable-job-ownership.md),
[fault tests and performance evidence](../benchmark-results/2026-10-02-r61b/README.md).
