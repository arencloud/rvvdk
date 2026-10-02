# Retained artifact admission and local conversion (R6.1c.4)

Linux `ownership::RetainedArtifact` owns a locked JobStore and a confined native
read-only source. `RetainedArtifact::open(store, id, source, options)` freshly admits
a retained CompletedLease artifact; it never constructs Job or a remote lease.
`convert_to` converts that source into a caller-owned, already-sized buffered local
RAW file using the existing controlled DataMover. No pathname is created, renamed,
published or automatically removed by this API.

These are synchronous blocking operations. Async callers must run them in a blocking
context and await completion, retaining destination ownership until workers drain.
No ESXi credentials or network connection are involved in local admission/conversion.

## Fresh admission and authority

Opening consumes the private JobStore, retaining its exclusive advisory lock for the
capability's lifetime. The following checks occur without releasing that lock:

1. Require a healthy private store and no transaction for the selected artifact.
   Validate the bounded journal envelope, store/source/artifact binding and sequence.
   Only CompletedLease is admitted. Aborted, uncertain, publication and cleanup
   states do not become a readable capability.
2. Check the stage directory, all fixed member identities and ownership marker.
   Open only the fixed metadata and payload members through that checked directory,
   read-only and without following links. Retain the journal inode identity as well.
3. Read at most 4 KiB metadata; reject unknown fields, wrong artifact/source/capacity,
   incomplete records and wrong container length. A metadata path/URL is never used.
   Serialized validation claims are untrusted, including LogicalReadbackVerified.
4. Adopt the owned payload descriptor into native StreamDisk. Apply its established
   default limits, require exact logical source capacity and decode every present
   grain. Absent grains remain structurally admitted zeros.
5. Reread journal/metadata/namespace facts and hash the entire container through a
   fresh checked descriptor. Compare with metadata regardless of its validation claim.
   Recheck ownership and metadata again after hashing. Content or identity drift
   fails admission; no record transition or repair is attempted.

Debug prints only `RetainedArtifact([private])`. No native source handle, store,
metadata hash, remote reference or mutable Job is returned. `logical_bytes()` exposes
only admitted logical size. Dropping the object closes its native source before
unlocking the store. It performs no cleanup or journal mutation.

## Conversion and lifetime

`convert_to(&mut self, &RawDisk<LocalFileBlockDevice>, CopyOptions, RetainedOptions,
observer)` accepts a caller-controlled buffered file of exactly the logical size.
The caller owns creation, destination authorization, disposal and eventual publication.
It must not pretruncate an artifact member to create that destination.

Each conversion drops its previous native map/decoder and re-admits current source
bytes while retaining the store lock. This avoids two full native maps being live
at once. Record, operation/stage, parsed metadata and journal inode must still match
the capability's original observations. A failed re-admission may leave only the
lock retained; a subsequent explicit attempt must freshly admit again.

Before any copy writes, destination identity must differ from every owned stage
member and the journal. Unknown identities, direct I/O destinations and wrong sizes
fail closed. DataMover additionally applies native-source alias checks, writable/
flush capability checks, extent validation and payload memory budgets. Source paths
are not reopened by the converter. The retained native source always goes through
logical decoding; it is never passed to a RAW descriptor-copy backend.

Controlled portable DataMover execution preserves existing sparse-zero behavior,
worker drain, progress and cooperative cancellation, then flushes the destination.
Afterward the wrapper rechecks journal/metadata/namespace and container digest,
revalidates the native source and checks destination facts. Only then does the method
return success. Full independent logical source/destination verification is separate;
this API does not produce a LogicalReadbackVerified metadata claim.

Progress callbacks describe engine execution after admission/planning. Engine
Completed means destination flush succeeded; a later wrapper check can still fail.
Callers must use the returned Result for wrapper success. Errors use a closed
vocabulary without operational names; observer counters retain partial progress.
Any failed/cancelled conversion can leave a partial caller-owned destination.
Neither a success nor a failure changes CompletedLease or publishes output.

## Bounds and cancellation

`RetainedOptions` defaults to a one-hour deadline and a cooperative cancellation
token; timeout must be positive and at most six hours. Each open/conversion gets its
own deadline. Checks occur between native grains/hash chunks and major phases;
DataMover receives the same cancellation/deadline control. Native map loading and
individual blocking I/O cannot be forcibly interrupted. Callbacks must follow the
existing DataMover contract, including not panicking.

Native default bounds still apply: 1 TiB logical capacity, 128 MiB requested map
slots, 256 MiB metadata reads, 4 × 2^20 present grains and a 512 KiB decode slot.
Metadata is at most 4 KiB; hashing uses 1 MiB and grain checks use 64 KiB. DataMover's
separate payload budget covers its copy buffers/extents, not the native index,
allocator overhead, filesystem cache or total process RSS. Conversion drops the
old map before admitting the new one. Eager re-admission and final hashing have a
measurable cost that remains tracked in PERF.0.

Trusted ancestors, private local filesystems, source quiescence and cooperating
writers remain required. The advisory lock excludes cooperating cleanup throughout
consumption. It does not stop malicious same-user mutation or offline store rollback;
identity/hash checks are observations rather than a filesystem snapshot.

## Recovery and next step

Process loss closes handles and releases the lock. The source journal stays
CompletedLease and the caller-owned RAW file remains, potentially partial. That
file has no new checkpoint, ownership record or automatic resume authority. A new
explicit consumer must re-admit the source and independently handle its destination.

R6.1c.5b now supplies [publication and explicit checked cleanup](durable-output-publication.md)
for separately owned output, with fresh admission and durable intent/acknowledgment. Do not label
this borrowed destination as journal-owned. Compose and qualify the live workflow
only after those boundaries exist; uncertain remote requests must never be replayed.

[ADR-0063](adr/0063-retained-artifact-local-conversion.md),
[tests, timing and plots](benchmark-results/2026-10-02-r61c4/README.md).

R6.1c.5a now supplies a separate [owned RAW operation](owned-raw-output.md) that
consumes this retained capability, journals private output, verifies every logical
byte and persists metadata. The borrowed `convert_to` contract above is unchanged.
R6.1c.5b adds no-replace bundle publication and explicit checked cleanup; composed
live qualification continues in R6.1c.5c.
