# ADR-0028: Endpoint identity and fresh descriptor inspection

## Status

Accepted for the bounded R0.5/R1.3 endpoint contract, 2026-09-28. Persistent
snapshot identity, consistency leases, and stronger durability guarantees remain
future decisions. Complements ADR-0027; does not change executor selection.

## Context

RAW preflight checks both logical backend claims and actual descriptor access.
For a local file, each side used to inspect the same FD independently: four
fstat/F_GETFL pairs per source/destination preflight. Removing logical checks
would let a writable FD bypass backend restrictions; trusting only backend
metadata would lose independent descriptor identity validation.

## Decision

`rvvdk-platform::FileInspection` holds one inspected `FileState` and its live
`BorrowedFd`. Its fields are private; construction performs fstat and F_GETFL.
The exact descriptor can be checked with `is_for`. A different descriptor for
the same inode is deliberately not interchangeable: access modes may differ.

`LinuxFdBackend::copy_endpoint_from_inspection` is an additive hook, callable
for sized backends that also implement `BlockDevice`. Keeping this hook outside
the vtable preserves existing `dyn LinuxFdBackend` use and FD-only backends.
Its default calls the existing
logical `copy_endpoint`, preserving custom-backend checks. It does not infer
logical identity or grant logical capabilities from FD access.

The local backend checks that the inspection belongs to its primary descriptor,
then derives size/identity and intersects actual access with its own capability
restrictions. Its portable `copy_endpoint` takes a fresh inspection and uses the
same conversion. RAW preflight takes one inspection per backend and uses it for
both logical and physical endpoint checks. Custom backends may retain independent
metadata work through the default hook. Overrides are responsible for preserving
their logical contract and accepting only the correct descriptor.

The platform crate now depends on the portable core endpoint types. Core does
not depend on platform or expose Linux types in its disk traits. Existing
LinuxFdBackend implementations need no new method implementation.

Planning and each execution invocation inspect afresh, before payload I/O and
observer callbacks. No inspection is stored in `CopyPlan`. Known backing aliases,
source READ, destination WRITE/FLUSH, current bounds, regular/non-append files,
and native Data identity binding remain required. Unknown identity still cannot
prove that endpoints are distinct and cannot authorize native Data execution.

## Consequences and limits

- Local RAW pair preflight requires two fstat/F_GETFL pairs, down from four.
  Duplicate logical/physical validation remains cheap and intentional.
- This is an observation, not a lock or atomic filesystem snapshot. Size and
  status flags can change even between the inspection's two syscalls. The live
  borrow prevents safe descriptor reuse; callers must keep endpoint state stable
  throughout copying. Never cache an inspection across calls.
- Distinct logical and physical metadata checks may report a different first
  error when multiple endpoint conditions are invalid. Endpoint role/cause and
  rejection before mutation or observation remain the guarantees.
- Low-level native APIs retain their own preflight boundaries. This step does
  not carry inspections through every native layer or prepare a persistent ring.
- Full DataMover success still requires its final flush to succeed. Low-level
  native callers still own flush. Neither contract claims hardware persistence,
  rollback, or a consistent snapshot of concurrently modified source contents.

See the [R1.3 measurements](../benchmark-results/2026-09-28-r13/README.md).
