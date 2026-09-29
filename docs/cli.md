# Local RAW CLI (R3.1)

The Linux command is **`rvddk`**, using the requested public spelling. Its crate
is `rvvdk-cli`; existing `rvvdk-*` library and repository names are unchanged.
This step supports inspection and read-only planning previews. Copy, verify,
cancellation, and terminal progress events are the following R3 increments.

```bash
cargo build --release -p rvvdk-cli
target/release/rvddk inspect source.raw --format raw
target/release/rvddk inspect source.raw --format raw --json --extents
target/release/rvddk plan source.raw new.raw --format raw --backend auto --json
target/release/rvddk plan source.raw existing.raw --format raw --overwrite
```

Both commands require `--format raw`. They do not infer a format from an extension
or signature. Inputs must be regular files; devices, directories, and ordinary
FIFO paths reject before payload processing. Source symlinks are followed to their
regular-file target. Files are opened buffered and read-only. No payload copy,
flush, ring setup, writable destination open, creation, or truncation occurs.
Ordinary filesystem access can still update access metadata; no metadata snapshot
or exclusion against concurrent namespace/content changes is promised.

## Inspect

Inspection reports file identity, logical size, backend-reported block sizes,
and logical Data/Zero/Hole totals with extent count. The local backend currently
reports 512/4096 logical/physical block sizes; these are its geometry defaults,
not device-sector discovery or format validation. Local RAW discovery normally
reports Data/Hole; an unavailable sparse query safely reports dense Data.

`--extents` adds the full offset/length/kind list; summary output is the default.
Extent discovery and topology validation use existing library APIs. No content
scan determines whether allocated bytes happen to be zero, and reported Hole
bytes do not predict how much physical destination space a copy will reclaim.

`--memory-budget BYTES` defaults to 268435456. It applies the library's planning
extent-capacity check. The backend query allocates its Vec before admission;
parsing, output, path storage, and other backend metadata are outside this payload
budget. Output serializes the existing extent slice without copying it into a
second extent Vec. This is not a process memory limit.

## Plan preview

A plan report has `status: "preview"`. It describes logical source work,
destination policy, requested settings, and a Threaded execution payload estimate.
It is **not an executable or reloadable CopyPlan**, a readiness guarantee, a
snapshot, a reservation, or permission to overwrite later. JSON input/replay is
not supported. It does not claim completed or estimated physical I/O counters.

| Destination at inspection | Preview behavior |
|---|---|
| Missing file in an existing directory | `create_new_no_clobber`; no file or temporary output is created |
| Existing regular file without `--overwrite` | Error `destination_exists` |
| Existing regular file with `--overwrite` | `overwrite_in_place`; capacity must cover the source |
| Existing larger destination | Copy range is source size; report the tail bytes to preserve |
| Same source inode, including hard links | Error `same_file`, including with `--overwrite` |
| Destination leaf symlink, dangling link, special file, or directory | Error; no replacement or traversal of a leaf symlink |
| Missing parent or invalid final file name | Error; no directory creation |

Destination inspection reads metadata only, so a preview can succeed for an
existing file that is not writable. `write_access_checked` is always false.
Neither permission bits nor a successful preview prove that later creation,
mutation, sparse operations, or durability will succeed. Parent symlinks are
resolved normally. Path/identity checks are observations, not race-free future
creation or overwrite authorization. Keep source contents, sizes, and namespace
stable while inspecting; R3.2 execution must validate again with actual endpoints.

`--backend threaded|auto|io-uring` defaults to `threaded`. The Threaded request
records `selected_backend: "threaded"` and `selection_reason: "requested_threaded"`.
Auto and explicit native record `selected_backend: null` with reason
`"deferred_until_destination_preparation"`. The executable RAW library planner
requires a writable destination, so the CLI does not fabricate a writable backend
or pretend a portable Threaded selection is the future RAW decision.
`runtime_prepared` is always false. Native descriptor/request compatibility,
ring availability, and runtime Auto fallback remain untested by this preview.

Tunables are decimal bytes/counts: `--block-size` (default 1048576), `--workers`
(default 1), `--queue-depth` (default 8), and `--memory-budget`. Block size, worker
count, and native queue depth must be positive and fit their supported integer
widths. Buffer alignment remains 4096, the work queue capacity 4, and native read
window is derived as ceil(queue depth / 2). No tuning defaults changed.

`threaded_payload_estimate_bytes` uses the library's existing accounting and is
reported for **every** requested backend as an explicitly named Threaded estimate.
`threaded_payload_fits_budget: false` is a valid preview outcome, not an execution
admission. No native memory estimate is implied. Live revalidation, actual native
selection, and invocation-specific memory admission are required before execution.

## Output and exit codes

Human output uses fixed summary labels, escaped paths, and optional tab-separated
extent rows. Prefer JSON for automation. `--json` emits one UTF-8 JSON success
object on stdout or one error object on stderr. No success object is emitted for
input/planning errors. Output failure returns nonzero; stdout may then contain a
partial document. Help and version remain plain text, including with `--json`.

| Exit | Meaning |
|---|---|
| 0 | Inspection/preview succeeded, or help/version displayed |
| 1 | File, policy, planning, memory-accounting, or output error |
| 2 | Invalid command-line usage |

A preview's successful exit says the preview was produced, not that copying can
start. Neither command reports transfer completion or durable output.

JSON **schema version 1** uses these fields:

| Field | Meaning |
|---|---|
| `schema_version`, `command`, `format`, `status` | Version 1, inspect/plan, raw, inspected/preview |
| `source.path` | `display` string; `bytes_hex` is additionally present for non-UTF-8 Unix paths, preserving exact path bytes |
| `source.logical_bytes`, block sizes, `identity.device/inode` | Observed local facts, not persistent snapshot identity |
| `summary` | Logical/data/zero/hole byte totals and extent count |
| `extents` | Present only with `--extents`; ordered offset, length, lowercase kind |
| `destination` | Plan only: path, new/existing state, policy, optional existing size (`null` for new), copy range, preserved tail, write-access check flag |
| `execution` | Plan only: requested/selected backend and reason, runtime flag, settings, payload budget, and explicitly Threaded estimate/fit |

Offsets and byte counts are unsigned integers. Optional field absence differs
from explicit nulls used for deferred selection and new-destination size. Future
additive fields may appear within version 1; consumers should ignore unknown
fields. Changed field meanings/types require a new schema version. This does not
stabilize Rust CopyPlan serialization or the wording of human diagnostics.

Errors have `schema_version: 1` and `error: { code, message, os_error? }`.
Version-1 codes are `usage`, `io`, `not_regular_file`, `invalid_destination`,
`same_file`, `destination_exists`, `destination_too_small`, `source_changed`,
`memory_budget`, `concurrent_access`, `planning`, and `output`.
Match `code` rather than message text; `os_error` is present when available.

## R3.2 execution contract to implement

New outputs should be created privately in the destination directory, copied,
flushed and optionally verified, then published with a no-replace operation.
An existing name appearing after preview must not be silently clobbered. Failed
new-output publication must preserve the existing name and define temporary-file
cleanup. Existing `--overwrite` will mean in-place modification of an independently
validated regular file, retaining bytes beyond the source range. Failures can leave
partial effects there; no rollback should be promised. Recheck aliases, access,
capacity, native compatibility, and memory with live endpoints before any write.

The [architecture decision](adr/0030-read-only-cli-preview.md),
[implementation log](implementation-log.md), and
[performance evidence](benchmark-results/2026-09-29-r31/README.md) track this step.
