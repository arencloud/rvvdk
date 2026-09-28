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
| R0.3 | Complete | Owned-only engine, retained descriptors, verified completion/shutdown rules; 18 lifetime/error regressions; ADR-0013 safety argument and API migration; committed with this record | [Native comparison and resource evidence](benchmark-results/2026-09-28-r03/README.md) recorded; mandatory safety fix accepted; performance qualification provisional because native/control timings drifted |
| R0.4 | Complete | Native configuration/range validation and supported-plan precheck | Accepted correctness cost: ~1–2 ns empty-call overhead; copy aggregates below 5%; shared-host qualification provisional |
| R0.5 | Complete | Endpoint capabilities/live capacity, known aliases, native binding, and contextual preflight errors | Required correctness cost accepted: planning +2.08–2.15 µs; final copy aggregates −1.93% to +2.37%; general qualification provisional |
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

## Next session

Start **R1.4**: separate logical copy intent from executor preparation. Introduce
a clear internal preparation result and execution-selection reason without
weakening current plan validation, endpoint checks, or callback ordering.
Keep runtime io_uring readiness/unaligned native policy in the R2 scope.
Continue recording benchmark baselines and commit each completed step.

Carry the R1.2 fragmented no-op observer and sequential Hole/Zero performance
qualification forward in PERF.0; do not declare those profiles qualified.
ESXi is still unnecessary; request the 60-day trial only when V0 is ready.
