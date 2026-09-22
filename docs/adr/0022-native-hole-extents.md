# ADR-0022: Native Hole Extent Handling

## Status

Accepted

## Context

M19B introduced native Data extent execution.

M19C added Zero extent semantics.

Hole remained the final unsupported VirtualDisk extent kind in the
native execution path.

A Hole represents a logically zero range whose sparse allocation
semantics should be preserved when the destination supports an
appropriate operation.

The portable DataMover already defines a priority for Hole handling:

1. discard
2. write zero
3. ordinary zero-filled writes

Using different semantics in native execution would make results depend
on execution strategy.

## Decision

Native extent execution applies the same Hole policy as the portable
DataMover.

For each Hole extent:

```text
DISCARD available
        |
       yes
        |
        v
    discard range

        no
        |
        v

WRITE_ZERO available
        |
       yes
        |
        v
   write zero range

        no
        |
        v

zero-filled write fallback
```
