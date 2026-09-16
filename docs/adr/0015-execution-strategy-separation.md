# ADR-0015: Separate Execution Strategy from Storage Abstractions

## Status

Accepted

## Context

rvvdk supports portable synchronous and threaded data movement and now
also contains a Linux io_uring execution path.

io_uring operates on Linux-specific resources such as file descriptors.

The generic `VirtualDisk` abstraction may represent storage backends
that do not expose file descriptors.

Examples include future remote, VMware, network, compressed, or
protocol-specific backends.

Adding Linux-specific methods to `VirtualDisk` would couple the core
storage abstraction to one execution mechanism.

## Decision

Execution policy is represented separately from storage semantics.

The initial execution strategies are:

```text
Threaded
IoUring
```
