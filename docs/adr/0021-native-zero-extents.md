# ADR-0021: Native Zero Extent Handling

## Status

Accepted

## Context

M19B introduced extent-aware native execution but only Data extents
could be executed natively.

Zero extents represent logical zero data and therefore do not require
source reads.

Treating Zero as ordinary Data would perform unnecessary source I/O and
discard semantic information supplied by the source backend.

## Decision

Native extent execution handles Zero separately from Data.

For a Zero extent:

1. use `write_zero_at` when the destination advertises `WRITE_ZERO`
2. otherwise write zero-filled blocks through the portable destination
   interface

Zero extents do not enter the io_uring source-read pipeline.

Native statistics separately account for logical zero processing and
physical fallback writes.

Automatic native execution accepts plans containing Data and Zero.

Hole remains unsupported by native execution at this stage.

## Execution model

```text
Data
  |
  v
io_uring read/write


Zero
  |
  +-- WRITE_ZERO
  |      |
  |      v
  |  write_zero_at
  |
  +-- fallback
         |
         v
   zero-filled writes


Hole
  |
  v
unsupported

```
