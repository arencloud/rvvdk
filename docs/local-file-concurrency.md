# Concurrent local file access

R2.6 adds cooperative request admission for Linux regular files. Every
`LocalFileBlockDevice` and regular-file `IoUringFile` in the same linked library
instance shares a coordinator identified by `(device, inode)`. Independent opens,
hard links, and the direct handle's buffered fallback therefore participate.
This also covers low-level native requests that bypass `BlockDevice` methods.

## Admission policy

Admission is synchronous and fail-fast. A conflicting request returns
`Error::ConcurrentFileAccess { offset, length }` before its syscall or SQE is
submitted. The error describes the rejected request. It does not roll back
previous requests in a copy or a `read_exact_at`/`write_all_at` loop.

| Active and incoming requests | Admission |
|---|---|
| Same payload mode, overlapping reads | Allowed |
| Any overlapping byte ranges with a writer | Rejected |
| Buffered and direct payload, overlapping OS pages | Rejected, including two reads or adjacent byte ranges |
| Same mode, disjoint byte ranges | Allowed, including direct blocks smaller than a page |
| Buffered and direct payload, disjoint OS pages | Allowed |
| Extent inspection and payload | Inspection conflicts with overlapping writers; concurrent readers are allowed |
| Flush and any nonempty request or another flush | Rejected across the whole file |
| Empty payload/inspection/sparse range | No admission needed; ordinary range/access validation still applies |

Ranges are half-open; page intersection uses the Linux base page size returned by
`sysconf(_SC_PAGESIZE)`. Sparse zero/discard is a buffered write intent, including
unaligned ranges and all fallback chunks. Flush takes a whole-file barrier
through both descriptor syncs. Extent inspection retains its read intent across
all seeks and the final size check. These are library admission rules, not a
claim that every filesystem offers independent physical page or block updates.

Aligned local requests use the direct descriptor when compatible; unaligned
requests use its buffered alias and acquire the corresponding buffered intent.
A native descriptor's mode is inspected once and cached. `IoUringFile::new`
performs inspection eagerly; the retained `From<OwnedFd>` conversion does so
on its first enqueue. Nonregular low-level descriptors do not participate.

Linux [open(2)](https://man7.org/linux/man-pages/man2/open.2.html) discourages
mixing direct and buffered access, particularly to overlapping regions. This
policy excludes that overlap among participating requests; it is not a new
cross-filesystem coherency guarantee or a reason to mix modes unnecessarily.

## Native ownership and shutdown

A native operation owns its admission before SQE publication. Queued and
submitted requests both exclude conflicts. A known final CQE releases admission,
even for a negative result; keeping a `CompletedOperation` buffer alive does not
keep the file range reserved. Confirmed shutdown and ordinary unwinding release
admission along with operation owners. Unconfirmed shutdown quarantines admission
with the buffer and file descriptor. Conflicting requests can consequently remain
rejected for the lifetime of that process. There is no unsafe force-release API.

The coordinator never waits for another active request to complete. Waiting
inside enqueue could prevent the same thread from draining the CQEs needed to
release a conflicting request. Its short mutex protects admission bookkeeping
only and is never held across an I/O syscall or user callback. Existing executors
propagate conflicts with their copy failure context; they do not spin, retry the
whole copy, switch executors, or report completion after partial failure.

Callers scheduling independent jobs should drain or join conflicting work before
retrying. Quarantined work requires resolving the process-level failure, not a
blind retry loop. Concurrent jobs sharing a destination can conflict at final
flush even if their payload ranges are disjoint. Small unaligned buffered tails
can also conflict with an active direct request in the same page; use
nonoverlapping pages or sequential scheduling for such mixed requests.

## Boundaries and cost

This is request-level cooperation, not a transaction, snapshot, copy-wide lock,
or cross-process lock. The default exact-read/all-write helpers can issue several
separately admitted requests. Extent discovery does not freeze the source after
its query returns. Application-level exclusion is still required for a stable
source throughout a plan and copy, including changes between individual requests.

Raw `File` calls, exposed FDs, `mmap`, external processes, custom backends that do
not join admission, and separately loaded library copies can bypass this policy.
The caller must coordinate them and keep size, contents, backing identity, and
file status flags stable as required by the endpoint/copy contracts. In particular,
do not change `O_DIRECT` after registration. No filesystem lock is installed.

A global weak registry is consulted on registration, not on every request. Each
live file has a short mutex and an active-range vector; admission and release scan
that file's active requests. Vector growth is fallible, and stale registry entries
are pruned on insertion. The owned native guard is included in the existing
operation payload budget. Shared registry/vector capacity, Arc allocations, and
backend bookkeeping remain outside that budget, which is not an RSS cap.

Tests cover the admission matrix, page edges, subpage direct pipelines, real
hard-link aliases, sparse/inspection/flush exclusion, cross-thread guard release,
negative CQEs, confirmed completion, and quarantined shutdown. Existing threaded,
mixed-endpoint, sparse, and native failure tests remain in force.
[Performance evidence and plots](benchmark-results/2026-09-29-r26/README.md)
measure the cost with matched harnesses and full copy readback.
