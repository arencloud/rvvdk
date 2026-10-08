# ADR-0067: Measure disjoint local costs with test-only instrumentation

Status: Accepted for R6.1c.5p measurement tools; no production tuning decision.

## Context

One live pipeline observation spent most of its time in local conversion/admission
and publication. Whole-stage timing alone cannot distinguish native decoding, hashing,
readback and durability work. Profiling adds overhead, and the smaller workstation
fixtures can use a different SHA backend and cache regime from the one-vCPU lab VM.

## Decision

Keep live repetitions on the frozen executable and bind each report to its hash and
unchanged logical source. Qualify complete runs with independent comparisons and
checked cleanup. Require three initial observations and three further observations
when the documented 5% investigation threshold triggers; retain unresolved variance.

Introduce only test-build profiling wrappers around existing expressions. Explicit
thread-local sessions collect disjoint wall/thread-CPU regions with byte/call counts;
reject nesting and preserve cleanup on unwind. Production builds omit every added
clock/counter. Do not expose a production option to bypass a check or sync barrier.

Use the same release test executable for alternating instrumented/disabled controls.
Disabled controls retain inactive thread-local checks; their ratio measures added
recording overhead, not total test-executable overhead relative to production.
Validate exact verification-work accounting and independent full authored RAW bytes.
Separate stored/deflated encodings and logical-capacity scaling with constant data
size. Record affinity, CPU features, working-set limits, filesystem, source/binary
identity, all samples and profiling overhead; repeat adverse/variable observations.

## Consequences

The result identifies costs within controlled local workloads without attributing a
before/after implementation speedup. Copy and journal regions include flush/sync,
while other explicit syncs are separate. Residual work and profiler bookkeeping stay
visible. Thread CPU is valid here only for serial copy; process CPU/RSS have broader
boundaries. Small warm fixtures cannot prove the private live disk's exact breakdown.

Measurements may justify a later correctness-preserving candidate. Hash equivalence,
source identity, full readback, durable ordering, cancellation and sparse allocation
remain requirements. Uncertain publication/recovery is a separate capability design.

[Contract](../pipeline-performance-profiling.md),
[evidence](../benchmark-results/2026-10-07-r61c5p/README.md).
