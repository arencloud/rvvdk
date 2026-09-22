# ADR-0023: Execution Semantic Parity

## Status

Accepted

## Context

RVVDK provides multiple execution strategies for moving virtual disk
contents.

The portable execution path uses the threaded DataMover implementation.

On Linux, compatible backends may also use the native io_uring execution
path.

Both execution engines operate on the same logical VirtualDisk extent
model:

- Data
- Zero
- Hole

M19A through M19D progressively extended native execution to understand
the complete extent model.

Native execution now supports:

```text
Data
  -> io_uring read/write pipeline

Zero
  -> WRITE_ZERO when available
  -> ordinary zero-filled write fallback otherwise

Hole
  -> DISCARD when available
  -> WRITE_ZERO when DISCARD is unavailable
  -> ordinary zero-filled write fallback otherwise
```
