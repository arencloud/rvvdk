# ADR-0018: Observable Execution Selection

## Status

Accepted

## Context

rvvdk supports multiple execution implementations.

Automatic execution selection can improve usability, but silent
fallback makes performance and runtime behavior difficult to reason
about.

A caller requesting io_uring should be able to distinguish successful
io_uring execution from a portable fallback.

## Decision

Execution strategy and execution result are represented separately.

Strategies include:

- `Threaded`
- `IoUring`
- `Auto`

Execution results identify the backend that actually executed the copy.

Native execution returns a `NativeCopyReport` containing:

- `ExecutionBackend`
- `NativeCopyStats`

Explicit `IoUring` selection does not silently fall back.

`Auto` may select io_uring for compatible native backend pairs.

Cross-capability fallback to the generic threaded path is deferred until
the type system can represent both required capability sets safely.

## Consequences

### Positive

Execution decisions are observable.

Explicit strategy selection remains deterministic.

Performance diagnostics can identify the actual execution backend.

Future automatic policy can be added without hiding fallback.

### Negative

The native and generic copy APIs remain separate temporarily.

`Auto` does not yet provide universal fallback.

Additional result metadata is exposed to callers.

## Future work

A unified dispatch interface may support:

```text
Auto
 |
 +-- native compatible
 |      -> io_uring
 |
 +-- otherwise
        -> threaded VirtualDisk
```


## R1.4 follow-up

CopyPlan now exposes planning-time requested strategy, selected backend, and a
structured selection reason. Reports continue to identify the backend actually
executed. Selection is not a runtime readiness guarantee, and a later executing
mover may have different native options. See
[ADR-0027](0027-portable-planning.md#r14--logical-intent-and-invocation-preparation-2026-09-28)
for the current preparation boundary and unchanged Auto limitations.
