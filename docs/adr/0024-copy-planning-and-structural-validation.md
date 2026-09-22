# ADR-0024: Copy Planning and Structural Validation

## Status

Accepted

## Context

RVVDK originally combined planning and execution inside DataMover.

A copy operation performed several responsibilities as part of the same
execution path:

1. inspect source geometry
2. discover source extents
3. validate the extent map
4. inspect source and destination backend capabilities
5. select an execution backend
6. determine execution alignment
7. execute the copy

This model was sufficient while RVVDK exposed only immediate copy
operations.

As the execution subsystem gained multiple backends and complete extent
semantics, planning became useful independently of execution.

A caller may need to inspect a migration before starting it.

Examples include:

- migration preflight
- dry-run reporting
- backend selection visibility
- sparse disk analysis
- policy decisions
- progress estimation
- migration orchestration
- future resumable execution

M20 therefore separates copy planning from copy execution.

## Decision

RVVDK introduces an immutable `CopyPlan` representing the structural
execution plan for a virtual disk copy.

The primary workflow becomes:

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
        |
        +----------+
        |          |
        v          v
    Threaded    IoUring
```
