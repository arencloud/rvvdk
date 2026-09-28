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

The engine now accepts only owned buffer operations. The former borrowed-slice
compatibility methods were removed in R0.3 because error returns could end the
borrow before kernel access ended. Native range/extent APIs take `BorrowedFd`;
`LinuxFdBackend` requires `AsFd`.

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
    +-- shared owned IoUringFile
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

Both owners enter the operation table before SQE publication. A matching final
CQE permits release, including a negative operation result. Interrupted waits
retry. Other errors stop new submissions while retaining pending resources.
Explicit `shutdown()` drains before closing the ring; copy functions call it
before returning. Drop uses the same protocol.

If cleanup cannot confirm completion, outstanding guards and FD references are
permanently retained and explicit shutdown reports their count. Ring close alone
is not treated as proof that buffer reuse is safe. This exceptional fallback can
also retain the guards' underlying pools; normal completion releases resources.
Shutdown can block on active synchronous I/O and does not provide rollback.
See [ADR-0013](adr/0013-io-uring-buffer-ownership.md) for the safety argument,
API migration, and limitations.

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

## Sparse-aware io_uring extent planning

The io_uring execution path is being extended from dense whole-disk
copying to execution based on the same logical extent model used by
the portable DataMover.

The source extent map is converted into a validated
`NativeExtentPlan` before native execution begins.

```text
VirtualDisk::extents()
        |
        v
validate_extents()
        |
        v
NativeExtentPlan
        |
        +-- Data
        +-- Zero
        +-- Hole
```
### Local backend extent reporting

`LocalFileBlockDevice` currently reports file-backed source ranges as
Data and Hole extents.

It does not currently synthesize Zero extents for allocated ranges whose
contents happen to be zero.

Native Zero handling is therefore validated at the generic
`NativeExtentPlan` execution layer.

The capability remains relevant to VirtualDisk backends that can
distinguish logical Zero extents from Data and Hole extents.

## Native entry-point validation (R0.4)

The exported native range and extent-copy functions validate block size and
alignment before allocation, io_uring setup, FD duplication, or destination
callbacks. This applies to empty ranges/plans and Zero/Hole-only plans too:

- Block size must be nonzero and fit the SQE's `u32` length field.
- Alignment must be a nonzero power of two, with a representable allocation
  layout for the configured block size.
- File offsets and the exclusive range end must fit the nonnegative `i64`
  domain. `RangeOverflow` also reports failure to represent a native file range.
  The owned engine validates each request before publishing its SQE; in
  particular, `u64::MAX` cannot become io_uring's current-file-position sentinel.
- The data-only `copy_extent_plan` scans for unsupported Zero/Hole extents
  before copying any Data extent. The destination-aware variant accepts all
  three kinds and validates the whole file range before calling the backend.

Queue depth and read window remain private, constructor-validated options; raw
queue depth arguments reject zero. Buffer capacity and request width checks in
the owned engine also precede publication. Invalid requests return their buffer
to the pool and leave a healthy engine available for subsequent valid requests.

This is configuration validation, not transactional execution. Backend failures,
allocation failure, or ring creation failure can still occur after earlier
extents have completed. Endpoint access, capacity, identity, and durability
preflight are R0.5; direct-I/O alignment/tail compatibility and logical discard
semantics remain R2 work. The allocation layout check does not impose an
aggregate memory budget or guarantee enough available memory.

## Native Zero extent execution

The native extent executor supports Data and Zero extents while
preserving their different storage semantics.

```text
NativeExtentPlan
        |
        +-- Data
        |     |
        |     v
        |  io_uring read/write pipeline
        |
        +-- Zero
        |     |
        |     +-- WRITE_ZERO available
        |     |       |
        |     |       v
        |     |  write_zero_at
        |     |
        |     +-- WRITE_ZERO unavailable
        |             |
        |             v
        |       zero-write fallback
        |
        +-- Hole
              |
              v
        unsupported
```
## Native Hole extent execution

Native extent execution supports all VirtualDisk extent kinds:

```text
NativeExtentPlan
        |
        +-- Data
        |     |
        |     v
        |  io_uring read/write
        |
        +-- Zero
        |     |
        |     +-- WRITE_ZERO
        |     |
        |     +-- zero-write fallback
        |
        +-- Hole
              |
              +-- DISCARD
              |
              +-- WRITE_ZERO
              |
              +-- zero-write fallback
```
## Execution semantic parity

Threaded and native execution are validated against the same
deterministic extent model:

```text
Data -> Zero -> Hole -> Data
```

## Copy planning

RVVDK separates copy planning from execution.

The destination-aware workflow is:

```text
source + destination
        |
        v
plan_with_destination()
        |
        v
     CopyPlan
        |
        v
execute_plan()
```

Both `execute_plan` and `execute_plan_with_observer` now share structural plan
validation before copying or notifying observers. Private validated dispatch
avoids repeating that scan for concurrent/native observed execution. Native
compatibility checks remain inside the native executor. See
[ADR-0025](adr/0025-shared-plan-validation.md).

## Concurrent worker shutdown

The coordinator releases its work-queue receiver before producing items. On a
worker or producer error, shared state retains the first recorded cause and stops
workers at work-item boundaries. Receiver destruction wakes a blocked producer;
sender destruction wakes idle workers. All scoped workers join before returning.
Successful copies still drain queued work and flush the destination.

Already-dispatched work can finish, and shutdown cannot interrupt a blocked
synchronous backend call. Public cancellation and partial-result reporting remain
future work. The contract and tests are documented in
[ADR-0008](adr/0008-streaming-work-scheduler.md#shutdown-contract--r02-2026-09-28).
