# ADR-0016: Linux Backend Capability Boundary

## Status

Accepted

## Context

The io_uring execution path requires Linux file descriptors and
backend-specific alignment information.

The generic `VirtualDisk` abstraction is intended to support storage
backends that may not expose Linux file descriptors.

Adding `RawFd` directly to `VirtualDisk` would couple the portable disk
model to Linux and to file-descriptor-based storage.

Placing the capability interface in `rvvdk-datamover` would also create
an undesirable dependency from backend crates such as `rvvdk-local`
back into the DataMover implementation.

## Decision

Platform-specific backend contracts are placed in a separate
`rvvdk-platform` crate.

On Linux, `rvvdk-platform` defines:

- `LinuxFdBackend`
- `LinuxFdCapabilities`

Backend implementations such as `LocalFileBlockDevice` implement this
contract.

Execution layers such as `rvvdk-datamover` consume the contract.

The generic `rvvdk-core` disk abstractions remain platform independent.

## Dependency direction

```text
             rvvdk-core

          rvvdk-platform
             /       \
            /         \
           v           v
    rvvdk-local   rvvdk-datamover
```