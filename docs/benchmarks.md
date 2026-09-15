# rvvdk Benchmarks

## Purpose

rvvdk uses benchmarks to validate performance changes rather than
assuming that architectural changes improve throughput.

The initial benchmark establishes a baseline for the sequential
DataMover.

## Current benchmark path

```text
LocalFileBlockDevice
        |
        v
     RawDisk
        |
        v
    DataMover
        |
        v
     RawDisk
        |
        v
LocalFileBlockDevice
```

## Concurrent DataMover baseline

The synchronous concurrent DataMover was benchmarked with a 1 MiB
block size.

The initial Milestone 12 baseline on the development system was:

| Workers | Throughput |
|---:|---:|
| 1 | 4.94 GiB/s |
| 2 | 8.36 GiB/s |
| 4 | 8.06 GiB/s |
| 8 | 6.27 GiB/s |

For this cached local-file workload, two workers provided the highest
measured throughput.

These results are environment-specific and must not be interpreted as
universal optimal concurrency settings.

## Streaming scheduler

The Milestone 13 scheduler replaces complete work-plan materialization
with incremental planning and a bounded producer/consumer queue.

```text
Extent
  |
  v
ExtentWorkIter
  |
  v
bounded queue
  |
  +---- worker
  +---- worker
  +---- worker
```

### Milestone 13 results

The bounded streaming scheduler produced the following cached
local-file results with a 1 MiB block size:

| Workers | Throughput |
|---:|---:|
| 1 | 5.02 GiB/s |
| 2 | 8.01 GiB/s |
| 4 | 8.10 GiB/s |
| 8 | 6.12 GiB/s |

The previous Milestone 12 scheduler produced approximately:

| Workers | Throughput |
|---:|---:|
| 1 | 4.94 GiB/s |
| 2 | 8.36 GiB/s |
| 4 | 8.06 GiB/s |
| 8 | 6.27 GiB/s |

The bounded scheduler therefore preserved approximately the same
performance profile while eliminating block-level work-plan memory
growth with disk size.

Queue-capacity testing with two workers produced:

| Queue capacity | Throughput |
|---:|---:|
| 1 | 7.78 GiB/s |
| 2 | 6.91 GiB/s |
| 4 | 7.31 GiB/s |
| 8 | 7.33 GiB/s |
| 16 | 7.10 GiB/s |
| 32 | 7.09 GiB/s |
| 64 | 6.98 GiB/s |
| 128 | 7.28 GiB/s |

No throughput advantage was observed from maintaining a large pending
work queue for this workload.

The default synchronous scheduler queue capacity is therefore kept
small. Queue capacity remains configurable because other storage and
network backends may behave differently.

These measurements are page-cache-heavy local-file benchmarks and are
not measurements of physical storage throughput.

## Direct I/O benchmark

The direct-I/O benchmark exercises:

```text
O_DIRECT source
      |
      v
LocalFileBlockDevice
      |
      v
RawDisk
      |
      v
DataMover
      |
      v
RawDisk
      |
      v
LocalFileBlockDevice
      |
      v
O_DIRECT destination
```

### Milestone 14 direct-I/O results

The Linux direct-I/O backend was verified using `strace`. The local
data file was opened with:

```text
O_RDWR | O_DIRECT | O_CLOEXEC
```

