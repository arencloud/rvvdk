# RAW copy and verification (R3.2)

`rvddk copy` and `rvddk verify` operate on Linux regular RAW files. Explicit
`--format raw` is required. The [inspect/plan guide](cli.md) remains applicable
to read-only previews; a preview is never replayed as an executable plan.

```bash
rvddk copy source.raw new.raw --format raw --verify --json
rvddk copy source.raw existing.raw --format raw --overwrite --backend auto --verify
rvddk verify source.raw existing.raw --format raw --block-size 65536 --json
```

Copy accepts the plan tuning flags: `--backend threaded|auto|io-uring` (Threaded
default), `--block-size` (1 MiB), `--workers` (1), `--queue-depth` (8), and
`--memory-budget` (256 MiB). `--verify` adds logical read-back before publication.
Verify accepts block size and memory budget; it needs no execution backend.
Neither command accepts `--extents`. Endpoints are buffered. Defaults are unchanged
and are not claimed to be optimal for every storage device.

## Destination and durability

| Destination observed when opening | Behavior |
|---|---|
| Missing, including with `--overwrite` | Create private output; publish only if the name is still absent |
| Existing without `--overwrite` | Reject without modifying its bytes |
| Existing with `--overwrite` | Require read/write access and capacity at least source length; modify the opened inode in place |
| Existing larger file | Preserve every byte beyond source length; do not truncate |
| Known source alias, including a hard link | Reject |
| Destination leaf symlink, directory or special file | Reject |

A new file is an **unnamed `O_TMPFILE` inode** in the destination directory,
created with mode 0600 subject to umask. The existing parent directory is opened
and retained. After fresh descriptor checks, planning, memory admission and runtime
preparation, the DataMover performs the copy and its final flush. Optional
verification follows; source identity/size/mtime/ctime are checked for observed
changes. The CLI then calls file `sync_all`, publishes with no-replace `linkat`
through its owned `/proc/self/fd` entry, syncs the parent directory, and rechecks
that the destination name identifies that inode. Only then is success reported.

```mermaid
flowchart LR
    A[Private inode] --> B[Plan and prepare]
    B --> C[Copy and flush]
    C --> D[Optional bounded verification]
    D --> E[Check source and sync file]
    E --> F[Link destination if absent]
    F --> G[Sync directory and check name]
    G --> H[Report completion]
```

New output requires Linux, a filesystem supporting `O_TMPFILE` and linking that
inode, accessible procfs, and a parent that can be opened for reading and synced.
There is no named temporary fallback. Missing parents are not created. The
filesystem may reject creation, linking or sync; those failures return nonzero.
A name appearing during copy is preserved, even if `--overwrite` was supplied
when the path was absent. New output does not inherit source permissions or times.

For overwrite, the CLI observes the leaf with `O_PATH|O_NOFOLLOW`, then opens it
without creation or truncation and compares descriptor identities before I/O.
It rejects source aliases and rechecks the name before and after execution.
Writes stay bound to that opened descriptor. Success includes file `sync_all`;
there is no new directory entry to publish or sync. Failure can leave a modified
prefix. Existing inode, permissions, and tail are retained; there is no rollback.

Parent symlinks resolve normally. Keep source contents and both namespaces stable
throughout a job. Descriptor checks and metadata stamps detect observed changes;
they do not provide snapshots or exclude unrelated writers. A moved parent remains
the pinned directory, and a path can change immediately after the final check.
Verification compares live logical reads, not power-loss recovery or physical
media contents. Sync success has the filesystem/device's durability semantics.

## Verification and memory

Public `rvvdk_datamover::Verifier` compares all logical bytes over source length,
including Hole/Zero regions and the final short block. It reports the first
mismatching byte offset. Destination tail bytes are outside verification.
Both endpoints must be readable, destination capacity must suffice, and known
aliases reject. Fresh endpoint identity/size checks surround comparison. The CLI
additionally compares metadata stamps; standalone verify stamps both endpoints.
Verify does not write or flush either file and does not preserve access times.

Two reusable Vec buffers each request `min(source_length, block_size)` bytes;
empty input needs no verification payload. Admission checks the expected payload
before allocation and actual capacities afterward. With `copy --verify`, buffers
are allocated before mutation and their retained capacities are subtracted from
the budget passed to DataMover, including planning/revalidation/execution.
The [payload budget](copy-memory.md) excludes allocator metadata, path/JSON
storage, backend working allocations, kernel cache and other process overhead.
No full-image buffer, hash artifact, snapshot, resume state or RSS cap is implied.

`LocalFileBlockDevice::from_buffered_file(File)` adopts an owned descriptor without
reopening a path. It derives read/write rights and rejects append, direct-I/O,
path-only and nonregular descriptors. The CLI retains a duplicated descriptor for
identity, durability and publication operations.

## Success reports

Exit 0 means the requested operation and its output report completed. Usage errors
return 2; all operational, verification and output errors return 1. Human output
is a short completion summary. With `--json`, schema version 1 uses these fields:

| Field | Meaning |
|---|---|
| `command`, `format`, `status` | copy/raw/completed or verify/raw/verified |
| `source`, `destination` | Path objects with `display` and optional `bytes_hex`, as in preview paths |
| `logical_bytes`, `destination_tail_bytes` | Source-length operation range and untouched/unverified tail |
| `elapsed_seconds` | Operation through durability/comparison and result construction; excludes serialization and process startup |
| `verification_payload_bytes` | Actual reserved two-buffer capacities; zero for copy without verification |
| `destination_policy` | Copy only: create_new_no_clobber or overwrite_in_place |
| `requested_backend`, `planned_backend`, `backend`, `selection_reason` | Copy request, logical selection, actual execution and selection reason |
| `runtime_fallback` | Copy: null or reason/os_error from Auto native runtime fallback |
| `stats` | Copy: confirmed bytes_read/written/zeroed/discarded, blocks_copied, extents_processed; read-back verification excluded |
| `verification` | Copy: null or bytes_verified/elapsed_seconds |
| `copy_memory_budget_bytes` | Copy budget after reserving verification payload |
| `durability` | Copy: file_synced for overwrite; file_and_directory_synced for new output |

Logical counts and payload I/O counts differ for sparse work. Actual backend may
differ from planned backend only through the library's supported pre-mutation
Auto fallback. Explicit io-uring failure does not silently switch executors.

## Failures and partial effects

The existing version-1 error envelope adds optional `details`. Added codes are
`verification_mismatch`, `destination_changed`, and `copy_failed`. Existing `io`,
`memory_budget`, `same_file` and other codes remain. Match codes, not diagnostic
text. Standalone mismatch details contain `mismatch_offset`.

Once a copy target exists, error details include `phase`, `destination_state`,
`confirmed_copy_stats` (null until the engine returns a success report) and `cause`.
Nested `cause` retains a mismatch offset or contextual engine failure, including
backend, operation, range, confirmed counters and unconfirmed-I/O state where
available. The operation label in nested diagnostics is not a stable enum schema.
Errors before opening a target can omit these details and have no copy mutation.

| Destination state | Meaning |
|---|---|
| `private_unpublished` | This invocation has not published its anonymous file |
| `existing_unchanged` | Overwrite execution has not been entered |
| `existing_may_be_modified` | Overwrite execution was entered; conservatively allow partial effects |
| `publication_unconfirmed` | A publication error leaves possible namespace effects uncertain |
| `published` | Linking succeeded; a later directory sync/name check failed; do not assume rollback |

Unpublished descriptors close on ordinary failure, reclaiming an unlinked inode.
An existing native safety quarantine can retain an owned descriptor after
unconfirmed I/O; no visible temporary name is left, but immediate reclamation is
not promised. The CLI never unlinks a visible destination after failure, including
uncertain publication. Inspect the filesystem and error state before retrying.

Output serialization/write/flush can fail **after successful mutation**. That
error carries `phase: report_output`, `operation_completed: true`, and the completed
`result` inside details. Stdout may contain partial JSON. A nonzero exit therefore
does not imply that no destination was created; retain and inspect stderr.
Progress streams, cooperative cancellation, signal handling and resumable jobs
are future work, starting with R3.3.

See [ADR-0031](adr/0031-local-copy-publication.md),
[tests and benchmarks](benchmark-results/2026-09-29-r32/README.md), and Linux
[open](https://man7.org/linux/man-pages/man2/open.2.html),
[linkat](https://man7.org/linux/man-pages/man2/linkat.2.html), and
[fsync](https://man7.org/linux/man-pages/man2/fsync.2.html) documentation.
