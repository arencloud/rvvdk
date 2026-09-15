# rvvdk Architecture

## Purpose

rvvdk provides reusable abstractions and high-performance components
for accessing and moving virtual disk data.

The architecture separates four concerns:

1. logical virtual disks
2. disk formats
3. storage transports
4. data movement

This separation allows disk formats and storage transports to evolve
independently.

## Architectural layers

```text
+--------------------------------------------------+
|                    API / CLI                     |
+--------------------------------------------------+
                         |
                         v
+--------------------------------------------------+
|                   Data Mover                     |
|                                                  |
| scheduling | concurrency | buffers | statistics  |
+--------------------------------------------------+
                         |
                         v
+--------------------------------------------------+
|                  VirtualDisk                     |
+--------------------------------------------------+
                         |
              +----------+----------+
              |          |          |
             RAW        VMDK       QCOW2
              |          |          |
              +----------+----------+
                         |
                         v
+--------------------------------------------------+
|                  BlockDevice                     |
+--------------------------------------------------+
                         |
           +-------------+-------------+
           |             |             |
       Local File       NBD           SAN
```

## BlockDevice

`BlockDevice` represents random-access storage independently of the
logical disk format.

Examples include:

- regular files
- Linux block devices
- NBD exports
- SAN LUNs

The current core interface exposes:

```text
geometry
capabilities
read_at
write_at
flush
```

## I/O execution model

The initial `BlockDevice` API uses synchronous positional I/O.

This is an architectural decision rather than an assumption that rvvdk
will operate sequentially.

Concurrency belongs to the data-movement layer.

A future high-performance backend may expose a separate
completion-oriented interface for Linux mechanisms such as `io_uring`.

The core crate remains independent of any specific asynchronous runtime.

See:

```text
docs/adr/0002-io-execution-model.md
```

## MemoryBlockDevice

`MemoryBlockDevice` is the first concrete implementation of the
`BlockDevice` abstraction.

It stores device contents in process memory and exists primarily for:

- validating the `BlockDevice` contract
- unit testing
- testing higher-level virtual disk implementations
- testing the future DataMover without requiring physical storage
- reproducing I/O edge cases deterministically

The implementation supports configurable capabilities.

A default memory device exposes:

```text
READ
WRITE
FLUSH
```

## Local file backend

`rvvdk-local` provides block-device implementations backed by local
operating-system storage.

The first implementation is:

```text
LocalFileBlockDevice
```

## VirtualDisk

`VirtualDisk` represents a logical virtual disk independently of its
physical storage representation.

The interface currently provides:

```text
geometry
capabilities
read_at
write_at
flush
extents
```

## DataMover

`rvvdk-datamover` provides data movement between `VirtualDisk`
implementations.

The initial implementation is intentionally sequential and
correctness-focused.

```text
Source VirtualDisk
        |
        v
      read
        |
        v
   reusable buffer
        |
        v
      write
        |
        v
Destination VirtualDisk
```

## Exact positional I/O

`BlockDevice` and `VirtualDisk` expose two levels of positional I/O.

Primitive operations:

```text
read_at
write_at
```

## Zero ranges and discard

rvvdk distinguishes between logical zeroing and storage discard.

### write_zero_at

`write_zero_at(offset, length)` guarantees that subsequent reads of
the specified logical range return zeroes.

Backends advertise support using:

```text
WRITE_ZERO
```

## Extent-aware data movement

The DataMover consumes the source `VirtualDisk` extent map rather than
assuming that every logical byte must be read from the source.

Extent processing depends on `ExtentKind`.

### Data

Data extents are transferred using normal exact positional I/O:

```text
source read_exact_at
        |
        v
      buffer
        |
        v
destination write_all_at
```

