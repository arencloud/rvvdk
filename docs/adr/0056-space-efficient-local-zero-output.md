# ADR-0056: Space-efficient local file zero output

Status: accepted (R5.12p). Extends the R2.2 local sparse output contract.

## Context

R5.12 converted the retained 30 GiB export correctly, but logical Zero work used
`ZERO_RANGE`, allocating space for missing source grains. The small XFS runner
failed with ENOSPC; Btrfs allocated all 30 GiB despite only about 3.5 GiB of Data.
Neither compressed size nor logical Data bytes promised output allocation.

## Decision

For writable Linux regular files, `write_zero_at` now tries
`PUNCH_HOLE | KEEP_SIZE` first, then `ZERO_RANGE | KEEP_SIZE` when punching is
explicitly unsupported, then bounded positional zero writes when both kernel
modes are unsupported. `discard` continues to try punching then bounded writes.

The [Linux fallocate contract](https://man7.org/linux/man-pages/man2/fallocate.2.html)
guarantees zero reads inside a successfully punched range, including partial
filesystem blocks, and preserves size with KEEP_SIZE. This is a backend-specific
implementation of existing zero semantics. It does not infer zero-reading discard
for arbitrary devices, change source Zero/Hole labels, or skip writes merely
because the caller says an output is new. Existing nonzero destinations work too.

One cooperative access guard spans all attempts and fallback writes. Exact byte
ranges, read-only/append/range/size checks and buffered aliases are unchanged.
Only EOPNOTSUPP and ENOSYS advance to a fallback and set per-open unsupported bits.
Zero and discard share the punch bit; zero-range has its own bit. EINTR retries
that mode; all other errors propagate immediately, including ENOSPC and partial
effects. The immutable 64 KiB fallback buffer does not grow with workers.

Copy counters retain requested-operation meaning: successful `write_zero_at`
contributes bytes_zeroed even when implemented by punching. Physical allocation
must be measured separately. No CLI flag or output schema changes.

## Consequences and qualification

Punching may reclaim whole blocks and preserves sparse output on qualified
filesystems. It does not reserve space for later writes; WRITE_ZERO never promised
such a reservation. Partial blocks, unsupported filesystems and fallback writes
may still allocate storage. No universal output-space bound is advertised.

[Contract](../local-sparse-output.md),
[tests, XFS/Btrfs allocation and benchmark evidence](../benchmark-results/2026-10-02-r512p/README.md).
Retain shared-host timing variation and adverse controls without attributing them
to code or noise without evidence. The production artifact/ownership gate remains
R6.1; this change does not alter VMware access or export identity.
