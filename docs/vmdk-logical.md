# Read-only FLAT/ZERO logical disks (R4.3)

`rvvdk-vmdk::VmdkDisk` implements the portable `VirtualDisk` contract over a
`ResolvedDescriptor`. It consumes the retained sources, revalidates them at
construction, and supplies logical reads and extent discovery. The
[descriptor subset](vmdk-descriptor.md) and [backing policy](vmdk-backing.md) still
apply. The public CLI supports explicit [VMDK sources](cli-vmdk.md).

## Reads and layout

Capacity comes from the checked sum of descriptor extent sizes. Logical and
reported physical block geometry are both 512 bytes; the latter is a conservative
logical-format value, not a host direct-I/O alignment assertion. Informational CHS
metadata does not determine capacity. Capabilities are READ, EXTENTS and SPARSE.
There is no WRITE, FLUSH, DISCARD, WRITE_ZERO, DIRECT_IO or native RAW endpoint.
Direct mutating trait methods and flush return Unsupported without backend calls.

Reads validate the complete requested byte range before changing the caller's
buffer. Unaligned byte requests are supported, including requests crossing any
number of FLAT/ZERO boundaries. Empty reads at EOF succeed; empty reads beyond EOF,
out-of-bounds ranges and arithmetic overflow reject. The first extent is located
by binary search; subsequent extents are traversed once. The read path allocates
no buffers or mapping vectors and holds no shared cursor or mapping lock.

FLAT translates to `backing_offset + (logical_offset - extent_start)` and reads
from the retained source. Repeated positive short reads are completed by the
existing BlockDevice exact-read contract. ZERO fills the caller's slice directly
and performs no source I/O. A successful `read_at` completes the requested range;
on error the buffer may contain a prefix from earlier extents/partial backing
reads. EOF and I/O failures propagate, never become successful zero-filled data.
Backing errors can contain physical offsets; they are not rewritten as logical
positions. Custom BlockDevices must honor the existing I/O contract.

`extents(offset, length)` clips the logical map, coalesces adjacent equal kinds,
and reports FLAT as Data and ZERO as Zero. It never infers Hole from host storage
allocation and never queries backing allocation metadata. Discovery allocates at
most a bounded map derived from the already resolved descriptor. Neither extent
discovery nor each read adds a full per-backing stat scan.

## Live validation and aliases

`revalidate()` exposes typed backing checks for standalone read sessions.
`copy_endpoint()` repeats validation for DataMover preflight and Verifier endpoint
observations. Known identity changes/loss, insufficient physical length or lost
read access fail. Construction also revalidates, catching changes between
resolution and disk creation. Once execution starts, external truncation/content
writes remain possible: retained handles are not a snapshot or multi-file lease.
Read failures propagate; callers must stabilize content when consistency matters.

A logical composite disk reports no single physical endpoint identity. To preserve
alias protection, `VirtualDisk` now has `validate_destination_identity`, called
by portable copy preflight and verification after ordinary fresh endpoint checks.
The default is a no-op; simple RAW identity checks remain in place. Wrappers must
forward this hook along with endpoint observations. The hook must not require
WRITE because verification destinations are read-only.

VmdkDisk checks the destination against **every** retained backing identity,
including repeated physical identities under hard-link names. Any match rejects
before copy mutation or verification reads. If a backing or destination identity
is unknown, copy/verification fails closed; standalone logical reads remain
available. A ZERO-only disk has no backing aliases to exclude. This is not a
content-identity test: two different files can contain equal bytes. The hook relies
on the fresh observation order specified by the trait; it is not an atomic lease.

Portable DataMover execution retains logical translation through one or multiple
workers, preserves destination tails and handles Zero under existing destination
policy. VmdkDisk does not implement BlockDevice or LinuxFdBackend, so its descriptor
or a backing file cannot silently enter the native RAW API. Auto through portable
APIs retains the established Threaded selection.

## Reference qualification

The [fixture runner](../scripts/vmdk/compare_reference.py) generates deterministic
backing bytes outside version control and invokes the small
[dump helper](../crates/rvvdk-vmdk/examples/dump_logical.rs). The helper creates a new
output only and exercises odd 65,537-byte read boundaries; it is a test utility,
not the CLI's publication/cancellation workflow.

QEMU's documented [convert and compare commands](https://www.qemu.org/docs/master/tools/qemu-img.html)
provide the external byte comparison. No QEMU implementation source or VMware SDK
is used. [Recorded evidence](benchmark-results/2026-09-29-r43/reference.json) retains
version, executable/generator hashes, commands, descriptor text and generated file
hashes. Qualification is intentionally stated per case:

| Fixture | Evidence |
|---|---|
| Synthetic monolithicFlat | rvvdk bytes, QEMU conversion/compare and expected RAW agree |
| Synthetic three-extent split flat | Same agreement, including repeated references and distinct files |
| QEMU-generated monolithic/split flat | RAW content and QEMU compare agree after explicitly recorded fixture-only removal of trailing NUL padding |
| custom FLAT/ZERO with offsets/repeated source | rvvdk matches an independently assembled byte oracle; QEMU rejects createType custom |

The direct text parser rejects NUL padding, embedded NUL and unsupported create
names. In R4.3, generated originals were preserved; normalized copies retained all
text and removed only terminal zero bytes. That evidence did not qualify unmodified
padded input. The R4.3 runner recorded the original rejection without counting it
as agreement. R4.5 acquisition and reference results below supersede that limitation. Custom ZERO/offset behavior has oracle/test coverage, not independent
reference-decoder qualification. Further interoperability work may extend these
limits explicitly; changing createType to obtain a reference decode is not a valid
substitute for validating the same descriptor.

Reproduce with QEMU installed and a fresh output directory:

```bash
cargo build --release -p rvvdk-vmdk --example dump_logical
python3 scripts/vmdk/compare_reference.py \
  --dump target/release/examples/dump_logical \
  --directory target/my-vmdk-fixtures --report target/my-vmdk-reference.json
```

## Validation and performance

Tests compare arbitrary byte ranges, all small-fixture start positions, and indexed
reads over 1,024 extents; cover maximum capacity, extent clipping/coalescing,
concurrent reads, short I/O/EOF/error behavior, immutable buffers on invalid ranges,
read-only operations, live truncation before execution, multi-backing aliases,
unknown-identity rejection, sparse portable copies and destination-tail preservation.
A local test checks a second backing's hard-link alias before mutation, successful
four-worker copies, and truncation after planning. Tests also run on Btrfs.

[Measured costs and plots](benchmark-results/2026-09-29-r43/README.md) cover memory
reads with contiguous/mixed/late-map requests and warm local FLAT/mixed copies with
final flush. Existing planning, copy, verification, parser and resolver controls
remain separately recorded; adverse pairs trigger longer repeats.

R4.4 provides [explicit CLI VMDK source selection](cli-vmdk.md) for inspect/plan/copy/verify while
destination remains RAW, reusing these contracts. Padded-descriptor acceptance is implemented by R4.5 below;
additional custom-layout reference qualification remains open.
No ESXi trial is needed for this local stage.


R4.5 update: [bounded acquisition](vmdk-padding.md) now accepts terminal NUL padding
while the direct text parser remains strict. [New reference evidence](benchmark-results/2026-09-29-r45/README.md)
qualifies unaltered generated hosted descriptors through both the dump helper and
CLI. The R4.3 results above remain historical; custom decoder qualification is open.
