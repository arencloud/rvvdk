# ADR-0042: Admit sparse parent metadata before logical fallback

Date: 2026-09-30. Status: Accepted.

The base sparse reader interprets unallocated grains as zero. Admitting parent
syntax through that same public type would silently return wrong child bytes.
Parent resolution also introduces cycles, namespace choices and cumulative work.

Introduce a separate `SparseLayerDescriptor` and metadata-only `SparseChain`.
Keep base parsing and CLI behavior intact. Resolve parents only through an explicit
caller policy returning owned descriptor/backing namespaces. Require known object
identities, matching adjacent CIDs/capacities and a terminal base; reject repeated
objects and shared backing handles. Admit all descriptors and maps under per-layer
and aggregate limits, and reobserve every retained source before success.

Default ceilings are 16 layers, 128 extents, 8 MiB outer descriptors, 128 MiB
conservative reservation and 256 MiB acquisition payload. Released scratch remains
charged. Parsing/resolver/allocator overhead and caller source storage are excluded;
these are neither RSS nor time limits. Source quiescence is the caller's obligation.

This permits metadata validation without exposing a misleading VirtualDisk. Logical
fallback, destination alias checks and CLI lifecycle support will be separate
qualified increments. The initial supported chain is sparse-only; broader parents
require their own contract. See [parent-chain contract](../vmdk-parent-chain.md) and
[R5.6 evidence](../benchmark-results/2026-09-30-r56/README.md).
