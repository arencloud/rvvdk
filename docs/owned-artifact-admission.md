# Private native artifact admission (R6.1c.3)

Linux `transfer_owned_artifact(source, credentials, store, artifact, options)` adds
native streamOptimized validation and private R6 artifact metadata to the owned
transfer lifecycle. It uses `OwnedTransferOptions` and returns `OwnedTransferReport`.
`is_artifact_success()` requires normal transfer success, native admission and
acknowledged metadata durability. The existing `transfer_owned_export` remains the
container-byte qualification API; it leaves metadata empty.

Both APIs retain a private stage. Neither publishes, converts, restores a lease
from a journal, or returns an admitted capability for subsequent local consumption.
The explicit source/pin, powered-off single-disk scope, bounded download, accepted
writer drain and conservative CompleteIntent boundary remain unchanged.

## Ordering and evidence

The journal worker owns all operations below; no file handle escapes its lifetime.
After the existing manifest check, payload sync and independent SHA-256/SHA-1
readback, and **before TransferComplete**, the artifact path:

1. Freshly checks stage/member identities and marker, then opens the owned payload
   read-only. `LocalFileBlockDevice` adopts that descriptor without reopening a
   pathname or following a descriptor/metadata reference.
2. Loads `StreamDisk` with the existing default native limits. Header, embedded
   descriptor, footer, directory, grain tables, record ranges/order and capacity
   must satisfy the already qualified streamOptimized rules. The encoded logical
   capacity must equal explicit source capacity.
3. Reads every present grain through the native decoder. Each record must be one
   complete checksummed zlib stream expanding to exactly 64 KiB, including a padded
   final grain. Absent grains are structurally admitted zeros; admission does not
   scan potentially enormous zero ranges.
4. Revalidates the native handle/header, releases it, checks ownership again and
   makes another bounded SHA-256 pass through a fresh read-only descriptor. Length
   and digest must match the sealed network/readback observation. This binds native
   admission back to the previously verified container, under source quiescence.
5. Constructs the bounded R6.1a `ExportArtifact` with the actual artifact/source
   binding, pin provenance, logical/encoded sizes and container digest. It records
   Complete and **ContainerDigestVerified**, never LogicalReadbackVerified.
6. Requires the existing owned metadata member to be empty. Writes it without
   replacing its inode, syncs it, closes it and independently reopens/rereads it.
   Exact bytes, parsed contract and stage identities must still match before the
   metadata acknowledgment is returned.

Only then may the coordinator persist TransferComplete, recheck source, persist
CompleteIntent, recheck source again and complete the live lease. Stage creation
already synced the metadata directory entry. This operation updates that existing
file and syncs its contents; it does not publish or rename a metadata object.

This proves native structure and decodability of present records for the admitted
container. It does not compare logical disk bytes against a guest or independent
source oracle. The private v1 validation claim deliberately remains conservative;
there is no schema change or promotion of an existing proof manifest.

## Bounds, cancellation and failures

The inherited native defaults limit logical capacity to 1 TiB, map slot memory to
128 MiB, metadata reads to 256 MiB, examined table entries to 32 million (binary
units), and present grains to 4 million (binary units). Envelope/descriptor budgets
apply separately. Decoder slot memory is capped at 512 KiB. The worker adds one
64 KiB grain buffer and one 1 MiB hash buffer; private metadata is at most 4 KiB.
These are explicit requested-buffer/index budgets, excluding allocator overhead,
HTTP/TLS state, inventory, fixed stack and kernel caches. A download within its byte
limit can still fail native admission because of format or native resource limits.

Lease heartbeats continue asynchronously during admission and metadata persistence.
Cancellation/deadline checks occur before and after map admission, between grains
and hash chunks, and around metadata persistence/readback. An accepted metadata
write/sync is drained before the coordinator handles cancellation. Map loading and
an individual blocking I/O call can outlast a deadline; no unsafe thread termination
or arbitrary blocked-kernel-I/O guarantee is claimed.

Native/metadata failures remain payload failures. They prevent TransferComplete and
permit the existing single abort only after a durable AbortIntent. Torn metadata,
failed sync/readback or process loss cannot acknowledge metadata durability. No
retry truncates or overwrites a nonempty metadata member. Fully parseable metadata
after a crash is still only an untrusted claim; a LeaseHeld/CompleteIntent journal
cannot authorize local cleanup or recreate a live remote capability.

The report exposes booleans and a count of successfully decoded present grains,
including partial progress on failure. Hashes, source identities, paths and lease
references remain absent from diagnostics. Native verification may be true when
metadata persistence fails; both may be true when remote completion is uncertain.
`is_artifact_success()` rejects each of those incomplete outcomes.

All prior trusted-ancestor/private-store/cooperating-writer assumptions remain.
Fresh identity/hash checks are observations, not snapshots or exclusion of a hostile
same-user writer. Metadata is private and correlatable; do not copy it into logs.

## Continuation

[R6.1c.4](retained-artifact-conversion.md) now freshly admits a retained CompletedLease
stage without reopening Job or a lease. Its read-only capability retains the store
lock and confined source through local conversion and final checks. Metadata paths
or claims do not confer authority. The RAW destination is caller-owned; R6.1c.5
must define actual output ownership and no-replace publication before composed live
qualification. No ESXi operation was required for this step.

[ADR-0062](adr/0062-private-native-artifact-admission.md),
[tests, raw samples and plots](benchmark-results/2026-10-02-r61c3/README.md).
