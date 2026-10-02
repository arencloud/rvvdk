# ADR-0057: Explicit source identity and versioned artifact claims

Status: accepted for R6.1a; workflow integration and durable ownership are separate gates.

## Context

The qualified export proof selects an unambiguous VM by single-disk capacity and
then rechecks its identity. Capacity is useful for a controlled experiment but
does not express the caller's intended source. Its diagnostic manifest is also
insufficient to authorize production recovery or connect a retained container to
an explicitly selected source.

## Decision

Add a pure Rust `rvvdk_vsphere::contract` module. A runtime `SourceSelection`
requires a pinned endpoint and pin provenance, VM managed reference and BIOS
UUID, disk device key and backing identity, and expected logical capacity.
Selection rejects duplicates and reuses the current single persistent disk scope
gate; it additionally requires powered-off state and no disabled export method.
Names, inventory position and capacity alone cannot select a source.

Persist a bounded, versioned, single-container `ExportArtifact` with a caller
artifact ID, domain-separated source binding, explicit pin provenance, container
format, separate logical/encoded sizes, completeness, validation claim and optional
encoded-content digest. Keep runtime endpoint/reference/backing strings and all
credentials, cookies, ticket URLs, lease handles and paths outside this metadata.
The fixed container member name is `disk-1.vmdk`; metadata supplies no path.

Provenance is supplied by the caller. Exact pin verification does not itself prove
how the pin was obtained. A pin, endpoint or provenance change invalidates the
source binding. Opaque hashes avoid storing guest names but remain sensitive,
linkable data. They are not anonymization, signatures or ownership capabilities.

Parsing returns internally consistent **untrusted claims**. It cannot certify
completion, past verification, freshness, file identity or authorization. Fresh
source and container observations remain necessary even for a record claiming
logical readback. No schema field or successful comparison permits resuming,
aborting, cleaning up or publishing resources. R6.1b defines those ownership rules.

## Consequences

JSON input is capped before deserialization, fixed shape and strict about unknown
and duplicate fields, versions, sizes, digests and incompatible state claims.
No raw serde error or operational identity appears in Debug/errors. Explicit
persistence includes private content/source digests and must stay out of logs.

R6.1a adds no remote calls, durable job store or production export command. The
capacity-selected qualification harness and its existing manifest stay unchanged.
R6.1c must use explicit selection at each required live revalidation point and
must not upgrade the old manifest through capacity matching. Current one-disk,
powered-off scope does not establish online, multi-disk or vCenter consistency.

[Contract and schema](../export-artifact-contract.md),
[qualification and synthetic benchmark baseline](../benchmark-results/2026-10-02-r61a/README.md).
