# Implementation tracking

Started: 2026-09-28. This is the persistent index of work performed against the
[roadmap](roadmap.md), with performance evidence governed by
[the benchmark policy](benchmarks.md#performance-policy--adopted-2026-09-28).

## Working rules

- User instruction (2026-09-28): commit after each completed implementation step
  without requesting confirmation again. Keep commits focused and include the
  relevant tests, benchmark evidence, and roadmap/log updates. Commit only project
  files needed for that step; exclude generated builds and unrelated local files.
  Record unresolved performance qualification explicitly. This authorizes local
  commits; publishing remains a separate action.
- Give each implementation package a stable roadmap ID, with child IDs for smaller
  steps. Record its starting state before editing and its results before closing it.
- Use `Planned`, `In progress`, `Implemented — validation pending`, `Complete`, or
  `Blocked`. An implementation is not Complete until its correctness checks and
  performance disposition are recorded.
- Link exact source revisions, tests, benchmark runs, and ADRs. For uncommitted work,
  mark the commit pending and capture the source fingerprint with benchmark evidence.
- Record failed experiments and accepted tradeoffs as well as successful changes.
  Keep enough detail to explain why a tuning decision was made.
- Update the roadmap checkboxes alongside this log. Finishing documentation does
  not complete an implementation milestone. Do not silently turn an untested
  assumption into a supported capability.

## Work index

| ID | Status | Scope / evidence | Performance disposition |
|---|---|---|---|
| DOC.1 | Complete | [Dated project review](project-review-2026-09-28.md) and [roadmap](roadmap.md); review session: 194 tests passed, fmt/Clippy passed; targeted defects recorded | N/A — analysis/documentation; historical benchmarks reviewed, no new throughput runs |
| DOC.2 | Complete | README presentation and accurate capability status; Rust example executed, local links and whitespace checked | N/A — documentation only |
| DOC.3 | Complete | Benchmark policy, this tracking index, and V0 lab/license requirements | N/A — documentation only; PERF.0 remains pending |
| WIP.OBS | In progress — checkpoint committed | `c951547` records the pre-existing observer/progress modules, exports, mover integration, and sparse progress test; F03 validation bypass is now fixed by R0.1; other progress lifecycle work remains open | [R0.1 comparison](benchmark-results/2026-09-28-r01/README.md) recorded; broader qualification remains pending |
| PERF.0 | In progress | Flush parity corrected; isolated builds, source patch/hashes, and repeated Btrfs planning/observer comparisons captured | Targeted evidence recorded; controlled-runner repeat, sustained direct I/O, and broader matrix remain pending |
| R0.1 | Complete | Shared structural validation before execution/notification; 8 regression cases; [ADR-0025](adr/0025-shared-plan-validation.md); committed with this step record | Required correctness fix accepted; [comparison and noise/overhead limits](benchmark-results/2026-09-28-r01/README.md) recorded; performance qualification provisional |
| R0.2 | Complete | Worker failure termination and first-recorded-error preservation; 11 regressions; ADR-0008 shutdown contract; committed with this record | [Comparison and failure latency](benchmark-results/2026-09-28-r02/README.md) accepted for this fix; initial memory increases investigated with affinity repeat; broader PERF.0 qualification remains open |
| R0.3 | Planned | io_uring buffer/FD lifetime and error-path repair | Pending — native throughput/resource comparisons on safe paths |
| R0.4 | Planned | Public low-level configuration and preflight validation | Pending — relevant setup/copy overhead |
| R0.5 | Planned | Access/durability preflight and endpoint identity checks | Pending — preflight overhead |
| V0 | Planned | Independent VMware access feasibility; licensed/evaluation host needed for representative API workflows | Pending — first transport baseline follows functional proof |

R1–R9 remain planned in the roadmap. Add their individual work packages here as
they are prepared; no implementation completion is implied by their omission.

## Per-step record template

Copy this template for each new implementation step. Keep records append-only
once complete, adding a dated correction if later evidence changes a conclusion.

```text
ID / title:
Date / status:
Problem and expected behavior:
Baseline commit + dirty/untracked source fingerprint:
Candidate commit + source fingerprint (or uncommitted):
Files / public API / ADR changes:
Implementation steps and decisions:
Correctness commands + results:
Benchmark workload IDs + exact commands:
Benchmark baseline and candidate artifact links:
Results: time, throughput, variance, available CPU/RSS/latency metrics:
Performance disposition: accepted / investigate / justified tradeoff / N/A:
Remaining risks and follow-up IDs:
Roadmap checkbox updated:
```

## Repository checkpoint — 2026-09-28

- Implementation checkpoint: `c951547` (`Add initial copy progress observers`).
  This preserves existing work; it does not complete R0.1 or qualify performance.
- Documentation checkpoint: this log, roadmap, review, benchmark policy, and README
  are committed together. Use this file's Git history to locate that revision.
- Validation rerun before committing: 194 tests passed, 0 failed, 0 ignored;
  formatting, strict Clippy across all targets, whitespace checks, and local
  documentation link checks passed.
- Performance: pending PERF.0. No new throughput measurements or tuning claims.
- Excluded artifact: `benchmark-m11.txt` remains on the development machine and is
  locally ignored through `.git/info/exclude`. It is historical output without
  sufficient environment/provenance to serve as the project's benchmark baseline.
- No implementation behavior was changed during commit preparation. The observer
  validation defect and other review findings remain scheduled in the roadmap.
- ESXi trial coordination: notify the user when V0 code/tests are ready; the user
  will then install the 60-day trial. Do not start the trial window for local work.

## R0.1 / PERF.0 — 2026-09-28

**Measured baseline:** `b579dfb`. **Candidate:** committed with this step record; the
[reconstructible source patch](benchmark-results/2026-09-28-r01/candidate.patch)
and [source/binary hashes](benchmark-results/2026-09-28-r01/environment.json)
identify the measured implementation.

Commit split: `9d45821` contains the benchmark harness and flush timing correction.
The following commit contains the validation fix, regressions, ADR, results, and
tracking updates; use this record's Git history to locate it. Measurements were
taken before committing, against the source state preserved in the patch; the
commit split does not change that measured source.

Implementation steps:

1. Added regressions for block-size/alignment mismatches, changed source size,
   insufficient destination capacity, stale Hole maps, and invalid concurrent
   observer notification. All seven failed on the original code; one reproduced
   a buffer-bound panic. Added a native observer rejection case as well.
2. Extracted shared structural plan validation. Both public execution paths now
   validate before copying or emitting progress. Private dispatch avoids a second
   validation scan in concurrent/native observed execution. The sequential copy
   loops remain separate until R1's semantic consolidation.
3. Corrected the native direct benchmark's flush boundary. This is a harness fix;
   historical direct-I/O throughput has not been requalified.
4. Added a focused planning/progress benchmark with dense and fragmented inputs,
   destination reset, identical flush timing, and output verification per iteration.
5. Discarded two failed tmpfs fixture attempts and a shared-build-directory attempt;
   used isolated baseline/candidate executables for accepted comparisons.
6. Recorded three baseline/candidate runs on Btrfs. The fragmented callback median
   crossed the 5% investigation threshold, so repeated the fragmented matrix with
   longer sampling and reversed order. Both sets and all limitations are retained.

Correctness validation:

```text
cargo test --workspace                                      202 passed, 0 failed
cargo fmt --all -- --check                                  passed
cargo clippy --workspace --all-targets -- -D warnings        passed
```

Performance disposition: accept the mandatory validation fix, with provisional
performance qualification. The callback's aggregate follow-up median remained
about 6% slower, but independent runs varied widely and paired changes did not
show a consistent >5% increase. The unobserved control also moved. This does not
establish a general regression-free bound. Follow up on a controlled runner under
PERF.0; no engine tuning defaults were changed. See the
[full report](benchmark-results/2026-09-28-r01/README.md), including all raw samples.

Roadmap: R0.1 checked off; PERF.0 remains in progress. R0.2 worker shutdown and
R0.3 native lifetime repair are not changed by this step.

## R0.2 — 2026-09-28

**Baseline:** `3302c1b`, initially clean. **Candidate:** the
[source patch](benchmark-results/2026-09-28-r02/candidate.patch) and
[environment/source hashes](benchmark-results/2026-09-28-r02/environment.json)
identify the measured implementation. The implementation, tests, benchmark harness,
evidence, and tracking are committed together; use this record's Git history to
locate the revision.

Implementation steps:

1. Added bounded subprocess reproductions. Six of the first seven tests timed
   out on the original runtime; the producer-error/idle-worker case already
   passed. Forced full queues behind read/write/zero/discard failures.
2. Released the coordinator's receiver before production. Added first-recorded
   error storage and an atomic stop flag checked at worker-item boundaries.
   Producer and worker failures share the same recording path; all scoped
   workers join before returning. Successful production still drains the queue.
3. Extended coverage to healthy-peer stopping, buffer return under contention,
   public-copy error preservation without flush, panic propagation, first-error
   ordering, and successful draining. Eleven new tests in total.
4. Updated ADR-0008 and architecture with ownership, wakeup, error ordering, and
   cooperative shutdown limits. Public APIs and execution defaults are unchanged.
5. Added memory/file scheduler comparisons and synthetic failure-to-return
   latency measurements. Corrected a read-only fixture flush in the first harness
   attempt, then rebuilt both variants identically with separate target paths.
6. Ran three successful-copy comparisons and three candidate failure runs.
   Memory results above the 5% investigation threshold triggered a longer repeat
   with five physical cores selected through CPU affinity; all results are kept.

Correctness: `cargo test --workspace` passed 213 tests; strict all-target Clippy
and formatting passed. The runtime is unchanged since that test run. The
[benchmark report](benchmark-results/2026-09-28-r02/README.md) records exact
commands, raw samples, the failed harness attempt, and performance disposition.

Performance disposition: accept this correctness fix with targeted evidence.
File-copy aggregate medians changed by −0.39% to +3.58%. Initial unpinned memory
increases reached +23.03%; a longer repeat with fixed CPU affinity ranged from
−3.63% to +0.30%, so those increases did not persist under that configuration.
Synthetic first-error-to-return medians were 17.0–18.3 µs, including worker joining.
This is not a guarantee for real backend stalls or a general regression-free
bound. Dedicated-runner and broader storage qualification remain under PERF.0.
No execution defaults were tuned. R0.2 is checked off in the roadmap.

Remaining limits: dispatched synchronous I/O can finish after cancellation and
must return for shutdown to complete. This is not a public cancellation API,
backend timeout mechanism, or partial-copy report. R0.3 native lifetimes remain
unrepaired by this work.

## Next session

Implement R0.3: io_uring buffer/FD lifetime and error-path repair, including a
safety argument and fault-injection tests. Continue PERF.0 qualification on
controlled storage, including the outstanding fragmented observer comparison,
corrected direct-I/O benchmark, and broader scheduler workloads. ESXi is still
unnecessary for these local steps; request the trial only when V0 is ready.
