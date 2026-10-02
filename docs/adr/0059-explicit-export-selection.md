# ADR-0059: Bind explicit source identity to the export proof

Status: Accepted for R6.1c.1; durable workflow integration remains open.

## Context

R6.1a defines explicit source identities, but the export proof selected a VM only
when its single disk had a unique requested capacity. The durable job foundation
in R6.1b is intentionally separate from network operations and artifact writers.
Combining every integration boundary in one change would obscure what has actually
been qualified.

## Decision

Add `export_selected_vm` alongside the existing capacity proof. Derive its network
policy from the source identity, select by reference, and validate UUID, disk key,
backing, capacity and the powered-off scope on that connection. Require matching
options and reject graceful shutdown before connecting. Add checks after staging,
before download and before completion; preserve the existing transfer implementation.
The example exposes an explicit all-or-nothing identity mode.

With a ready lease, refresh progress before each new read and limit the read to
one third of the lease timeout, capped at ten seconds and the operation deadline.
Await a property operation and its bounded pagination cleanup; observe cancellation
before the next operation. No automatic retry or alternate selection is introduced.

## Consequences

Equal-capacity VMs are selectable without relying on names or inventory order.
Repeated observations reduce the interval in which unnoticed changes can occur,
but do not exclude remote races. Three additional property calls and a progress
refresh have a measurable fixed cost; retain matched synthetic measurements rather
than describing this as a throughput optimization.

This bounded package completes explicit selection in the export proof. It does
not complete production R6.1c or confer durable ownership on the existing writer.
The legacy API remains available for qualification. The next package must bind
real lease capabilities and owned resource handles to durable intent, and resolve
heartbeat/cancellation, uncertain completion and outstanding-writer coordination.
No old manifest is automatically upgraded into job ownership.

[Contract](../explicit-export-selection.md),
[evidence](../benchmark-results/2026-10-02-r61c1/README.md).
