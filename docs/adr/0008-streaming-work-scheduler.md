# ADR-0008: Bounded Streaming Work Scheduler

## Status

Accepted

## Context

The first concurrent DataMover converted the complete source extent map
into a `Vec<WorkItem>` before starting execution.

The number of work items grows approximately with:

```text
logical data size / block size
```
