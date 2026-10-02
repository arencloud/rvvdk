# ADR-0063: Admit retained artifacts without reconstructing a live job

Status: Accepted for R6.1c.4 local consumption/conversion; actual publication remains open.

## Context

The artifact transfer persists native-checked container metadata and CompletedLease,
but neither a journal assessment nor metadata grants a consumer access to an arbitrary
path. Reconstructing Job would also blur local consumption with remote ownership.
The existing DataMover already provides bounded native logical conversion, alias
admission, sparse output, cancellation/progress and destination flush.

## Decision

Consume JobStore into a separate RetainedArtifact. Admit only healthy CompletedLease
records with no transaction, explicit source/artifact binding, checked stage/member/
marker identities and private bounded metadata. Freshly validate native structure,
decode present grains and hash the entire container; no stored claim skips checks.
Retain the lock and confined read-only source. Expose no source handle or mutable Job.

Conversion accepts an explicit caller-owned buffered RAW destination of exact size.
Drop the prior native map before fresh re-admission to bound peak map memory, retaining
the store lock throughout. Compare original record/metadata/journal identities and
reject destination aliases to all owned members and journal. Execute controlled
portable DataMover with existing budgets and zero/flush rules. Recheck source content
and identities after destination flush before returning success.

The source journal remains unchanged. The destination is not journal-owned, published
or automatically deleted. Progress Completed describes the engine's flush; callers
must await the wrapper Result because final source checks can still fail. Errors are
closed diagnostics, and partial progress remains available through the observer.

## Consequences

Cooperating cleanup cannot run while the capability exists. Process loss releases
its lock and leaves the source CompletedLease plus a potentially partial caller-owned
output. No local checkpoint or remote capability is inferred. Same-user hostile
writers and offline rollback remain outside the cooperating private-store model.

Fresh map/decode/hash checks and post-copy hashing add CPU/I/O cost. Compare against
bare native conversion, retain phase/CPU/RSS/allocation evidence and repeat adverse
observations. Preserve barriers and source checks rather than declaring the added
cost solved. Logical oracle verification and publication remain separate claims.

R6.1c.5 will define output ownership and actual descriptor-bound no-replace publication,
then qualify the composed live workflow. No new ESXi operation is needed for this
local contract and synthetic qualification.

[Contract](../retained-artifact-conversion.md),
[tests and measurements](../benchmark-results/2026-10-02-r61c4/README.md).
