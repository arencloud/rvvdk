# Local sparse output (R2.2, R5.12p)

Linux writable LocalFileBlockDevice handles now advertise WRITE_ZERO, DISCARD,
and DISCARD_ZEROES. Both operations guarantee zero-reading logical contents
inside the requested range, unchanged bytes outside it, and unchanged file size
on success. Read-only handles advertise none of these operations and reject even
empty mutation requests. RawDisk forwards the capabilities.

## Execution and fallback

R5.12p makes `write_zero_at` try `fallocate(PUNCH_HOLE | KEEP_SIZE)` first.
If punching returns EOPNOTSUPP or ENOSYS, it tries `ZERO_RANGE | KEEP_SIZE`;
if that is also unsupported, it uses bounded writes. `discard` continues to try
`PUNCH_HOLE | KEEP_SIZE` followed by bounded writes. Linux defines zero reads for
successful calls and zeroes partial blocks within punched ranges. We pass the
exact byte range instead of rounding into neighboring data. See the primary
[fallocate manual](https://man7.org/linux/man-pages/man2/fallocate.2.html).

The backend handles EOPNOTSUPP and ENOSYS with ordinary positional zero writes
of at most 64 KiB. A shared immutable zero array supplies the bytes without a
per-operation zero buffer. Admission bookkeeping may grow its range vector. Unsupported kernel modes are cached independently
on each open device using atomic flags. Zero and discard share the punch-mode bit;
the zero-range bit remains separate. Concurrent first
calls may each probe; later calls use the fallback. Reopening probes again.
Capabilities describe logical support, so they remain advertised after fallback.

Interrupted fallocate calls retry. Other errors, including EINVAL, EPERM, EIO,
ENOSPC, and EFBIG, propagate without fallback or capability caching. This avoids
hiding permissions, malformed requests, storage failures, or partial effects.
Fallback is internal to the advertised operation: a returned failure is still
not retried by DataMover. The destination may be partially modified on error.

Both operations validate write permission, checked range arithmetic, cached
logical geometry, fresh descriptor access/append state and current file size,
and representability in off_t. Empty in-bounds requests return without issuing
fallocate. Truncation observed since open is rejected before writes can regrow
the file. Neither operation implicitly flushes; DataMover keeps its existing
success flush boundary, and direct callers must flush as required.

For direct-open devices, fallocate and fallback writes use the existing buffered
alias. Its current append/access flags are checked as well as the primary
handle's. Construction already verifies that both descriptors refer to the same
inode. Tests cover buffered and direct-open operations, partial filesystem-block
edges, and mixed native/threaded copies with aligned Data requests. They do not
qualify arbitrary buffered/direct I/O or native unaligned Data tails. R2.6 now
[admits participating alias requests](local-file-concurrency.md), holding one
buffered-write intent across each sparse syscall and all fallback chunks.

As with existing preflight, inspection is not a lock. Callers must stabilize file
size, status flags, and nonparticipating writes during operations. Concurrent external
truncation or flag changes after inspection are outside this contract.

## Contents, allocation, and statistics

Hole punching can release complete filesystem blocks; unaligned edge bytes are
zeroed without requiring neighboring blocks to disappear. Unsupported operation
fallback preserves contents but may allocate storage. The secondary ZERO_RANGE path may allocate
unwritten extents rather than release space. Therefore SPARSE/DISCARD_ZEROES
are not promises that every call reclaims physical space.

Copy statistics retain their operation-based meanings:

- Data writes contribute bytes_written.
- Successful backend zeroing contributes bytes_zeroed, even if the backend used
  its ordinary-write fallback internally.
- Successful backend discard contributes bytes_discarded, even if physical space
  was not reclaimed. This is not a measure of blocks released.

Local Hole copies previously contributed ordinary-write counters. They now
contribute discard counters; Zero output contributes zero counters. Counters and
callback cadence follow the existing accelerated-operation policy. Logical bytes
and the final success/flush contract are unchanged. The core copy memory budget
still excludes backend storage; the new fallback uses one shared 64 KiB array.

## Validation and limits

Nine added tests cover forced unsupported modes/cache independence, interruption,
real errors with partial effects, empty/overflow/truncated ranges, append flags
on primary and buffered aliases, read-only rejection, odd boundaries/readback,
and mixed Data/Zero/Hole copies through one/four-worker and native executors.

The ordinary workspace run leaves two explicit allocation tests ignored because
it requires a supporting storage filesystem. Run all four local output tests,
including that allocation check, with:

```bash
RVVDK_TEST_DIR="$PWD/target/sparse-storage-tests" \
  cargo test -p rvvdk-local --test sparse_write -- --include-ignored --nocapture
```

On the recorded Btrfs storage mount, an 8 MiB incompressible file fell from
16,384 to 4,096 allocated 512-byte blocks after punching its middle 6 MiB.
Full readback verified zeroes in the requested range and unchanged surrounding
contents and size. This is evidence for that filesystem/run, not universal
allocation behavior. Forced-error tests separately validate fallback when kernel
acceleration is unavailable. [R2.2 evidence](benchmark-results/2026-09-29-r22/README.md)
contains commands, raw output, matched timings, and performance limitations.

[Sparse source discovery fallback](local-sparse-discovery.md) is implemented by R2.3. Native request
compatibility and runtime resource reuse are implemented by R2.4/R2.5. R2.6
covers cooperative concurrent alias admission.
The [logical Hole contract](adr/0026-logical-hole-guarantee.md) remains authoritative.


## R5.12p qualification

The retained-export ENOSPC failure exposed why logical Zero writes should avoid
unnecessary allocation. The change is local to the regular-file backend; source
Zero/Hole extents, endpoint capabilities and operation-based copy statistics stay
the same. A successful zero call may deallocate complete blocks; it does not reserve
space for later writes. Existing nonzero destinations and partial edges use the
same exact-range semantics. One access guard spans punching, zero-range fallback
and bounded writes. Real errors are never retried through a different mode.

New unit cases qualify punch-first completion, secondary zero-range support,
shared unsupported-mode caching and real-error propagation for both public
operations. Allocation tests cover populated and fresh sparse files as well as
partial-block sentinels. [ADR-0056](adr/0056-space-efficient-local-zero-output.md),
[complete storage qualification and performance disposition](benchmark-results/2026-10-02-r512p/README.md).
