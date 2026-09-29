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
| DOC.4 | Complete | Reusable benchmark plot generator; six R2.3 charts in SVG/PNG; [guide](../scripts/benchmarks/README.md) | N/A — presentation/tooling only; original samples and performance dispositions preserved |
| DOC.3 | Complete | Benchmark policy, this tracking index, and V0 lab/license requirements | N/A — documentation only; PERF.0 remains pending |
| WIP.OBS | In progress — checkpoint committed | `c951547` records the pre-existing observer/progress modules, exports, mover integration, and sparse progress test; F03 validation bypass is now fixed by R0.1; other progress lifecycle work remains open | [R0.1 comparison](benchmark-results/2026-09-28-r01/README.md) recorded; broader qualification remains pending |
| PERF.0 | In progress | Flush parity corrected; isolated builds, source patch/hashes, and repeated Btrfs planning/observer comparisons captured | Targeted evidence recorded; controlled-runner repeat, sustained direct I/O, and broader matrix remain pending |
| R0.1 | Complete | Shared structural validation before execution/notification; 8 regression cases; [ADR-0025](adr/0025-shared-plan-validation.md); committed with this step record | Required correctness fix accepted; [comparison and noise/overhead limits](benchmark-results/2026-09-28-r01/README.md) recorded; performance qualification provisional |
| R0.2 | Complete | Worker failure termination and first-recorded-error preservation; 11 regressions; ADR-0008 shutdown contract; committed with this record | [Comparison and failure latency](benchmark-results/2026-09-28-r02/README.md) accepted for this fix; initial memory increases investigated with affinity repeat; broader PERF.0 qualification remains open |
| R0.3 | Complete | Owned-only engine, retained descriptors, verified completion/shutdown rules; 18 lifetime/error regressions; ADR-0013 safety argument and API migration; committed with this record | [Native comparison and resource evidence](benchmark-results/2026-09-28-r03/README.md) recorded; mandatory safety fix accepted; performance qualification provisional because native/control timings drifted |
| R0.4 | Complete | Native configuration/range validation and supported-plan precheck | Accepted correctness cost: ~1–2 ns empty-call overhead; copy aggregates below 5%; shared-host qualification provisional |
| R0.5 | Complete | Endpoint capabilities/live capacity, known aliases, native binding, and contextual preflight errors | Required correctness cost accepted: planning +2.08–2.15 µs; final copy aggregates −1.93% to +2.37%; general qualification provisional |
| V0 | Planned | Independent VMware access feasibility; licensed/evaluation host needed for representative API workflows | Pending — first transport baseline follows functional proof |

R1.1–R1.6 and R2.1–R2.6 are complete within their documented scopes; their detailed
records appear below. R3 local CLI and R4.1–R4.4 local FLAT/ZERO VMDK workflows are complete within their
documented contracts; R4.5, later milestones and V0 remain planned. Performance qualification
remains provisional as recorded for each step.

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

## R0.3 — 2026-09-28

**Baseline:** `52ddb14`, initially clean. **Candidate:** the
[source patch](benchmark-results/2026-09-28-r03/candidate.patch) and
[environment/source fingerprints](benchmark-results/2026-09-28-r03/environment.json)
identify this implementation. Code, tests, ADR, benchmark evidence, and tracking
are committed together; use this record's Git history to locate the revision.

Implementation steps:

1. Traced borrowed/owned submission, CQE consumption, unwinding, and teardown.
   Reviewed the pinned io-uring implementation and upstream buffer/cancellation
   contracts; references and assumptions are in ADR-0013.
2. Reproduced the old raw-FD reuse defect: a queued read used `/dev/zero` after
   the original descriptor was closed and reused. Preserved the exact old-API
   reproducer and failing output. No use-after-free experiment was performed.
3. Removed borrowed-slice engine I/O. Operations now retain both BufferGuard and
   shared owned IoUringFile before publishing SQEs. Added typed descriptor APIs
   and migrated repository call sites. Native payload copying remains zero-copy
   between completed reads and submitted writes.
4. Added interrupted-enter retries, explicit in-flight capacity bounds, final-CQE
   validation, and explicit shutdown. Cleanup drains confirmed operations, rejects
   new submissions after errors, and permanently retains unconfirmed owners.
   High-level copy errors preserve the original cause and cleanup-retention count.
5. Added 18 bounded lifetime/fault-injection tests; migrated existing positional
   tests to owned buffers. Covered actual partial submission, errors before/after
   acceptance, waits, CQEs, FD reuse, unwinding, capacity, and shutdown. Public
   pipeline EOF/write-error regressions verify original causes and FD release.
6. Built identical high-level benchmarks against isolated source variants.
   Recorded buffered/direct copy comparisons, threaded controls, per-copy FD
   checks, and whole-process GNU time resource measurements. A noisy direct
   control triggered a longer repeat alongside the native 64 KiB case.

Correctness: 231 workspace tests passed; formatting and strict Clippy across all
targets passed. Safety argument, API migration, and exceptional-retention costs
are recorded in [ADR-0013](adr/0013-io-uring-buffer-ownership.md).

Performance disposition: accept the mandatory safety repair; qualification remains
provisional. Initial native aggregate medians changed by −6.10% to +2.38%, but the
unchanged direct threaded control moved +15.25%. A longer repeat produced +12.64%
for the direct native 64 KiB case and +38.39% for its control. These noisy results
do not establish a general regression bound; retain the full evidence and repeat
under controlled PERF.0 conditions. Every benchmark iteration verified output and
stable FD counts. Process CPU/RSS statistics include harness overhead and unequal
adaptive iteration counts. No execution defaults were tuned. R0.3 is checked off.

After measuring, two public pipeline tests and a stronger live-ID wraparound case
were added without runtime/harness changes. Rebuilding the candidate benchmark
reproduced the measured binary SHA-256 exactly.

Remaining limits: unconfirmed cleanup can retain buffers, pools, and FDs until
process exit; draining can block on stalled I/O and is not rollback. Preflight,
logical identity, public cancellation/deadlines, and sustained-device qualification
remain separate work. The [benchmark report](benchmark-results/2026-09-28-r03/README.md)
records the performance disposition and measurement limits.

## R0.4 — Native entry-point validation

Baseline: `2da7c8f` (clean worktree). Scope: validate exported native copy
configuration and request ranges before I/O; reject unsupported data-only plans
before any destination change. Findings covered: F08 plus native mutation ordering.

Implementation sequence:

1. Audited public entry points. `CopyOptions` already rejects zero blocks and
   invalid alignment; `IoUringExecutionOptions` privately enforces queue depth
   and read-window constraints. Native extent execution bypassed range-copy
   validation on empty and Zero/Hole paths.
2. Added eight regressions. Before the repair, seven failed and the valid-empty
   case passed. The Zero fallback child hit its five-second kill/reap deadline;
   other failures exposed partial writes, backend calls before rejection, invalid
   empty-copy acceptance, and publication of invalid native offsets. Preserved
   the original failure log in the benchmark evidence directory.
3. Added allocation-free shared validation for nonzero/u32 block size,
   power-of-two alignment and allocation layout, and representable signed file
   ranges. All native range/extent entry points use it before any setup or I/O.
4. Added a full supported-kind precheck to the data-only executor. Added request
   range validation before owned SQE publication, including rejection of the
   current-file-position sentinel. Invalid requests release their buffer and do
   not poison the engine; the regression then performs a valid read.
5. Added entry-point microbenchmarks, memory Zero fallback, and a fragmented
   native Data-plan benchmark. Compared isolated baseline/candidate builds and
   reused dense buffered/direct benchmarks with unchanged threaded controls.
   Reset and readback are untimed; flush is timed for both source variants.
6. Corrected Clippy's hexadecimal digit-grouping warning in benchmark seed
   spelling; seed values and workloads are unchanged. Rebuilt validation
   benchmark hashes changed, so retained the initial patch/results and repeated
   both final variants. Dense benchmark hashes were unchanged. Also repeated
   the full direct-I/O profile with longer sampling after pair-to-pair drift.

Validation: 239 workspace tests passed (eight new); formatting and strict
all-target Clippy passed. The [architecture contract](architecture.md#native-entry-point-validation-r04)
records validation order and limits. Performance results and disposition are in
[the R0.4 report](benchmark-results/2026-09-28-r04/README.md).

Performance disposition: accept the mandatory validation repair. Final empty
calls added 1.12–1.26 ns; fragmented native copy was +1.73%; repeated direct
native aggregates were −3.01% to +0.95%. Initial buffered native was +2.19%.
Individual direct pairs still varied widely (−12.35% to +11.55% for QD8/64 KiB),
so general performance qualification remains provisional. The apparent −14.60%
memory fallback improvement is not a tuning claim. All benchmark output checks
passed; no defaults changed. Both saved patches passed `git apply --check`
against the clean baseline, and the temporary worktree was removed.

Remaining limits: these checks do not guarantee available memory, a supported
kernel queue size, endpoint access, durability, identity, or direct-I/O alignment.
Runtime failure can still leave completed extents written. R0.5 covers basic
endpoint preflight; R2 covers native tail compatibility and sparse semantics.

## R0.5 — Endpoint access, capacity, durability, and alias preflight

Baseline: `eb10da1`, clean worktree. Scope: validate endpoint requirements before
copying or notifying observers. F04/F10 are partially covered; runtime io_uring
compatibility, sparse semantics, and snapshot/plan identity remain later work.

Implementation sequence:

1. Audited all copy, planning, and observed execution paths. Portable execution
   could discover missing flush support after writing; native FD execution could
   bypass logical access checks, extend short destinations, or overwrite aliases.
2. Added the core `CopyEndpoint` contract with optional live backing identity.
   RAW forwards it; memory identifies the borrowed object; local Linux files
   refresh metadata/access from their open FDs. Platform inspection retains
   OS errors and retries interrupted metadata/flag queries.
3. Added shared access, size, alias, and durability preflight. Full-copy APIs
   require READ/WRITE/FLUSH and unchanged current source size. Native file APIs
   require regular files, explicit-offset-compatible access, and enough existing
   capacity; their caller remains responsible for flushing.
4. Bound native Data execution to known backend/descriptor identities and checked
   local direct/buffered handles at construction. A review caught the binding
   check occurring after initial observation; moved it into shared planning and
   execution preflight and added adversarial source/destination backend coverage.
5. Ran fourteen integration regressions on the original baseline: thirteen failed
   and the valid partial-range copy passed. The candidate adds nineteen tests
   total, including live size changes, hard links, append/read-only access,
   missing flush, no-callback rejection, errno preservation, and paired handles.
6. Updated existing fixtures/contracts deliberately: the worker-error backend
   advertises FLUSH so the intended worker error remains reachable; the two
   public native cleanup tests now assert contextual rejection without leaked
   descriptors because their special/short-file inputs are rejected before
   submission. Owned-engine fault-injection cleanup tests remain unchanged.
7. Added planning benchmarks, reused fragmented/zero/dense copy cases, and built
   isolated baseline/candidate variants. The Zero fallback benchmark now uses
   a regular source file in both variants to match the explicit native contract.
   Preflight runs once per native extent plan, not once per Data extent.
8. A final flag audit found that excluding only the opposite access mode would
   incorrectly advertise access for Linux's ioctl-only mode. Tightened the
   predicate to accept only named read/write modes and added a real-FD regression.
   Preserved initial results and repeated the full comparison for the final code.

Validation: 258 workspace tests passed; formatting and strict all-target Clippy
passed. Performance evidence and disposition are recorded in the
[R0.5 report](benchmark-results/2026-09-28-r05/README.md).

Performance disposition: accept the required checks and explicit planning cost.
Initial planning added 1.58–1.65 µs; final planning added 2.08–2.15 µs (+181–185%).
Eight new metadata/flag queries per RAW plan account for logical and descriptor
inspection; R1 should investigate sharing the descriptor snapshot safely. Final
copy aggregates were −1.93% to +2.37%. A longer direct-only initial repeat was
−1.09%, but final individual pairs still drifted; general PERF.0 qualification
remains provisional. Every output/FD-count check passed. No defaults were tuned.
Both source states, all initial/follow-up/final samples, and resource logs are
saved. The source patch was checked and the temporary baseline worktree removed.

Limits: checks are point-in-time; callers must stabilize contents, sizes, flags,
and mappings. Unknown generic identities cannot prove that endpoints are distinct;
custom native RAW backends must forward known identity. Plans remain structural,
not persisted snapshot identities. Runtime failures can still leave partial
writes. Low-level empty native calls remain no-ops after configuration validation.
See the [endpoint contract](architecture.md#copy-endpoint-preflight-r05).

## R1.1 — Portable planning, execution, and explicit RAW adapters

Baseline: `92f0350`, clean worktree. Scope: the portable lifecycle acceptance of
R1, canonical extent topology validation, and explicit separation of logical
access from Linux RAW native execution. The rest of R1 remains open.

Implementation sequence:

1. Generalized destination-aware planning, plan execution, report copying, and
   observers to `VirtualDisk + ?Sized`; widened worker/helper bounds so borrowed
   trait objects also work with multiple workers. Portable methods are available
   outside Linux and never consult physical descriptors.
2. Made portable selection explicit: Auto uses Threaded; explicit IoUring and
   native plans are rejected before payload I/O/notification. Renamed the old
   RAW-bound methods to `plan_raw_with_destination`, `execute_raw_plan`,
   `execute_raw_plan_with_observer`, and `copy_raw_with_report`. Updated native
   tests/benchmarks and documented this pre-release API migration in ADR-0027.
3. Shared endpoint/structural validation and initial/final observation across
   portable and RAW execution. Preserved separate endpoint policies and native
   descriptor binding. Reused the canonical validator in both plan constructors;
   removed the duplicate native validator and redundant caller scans. The existing
   malformed-map tests caught an accidentally removed direct-copy validation call
   during implementation; restored it before final checks and measurements.
4. Added nine integration tests covering memory/local/translated disks, trait
   objects, one/four workers, sparse capability combinations, endpoint/configuration
   rejection, empty plans, failed flush, and native rejection. The translated fixture
   shifts physical offsets, preserves surrounding guard bytes, and panics if native
   capabilities or descriptors are accessed. Added a constructor topology regression.
5. Ran an identical external trait-object consumer against baseline/candidate:
   baseline does not compile; candidate executes and verifies output. Checked
   core/datamover libraries for `wasm32-unknown-unknown`; this checks platform
   boundaries, not runtime/thread support. Removed Linux assumptions from unrelated
   generic unit tests and gated a Linux-only report constructor.
6. Compared the unchanged progress and native-lifetime harnesses using isolated
   optimized builds and alternating baseline/candidate order. Added candidate-only
   memory plan execution measurements for concrete versus trait-object calls.
   Setup/reset/readback are untimed; execution includes flush. Every copy verifies
   complete output, and native controls also check descriptor counts.

Validation: 268 workspace tests passed (ten new), strict all-target Clippy,
formatting, and non-Linux library compilation passed. Benchmark results and the
performance disposition are recorded in the
[R1.1 report](benchmark-results/2026-09-28-r11/README.md).

Performance disposition: accept with qualification remaining provisional. Initial
median paired copy changes were −3.51% to +2.22%. Longer repeats did not reproduce
the two individual >5% slowdowns: fragmented observed copy was −1.50%/−1.48%,
buffered native −6.53%/+0.01%. Portable planning avoids unnecessary RAW descriptor
inspection; RAW snapshot deduplication remains open. Candidate-only dynamic memory
execution includes a four-worker +26.64% outlier (other pairs +2.96%/+0.99%), so no
tight dispatch overhead or tuning claim is made. All evidence is retained and no
defaults changed. Patch application and source/executable fingerprints were checked.

Limits: plans remain structural and require stable logical mappings/content.
Canonical Data/Zero/Hole execution policy, contextual errors/partial progress,
memory budgets, and descriptor snapshot reuse remain R1 follow-ups. Existing
DISCARD guarantees and runtime native selection limitations remain R2 work.

## R1.2 — Shared semantic policy and sequential execution

Baseline: `b7162fc`, clean worktree. Scope: one Data/Zero/Hole operation policy
across portable/native execution, plus one sequential lifecycle for direct,
planned, and observed copies. No public API changes or stronger sparse guarantees.

Implementation sequence:

1. Audited the duplicated decisions in unobserved sequential, observed sequential,
   worker, and destination-aware native execution. Recorded existing precedence:
   Zero uses WRITE_ZERO or ordinary writes; Hole uses DISCARD, then WRITE_ZERO,
   then ordinary writes. Failure of an advertised operation is propagated.
2. Added private `policy::select` and routed all those execution paths through it.
   Retained whole-extent operations for sequential/native copies and bounded work
   items for workers. Data selection avoids querying sparse capabilities.
3. Replaced the duplicated sequential loops with `sequential::execute`. It owns
   allocation, block/tail transfers, statistics, and flush. Compile-time progress
   hooks preserve the observer cadence without adding observer state to ordinary
   copies; a const-generic transfer helper shares copy/fallback mechanics.
4. Added five contract tests for exact operation traces, odd tails, all sparse
   capability combinations, advertised-operation failure without retries,
   64 MiB progress thresholds, and flush failure. All five also pass on baseline,
   confirming behavior preservation. Existing translated-disk and native parity
   tests continue to cover byte equality and logical/native separation.
5. Explicitly recorded a pre-existing progress limit: an exact byte threshold can
   emit 100% before the final extent is counted/flushed. Flush failure suppresses
   the outer final snapshot, but snapshots have no terminal lifecycle tag. Do not
   interpret percentage alone as durable completion; check the execution result.
6. Added a matched memory sparse-policy harness and compared isolated optimized
   baseline/candidate builds. Reused the progress, dynamic memory, and native Zero
   fallback harnesses. All timed copies include their flush boundary; reset and
   full readback remain untimed. No executor defaults or buffer budgets changed.

7. Initial repeats left accelerated mixed four-worker copies above the 5%
   threshold (+10.23% median). A same-baseline-binary control varied by only
   −0.16%/−3.45%/−0.75%, so the slowdown was not dismissed as host noise. Reviewed
   worker code generation and changed the central selector to branch directly by
   extent kind, with a shared zero/fallback helper. Zero selection no longer goes
   through Hole's discard condition. The same capabilities are still queried once.
8. Retained both patches/hashes and all initial, follow-up, and same-binary control
   results. Rebuilt the candidate and repeated the targeted mixed workload:
   −0.62%/−2.85%/−15.10%. Repeated all tests and the full matched benchmark matrix
   for that intermediate source state; no initial timings are presented as later-code data.

9. The second full matrix still showed sequential memory/Zero fallback costs.
   Preserved that result instead of calling it final qualification. Restored the
   original configured block-size bound in the shared transfer helper, rather than
   deriving the bound from the buffer's runtime length. Allocations and actual
   ranges are identical. Three targeted pairs gave +1.92%/+6.22%/−2.79% for dynamic
   one-worker memory and −2.81%/+0.09%/−17.31% for observed Zero fallback. Re-ran
   the tests/checks and paired the entire final matrix case by case, reducing the
   gap between corresponding baseline/candidate timings. All three source patches
   and experiment results remain recorded.

Validation: 273 workspace tests passed (five new). Strict all-target Clippy,
formatting, and core/datamover library compilation for `wasm32-unknown-unknown`
passed. The latter is a portability compile check, not a runtime qualification.
Benchmark evidence and performance disposition are in the
[R1.2 report](benchmark-results/2026-09-28-r12/README.md).

Performance disposition: accept the implementation/refinements; PERF.0 remains
open, not a clean performance pass. Final dense median changes were −2.11% to
+0.83%; dynamic memory −0.40%/+0.26%; accelerated mixed −2.87%/+1.30%. Three final
profiles remain above 5% by median: fragmented no-op observer +7.66%, one-worker
Hole fallback +8.29%, and one-worker Zero fallback +14.12%. Sparse individual pairs
conflict and longer final-source Zero repeats do not reproduce that median cost.
Keep all three as controlled-runner follow-ups, including the possibility of real
callback/fallback overhead. No defaults changed and no universal speedup is claimed.
All source states, raw samples, outliers, and process resource logs are retained.

Limits: DISCARD still relies on the existing backend contract; R2 must introduce
an explicit zero-read guarantee. Native and worker payload mechanisms remain
separate and retain their existing lifetime/shutdown behavior. Contextual errors,
partial statistics, terminal events, memory budgets, and prepared endpoint
snapshots remain future work.

## R1.3 — Share fresh local RAW endpoint inspections

Date: 2026-09-28. Baseline: `28221ec` (R1.2). Scope: remove duplicate
local descriptor metadata work while preserving logical endpoint authorization,
native identity binding, and execution-time refresh.

Implementation sequence:

1. Added platform `FileInspection`, with private state and a live descriptor
   borrow, and the optional `LinuxFdBackend::copy_endpoint_from_inspection` hook.
   Custom backends default to their original logical endpoint checks.
2. Local files check the exact descriptor before deriving current capacity,
   identity, and restricted capabilities. Portable calls take a fresh inspection;
   RAW preflight shares it between logical and physical checks. Neither planning
   nor execution caches it across calls.
3. Kept source size, logical READ/WRITE/FLUSH, bounds, aliases, file modes, and
   native Data binding validation. Core remains independent of platform; platform
   now imports core's portable endpoint contract.
4. Added tests for a different descriptor to the same inode, logical restrictions
   on a writable FD, refreshed capacity/append flags, default custom-backend
   checks on every plan/execute call, and unknown native identity. The custom
   backend contract test also passes on the unchanged baseline runtime.
5. Added complete RAW copy controls to the existing preflight benchmark: reset
   and readback outside timing, planning/execution/final flush inside timing.
   Measured isolated optimized baseline/candidate builds with adjacent pairs.
6. Final review caught the optional hook making the trait non-dyn-compatible.
   Restricted that hook to sized backends, preserving the pre-existing FD trait
   object API and FD-only implementations. Added a compile/runtime regression,
   repeated validation and the benchmark matrix, and retained both source states
   with their raw evidence.

Validation: **277 workspace tests passed** (four new), formatting, strict
all-target Clippy, and core/datamover library compilation for
`wasm32-unknown-unknown`. The portability check is compilation, not a runtime
qualification. Freshness, alias, mismatch, and rejection-before-observation
regressions continue to pass.

Performance: strace confirms **eight → four metadata/access syscalls per local
RAW pair preflight**, for both Threaded and IoUring planning (100 plans each).
The [R1.3 report](benchmark-results/2026-09-28-r13/README.md) records final paired
planning/copy latency, all initial/final samples, source/binary fingerprints,
process resource logs, and the qualification decision. Final planning medians
improve 14.16% (Threaded) and 28.34% (IoUring); complete-copy medians change
−0.05%/+0.25%. One native copy pair is +9.92%; keep its variability for
controlled-runner qualification rather than declaring a clean performance pass.

Limits: inspection is point-in-time and its syscalls are not atomic. Custom
backends may still perform independent metadata work. Lower native layers retain
their preflight boundaries; persistent native preparation/ring reuse remains R2.
This does not establish snapshot consistency, extend durability, or close PERF.0.
The R1.2 fragmented observer and sequential Hole/Zero follow-ups remain open.
[ADR-0028](adr/0028-endpoint-inspection.md) records the accepted boundary.

## R1.4 — Separate logical intent and invocation preparation

Date: 2026-09-28. Baseline: `5e75d51` (R1.3). Scope: explicit logical intent,
observable planning selection, and one fresh preparation boundary for both
observed and unobserved plan execution.

Implementation sequence:

1. Split CopyPlan's private storage into LogicalCopyPlan (canonical extents,
   accounting, and fingerprint) and execution selection/configuration. Retained
   existing public getters and added `execution_selection()`, with requested
   strategy/options, selected backend, and a non-exhaustive reason enum.
2. Centralized portable/RAW selection policy. Portable Auto records the API
   boundary; RAW records explicit native requests or descriptor compatibility.
   Kept explicit native rejection and existing backend selection behavior.
3. Moved shared structural and endpoint validation into the preparation module.
   Each PreparedExecution borrows its checked plan and endpoints. Native
   preparation resolves current native options, constructs the native extent
   plan, and checks descriptor/buffer alignment before initial observation.
4. Dispatch now uses prepared native configuration. This fixes native strategy
   and alignment rejections emitting an initial callback: both APIs now reject
   before observation. Preserved successful callbacks, flush ownership, payload
   timer boundaries, and the direct portable copy fast path.
5. Added seven tests for selection/provenance, identical logical intent across
   choices, explicit-native versus Auto rejection policy, native strategy/config
   changes, and live alignment changes on either endpoint. The three behavioral
   regressions fail on baseline with one unexpected callback and pass on candidate.
6. Ran an eight-profile matched benchmark matrix using unchanged harnesses and
   isolated builds. Recorded all raw samples, source/binary fingerprints, resource
   logs, and a longer targeted follow-up for fragmented planning.

Validation: **284 workspace tests passed** (seven new), formatting, strict
all-target Clippy, and core/datamover library compilation for
`wasm32-unknown-unknown`. The latter is a portability compile check, not runtime
qualification. The [R1.4 report](benchmark-results/2026-09-28-r14/README.md)
contains exact commands, before/after regressions, and performance disposition.

Performance disposition: accept the measured RAW planning cost of +57 ns/+3.35%
(Threaded) and +28 ns/+1.55% (native). Complete RAW copy medians are −0.04%/+0.78%.
Fragmented planning initially measured +5.01%; a longer unchanged-binary repeat
measured −0.05%, with pairs −0.37%/−0.05%/+3.44%. Retain both. Fragmented no-op
observation is +4.41% and dynamic memory −1.79% by aggregate, with conflicting
pairs. This is not a clean performance qualification pass; PERF.0 remains open.

Limits: planning reasons are historical provenance, not runtime readiness.
The RAW compatibility evaluator still accepts descriptor pairs and combines
alignment claims; full request checks and runtime io_uring fallback remain R2.
Private preparation does not reserve a ring/buffers or stabilize source state.
Later allocation/kernel failures can still follow initial observation. Native
execution uses the executing mover's current native options; plans retain their
selected backend. Low-level native APIs and copy/flush contracts are unchanged.
PERF.0 and R1.2/R1.3 controlled-runner follow-ups remain open.

## R1.5 — Contextual failures and confirmed partial progress

Date: 2026-09-29. Baseline: `8adfabd` (R1.4). Scope: typed failure context and
conservative progress accounting for sequential, concurrent, and native copies.

Implementation sequence:

1. Added core CopyOperation, CopyProgress, CopyFailure, and boxed
   Error::CopyExecution. Error::copy_failure finds context through native cleanup
   wrappers; the standard source chain retains the underlying core/OS cause.
2. Separated InvalidCopyConfiguration, StaleCopyPlan, and live EndpointChanged
   preflight errors from CorruptMetadata. Invalid extent topology remains corrupt
   metadata; numeric/capacity/preflight variants retain their existing forms.
3. Sequential execution captures the attempted block/sparse operation and completed
   counters. A fully read block counts as read even when its write fails. Failed
   opaque backend calls are marked as possibly having additional uncounted I/O.
4. Workers retain their counters on failure and aggregate after all workers join.
   Preserve the first recorded cause and uncertainty from later worker failures.
   No per-block counter atomics were added; the existing cancellation/shutdown
   behavior and buffer return guarantees remain.
5. Native range copying retains observed CQE progress before leaving the pipeline.
   Positive short transfers count confirmed bytes without falsely completing a
   block. Sparse/extent execution adds prior completed extents. Shutdown preserves
   the original failure and unconfirmed-operation diagnostic. Engine ownership
   implementation is unchanged; cleanup I/O can remain outside the lower bounds.
6. Added flush context for all executors. Failure retains complete payload
   counters and suppresses final success observation; counters never imply
   durability, rollback, or a contiguous resume point.
7. Added ten tests covering errno/source chains, partial writes, sequential
   operation/range/counters, worker aggregation after joining, later uncertainty,
   native short I/O/buffer return, native sparse/fallback errors, cleanup chains,
   typed plan rejection, and threaded/native flush failures.
8. Updated existing error-shape assertions to inspect the original cause inside
   copy context while preserving worker shutdown and no-retry guarantees.
   Measured ten unchanged success-path harness profiles with isolated builds,
   retained all pairs, and repeated a variable four-worker sparse profile.

Validation: **294 workspace tests passed** (ten new), formatting, strict
all-target Clippy, and core/datamover library compilation for
`wasm32-unknown-unknown`. The portability result is a compile check, not runtime
qualification. Exact commands, samples, and performance disposition are in the
[R1.5 report](benchmark-results/2026-09-29-r15/README.md).

Performance disposition: complete RAW copy medians are +0.14%/+0.28%, and all
main aggregate changes stay within −3.05% to +2.18%. Four-worker mixed sparse
copying remains variable: +19.60% in one main pair and +16.44% in one longer
repeat, despite aggregates +0.10%/−2.93%. A same-baseline-binary control is tighter
(−2.59%/−0.02%/−0.53%). Accept functionality provisionally; retain this as a PERF.0
investigation, including possible code/scheduling effects. Do not dismiss it as
host noise or declare qualification complete.

Limits and migration: payload errors now carry a CopyExecution wrapper; callers
matching raw Io/Unsupported errors must inspect the cause/source chain. Error
counters are confirmed lower bounds. Failed backend calls and native shutdown
can perform additional I/O; worker errors do not identify fully completed
extents. Unknown native completion identity is reported at the attempted range
level. Panics, cancellation, terminal lifecycle tags, durable checkpoints, and
full native runtime preparation are outside this step.
[The error contract](copy-errors.md) records these boundaries. PERF.0 and prior
controlled-runner performance follow-ups remain open.

## R1.6 — Accounted copy memory budget

Date: 2026-09-29. Status: **Complete within the payload-accounting scope**;
process RSS and controlled-runner performance qualification are not claimed.
Baseline: `67d245f` (R1.5), initially clean working tree. Candidate is committed
with this record; [source patch, binary hashes, and measurements](benchmark-results/2026-09-29-r16/README.md)
identify the implementation independently of the final documentation commit.

Problem: bounded pools and channels alone did not constrain their aggregate
storage or fragmentation-dependent extent metadata. Structural revalidation
retained two extent Vecs; concurrent scheduling unnecessarily cloned the plan;
native preparation owns another plan copy.

Implementation:

1. Added a configurable **256 MiB per-invocation default** in CopyOptions. This
   is an admission policy, not a tuned optimum or reservation. All public
   DataMover planning/copy/report/observed/RAW/native routes check their relevant
   allocation phases. Existing low-level pool/native functions remain separate.
2. Added CopyMemoryUsage and DataMover::execution_memory for the execution
   breakdown using current mover settings. Charge buffer payload and pool
   descriptors, bounded queue/worker entries, and extent **capacity**, including
   unused slots. Native accounting uses executing queue depth and reserves sparse
   fallback storage conservatively.
3. Added typed MemoryBudgetExceeded (phase/required/budget) and
   MemoryAccountingOverflow. Reject known execution excess before another live
   extent query; check returned live capacity with the retained plan before
   observation, payload operations, or flush. Exact limits are admitted.
4. Charge peak phases separately: revalidation Vecs drop before executor
   allocation; native fallback and Data pools do not coexist. Check native clone
   requirements before allocation and actual clone capacity before dispatch.
5. Removed the concurrent scheduling extent clone by borrowing the plan's slice
   through scoped execution. No new per-block accounting or synchronization.
6. Added five regression tests covering overallocated metadata, exact limits,
   observed/unobserved/direct/report rejection without I/O, current metadata
   growth, arithmetic overflow, executing native queue depth, native plan copies,
   and exact-budget native readback. Existing failure/shutdown checks still pass.
7. Recorded the accounting model and explicit exclusions in
   [copy-memory.md](copy-memory.md), architecture, error contracts, ADR-0027,
   README, and roadmap. Backend query allocation happens before capacity can be
   checked; opaque allocator/container overhead, stacks, backend/observer memory,
   kernel resources, other invocations, and prior native quarantine remain
   external. Fragmentation-dependent total memory and resource availability are
   not solved by this budget.

Validation: **299 workspace tests passed**, formatting, strict all-target Clippy,
and core/datamover library compilation for wasm32-unknown-unknown. The portable
check is compilation only. All eleven benchmark profiles use unchanged harnesses
and isolated optimized builds; byte/counter checks remain enabled. Raw results
and resource logs are retained. Performance disposition follows below.

Performance disposition: all eleven main aggregate changes fall between −4.91%
and +1.90%; complete RAW copies are +1.48%/+0.95% Threaded/native. Four-worker
mixed sparse pairs vary from −23.52% to +1.71%; no general speedup is claimed.
Fragmented no-op observation includes a **+39.16% main pair** and **+16.94% longer
repeat**, despite aggregates −3.28%/+0.78%. Accept functionality provisionally,
retain these samples, and keep performance qualification open. The identical
baseline-binary control also varies (+19.85%/−0.22%/+3.15%), demonstrating
run-to-run variation without ruling out candidate effects. Full interpretation is in the
[R1.6 report](benchmark-results/2026-09-29-r16/README.md). No buffer/queue tuning or
ESXi work was needed for this local step.

## R2.1 — Logical Hole zero-read guarantee

Date: 2026-09-29. Status: **Complete within the logical-content contract**;
local filesystem allocation and performance qualification remain open.
Baseline: `25977f6` (R1.6), initially clean working tree. Candidate is committed
with this record; [source/harness patches, binary hashes, and results](benchmark-results/2026-09-29-r21/README.md)
identify the measured implementation.

Problem: the shared policy chose DISCARD for Hole output without a guarantee
that a successful call would produce zero-reading bytes. A backend could report
success while leaving nonzero destination contents.

Implementation:

1. Defined Zero and Hole as logical zero-read guarantees. Physical unallocation
   must not hide inherited parent data; logical backends resolve it before
   reporting Hole. Source snapshot/consistency requirements remain unchanged.
2. Added Capabilities::DISCARD_ZEROES (bit 8), a modifier requiring DISCARD.
   Successful calls must zero the complete requested logical range, preserve
   surrounding bytes and disk size, and do not promise physical reclamation,
   atomic rollback, or durability. WRITE_ZERO retains its zero-content contract.
3. Updated the shared selector used by sequential, worker, and native sparse
   execution. Hole chooses discard only with both flags, otherwise WRITE_ZERO,
   otherwise bounded zero writes. Data and Zero selection are unchanged.
4. Default MemoryBlockDevice advertises the new guarantee because its discard
   fills bytes with zero. Explicit capability masks are not silently upgraded;
   read-only construction remains without discard. RawDisk forwards the flags.
   Local filesystem operations remain R2.2, so local Hole output still uses
   bounded zero writes.
5. Preserved no-retry behavior for a failed advertised sparse operation, even
   Unsupported after a partial mutation. Failure context/counters and flush/
   observer boundaries remain unchanged. bytes_discarded measures logical bytes
   successfully processed, not actual physical space reclaimed.
6. Added six tests: explicit memory capability advertisement; all eight flag
   combinations across four portable entry paths with one/four workers;
   native sparse dispatch; partial guaranteed-discard failure; and two mixed
   RAW/native parity profiles with unqualified DISCARD. Expanded existing
   translated-disk/semantic matrices and migrated zero-guaranteeing fixtures so
   accelerated and failure-path coverage still exercise their intended calls.
7. Reproduced both portable and native regressions on the baseline with only a
   new flag declaration and the new fixture. Baseline reports two expected test
   failures; candidate passes. The reproduction patch, commands, and output are
   retained, then baseline production code was restored before benchmarking.
8. Updated semantic-policy/failure benchmark fixtures to explicitly advertise
   zero-reading discard using a bit literal compilable on both revisions. Added
   exact accelerated zero/discard counter assertions. Matched benchmark harness
   edits are saved separately; baseline and candidate use identical harnesses.
9. Accepted [ADR-0026](adr/0026-logical-hole-guarantee.md), superseding the unsafe
   unconditional preference in ADR-0003/0022. Updated trait contracts,
   architecture, README, error counter definitions, and roadmap.

Validation: **305 workspace tests passed** (six new), formatting, strict
all-target Clippy, and core/datamover library compilation for
wasm32-unknown-unknown. The portable result is compilation, not runtime
qualification. Tests validate logical bytes and operation selection; memory and
synthetic discard implementations do not prove filesystem space reclamation.

Performance disposition: main aggregates range from −2.16% to +4.01%; complete
RAW copies are +0.82%/+0.29% Threaded/native. Four-worker dynamic memory has
+4.01% main and +2.60% longer-repeat aggregates, including +5.92%/+7.49% slow
pairs. Fragmented no-op observation retains a +10.39% pair despite aggregate
−0.61%. Accept the correctness fix provisionally; no general speedup or clean
performance qualification is claimed. [The report](benchmark-results/2026-09-29-r21/README.md)
records the focused repeat and identical-baseline control (+1.92%/−1.76%/+4.95%
pairs). That variability does not rule out candidate-specific effects. Prior
PERF.0 issues remain open.

Additional harness audit: the pre-existing scheduler_failure benchmark omits
FLUSH and asserts raw Io errors, predating current preflight/CopyExecution
contracts. Its new guarantee declaration is migrated, but its runtime repair
remains PERF.0 work before those failure-latency timings can be reused. It was
not used for the R2.1 measurements.

## R2.2 — Local zeroing and hole punching

Date: 2026-09-29. Status: **Complete within the local sparse-output contract**;
filesystem-independent allocation and general performance qualification are not
claimed. Baseline: `a4a04fb` (R2.1), initially clean working tree. Candidate is
committed with this record; [source/harness patches, hashes, and measurements](benchmark-results/2026-09-29-r22/README.md)
identify the measured implementation.

Problem: local output always materialized logical holes through ordinary zero
writes, despite R2.1 having established the guarantee needed for safe deallocation.
Local Zero ranges also lacked a backend operation. Both needed exact boundaries,
access/range checks, and an honest fallback contract before advertising support.

Implementation:

1. Writable Linux LocalFileBlockDevice now advertises WRITE_ZERO and
   DISCARD|DISCARD_ZEROES. Read-only handles do not. Fresh endpoint inspection
   removes the new write capabilities when current descriptor access is not
   writable. RawDisk and shared DataMover policy automatically use these paths.
2. Implemented ZERO_RANGE|KEEP_SIZE and PUNCH_HOLE|KEEP_SIZE on exact byte ranges.
   Kernel partial-block semantics preserve surrounding bytes and file size.
   Direct-open devices use their verified buffered alias for sparse operations.
3. Added cached-geometry and fresh-size checks, checked range/off_t arithmetic,
   logical write permission, and current primary/buffered append/access checks.
   Empty valid requests avoid fallocate. Observed truncation cannot silently
   regrow the file through fallback. Checks remain inspections, not external
   mutation locks.
4. EOPNOTSUPP/ENOSYS use bounded positional writes from a shared 64 KiB zero
   array, without per-operation heap allocation. Unsupported acceleration is
   cached independently per mode/open device. EINTR retries; EINVAL, EPERM,
   EIO, ENOSPC, and other errors propagate without fallback or caching.
5. Preserved copy flush ownership and failure/no-retry behavior. Successful
   backend zero/discard contributes its operation counter even if the backend
   used ordinary writes internally. Counters do not measure physical allocation.
   Updated old tests/harness assertions to reflect this intentional migration.
6. Added nine tests covering unsupported-mode injection/cache independence,
   interrupted and real errors with partial effects, empty/invalid/truncated
   ranges, append flags on both handles, read-only access, odd edges and readback
   through buffered/direct opens, storage allocation, and mixed copies through
   one/four workers and io_uring with/without observation.
7. Ran local output tests on the recorded storage filesystem, including the
   explicitly gated physical-allocation test. Punching 6 MiB from an 8 MiB
   incompressible file reduced allocated 512-byte blocks **16,384 → 4,096**;
   full logical readback and file size remained correct.
8. Added a matched 32 MiB mixed local-output benchmark (Data 8 MiB, Zero 4 MiB,
   Hole 20 MiB) for sequential, four-worker, and native execution. Migrated
   progress/native-zero harness counters with capability-aware assertions,
   preserving identical harnesses across builds. Fixed the known scheduler
   failure fixture's FLUSH capability and CopyExecution-aware error assertion;
   smoke-ran all eight failure profiles on both builds without claiming renewed
   latency qualification.
9. Updated [local sparse output](local-sparse-output.md), architecture,
   ADR-0026's implementation record, README, roadmap, and this log. Physical
   reclamation remains filesystem dependent; fallback may allocate space.

Validation: **314 distinct tests covered**: 313 pass in the workspace run, one
storage-allocation test is explicitly gated there and passes in the separate
storage run. All three local sparse-output integration tests also pass on that
mount. Formatting, strict all-target Clippy, and the core/datamover wasm32 library
compile check pass. The latter is not runtime portability qualification.

Initial workspace runs exposed old local Zero/Hole counter expectations (four
native-extent and two unified-dispatch assertions). Their failure logs are
retained; expectations were migrated and the final suite passes. No payload
readback failure was hidden. [The report](benchmark-results/2026-09-29-r22/README.md)
records commands, smoke checks, allocation evidence, and timing samples.

Performance disposition: 32 MiB mixed-copy aggregates improve 34.67% sequentially,
27.53% with four workers, and 44.79% natively on this host. Dense RAW controls are
−0.47%/−1.40%. However, fragmented no-op observation is **+26.67%** in the main
comparison and **+15.39%** in a longer repeat, with every paired result slower.
Accept the sparse-output feature with this explicit measured cost; do not dismiss
it as noise or claim universal acceleration. Repeated small hole punches and
fresh checks replace zero writes, and need targeted batching/backend-cost work
before qualification. Full samples, counter checks, control variability, and
limits are in the [R2.2 report](benchmark-results/2026-09-29-r22/README.md).
Prior PERF.0 issues remain open.

## R2.3 — Dense fallback for unavailable sparse discovery

Date: 2026-09-29. Status: **Complete within the local discovery contract**.
Baseline: `2755be0` (R2.2), initially clean working tree. Candidate is committed
with this record; [source, hashes, and measurements](benchmark-results/2026-09-29-r23/README.md)
identify the measured implementation.

Problem: unsupported SEEK_DATA/SEEK_HOLE prevented local RAW copying altogether.
Discovery also trusted the size captured at open, so truncation could turn ENXIO
into a false trailing Hole. Partial maps needed a conservative all-or-error
policy before fallback could be safe.

Implementation:

1. Extracted private discovery logic with injectable seek results. EINVAL,
   EOPNOTSUPP, or ENOSYS from either selector returns one Data extent for the
   entire requested range, discarding any partial map. No unknown allocation
   state is classified as Hole.
2. Validate checked range arithmetic, original geometry, off_t limits, and fresh
   size before scanning; recheck fresh size before returning a nonempty map.
   Empty valid requests do not seek. Observed truncation cannot produce a false
   Hole or a dense map extending beyond current EOF.
3. Retry EINTR and preserve genuine errors. ENXIO from SEEK_DATA means trailing
   Hole only with the final range check; ENXIO from SEEK_HOLE is an error.
   Reject backward, nonprogressing, and past-EOF offsets before clipping valid
   results to the query.
4. Deliberately avoid a negative discovery cache, so later errors or changed
   allocation maps remain visible. Existing extent validation and stale-plan
   fingerprints are unchanged. Caller-managed source stability remains required;
   metadata checks do not create a snapshot or detect every concurrent mutation.
5. Added ten regression tests: unsupported selectors at the first/late seek,
   discarded partial maps, full logical readback through real holes and nonzero
   regions, exact subranges, interruption, real errors, invalid/empty requests,
   malformed maps, and truncation before/during discovery.
6. Added an opt-in dense-map expectation to the existing progress harness, used
   identically in both builds. A process-local lseek interposer forces EINVAL
   for a separate candidate experiment, without adding production fault hooks.
   This is injected unavailability on the recorded storage filesystem, not
   qualification of a second filesystem.
7. Updated README, architecture, roadmap, sparse-output cross-reference, and the
   [source discovery contract](local-sparse-discovery.md). Benchmarks retain raw
   samples, commands, resource logs, matched harness/source patches, and hashes.

Validation: **323 workspace tests pass**, one existing storage-allocation test
is explicitly gated (324 distinct tests in the inventory). Formatting, strict
all-target Clippy, and core/datamover wasm32 library compilation pass. The
allocation test is unchanged and was exercised on Btrfs in R2.2; this step does
not claim a renewed physical-allocation result or runtime portability.

Injected-discovery smoke checks reproduce the baseline Io/EINVAL failure and
verify candidate complete RAW Threaded/native copies plus all eight progress
planning/copy profiles. The sparse fixture becomes one Data extent, with every
logical byte read and written and complete output verified on a nonzero prefill.

Performance disposition: native small-plan aggregate **+34.54%** (1.766 →
2.376 µs), accepted as an explicit correctness cost. Complete RAW Threaded/native
are **+0.94%/+1.07%**; dense/fragmented copies **−0.30%/+0.05%**. Fragmented planning
is **+3.02%** with a **+9.10%** pair; the longer repeat is
**+1.35%**, with pairs **+1.40%, -2.36%, +0.35%**.
Keep this concern provisional; no same-binary control was run. Candidate-only
injected fallback timings characterize a different I/O path, not a baseline
speedup. [All results and limits](benchmark-results/2026-09-29-r23/README.md)
are retained. Prior PERF.0 and R2.2's fragmented-output cost remain open.

## DOC.4 — Benchmark plots and reusable report generation

Date: 2026-09-29. Status: **Complete**. Baseline: `8ed67ec` (R2.3), initially
clean working tree. This step responds to the request for visual benchmark
reports before continuing implementation.

Added a headless Matplotlib generator with a pinned plotting dependency, explicit
report configuration, and usage guide. R2.3 now embeds six SVG charts with PNG
copies: separate planning/copy bars, aggregate changes with paired-run dots,
per-run sample box plots, the main/longer-repeat comparison, and candidate-only
injected fallback. Each figure carries measured source identities, units, and
conditions. Outliers remain visible; candidate-only results claim no speedup.

The generator normalizes Criterion samples by iteration count, matches explicit
pair IDs, and checks raw medians and all aggregate/paired results against saved
summaries. It records computed values and hashes of inputs, generator, and images,
plus the Python/package environment. Future measured steps must include applicable
plots under the [benchmark policy](benchmarks.md#plots-for-recorded-results).
Historical trends require matched conditions; no misleading cross-step trend is
inferred from different fixtures or host conditions.

Validation: five focused Python tests pass, covering pair identity, aggregation,
normalization/outlier preservation, missing/duplicate pairs, mismatched recorded
values, and invalid samples. All 48 saved runs validate (36 main, six repeat,
six candidate-only); all six figures were visually inspected. SVG/XML, PNGs,
links, and provenance hashes check successfully, and regeneration in the recorded
environment produces identical image bytes. Original benchmark evidence remains
unchanged. Rust tests and benchmarks were not rerun: no Rust/runtime changes.
Performance disposition: **N/A — plotting and documentation only**; R2.3 and
all earlier qualifications remain as recorded. R2.4 remains next.

## R2.4 — Native request compatibility and whole-plan fallback

Date: 2026-09-29. Status: **Complete within DataMover's declared request contract**.
Baseline: `5b64bf7` (R2.3 plus plots), initially clean working tree. Candidate is
committed with this record; [source/harness patches and hashes](benchmark-results/2026-09-29-r24/README.md)
identify the measured builds.

Problem: RAW descriptor evaluation accepted every pair, so Auto could select
io_uring for odd tails, unaligned Data, or block splits incompatible with a direct
endpoint. A sparse prefix or observer callback could occur before a later native
request failed. Mixed endpoint evaluation did not enforce the direct side's
request alignment independently.

Implementation:

1. Added one allocation-free checker for declared native request intent: u32
   block width, nonnegative i64 ranges, power-of-two endpoint alignments, valid
   allocation layout, Data offsets/lengths, and block boundaries actually used
   by multi-request Data extents. Buffered pairs avoid a per-Data alignment scan.
2. Auto records RawRequestsIncompatible(NativeRequestIssue) and chooses Threaded
   for the whole plan. Explicit native returns the typed issue. The new core
   error/reason carries endpoint, range, and alignment information. Logical
   extents, zero/discard policy, and source validation remain intact.
3. Preparation rechecks native intent before allocating its extent clone or
   invoking observers. Changed compatibility rejects the existing plan for both
   strategies; callers may replan with Auto. Preserve R1.4's historical plan
   selection and R1.6's backend-specific budget rather than changing execution
   backend underneath an already prepared structural plan.
4. Existing fresh descriptor inspection now includes O_DIRECT. RAW planning and
   preparation reject disagreement with backend declarations, without adding
   fstat/fcntl calls. Alignment declarations are reread, not rediscovered via
   statx on each execution; local discovery still occurs at open.
5. The explicitly native FD-only DataMover wrapper also rejects declared request
   incompatibility and never silently falls back. Standalone low-level FD copy
   functions retain their caller-managed direct alignment contract. Runtime
   resource preparation, ring reuse, and general concurrent alias policy remain
   open; no retry after mutation was added.
6. Added five request-math tests and six integration tests. The latter include
   48 tiny/odd/aligned copies across buffered/direct/mixed endpoints, one/four
   workers, and observed/unobserved execution, plus sparse-prefix rejection,
   changed declarations/flags, replanning, split boundaries, and fallback budgets.
7. Added a matched 16 MiB benchmark for both mixed endpoint directions and a
   candidate-only 16 MiB + 7-byte Auto fallback workload. Complete copies time
   planning and final flush, assert selected backend/counters, and verify all
   output bytes on a nonzero-prefilled destination. Extended plot tooling to
   label generic candidate-only experiments without calling them injected EINVAL.
8. Review found that an inline request diagnostic increased core Error from
   40 to 48 bytes on this target. Boxed the diagnostic only on the error path
   to preserve the existing layout; retained the first candidate's source and
   measurements in a separate archive and remeasured the final implementation.
9. Updated README, architecture, roadmap, ADR-0027, and the
   [request contract](native-request-compatibility.md), including migration and
   the distinction between request acceptance and runtime readiness.

Validation: **334 workspace tests pass**, one unchanged storage-allocation test
remains gated (335 distinct tests). Formatting, strict all-target Clippy, and the
core/datamover wasm32 library check pass. All six new integration tests also pass
on the recorded Btrfs mount; this is not renewed physical-allocation qualification.
Five plot-calculation tests pass. The first focused build lacked a direct libc
dev dependency for the fcntl test; its diagnostic is retained, the Linux-only test
dependency was added, and the final suite passes.

Performance disposition: final native small-plan aggregate **+3.59%** (about
0.096 µs); complete buffered RAW Threaded/native **−4.55%/+3.15%**; mixed direct
source/destination **+4.33%/−3.00%**. Native RAW and mixed direct-source retain
**+15.57%/+16.61%** adverse pairs. Longer-repeat aggregates are
**-0.07%/-2.68%**, with paired results preserved.
Same-baseline-binary controls characterize mixed-I/O variation but do not rule out
candidate effects. Accept request-safety functionality with performance
qualification still open. The [report](benchmark-results/2026-09-29-r24/README.md)
retains all final/initial samples, controls, candidate-only fallback, plots, and
limitations. No timing gain is attributed to boxing the diagnostic.

## R2.5 — Native runtime preparation and per-job reuse

Date: 2026-09-29. Status: **Complete within the documented runtime contract**.
Baseline: `6680a0b`, initially clean. Source and matched harness identities are
retained with the [benchmark evidence](benchmark-results/2026-09-29-r25/README.md).

Moved native ring, pool, and descriptor setup before initial callbacks and sparse
prefix writes. Invocation-scoped preparation transfers ownership into execution;
all Data extents reuse that resource set. A Data pool buffer supplies zero-write
fallback, while sparse-only jobs prepare one zeroed block and need no ring.
Shutdown runs once per job, preserving lifetime quarantine and prior progress on
failure. Observer panic drops unsubmitted resources. No payload scheduling,
queue, worker, or read-window defaults changed.

Auto runtime fallback is restricted to recognized ring-construction unavailable/
denied errors. Explicit native, invalid configuration, exhaustion, descriptor
failure, and post-submission errors never fall back. Threaded fallback gets its
own budget check after native-only metadata is released. Planning selection stays
immutable; successful reports expose runtime_fallback(), and every progress
snapshot uses the actual backend. Native stats retain setup time explicitly.

Added eight isolated integration tests, including syscall-denial fault injection,
ring reuse after blocking further setup, sparse prefixes, one/four workers,
observed/unobserved behavior, budget failure, and descriptor/panic cleanup.
The first fixture omitted EXTENTS capability and exposed dense Data; its failed
output is preserved, and the fixture was corrected before measurement. Strict
Clippy identified a large dispatch enum; boxing its native-only resources keeps
the portable dispatch small before benchmarking the final source.

Added matched fragmented native benchmarks with and without a no-op observer,
and a candidate-only unavailable-ring workload. Updated runtime/request/memory
contracts, architecture, ADR-0027, README, and roadmap.

Validation: **342 workspace tests pass**, one unchanged allocation test is gated
(343 distinct tests). Formatting, strict all-target Clippy, and core/datamover
wasm32 compilation pass. All eight runtime and six request integration tests also
pass on Btrfs. Raw outputs and commands are retained with the report. No new
physical-allocation or cross-kernel qualification is claimed.

Performance: 66 matched runs show fragmented native elapsed time **−59.46%**
unobserved / **−59.19%** with a no-op observer; 16 Data extents **−6.62%**;
memory zero fallback **−25.24%**; mixed local sparse **−6.38%**. Dense RAW native,
mixed directions, and direct QD1 aggregate changes are below 0.1%, but the
**+38.16% native RAW pair** remains open. Six longer-repeat runs yield
**+38.23%**, and same-baseline-binary controls retain
+3.22%, -0.48%, -0.98% paired changes. Controls do not
explain away candidate effects. Candidate-only denied-ring fallback is
**13.629 ms**, with no baseline speedup claim. Accept functionality and
reuse while leaving controlled-runner performance qualification open. All raw
runs, source/harness/binary identities, failed fixture output, and six SVG/PNG
plots are in the [report](benchmark-results/2026-09-29-r25/README.md).
Twelve additional short diagnostic runs yield +0.14% aggregate (paired −3.50%
to +2.10%). Separate syscall traces show flush dominates their measured cycles;
tracing perturbs timing and does not attribute the earlier regression. Both
longer-repeat and main adverse results remain open.

## R2.6 — Cooperative concurrent local alias admission

Date: 2026-09-29. Status: **Complete within the documented cooperative contract**.
Baseline: `2d5c68c`, initially clean. Exact source patch, unchanged benchmark
harness hashes, and separately built executable identities are retained with
[the evidence](benchmark-results/2026-09-29-r26/README.md).

Added per-file admission shared by Linux local backend calls and native requests,
keyed by device/inode across independent opens and hard links. Admission fails
before I/O for overlapping writers, page-overlapping mixed buffered/direct modes,
and whole-file flush conflicts. Extent inspection takes read intent; sparse
operations take one buffered-write intent across syscall and all fallback chunks.
Same-mode readers and disjoint direct subpage requests remain concurrent.

Native operations own admission before SQE publication and retain it until a
known final CQE or quarantine. Completed buffers do not retain admission.
Negative completions, successful shutdown, and unwinding release confirmed
requests; uncertain shutdown retains the range with buffer and descriptor owners.
There is no automatic copy retry or executor fallback for admission conflicts.

The registry is consulted at registration; short per-file mutex sections scan
active ranges without holding a lock across I/O. Shared registry/vector allocations
remain backend metadata outside the payload budget, while the owned native guard
is charged in operation entries. File modes are inspected once per IoUringFile;
infallible OwnedFd conversion remains available with lazy first-enqueue inspection.
No native scheduling, worker, buffer, or queue defaults changed.

Added 14 tests: five coordinator tests, three bounded native lifetime tests,
four real local/native integration tests, and two sparse/inspection lifetime tests.
The first two development compilations caught wrong sparse method names in the
new integration test (`punch_hole`, then `deallocate_at`); corrected to the existing
`discard` API. Their diagnostics are retained. Production admission compiled on
its first check; the initial full executable suite then passed. Existing fixtures
and correctness assertions were not weakened to accommodate the new policy.

[ADR-0029](adr/0029-local-file-admission.md) and the
[contract](local-file-concurrency.md) record fail-fast rationale, migration,
page granularity, quarantine behavior, and external-writer responsibility.
This is request-level cooperation; raw FDs, mmap, external processes, stable
source snapshots, and copy-wide scheduling remain caller responsibilities.
Updated ownership/planning ADRs, runtime/request/memory/sparse contracts,
architecture, README, and the R3 implementation sequence.

Validation: **356 workspace tests pass**, one unchanged allocation test remains
gated (357 distinct tests). Formatting, strict all-target Clippy, and the
core/datamover wasm32 library check pass. Four new concurrency, six request,
and eight runtime integration tests also pass on Btrfs. Exact commands and raw
outputs are retained in the report; no new allocation qualification is claimed.

Performance: 54 matched runs show native planning **+6.72%** (about 0.171 µs).
Six longer-repeat runs yield **+2.20%** (about 0.053 µs), retaining **+5.71%/+5.32%**
pairs. Accept the observed planning cost for admission correctness; attribution
and broader qualification remain open. Eight copy aggregates range **−5.06% to
+4.89%**, with no adverse pair above +5%; these shared-host observations do not
establish a speedup. Small-block buffered/direct native and four-worker controls,
fragmented native, and sparse output are included. Five reproducible SVG/PNG
charts and exact source/harness/binary identities accompany all samples.
Prior R2.5 adverse results and PERF.0 remain open. Registration scaling and
heavily contended multi-job costs are explicitly unmeasured.

## R3.1 — Read-only RAW CLI

Date: 2026-09-29. Status: **Complete within the preview contract**.
Baseline: `bfd5355`, initially clean. The [report](benchmark-results/2026-09-29-r31/README.md)
records exact source patch, unchanged control harnesses, compiler/build commands,
and separate release artifact hashes. Public binary spelling is `rvddk`; the
crate is `rvvdk-cli`, with existing repository/library names unchanged.

Implemented inspect and read-only plan preview, explicit RAW format, human output,
schema-versioned JSON, stable error codes/exit status, optional extent lists,
non-UTF-8 paths, and validated backend/tuning requests. Reuse LocalFileBlockDevice
and DataMover logical planning for discovery/topology/budgets; no existing library
implementation changed. Source and destination bytes are never mutated.

Executable RAW planning requires a writable destination. Instead of fabricating
capabilities or opening output writable, plan records logical work and destination
intent. Threaded selection follows its explicit request; Auto/native selection
and all runtime preparation are deferred. A labeled Threaded payload estimate
can exceed budget while producing a valid preview; execution admission remains
separate. JSON is not a serialized/reloadable CopyPlan.

New output intent is no-clobber without creating files or parents. Existing output
requires --overwrite and adequate capacity; intent is in-place over source length,
preserving any tail. Reject known aliases, leaf symlinks, special files, and invalid
paths. Read-only output can be previewed; permission and race-safe publication
checks remain execution work. [ADR-0030](adr/0030-read-only-cli-preview.md) and the
[CLI contract](cli.md) define schema, resource limits, and R3.2 obligations.

Twelve new tests cover dense/empty/odd/sparse topology, preservation, policies,
aliases, path encodings, flags, budgets, output failures, and real binary use.
Six bounded exec cases use seccomp to deny writable opens and native setup while
previewing new/existing destinations through each backend request. The initial
scaffold check preceded creation of its declared benchmark file; that manifest
error is retained. All executable test runs pass. Cargo.lock adds only the new
workspace package; no third-party versions were changed.

Validation: **368 workspace tests passed**, one unchanged allocation test remains
gated (369 distinct tests). Formatting, strict all-target Clippy, and the
core/datamover wasm32 library check pass. All twelve CLI tests also pass on Btrfs.

Performance: twelve matched library controls yield **−0.47% Threaded / +0.48%
native planning**, with no +5% aggregate/pair threshold crossing. No engine
speedup is claimed. Nine candidate-only Criterion runs measure about **16.913 µs**
for dense in-process inspect, **22.715 µs** for new-output plan, and **1.638 ms**
for fragmented full-map inspect. Six whole-process runs (900 measured launches)
yield about **1.381/1.388 ms** for inspect/plan, including taskset/startup and
captured output. Different boundaries have no before/after speedup interpretation.
Four reproducible SVG/PNG charts retain all measurements and identities. No tuning
defaults changed; PERF.0 and prior adverse results remain open.

## R3.2 — Copy, verification, and safe publication

Date: 2026-09-29. Status: **Complete within the local transfer contract**.
Baseline: `cd41872`, initially clean. The [report](benchmark-results/2026-09-29-r32/README.md)
retains initial/final source patches, separate binaries, raw samples, resource
logs, validation and reproducible SVG/PNG charts. The enclosing commit records
this step; no third-party dependency versions or tuning defaults changed.

Implemented reusable bounded logical verification and buffered owned-file
adoption. CLI copy independently prepares live endpoints; optional verification
buffers are reserved from the same payload budget before mutation. New output
uses a private anonymous inode, copy/flush/optional verify, source metadata checks,
file sync, no-replace linking, directory sync and final name check. Existing
--overwrite is in place and preserves its inode and tail. Errors retain phase,
conservative destination state, confirmed counters and nested context. Failure
to print a successful result identifies the already-completed operation.

[ADR-0031](adr/0031-local-copy-publication.md) and the
[transfer contract](cli-transfer.md) record O_TMPFILE/procfs/directory requirements,
metadata observation limits, partial effects and no rollback. Stable contents
and namespace remain caller obligations. Native quarantine can retain an unnamed
file descriptor after uncertain I/O. Cancellation and progress remain R3.3.

Validation: **382 tests passed; one unchanged allocation test gated** (383 total).
Fourteen new tests exercise verification, budgets, alias/path policies, all
backend requests, output failure and publication boundaries. Two bounded child
cases deny ring setup. Sync-error hooks model boundary errors, not real fsync
fault injection. All 18 CLI integration tests pass on Btrfs; fmt, strict all-target
Clippy and portable core/datamover wasm32 checks pass. Initial dependency-check
and zero-in-hole corruption-fixture diagnostics are retained, with successful
subsequent validation.

Initial controls found +25.27%/+24.01% inspect/plan cost from the eager command
tree. Deferred selected-command construction removed unrelated option building;
all initial evidence remains, including build-confounded later repeats. Final
24 matched controls yield +0.07%, -0.75%, -5.69%, -0.60%
(Threaded copy, native copy, inspect, plan). Longer repeats yield +1.06%, retaining
a +6.20% plan pair; attribution remains open.
Every pair is in the report; no engine speedup or broad storage qualification is
claimed. Twelve new 16 MiB workflow runs record median costs of
20.153 ms, 31.896 ms, 31.328 ms, 2.736 ms for Threaded new, Threaded new+verify,
native new+verify and verify-only respectively. Copy timers include file and
directory durability; verify-only has a different boundary. Earlier PERF.0 and
adverse qualification results remain open.

## R3.3 — Lifecycle progress and cancellation

Date: 2026-09-29. Status: **Complete within the cooperative local contract**.
Baseline: `d83dcc3`, initially clean. [Evidence](benchmark-results/2026-09-29-r33/README.md)
retains initial/final source and binary identities, raw samples, diagnostics,
resource logs, validation and reproducible SVG/PNG charts. The enclosing commit
records this step; dependency versions and execution defaults are unchanged.

Added controlled portable/RAW execution APIs, CancellationToken/predicate,
coordinator CopyEvent lifecycle states and verification checkpoints. Concurrent
workers aggregate counters and stop taking work; native cancellation preserves
owned shutdown/quarantine. Unobserved APIs compile out generic checks and legacy
snapshot cadence remains compatible. CLI --progress uses human lines or JSON
lines on stderr. Binary-only SIGINT/SIGTERM handlers record an atomic request;
programmatic cancellation is available without process-global handlers.

Engine flush completion is distinct from CLI verification/publication completion.
Before linking, cancellation leaves new output private. After linking, finish
directory sync/name inspection and report published cancellation; never unlink.
In-place partial effects and tails retain R3.2 semantics. Progress-output failure
stops cooperatively. No hard deadline, forced syscall cancellation or rollback is
promised. [Contract](cli-progress.md); [ADR-0032](adr/0032-copy-lifecycle-cancellation.md).

Validation: **393 passed, one unchanged gated allocation test** (394 total).
Eleven new tests cover progress/terminal order, flush failure and cancellation,
worker counters/joins, native reuse, verification prefixes, sparse processing,
publication boundaries, overwrite preservation and progress output failure.
Three bounded real binary cases exercise SIGINT/SIGTERM. All 24 CLI integration
and five controlled-execution tests pass on Btrfs. Formatting, strict all-target
Clippy and core/datamover wasm32 checks pass. Initial test-helper/fixture compile
errors and the missing platform guard are retained. After the guard fix, some
release hashes changed; all timings were rerun on the final source.

Final performance: 48 matched runs yield -4.20%, +1.55%, +1.76%, -0.88%,
-1.83%, +1.62%, +0.34%, -0.58%
for Threaded 1 MiB, native 1 MiB, four-worker 16 MiB, CLI plan, CLI new,
CLI new+verify, CLI native+verify and verify-only respectively.
The +5.60% four-worker pair triggered repeats: -6.07% aggregate, no adverse pair
above +1.53%, and substantial timing spread; no speedup is claimed.
Twelve candidate-only runs yield 48.785 ms, 48.697 ms, 772.529 µs, 817.444 µs for
Threaded/native full progress and Threaded/native early stop respectively. Stop
timers cover the whole invocation through cleanup, not signal reaction time.
Sink/capture boundaries differ. No engine speedup or earlier qualification closure
is claimed; PERF.0 and prior adverse results remain open.

## R4.1 — Bounded descriptor parsing

Date: 2026-09-29. Status: **Complete within the parser-only contract**.
Baseline: `cde5a70`, initially clean. [Evidence](benchmark-results/2026-09-29-r41/README.md)
retains source/harness/binary identities, raw samples, validation and reproducible
SVG/PNG plots. The enclosing commit records this step.

Added portable `rvvdk-vmdk` for version-1 hosted base descriptors: monolithicFlat,
split flat, and custom FLAT/ZERO. Input/line/extent/metadata/name bounds, checked
capacity/backing-offset arithmetic, strict feature/duplicate/order checks and typed
line errors precede future backing I/O. Strings borrow input. Paths are explicitly
untrusted metadata. Synthetic fixtures record provenance against VMware's format
note; no third-party implementation or SDK is used. See the
[contract](vmdk-descriptor.md) and [ADR-0033](adr/0033-bounded-vmdk-descriptors.md).

Validation: **405 distinct passed, one unchanged gated allocation test** (406 total).
Twelve parser tests cover grammar, limits, overflow, variant rejection and bounded
mutation smoke coverage. Formatting, strict all-target Clippy and core/datamover/
parser wasm32 checks pass (existing datamover dead-code warning retained). No
backing-file validation, reference byte comparison or ESXi qualification is claimed.

Performance: 12 matched runs show +1.33%, -0.74%
for RAW planning and one-worker copy+flush. Every paired result is retained;
none exceeded the adverse +5% threshold. Twelve parser runs establish
1.044 µs, 105.812 µs, 104.596 µs, 7.85 ns for small, 1,024-extent,
late extent-limit and early input-size rejection respectively. Input construction
is excluded; parsing, vector allocation/drop are timed. No prior parser baseline,
engine speedup claim, tuning-default change or earlier qualification closure.

## R4.2 — Bounded backing resolution

Date: 2026-09-29. Status: **Complete within the owned-source contract**.
Baseline: `380acc2`, initially clean. [Evidence](benchmark-results/2026-09-29-r42/README.md)
retains source/harness/binary identities, raw samples, validation and reproducible
SVG/PNG plots. The enclosing commit records this step.

Added bounded owned descriptor acquisition and portable caller-supplied backing
resolution. Count limits precede resolver calls; repeated exact references share
sources. Retained read-only wrappers validate live access/physical ends and support
identity/size/access revalidation. Linux pins the descriptor parent and uses
openat2 beneath/no-symlink/no-mount-crossing lookup, O_PATH regular-file inspection
and procfs reopen with identity comparison. Existing buffered local I/O admission
is reused. No weaker fallback; no snapshot guarantee. See the
[contract](vmdk-backing.md) and [ADR-0034](adr/0034-confined-vmdk-backing-resolution.md).

Validation: **420 distinct passed, one existing gated allocation test** (421 total).
Fifteen new tests cover portable resource/ownership/live-state contracts and Linux
path/object/reopen/truncation behavior. All 27 VMDK tests pass on Btrfs. Formatting,
strict all-target Clippy and core/datamover/VMDK wasm32 checks pass (unchanged
control::sum warning). Dependency versions, parser implementation and RAW execution
remain unchanged.

Performance: 18 matched runs show -0.80%, +0.90%, +0.01%
for small parser, 1,024-extent parser and RAW Threaded copy+flush respectively.
Longer repeats: -0.98%. Every main and repeat pair remains in the report.
The RAW control executable is byte-identical across builds; its timing variation
cannot be attributed to a changed RAW implementation. Twelve new-mode runs yield
8.998 µs, 286.475 µs, 26.421 µs, 11.685 µs for one source,
32 sources, 1,024 references sharing one source and descriptor load/parse/drop.
Resolution includes open/check/close, excludes parsing; loading has a different
boundary. No engine speedup or prior performance-qualification closure is claimed.

## R4.3 — Read-only logical VMDK mapping

Date: 2026-09-29. Status: **Complete within the documented local subset**.
Baseline: `4eee003`, initially clean. [Evidence](benchmark-results/2026-09-29-r43/README.md)
retains source/harness/binary identities, samples, validation, reference-tool
records and reproducible SVG/PNG plots. The enclosing commit records this step.

Added VmdkDisk over retained FLAT/ZERO sources: binary first-extent lookup,
allocation-free caller-buffer reads, cross-extent offset translation, checked
ranges, clipped/coalesced Data/Zero discovery, live preflight validation and no
writable/native RAW interface. EOF/errors propagate without zero recovery.
A new default VirtualDisk identity hook lets copy/verification preflight reject
aliases to any backing and fail closed for unknown identities. Wrappers must
forward it. [Contract](vmdk-logical.md); [ADR-0035](adr/0035-read-only-vmdk-logical-mapping.md).

Validation: **432 distinct passed, one existing gated allocation test** (433 total).
Twelve new logical/local tests cover ranges, boundaries, short/error I/O,
concurrency, alias protection, truncation, portable copies and tail preservation.
All 39 VMDK tests pass on Btrfs. Formatting, strict all-target Clippy and portable
core/datamover/VMDK wasm32 checks pass (existing control::sum warning).

Five generated reference cases are explicitly qualified: synthetic monolithic/split
agree with QEMU and expected RAW; QEMU-generated hosted layouts agree after
recorded fixture-only trailing NUL removal; custom matches the byte oracle only
because QEMU rejects its createType. Original padded descriptors still reject.
Initial reference/test diagnostics are retained; no unsupported reference case
is counted as agreement. No SDK or third-party implementation source was used.

Performance: 36 matched runs show -0.16%, -0.19%, +0.07%, +1.74%, -1.02%, -0.79%
for RAW planning, one-worker copy, four-worker copy, 1,024-extent parsing,
one-source resolution and CLI verification respectively.
An adverse +8.71% four-worker pair triggered longer repeats, yielding -1.58%
aggregate. Every initial and repeat pair remains in the report.
Fifteen new-mode runs yield 1.185 µs, 1.098 µs, 0.070 µs, 7.579 ms, 15.795 ms
for memory FLAT 64 KiB, mixed 64 KiB, final-of-1,024 4 KiB, local FLAT 1 MiB copy
and local mixed 1 MiB copy respectively. Copy includes final flush; memory reads
exclude construction/revalidation. No speedup or earlier qualification closure.

## R4.4 — Explicit CLI VMDK sources

Date: 2026-09-29. Status: **Complete within the documented local subset**.
Baseline: `e60a3eb`, initially clean. The enclosing commit records this step.
[Evidence](benchmark-results/2026-09-29-r44/README.md) retains source/harness/binary
identities, all samples and pairs, validation and reproducible SVG/PNG plots.

Added explicit `--format vmdk` to inspect/plan/copy/verify, with RAW destinations.
An owned source enum preserves typed native RAW execution and uses portable
VmdkDisk execution. VMDK Auto selects Threaded; explicit io-uring rejects before
destination effects. Logical reports include capacity, Data/Zero ranges and
bounded source identity details. The confined resolver exposes file acquisition
so the CLI can retain observations of the exact descriptor/backing objects.
Every source alias rejects before overwrite; size/mtime/ctime observations detect
changes during acquisition and copy/verification. Existing private publication,
verification, cancellation and overwrite-tail policies apply unchanged.
[Contract](cli-vmdk.md); [ADR-0036](adr/0036-cli-vmdk-sources.md).

Validation: **440 distinct passed, one existing gated allocation test** (441 total).
Eight integration tests cover logical reports, mixed reads/verified copies,
overwrite tails, mismatch offsets, every descriptor/backing alias, early native
rejection, strict format/confinement errors, budgets, Zero-only/explicit RAW input,
same-size source changes, cancellation and publication races. A separate 76-test
CLI/VMDK run passes with integration fixtures on Btrfs; five existing CLI unit
fault tests retain their system-temporary fixtures. Formatting, strict all-target Clippy and core/datamover/VMDK wasm32
checks pass (existing control::sum warning). A test-only byte-string Clippy style
correction is retained in the evidence. No parser/mapping algorithms or dependency
versions changed; no ESXi or VMware SDK was used.

Performance: 30 matched runs show +9.86%, +8.31%, -0.48%, +1.42%, +0.82%
for RAW inspect, RAW plan, Threaded copy+verify, native copy+verify and verify-only.
Adverse preview aggregates/pairs and one +5.72% native pair triggered 18 longer
runs: +10.67%, +5.36%, +1.44% respectively. All original/repeat pairs remain visible.
The persistent preview cost is about 1–2 µs, accepted for nonblocking owned-file
acquisition, descriptor adoption/retention and stronger metadata observations.
Code inspection identifies the additional work; no isolated syscall attribution
is claimed. No per-block scan was added. Controlled-runner tuning remains open.
Fifteen new-mode runs yield 53.367 µs / 61.726 µs for mixed VMDK inspect/plan,
22.453 ms / 24.671 ms for 1 MiB FLAT/mixed copy+verify with full publication, and
159.038 µs for mixed verification. These are distinct workloads, not engine speedups.

## Next session

Start **R4.5**: define and implement bounded trailing-NUL descriptor acquisition
compatibility, with unmodified generated hosted descriptor reference comparisons.
Keep parser/resource/error rules explicit and custom independent-decoder
qualification separate; neither limitation is silently closed by CLI integration.
Each step gets tests, benchmarks, plots and a commit.

ESXi remains unnecessary for local work; request the 60-day trial when V0's lab
proof is ready. Keep PERF.0, R4.4 preview overhead/tuning and earlier adverse timing
pairs open for a controlled runner. Do not infer VMware live-access compatibility
from local format support.
