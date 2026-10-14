# ADR-0068: Reconcile only a freshly verified, unambiguous output PublishIntent

Status: Accepted in R6.1d.1a and implemented in R6.1d.1b. Bounded local evidence and
remaining filesystem/performance limits are recorded in the
[qualification report](../benchmark-results/2026-10-14-r61d1b/README.md).

## Context

The output publication protocol can survive process loss before or after its bundle
rename with PublishIntent still recorded. Read-only assessment distinguishes the
two namespaces but proves neither content nor durability. Normal VerifiedOutput
admission rejects PublishIntent, and its publication success report requires a
rename observed during that call. Neither API can safely stand in for reconciliation.

A visible intent may itself have survived between journal rename and store-directory
sync. A visible Published record after failed acknowledgment likewise does not prove
the previous invocation completed its persistence work.

## Decision

Add a separate explicit local operation that consumes the original locked store
and destination, explicit source/artifact/output selection and bounded controls.
Accept only valid output version-2 PublishIntent sequence 7, no source or output
transaction, a freshly admissible CompletedLease source and exactly one owned bundle
at the recorded stage or destination. Both/neither/foreign states remain blocked.

Freshly verify native/encoded source, all logical RAW bytes, digest, metadata and
identities. Sync the checked bundle and re-establish durability of the exact admitted
intent journal and store before attempting a new rename. Staged recovery performs
one no-replace rename; already-renamed recovery performs none. Both branches sync
both parents and recheck bindings before committing Published sequence 8.

Use a distinct reconciliation report with origin, verification and persistence
observations, primary error and independent assessment error. Success requires
durable acknowledgment in this invocation and consistent final namespaces. Preserve
the existing PublicationReport predicate. A failed call cannot become success merely
because assessment sees Published or an owned final bundle.

Keep one deadline, cooperative cancellation and lock/descriptor custody across the
call. After rename or entry into the already-renamed parent-sync phase, finish both
parent syncs before checking cancellation unless I/O fails. Never interrupt an
acknowledgment commit midway for cancellation. Never roll back, delete, repair a
transaction, adopt an unstamped resource or reconstruct a remote request.

## Consequences

The wire format and normal publication/cleanup eligibility remain unchanged.
Refactor only private content validation to support a checked bundle at either
parent; do not broaden VerifiedOutput admission. The original retained source is
required even when the final RAW appears intact. A caller must preserve it until
reconciliation finishes.

Published-at-entry remains outside this intent-only operation, including a retry
after a lost success response. Separate future work must define fresh Published
attestation and transaction repair. Process-loss qualification is not physical
power-loss qualification. The private cooperating-writer trust boundary is unchanged.

The implementation must qualify both branches with independent bytes, fault hooks,
SIGKILL, cancellation, namespace substitution and conservative exclusions. Measure
recovery phases separately from fixture/crash setup; compare unchanged normal
publication against a frozen baseline, retain repeated samples and standalone plots.
No performance improvement is claimed by this design and PERF.0 remains open.

[Contract and complete state table](../publish-intent-reconciliation.md),
[qualification and benchmark plan](../publish-intent-reconciliation-qualification.md),
[existing publication contract](../durable-output-publication.md).
