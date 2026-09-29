# VMDK sources in the CLI (R4.4)

All four commands accept explicit `--format vmdk`. Destinations are always RAW;
there is no format detection or VMDK writer. Linux and the existing
[confined resolver](vmdk-backing.md) are required.

```bash
rvddk inspect disk.vmdk --format vmdk --extents --json
rvddk plan disk.vmdk output.raw --format vmdk --backend auto --json
rvddk copy disk.vmdk output.raw --format vmdk --backend auto --workers 4 --verify --progress
rvddk verify disk.vmdk output.raw --format vmdk --json
```

The [descriptor subset](vmdk-descriptor.md) and [logical reader](vmdk-logical.md)
remain the format contract: hosted base `monolithicFlat`, split flat, and custom
FLAT/ZERO layouts only. Parent chains, sparse/compressed/encrypted images and
managed variants reject. Terminal NUL padding still rejects; R4.3's QEMU-generated
fixture normalization is not automatic CLI behavior. Custom layouts have byte
oracle coverage, but independent decoder qualification remains open.

## Opening and identity

The caller authorizes the descriptor's parent directory. Its basename and every
backing reference use the resolver's `openat2` confinement; symlinks, traversal,
mount crossings and special files reject with no weaker fallback. A trusted
`/proc/self/fd` is required. Parent path components remain caller-trusted.

The CLI retains the descriptor plus an observation handle for each distinct
backing reference. These handles share the exact opened objects with the logical
reader; it never reopens descriptor-provided paths to check metadata. Observations
cover device/inode, size, mtime and ctime, including nanoseconds, before acquisition
or backing adoption and after acquisition. Copy checks them before execution and
after copy/optional verification; verify checks after comparison. Changes cause
`source_changed`. This is detection at observation boundaries, **not a snapshot or
external-writer exclusion**. Keep sources quiescent throughout the operation.

Plan, copy and verify reject a destination alias of the descriptor or **any**
backing, including hard links, with `same_file`. Copy validates the destination's
opened identity again through the existing target and DataMover contracts. The
reader's composite alias hook remains active during portable copy and verification.
Namespace replacement does not redirect already opened reads. Output name races
retain the existing no-replace publication semantics.

Acquisition retains the parser limits (1 MiB text, 8 KiB line, 1,024 extents,
128 DDB entries) and resolver limits (1,024 extents, 128 distinct backing names).
Each backing has one extra observation FD; deduplicated references reuse it.
These bounded resources, descriptor strings and reports are outside the
copy/verification payload budget. `--memory-budget` is not an RSS or FD limit.

## Execution and reports

| Request | VMDK behavior |
|---|---|
| `--backend threaded` | Portable threaded logical copy; `requested_threaded` |
| `--backend auto` | Portable threaded logical copy; `portable_api` |
| `--backend io-uring` | `unsupported_backend`, before destination opening/creation |
| `inspect` / `plan` | Read-only logical preview, no output inode creation |
| `copy --verify` | Logical read-back before publication |
| `verify` | Compare logical source bytes to the RAW destination prefix |

A VMDK never enters the native RAW adapter. RAW sources retain native selection
and runtime fallback behavior. Workers/block size tune portable copies; queue
depth is accepted for the shared CLI interface but does not configure a VMDK ring.

Schema version 1 is retained. `format` identifies the source; all sizes, extent
counts, Data/Zero totals, progress and verification offsets describe logical disk
bytes. `inspect`/`plan` report 512/512 block geometry, no composite `source.identity`,
and a `source.vmdk` object with `descriptor_identity`, `backing_identities`,
`backing_file_count`, `descriptor_extent_count` and hexadecimal `cid`. The original
extent count may differ from the coalesced logical count. CID is informational,
not a content hash or consistency guarantee. RAW's existing identity shape remains.
Copy/verify retain their path report shape and return `format: "vmdk"`.

The [copy publication contract](cli-transfer.md) and [progress/cancellation
contract](cli-progress.md) apply: private output, logical verification, file sync,
no-replace link and directory sync for new files; explicit in-place overwrite
preserves the destination tail and may leave partial bytes on failure. Cancellation
does not roll back existing or already published output. No live VMware access or
ESXi installation is needed for these local commands.
