# ADR-0069: Attest current Published output without rewriting its acknowledgment

Status: Accepted in **R6.1d.2a**, 2026-10-14. Implementation and qualification pending
**R6.1d.2b**. Reviewed against `b569060`; this decision changes no runtime behavior.

## Context

Normal publication and intent reconciliation both rename their Published journal
before syncing the store directory. A process may fail in that gap or lose the
response after success. Later read-only assessment sees Published in either case.
Neither a visible record nor a saved report proves the earlier invocation completed.
Intent reconciliation intentionally rejects Published-at-entry.

A consumer needs an explicit way to verify and re-establish the current output's
persistence barriers while retaining this distinction. Cleanup is a separate request
and cannot stand in for verification.

## Decision

Design `JobStore::attest_publication`, consuming the original store/destination locks,
explicit output/artifact/source selection and one bounded control. Accept only valid
version-2 Published sequence 8, no source/output transaction, retained CompletedLease
source, absent stage and original owned final bundle. Every other state and namespace
pair remains ineligible. Keep the cooperating-writer/private-store trust model.

Freshly admit encoded/native source and fully compare RAW, including zero ranges,
metadata, SHA-256, exact length and post-read identities. Sync three members and
bundle, then store and destination, then the exact existing Published journal and
store again. Recheck original bindings, transactions and namespace; perform independent
read-only assessment and final control observation while locks remain held.

Do not rewrite Published, increment sequence, create a transaction, rename, recopy,
delete or reconstruct any remote action. The existing journal's bytes and inode
remain unchanged. Its final sync barrier is repeated after namespace barriers;
earlier journal visibility is never treated as an observed historical acknowledgment.

Use a separate `PublicationAttestationReport` for fresh content, bundle/parent/record
syncs and final binding observations, with primary and assessment errors. Success
requires every flag, no errors and consistent Published sequence-8 final state.
There is no prior-success or new-acknowledgment flag. A later failure cannot become
success merely because the record still says Published.

One absolute deadline includes source admission and post-assessment control checks.
Control is checked after potentially lengthy binding checks and before each ordinary
sync. Finish each parent pair or journal/store pair before observing cancellation,
unless I/O fails. No in-call retry, rollback, recovery dispatch or cleanup occurs.

## Alternatives and consequences

Returning success from assessment would omit content and new sync evidence. Reusing
intent reconciliation would weaken its state gate and confuse new acknowledgment
with a fresh observation. Rewriting Published or adding an Attested state would add
journal transactions, inode/sequence changes and uncertain recovery paths without
proving earlier success. This bounded operation therefore preserves the existing
wire format and uses its own report.

The source must remain available even after normal publication succeeds. After source
cleanup this operation is unavailable; explicit identity-checked Published cleanup
retains its existing independent eligibility. Repeated attestation is permitted only
after repeating full admission, verification and all eight sync calls. Reports are
not certificates or reusable authority. Same-user hostile writers, offline rollback,
prior storage-error repair and physical power-loss guarantees are outside this scope.

Reuse private `CheckedOutput` and retained admission; keep normal Verified admission,
PublicationReport and ReconciliationReport unchanged. Any moved private binding/sync
helpers need frozen-build regression checks for both existing paths. Do not introduce
a generalized recovery engine or production profiling/skip options.

Implementation requires independent bytes, source/output journal preservation,
zero writes/commits/renames, conservative exclusions, fault/cancellation tests and
SIGKILL at both originating acknowledgment and new attestation boundaries. Retain
phase/wall/CPU/RSS/allocation/cleanup measurements, overhead controls, required longer
repeats and SVG/PNG plots. This design has no runtime performance result; PERF.0
and general production recovery remain open.

[Contract, state tables and report](../published-output-attestation.md),
[implementation and acceptance matrix](../published-output-attestation-qualification.md),
[preceding intent-reconciliation decision](0068-explicit-publish-intent-reconciliation.md).
