# ADR-0070: Promote only a matched Published acknowledgment transaction

Status: accepted for implementation in **R6.1d.3a**, 2026-10-14.
Reviewed against `6c0134e`. **Documentation only; runtime implementation and
qualification remain open in R6.1d.3b.**

## Context

Both normal publication and PublishIntent reconciliation write a Published
transaction after syncing the final bundle namespace. Process loss before journal
replacement leaves the old intent and a partial or complete transaction. Existing
reconciliation, fresh attestation and cleanup reject every pending transaction.
Read-only assessment reports existence, not the transaction's validity or authority.

A complete candidate can be present before its file sync, so visibility or checksum
cannot establish prior durability. A separate operation must freshly validate the
source, output and exact old/new pair before changing the journal name.

## Decision

Add the consuming `JobStore::recover_publication_acknowledgment` API and a separate
`PublicationAcknowledgmentRecoveryReport`. Accept only original v2 PublishIntent
sequence 7 plus a complete v2 Published sequence 8 transaction equal to the old
record with exactly state and sequence changed. Both envelopes must independently
validate; pin their exact bytes and distinct inodes. Preserve all binding, contract,
stamp, digest and destination fields and require absent cleanup markers.

Require original store/destination locks, explicit source/artifact/output, retained
CompletedLease source, no source transaction, absent stage and owned final bundle.
Freshly validate encoded/native source and all logical RAW bytes including zeros,
metadata, exact length, digest and post-read bindings. A matched pair with staged,
missing or ambiguous output is still ineligible.

Sync the three bundle members and directory, exact old intent and store, store and
destination namespaces, then exact candidate. Recheck pair/source/namespace and
control immediately before replacing the old journal with that candidate. Sync the
store after rename before observing cancellation. This successful path has ten sync
calls, one journal rename, no bundle rename, no application byte writes, no new file
and no commit_output call. Candidate bytes/inode become the journal bytes/inode;
the old pathname is intentionally replaced. There is no backup or rollback.

Use one deadline and retain both locks through source/output binding checks before
and after independent assessment. Keep primary and assessment errors separate.
Report content, bundle, intent, parents, candidate sync, observed promotion, terminal
store sync and final bindings independently. Require every flag and correct final
state for success; never infer an earlier invocation's success. A failed rename or
store sync remains failure even when Published becomes visible.

Keep the current cooperating-writer/private-store trust model. Immediate name checks
plus advisory locks are not protection from hostile same-user mutation; rename is
not an inode-conditional compare-and-swap. Physical power loss, prior writeback-error
repair and network filesystem behavior remain unqualified.

## Alternatives and consequences

Deleting a partial transaction and committing anew would discard unresolved state
and expand authority. Reconstructing an old journal from an orphan candidate would
remove the original-binding prerequisite. Treating duplicate Published records as
redundant would add a separate deletion/reconciliation policy. All remain excluded.
An in-place journal rewrite would lose the existing transaction's inode/byte evidence
and add another torn-write path. Promotion retains the original complete candidate.

RENAME_NOREPLACE cannot replace the known existing intent journal; it remains the
correct separate policy for bundle publication. Exchange/backup protocols would add
names and recovery states. The selected single replacement follows the current
journal pattern, restricted to a freshly verified exact pair.

Do not broaden existing public state gates or add a generic recovery dispatcher.
A private validated pair can supply binding checks to shared full-content validation;
existing callers must retain transaction-absence checks. No public bypass flag,
transaction editing, new wire format, cleanup permission or remote capability.

A retry before promotion repeats all work. After promotion, this API rejects
Published-at-entry and the caller may explicitly request fresh attestation. Source
cleanup prevents this operation; pending transactions still block output cleanup.
Current publication, reconciliation, attestation and cleanup behavior stays unchanged.

Implementation must qualify exact pairing, conservative exclusions, independent
bytes/inodes, cancellation, syscall failures, originating and recovery SIGKILL, and
measured syscall counts. Freeze normal/reconciliation/attestation baselines before
shared-helper changes; retain phase/CPU/RSS/allocation/cleanup, recording controls,
required longer repeats and SVG/PNG plots. PERF.0 remains open. This design makes no
runtime correctness, speedup or durability qualification claim.

[Contract, state tables and protocol](../published-acknowledgment-recovery.md),
[implementation and qualification matrix](../published-acknowledgment-recovery-qualification.md),
[preceding intent reconciliation](0068-explicit-publish-intent-reconciliation.md),
[separate fresh attestation](0069-fresh-published-output-attestation.md).
