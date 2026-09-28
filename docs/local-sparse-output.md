# Local sparse output (R2.2)

Linux writable LocalFileBlockDevice handles now advertise WRITE_ZERO, DISCARD,
and DISCARD_ZEROES. Both operations guarantee zero-reading logical contents
inside the requested range, unchanged bytes outside it, and unchanged file size
on success. Read-only handles advertise none of these operations and reject even
empty mutation requests. RawDisk forwards the capabilities.

## Execution and fallback

`write_zero_at` uses `fallocate(ZERO_RANGE | KEEP_SIZE)`;
`discard` uses `fallocate(PUNCH_HOLE | KEEP_SIZE)`. Linux defines zero reads for
successful calls and zeroes partial blocks within punched ranges. We pass the
exact byte range instead of rounding into neighboring data. See the primary
[fallocate manual](https://man7.org/linux/man-pages/man2/fallocate.2.html).

The backend handles EOPNOTSUPP and ENOSYS with ordinary positional zero writes
of at most 64 KiB. A shared immutable zero array supplies the bytes, with no
per-operation heap allocation. Unsupported acceleration is cached independently
for zeroing and punching on each open device using atomic flags. Concurrent first
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
qualify arbitrary overlapping buffered/direct I/O or native unaligned Data tails.

As with existing preflight, inspection is not a lock. Callers must stabilize file
size, status flags, and overlapping writes during operations. Concurrent external
truncation or flag changes after inspection are outside this contract.

## Contents, allocation, and statistics

Hole punching can release complete filesystem blocks; unaligned edge bytes are
zeroed without requiring neighboring blocks to disappear. Unsupported operation
fallback preserves contents but may allocate storage. ZERO_RANGE may allocate
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

The ordinary workspace run leaves one explicit allocation test ignored because
it requires a supporting storage filesystem. Run all three local output tests,
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

Sparse source discovery fallback is R2.3. Native runtime availability/request
compatibility and persistent per-job ring reuse remain subsequent R2 work.
The [logical Hole contract](adr/0026-logical-hole-guarantee.md) remains authoritative.
