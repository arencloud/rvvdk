# ADR-0020: Native Extent-Aware Execution

## Status

Accepted

## Context

The initial io_uring DataMover integration copied the complete logical
disk range as dense data.

The portable DataMover already operates on an extent model containing:

- Data
- Zero
- Hole

Treating the entire disk as dense in the native path would discard
those semantics and could cause unnecessary reads and writes.

Automatic fallback also cannot safely occur after native execution has
already modified part of the destination.

## Decision

Native execution consumes a validated `NativeExtentPlan`.

Extent validation is shared with the portable DataMover.

At the M19B stage:

- Data extents are copied using the io_uring range-copy pipeline.
- Zero extents are explicitly unsupported by native execution.
- Hole extents are explicitly unsupported by native execution.

Explicit io_uring execution returns an error when an unsupported
extent kind is encountered.

Automatic execution evaluates the complete plan before native I/O
begins.

Auto selects io_uring only when both:

```text
backend pair is native compatible
AND
extent plan is Data-only
```