# ADR-0026: Logical Hole and zero-reading discard

## Status

Accepted, 2026-09-29, implemented by R2.1. Supersedes the unconditional discard
preference in [ADR-0003](0003-extent-copy-semantics.md) and
[ADR-0022](0022-native-hole-extents.md).

## Problem

The copier previously selected `discard` whenever DISCARD was advertised.
Ordinary discard does not promise particular read-back bytes in our backend
contract. A successful discard that preserves old nonzero contents could make
a Hole copy report success with incorrect logical output. Source sparse metadata
and destination deallocation guarantees are separate concerns.

## Decision

Both `ExtentKind::Zero` and `ExtentKind::Hole` promise that **every logical byte
in the range reads zero**. Their payload may be omitted from source reads. Hole
adds a preference for zero-guaranteed discard on output; it is not permission to
leave the destination untouched. Data continues to require source reads.

A backend resolving a layered/parent-backed disk must resolve inherited content
before reporting a logical Hole. Physical unallocation alone is insufficient if
a read would expose nonzero parent data. Callers must still stabilize their
source; extent metadata is not a snapshot or a read-back verification mechanism.

Introduce the `Capabilities::DISCARD_ZEROES` modifier at bit 8. In combination
with DISCARD, it promises that **successful discard of the complete requested
range** makes subsequent logical reads return zero, preserves disk size, and
preserves bytes outside the range. This includes boundary bytes of unaligned
ranges. The guarantee is subject to subsequent writes and normal concurrent
access rules. The modifier alone does not advertise a callable discard operation.

It does not guarantee physical reclamation, secure erasure, atomicity, or
persistence across a crash. WRITE_ZERO makes the same logical zero-content
promise for `write_zero_at`. Ordinary DISCARD remains available to callers that
explicitly want it without assuming zero reads.

All copier executors share this priority:

| Intent | Capabilities | Operation |
|---|---|---|
| Data | READ/WRITE preflight as before | Read source, write destination |
| Zero | WRITE_ZERO | write_zero_at |
| Zero | Otherwise | Bounded zero-filled writes |
| Hole | DISCARD **and** DISCARD_ZEROES | discard |
| Hole | Otherwise, WRITE_ZERO | write_zero_at |
| Hole | Otherwise | Bounded zero-filled writes |

The policy applies to direct copying, planned/observed sequential execution,
workers, and destination-aware native extent execution. Data/Zero policy,
endpoint preflight, copy memory budgets, and flush ownership remain unchanged.
Capabilities are read when selecting an operation; custom backends must keep
their advertised guarantee true for every successful call.

An advertised operation that fails is **not retried** using a weaker operation,
including when it returns Unsupported. The failed operation may have partially
changed the range. Failure context retains the operation and confirmed lower
bounds, marks potentially unconfirmed effects, and suppresses final success
observation and copy flush as before. Capability-based fallback happens before
the operation is attempted.

## Migration and built-in backends

- Custom backends advertising only DISCARD now receive WRITE_ZERO or ordinary
  zero writes for Hole output. This deliberate behavior change prioritizes
  logical correctness; it may perform more I/O or allocate more physical space.
- Add DISCARD_ZEROES only after establishing the full requested-range guarantee,
  including unaligned edges and bytes surrounding the range. Do not infer it
  from DISCARD or SPARSE. Do not add it merely to retain an old fast path.
- Default MemoryBlockDevice advertises both flags because its discard fills the
  requested bytes with zero. It does not release physical storage. Explicit
  `with_capabilities` masks remain explicit; they are not silently upgraded.
  Read-only construction does not advertise discard or the guarantee.
- RawDisk forwards its backend's capabilities and operations unchanged. A
  translated VirtualDisk must provide the guarantee for its **logical** mapping.
- LocalFileBlockDevice does not yet advertise discard or WRITE_ZERO. It still
  materializes Hole output with bounded zero writes. Filesystem hole punching,
  range/access checks, partial-block handling, and filesystem fallback are R2.2.

`bytes_discarded` counts logical bytes successfully processed through discard;
it is not a measurement of reclaimed space. Accelerated zero calls contribute
`bytes_zeroed`; fallback writes contribute `bytes_written`. A successful copy
still follows the destination's flush contract. No transactional rollback or
physical-allocation qualification is added here.

## Validation and performance

The adversarial backend returns successful ordinary discard with nonzero bytes.
Tests cover all eight combinations of DISCARD, DISCARD_ZEROES, and WRITE_ZERO
across direct/report/planned/observed portable copying with one/four workers,
sparse-only native dispatch, odd extent boundaries, destination guard bytes,
logical read avoidance, counters, and partial discard failure without retry.
Mixed RAW/native parity adds unqualified-discard fallback profiles. Existing
sparse-operation failure and lifecycle regressions retain their intended paths
by explicitly advertising the guarantee where their fixture provides it.

The old policy fails the new portable and native regressions when supplied only
with the new flag declaration and fixture. [R2.1 evidence](../benchmark-results/2026-09-29-r21/README.md)
records this reproduction, correctness checks, matched benchmark harness changes,
and performance limits. Local sparse allocation and native availability/tail
compatibility remain separate R2 milestones. No ESXi host is required.
