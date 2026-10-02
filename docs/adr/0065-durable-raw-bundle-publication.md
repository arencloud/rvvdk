# ADR-0065: Publish the verified RAW bundle atomically and clean only checked resources

Status: Accepted for R6.1c.5b local publication and explicit cleanup; live composition
and uncertain-publication reconciliation remain open.

## Context

R6.1c.5a creates private owned RAW output with durable verification metadata. Moving
only the RAW would separate it from metadata/ownership evidence. A Verified record
may survive process loss at an acknowledgment boundary, and stored claims cannot
recreate a safe publication or deletion capability.

## Decision

Admit a separate VerifiedOutput by consuming JobStore, requiring retained CompletedLease
source and fresh full source/output comparison, metadata/digest checks and identity
validation. No output writer or remote capability is exposed. Publication consumes
this capability and a separately locked PublicationDirectory; revalidate content at
publication because admission and use may be separated in time.

Require a private caller-chosen destination outside the store on the same filesystem,
with stable trusted ancestors. Generate the final bundle component from explicit
OutputId. Move RAW, metadata and marker together by descriptor-relative atomic
RENAME_NOREPLACE. Reject unknown bundle entries and preserve all destination collisions.
Sync members and bundle directory; persist PublishIntent before rename, sync both
parents afterward, and persist Published. Separate observed rename from durable
acknowledgment. Never roll back or automatically delete an uncertain destination.

Keep conversion records at version 1. Omitted new optional fields preserve their
canonical hash. Version 2 is introduced only for publication/cleanup transitions and
binds destination parent identity or eligible cleanup origin. Validate versions,
state/sequence, stamp coherence and bindings explicitly; older readers fail closed.
Factor the existing output commit primitive so new transitions use the same exclusive
transaction/file-sync/rename/directory-sync protocol and fault boundaries.

Provide read-only namespace assessment. Provide explicit cleanup of stamped private
partial output or acknowledged Published bundles, with checked partial CleanupIntent
retry. Preserve unknown entries/replaced inodes and refuse pending transactions,
unstamped stages and PublishIntent. Sync the parent before Cleaned even when a
recovered directory is already absent. Source cleanup stays separate; output cleanup
requires source identity binding but not source bytes. Keep the terminal journal.

## Consequences

Publication is an atomic bundle rename with an explicit same-filesystem boundary;
there is no single-file or cross-filesystem fallback. The destination lock protects
cooperating readers/writers only. Root/hostile same-user changes, offline rollback
and physical storage failure remain outside current guarantees.

Fresh output admission plus publication revalidation adds two full logical comparisons
and hashes beyond conversion's readback. Two publication journal commits and both
parent syncs add latency. Benchmark complete conversion with/without publication,
record separate admission/publication/cleanup phases, CPU/RSS/allocation and oracle
correctness, keep standalone plots and repeat adverse comparisons above 5%. Retain
PERF.0; reducing repeated verification requires a new trust/lifetime argument.

R6.1c.5b supplies bounded explicit recovery actions, not general reconciliation.
Uncertain publication/transactions and unstamped stages remain retained and blocked.
R6.1c.5c will wire the Rust runner and qualify the composed authorized live workflow.

[Contract](../durable-output-publication.md),
[tests and measurements](../benchmark-results/2026-10-02-r61c5b/README.md).
