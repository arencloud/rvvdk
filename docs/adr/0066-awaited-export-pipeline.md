# ADR-0066: Compose export and publication through awaited owned capabilities

Status: Accepted for R6.1c.5c bounded Linux qualification.

## Context

The individual export, conversion and publication capabilities have separate durable
journals, locks and reports. A runner that drops a blocking task on cancellation,
reopens a substituted store path, or retries an uncertain remote completion would
break those guarantees even if each individual operation is correct.

## Decision

Preflight the destination and output ID before remote effects. Hold the destination
capability throughout. Pin the original job-store inode with an internal descriptor
anchor; reacquire the lock and freshly admit each local capability after export's
worker drains. Preserve existing report boundaries and success predicates.

Await export and one blocking local task. Share one cancellation token and one local
absolute deadline across retained admission, conversion, fresh output admission and
publication. On cancellation, drain accepted work rather than race it against a
future drop. Report a local worker panic as unknown local outcome; preserve remote
results, and never infer rollback or cleanup authority.

Provide a private-config Rust example for fresh discovery/inspection, execution and
separate explicit cleanup. Passwords remain terminal-only. Cleanup uses existing
checked operations; no new uncertain-state repair capability is introduced.

## Consequences

The runner can qualify a complete powered-off export through a durable RAW bundle
without a Python VMware/data implementation. Offline measurement and independent
oracle tools remain separate. Success retains evidence until explicit cleanup.
The future must be awaited; process loss still requires conservative recovery.
Store locks are reacquired between phases, so freshness checks remain necessary.
Repeated full byte verification and durability barriers remain measurable costs.
This step does not close production recovery or grant authority over unknown leases.

[Contract](../composed-export-pipeline.md),
[evidence](../benchmark-results/2026-10-02-r61c5c/README.md).
