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

## Linux sparse-file extent discovery

`LocalFileBlockDevice` supports sparse extent discovery on Linux using
`SEEK_DATA` and `SEEK_HOLE`.

The backend advertises:

```text
EXTENTS
SPARSE
```

## Aligned I/O buffers

rvvdk uses an explicit aligned buffer abstraction for DataMover I/O.

The initial implementation is:

```text
AlignedBuffer
```

## Buffer pool

rvvdk provides a bounded reusable buffer pool built from
`AlignedBuffer` allocations.

```text
BufferPool
   |
   +-- AlignedBuffer
   +-- AlignedBuffer
   +-- AlignedBuffer
```


## Concurrent DataMover

rvvdk supports a synchronous multi-worker DataMover execution path.

The source extent map is translated into block-sized work items.

```text
VirtualDisk extents
        |
        v
    Work planner
        |
        v
   bounded work
        |
   +----+----+----+
   |    |    |    |
   v    v    v    v
  W0   W1   W2   Wn
   |    |    |    |
   v    v    v    v
BufferPool buffers
   |    |    |    |
   +----+----+----+
        |
        v
Destination
```

## Streaming work scheduler

The concurrent DataMover does not materialize the complete block-level
work plan before copying.

Each source extent is converted lazily into `WorkItem` values using
`ExtentWorkIter`.

```text
VirtualDisk extents
        |
        v
 ExtentWorkIter
        |
        v
 bounded channel
        |
   +----+----+
   |         |
   v         v
 worker    worker
```

## Linux direct I/O

`LocalFileBlockDevice` supports an optional Linux direct-I/O mode using
`O_DIRECT`.

Direct I/O is explicitly selected when opening a local file and is not
the default behavior.

```text
LocalFileBlockDevice
        |
        +-- buffered
        |
        +-- direct
              |
              v
           O_DIRECT
```

## Hybrid direct-I/O fallback

A direct `LocalFileBlockDevice` maintains two descriptors for the same
logical file:

```text
O_DIRECT descriptor
        |
        +-- aligned requests

buffered descriptor
        |
        +-- unaligned requests
```

## Direct-I/O alignment discovery

Direct-I/O alignment is discovered at runtime rather than assumed to
be universally 4096 bytes.

The local Linux backend represents direct-I/O constraints as:

```text
DirectIoAlignment
    |
    +-- memory_alignment
    |
    +-- offset_alignment
```
Alignment metadata also records how the values were obtained.

```text
DirectIoAlignmentSource

Statx
Fallback
```

## io_uring execution foundation

rvvdk includes a Linux-specific `io_uring` integration layer.

The initial integration provides runtime capability probing only.

```text
DataMover
   |
   +-- sequential engine
   |
   +-- threaded engine
   |
   +-- io_uring foundation
            |
            v
        Linux kernel
```

## io_uring engine lifecycle

The Linux `IoUringEngine` owns an `io_uring` instance and tracks:

```text
queue depth
next user-data identifier
operations in flight
```

## io_uring buffer ownership

io_uring read and write operations contain pointers to userspace
buffers.

Those buffers must remain valid from SQE submission until the
corresponding CQE has been consumed.

rvvdk therefore does not expose raw asynchronous submission using
ordinary borrowed slices as a public API.

Ownership-safe asynchronous operations transfer a `BufferGuard` into
an in-flight operation:

```text
BufferPool
    |
    v
BufferGuard
    |
    v
InFlightOperation
    |
    +-- user_data
    +-- operation kind
    +-- offset
    +-- length
    +-- owned BufferGuard
    |
    v
io_uring SQE
    |
    v
Linux kernel
    |
    v
CQE
    |
    v
CompletedOperation
    |
    v
BufferGuard
```

## Balanced io_uring pipeline scheduling

The initial io_uring copy pipeline correctly maintained buffer
ownership but could produce read/write waves.

For example, with a queue depth of eight, filling the complete queue
with reads could produce:

```text
R R R R R R R R
        |
        v
W W W W W W W W
        |
        v
R R R R R R R R
```
## DataMover execution strategy

DataMover execution policy is represented independently from storage
backend abstractions.

```text
ExecutionStrategy
    |
    +-- Threaded
    |
    +-- IoUring
            |
            +-- queue depth
            +-- read window
```

## Linux backend capability boundary

Linux-specific execution capabilities are separated from the generic
disk abstraction.

The generic `VirtualDisk` interface remains portable and does not
expose Linux file descriptors.

Platform-specific backend contracts are provided by the
`rvvdk-platform` crate.

```text
                    rvvdk-core
                        |
                portable disk model

                 rvvdk-platform
                        |
                 LinuxFdBackend
                    /       \
                   /         \
                  v           v
          rvvdk-local    rvvdk-datamover
               |              |
          implements       consumes
               |              |
               +------+-------+
                      |
                      v
             Linux execution path
```

## DataMover native execution dispatch

DataMover supports explicit selection of an execution strategy while
preserving the existing threaded behavior as the default.

```text
DataMover
    |
    +-- CopyOptions
    |
    +-- ExecutionStrategy
            |
            +-- Threaded
            |
            +-- IoUring
                    |
                    +-- queue depth
                    +-- read window
```

## Execution reporting and automatic native selection

DataMover execution results identify the backend that actually
performed the copy.

```text
NativeCopyReport
    |
    +-- ExecutionBackend
    |
    +-- NativeCopyStats
```

## Unified typed execution dispatch

DataMover provides unified execution dispatch for raw disks whose
underlying block devices expose Linux native execution capabilities.

```text
RawDisk<S>                     RawDisk<D>
    |                              |
    v                              v
BlockDevice +                  BlockDevice +
LinuxFdBackend                LinuxFdBackend
        \                         /
         \                       /
          +---------------------+
                    |
                    v
        DataMover::copy_with_report
                    |
                    v
            ExecutionStrategy
           /        |         \
          /         |          \
   Threaded      IoUring       Auto
      |             |            |
      |             |       capability
      |             |         evaluation
      |             |        /        \
      |             |      yes          no
      |             |       |            |
      v             v       v            v
  portable       io_uring io_uring    threaded
  DataMover
      \             |       |            /
       \            |       |           /
        +-----------+-------+----------+
                    |
                    v
                CopyReport
                    |
              +-----+-----+
              |           |
           backend      CopyStats
```
