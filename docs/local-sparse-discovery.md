# Local sparse source discovery (R2.3)

`LocalFileBlockDevice::extents(offset, length)` returns a complete, ordered map
of the requested logical range. On Linux it queries `SEEK_DATA` and `SEEK_HOLE`.
If either query reports unsupported discovery, it returns **one Data extent for
the entire requested range**, discarding any partial map. Reading that extent
reads every logical byte, including zeros stored in filesystem holes. Sparse
output preservation is then unavailable; copying can consume more I/O and space.

## Errors and bounds

The backend checks addition overflow, the handle's original logical geometry,
`off_t` representability, and current file size before discovery. It checks size
again before returning a nonempty map, including dense fallback. Valid empty
requests return an empty map without seeking. File growth does not enlarge the
handle's logical geometry; observed truncation below the requested end fails.

| Result | Behavior |
| --- | --- |
| `EINVAL`, `EOPNOTSUPP`, `ENOSYS` | Replace the whole query with Data |
| `EINTR` | Retry the same seek |
| `ENXIO` from `SEEK_DATA` | Trailing Hole, subject to the final size check |
| `ENXIO` from `SEEK_HOLE` | Propagate the I/O error |
| Other errors, including `EIO`, `EBADF`, `EPERM`, `EOVERFLOW`, `ESPIPE` | Propagate; no fallback |
| Backward, nonprogressing, or out-of-file offsets | Reject corrupt metadata |

`EINVAL` is accepted as unsupported only after validating a nonnegative,
representable range on a regular file with known seek selectors. Linux documents
`EINVAL` for unsupported selectors and `ENXIO` for no remaining Data or a seek
beyond EOF; a successful final `SEEK_HOLE` can return EOF. See
[Linux lseek(2)](https://man7.org/linux/man-pages/man2/lseek.2.html).

## Consistency and planning

Discovery support is deliberately **not cached**. Each query can report current
errors or a changed map. Existing DataMover extent validation and plan
fingerprints remain in force: a sparse/dense map change between planning and
execution can reject a stale plan before mutation. Replan against a stable
source rather than mixing maps within a copy.

Fresh checks are observations, not a snapshot or an external-writer lock. A
truncate-and-regrow race or same-size content change can escape them. The caller
must keep source contents, size, and allocation stable throughout planning and
copying. Local payload I/O is positional; discovery still changes the descriptor's
shared file cursor through `lseek`.

`EXTENTS` means the backend can supply a valid logical map; `SPARSE` means it can
attempt sparse discovery. Neither promises that every filesystem reports holes
or that all zero bytes are holes. Destination zeroing and punching have a
[separate contract](local-sparse-output.md).

## Verification and cost

Ten deterministic tests inject unsupported discovery at both selectors and after
partial maps, real errors, interruption, malformed offsets, and truncation.
They verify exact subranges and complete logical readback across actual holes and
nonzero regions. Existing integration tests exercise supported filesystem maps
and copies. The benchmark report records normal-path comparisons and an explicit
unsupported-discovery experiment with complete output checks.

Nonempty successful queries add two metadata inspections. Unsupported discovery
performs one failed seek per query rather than maintaining a permanent negative
cache. See [R2.3 measurements](benchmark-results/2026-09-29-r23/README.md) for the
measured tradeoff and qualification limits. Prior PERF.0 issues remain open.
