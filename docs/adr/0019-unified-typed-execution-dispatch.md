# ADR-0019: Unified Typed Execution Dispatch

## Status

Accepted

## Context

DataMover has separate portable and Linux-native execution paths.

The portable path operates on `VirtualDisk`.

The native io_uring path operates on backend resources implementing
`LinuxFdBackend`.

For raw disks, the logical disk and backend capability layers can be
composed statically because `RawDisk<D>` provides `VirtualDisk`
semantics while its underlying `D` provides backend capabilities.

## Decision

DataMover provides `copy_with_report` for:

```text
RawDisk<S>
RawDisk<D>
```
