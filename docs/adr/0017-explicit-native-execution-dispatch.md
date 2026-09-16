# ADR-0017: Explicit Native Execution Dispatch

## Status

Accepted

## Context

DataMover historically executes through the portable `VirtualDisk`
interface.

The io_uring implementation requires Linux file descriptors and
platform-specific backend capabilities.

Making every `VirtualDisk` expose a Linux file descriptor would break
the portability of the core storage abstraction.

Automatically selecting io_uring could also silently change execution
semantics for existing DataMover callers.

## Decision

DataMover stores an explicit `ExecutionStrategy`.

`Threaded` remains the default strategy.

Linux native execution is exposed through a capability-constrained
`copy_native` operation.

Both source and destination must implement `LinuxFdBackend`.

The io_uring strategy uses:

- backend file descriptors
- backend alignment requirements
- DataMover copy options
- `IoUringExecutionOptions`

The existing generic `copy` operation remains unchanged.

No silent fallback between execution strategies is performed in this
stage.

## Execution paths

```text
Portable:

DataMover
    |
    v
copy
    |
    v
VirtualDisk
    |
    v
Threaded execution


Linux native:

DataMover
    |
    v
copy_native
    |
    v
LinuxFdBackend
    |
    v
compatibility evaluation
    |
    v
io_uring execution
```
