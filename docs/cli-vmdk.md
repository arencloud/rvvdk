# VMDK sources in the CLI (R4.4–R5.4)

All four commands accept explicit `--format vmdk`. Destinations are always RAW;
there is no RAW/VMDK autodetection or VMDK writer. Linux and the existing
[confined resolver](vmdk-backing.md) are required.

```bash
rvddk inspect disk.vmdk --format vmdk --extents --json
rvddk plan disk.vmdk output.raw --format vmdk --backend auto --json
rvddk copy disk.vmdk output.raw --format vmdk --backend auto --workers 4 --verify --progress
rvddk verify disk.vmdk output.raw --format vmdk --json
```

The [descriptor subset](vmdk-descriptor.md) and [logical reader](vmdk-logical.md)
define hosted base `monolithicFlat`, split flat and custom FLAT/ZERO layouts.
R5.4 additionally accepts the [clean version-1 base sparse subset](vmdk-sparse-disk.md):
`monolithicSparse` containers and external `twoGbMaxExtentSparse`/`2GbMaxExtentSparse`
descriptors. Parent chains, version 2, dirty state, compressed/encrypted images and
managed variants reject. [Bounded terminal NUL padding](vmdk-padding.md) is accepted
during acquisition; embedded NULs or nonzero suffixes reject. Files are never rewritten. Custom layouts have byte
oracle coverage, but independent decoder qualification remains open.

## Opening and identity

The caller authorizes the descriptor's parent directory. Its basename and every
backing reference use the resolver's `openat2` confinement; symlinks, traversal,
mount crossings and special files reject with no weaker fallback. A trusted
`/proc/self/fd` is required. Parent path components remain caller-trusted.

The CLI retains the descriptor plus an observation handle for each distinct
backing reference for FLAT/ZERO, or each sparse extent (including repeated names).
These handles share the exact opened objects with the logical
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

Acquisition reads an initial chunk of at most 4 KiB from the opened source. Within
explicit VMDK mode, the hosted sparse magic selects binary header admission; all
other input follows bounded text acquisition. Text reuses that first chunk, reads
at most 1 MiB plus one oversize probe, then parses FLAT/ZERO or the explicit split
sparse grammar. The first successful flat parse is reused for resolution. Original
bytes (including terminal padding) are never rewritten.

For a binary container, the CLI rereads the exact 512-byte header and validates its
geometry and advertised descriptor range against the initial observed file length.
Only then does it allocate/read the descriptor region (at most 1 MiB). That text
must be monolithicSparse; its extent reference must resolve to the same device/inode
as the opened container **before** backing metadata reads. A different file with
identical bytes is rejected; a hard link to the same inode is accepted. An external
text monolithicSparse descriptor is rejected: pass the actual container. Passing a
split extent file instead of its external descriptor is also rejected.

Text limits remain 1 MiB input, 8 KiB line, 1,024 extents and 128 DDB entries.
FLAT/ZERO resolution retains 1,024 extents / 128 distinct backing names. Sparse
sources use SparseDisk defaults: 128 loaded extents, 128 MiB aggregate metadata
reservation, 256 MiB aggregate metadata reads, and 65,536 coalesced query entries,
plus the per-extent limits. Sparse repeated references are charged separately.
The entry probe/header/descriptor acquisition is additional to loader counters:
up to 4 KiB + 512 bytes + 1 MiB for a container, or 1 MiB + one byte for text.
Each backing also has an observation FD. These bounded source resources and JSON
reports are outside the copy/verification payload budget. `--memory-budget` is
not a total RSS, source-metadata or FD limit.

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
Sparse previews additionally report `layout: "hosted_sparse"`,
`metadata_memory_reservation_bytes` and `metadata_read_bytes` (loader counters,
excluding CLI entry acquisition). Repeated sparse references count as separate
loaded backings. Existing FLAT/ZERO preview fields are unchanged.
Copy/verify retain their path report shape and return `format: "vmdk"`.

The [copy publication contract](cli-transfer.md) and [progress/cancellation
contract](cli-progress.md) apply: private output, logical verification, file sync,
no-replace link and directory sync for new files; explicit in-place overwrite
preserves the destination tail and may leave partial bytes on failure. Cancellation
does not roll back existing or already published output. No live VMware access or
ESXi installation is needed for these local commands.


R5.4 [evidence and plots](benchmark-results/2026-09-30-r54/README.md) qualify all four
commands against QEMU-produced monolithic/split fixtures, including a two-file split
disk. See [ADR-0041](adr/0041-sparse-cli-source-acquisition.md). R5.5 adds
[adversarial and scaling qualification](vmdk-sparse-qualification.md). Next is R5.6
bounded parent-chain metadata admission, before logical fallback and CLI exposure.
