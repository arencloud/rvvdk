# ADR-0062: Validate native container structure before persisting private artifact metadata

Status: Accepted for R6.1c.3; local consumption, conversion and publication remain open.

## Context

Owned transfer validates container bytes and preserves uncertain completion, but a
matching manifest/prefix does not establish a usable VMDK. Its metadata file is empty.
The R6.1a artifact schema can express private claims, while the existing native
StreamDisk can validate bounded metadata and decode compressed records.

## Decision

Add an explicit `transfer_owned_artifact` API sharing the owned coordinator. Keep
the container-byte API's existing scope. Within the serial owner, after seal and
before TransferComplete, adopt a freshly checked read-only payload descriptor into
StreamDisk. Use established default resource limits, require matching source capacity,
and decode every present grain. Skip structurally absent zero ranges.

Recheck the container length/SHA-256 afterward against the sealed observation. Write
the private ExportArtifact into the existing empty owned metadata member, retaining
its inode. Sync, independently reread/parse and recheck ownership before acknowledging.
Persist Complete/ContainerDigestVerified only; native decodability is not an independent
logical oracle comparison and does not justify LogicalReadbackVerified.

Keep heartbeat and cancellation coordination on the current async/blocking-worker
boundary. Native and metadata errors block TransferComplete but leave the journal
available for a durably ordered abort. Once CompleteIntent is submitted, existing
no-abort/no-retry rules apply even when metadata is complete.

## Consequences

A private successful artifact has format and present-grain evidence plus bounded
metadata bound to the exact container/source. It is still not a published object or
a recovered capability. Torn metadata and uncertain completion require conservative
assessment; parseable JSON alone never authorizes consumption or cleanup.

Map admission, decoding, another hash pass and metadata sync add CPU/I/O and memory
cost. Measure them against the same owned container lifecycle, retain adverse results,
and preserve all barriers. Source quiescence remains required; repeated checks are
not a snapshot. Default native resource limits can reject a byte-bounded export.

R6.1c.4 will admit retained artifacts freshly for read-only local consumption and
conversion, keeping lock/descriptor lifetimes and alias checks explicit. Actual
publication and composed live qualification remain separate gates.

[Contract](../owned-artifact-admission.md),
[qualification and plots](../benchmark-results/2026-10-02-r61c3/README.md).
