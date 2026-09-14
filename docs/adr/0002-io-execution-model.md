# ADR-0002: I/O Execution and Concurrency Model

## Status

Accepted

## Context

rvvdk is intended to provide a high-performance virtual disk data
plane.

The core `BlockDevice` abstraction must support multiple backend types:

- local files
- Linux block devices
- NBD
- SAN
- future VMware-specific transports

Different backends have different I/O execution models.

A local file may use:

- blocking positional I/O
- `pread` / `pwrite`
- `io_uring`

A network transport such as NBD may use:

- asynchronous socket I/O
- multiple outstanding requests
- protocol-level request identifiers

The data mover must also support queue depth greater than one so that
reads and writes can overlap.

Several possible designs were considered.

## Option 1: Async trait methods

Example:

```rust
async fn read_at(
    &self,
    offset: u64,
    buffer: &mut [u8],
) -> Result<usize>;