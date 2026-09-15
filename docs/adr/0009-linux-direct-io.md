# ADR-0009: Linux Direct I/O Backend

## Status

Accepted

## Context

The initial local-file backend uses normal buffered positional I/O.

Buffered I/O is appropriate for portability and correctness but makes
storage benchmarks heavily dependent on the Linux page cache.

rvvdk also requires a local I/O path suitable for future
high-performance mechanisms such as `io_uring`.

Linux provides `O_DIRECT` for I/O that minimizes normal page-cache
interaction.

Direct I/O introduces alignment requirements that do not exist for the
buffered path.

## Decision

`LocalFileBlockDevice` will support explicit direct-I/O constructors.

Buffered I/O remains the default.

Direct-I/O files are opened using:

```text
O_DIRECT