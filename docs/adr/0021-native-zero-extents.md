# ADR-0021: Native Zero Extent Handling

## Status

Accepted

## Context

M19B introduced extent-aware native execution but supported only Data
extents.

Zero extents contain logical zero data and do not require source reads.

Copying them through the Data io_uring pipeline would perform
unnecessary I/O and lose the semantic information supplied by the
source extent map.

## Decision

Native extent execution handles Zero extents using destination
semantics.

If the destination advertises `WRITE_ZERO`, DataMover invokes
`write_zero_at`.

Otherwise, DataMover performs a zero-filled write fallback.

Zero extents do not generate source reads.

The native extent statistics separately account for logical zero
processing and physical fallback writes.

Auto execution considers Data and Zero extents native-compatible.

Hole extents remain unsupported until the next milestone.

## Consequences

### Positive

Zero regions no longer require source reads.

Destination-native zero operations can be used when available.

The native path preserves more of the logical extent model.

Auto can remain on native execution for Data+Zero workloads.

### Negative

Native execution is now a hybrid executor: Data operations use
io_uring while Zero operations may use the portable destination API.

Hole semantics remain incomplete.

## Future work

M19D will add Hole processing using the same policy as the portable
DataMover:

1. discard when supported
2. otherwise write-zero when supported
3. otherwise zero-write fallback