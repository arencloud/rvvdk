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