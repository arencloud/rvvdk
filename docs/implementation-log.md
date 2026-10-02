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
records appear below. R3 local CLI and R4.1–R4.5 local FLAT/ZERO VMDK workflows are complete within their
documented contracts. R5.1 header admission is complete; R5.2 onward and V0 remain planned. Performance qualification
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
Sixteen new tests cover portable resource/ownership/live-state contracts and Linux
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

## R4.5 — Bounded descriptor padding

Date: 2026-09-29. Status: **Complete within the documented acquisition policy**.
Baseline: `ce193cf`, initially clean. The enclosing commit records this step.
[Evidence](benchmark-results/2026-09-29-r45/README.md) retains source/harness/binary
identities, samples, reference commands/hashes, validation and SVG/PNG plots.

DescriptorText now accepts a contiguous terminal NUL run after bounded acquisition
reaches EOF. The unchanged total-byte limit includes padding; oversize input still
uses only one extra probe. Original bytes remain intact through as_bytes(); new
text_bytes()/padding_bytes() expose the cached prefix and padding count. Repeated
parse() calls use that prefix without rescanning padding or allocating a normalized
copy. Direct Descriptor parsers remain strict. Embedded NUL/nonzero suffixes,
invalid/all-zero input and I/O failures after padding reject. Local loading and CLI
inherit the policy with no source-file rewrites or logical mapping changes.
[Contract](vmdk-padding.md); [ADR-0037](adr/0037-bounded-vmdk-padding.md).

Validation: **447 distinct passed, one existing gated allocation test** (448 total).
Six acquisition tests cover strict parsing, provenance, suffix/chunk boundaries,
interruptions, errors after padding and exact/over-limit admission. A CLI test
exercises all four commands on an unchanged padded source. The separate 83-test
CLI/VMDK run passes with integration fixtures on Btrfs; five existing CLI unit
fault tests retain system-temporary fixtures. Formatting, strict all-target Clippy
and core/datamover/VMDK wasm32 checks pass (existing control::sum warning).

The reference runner now requires direct acceptance of original QEMU-generated
monolithicFlat and twoGbMaxExtentFlat descriptors, with 157 and 149 terminal NULs.
Dump output and public CLI copy/verification equal the 1 MiB RAW oracle, QEMU
comparison succeeds, and original descriptor hashes remain unchanged. Inspect/plan
reports also pass and plan creates no output. Synthetic hosted reference cases
still agree; custom remains byte-oracle-only because QEMU rejects its createType.
Historical R4.3 normalization evidence remains immutable. No SDK or ESXi was used.

Performance: 30 matched runs show +0.35%, +1.01%, +3.28%, +0.18%, -1.03% for strict
small parsing, local unpadded acquisition, VMDK inspect, VMDK copy+verify and RAW
verify. An initial +7.80% inspection pair triggered six longer runs: -0.61%
aggregate, paired -0.61%, -1.62%, +0.06%. Every original/repeat pair remains saved.
Twelve candidate-only runs yield 0.582 µs / 365.609 µs for acquiring 512 B / 1 MiB
padded inputs, 47.373 µs for oversize rejection and 0.372 µs for reparsing a cached
prefix. The one-time bounded scan is accepted; repeated parsing avoids that work.
Different boundaries are not speedups, and prior performance follow-ups stay open.

## R5.1 — Bounded hosted sparse header admission

Date: 2026-09-30. Status: **Complete within the header-only subset**.
Baseline: `8de73fd`, initially clean. The enclosing commit records this step.
[Evidence](benchmark-results/2026-09-30-r51/README.md) retains source/harness/binary
identities, all samples, specification/reference hashes, validation and SVG/PNG plots.

Added portable SparseHeader/SparseLimits/SparseRegion/SparseError APIs. Decode an
exact 512-byte sector without allocation, packed casts or unsafe code. Admit clean,
uncompressed version 1 with supported flags, checked grain/capacity geometry,
directory/table counts, independent resource limits, metadata range/overlap checks
and observed file bounds. A stack Read helper consumes exactly one sector and
propagates interrupted/short/error I/O correctly. Header admission does not validate
region contents or make a disk readable. Existing FLAT/ZERO APIs and CLI sparse
rejection remain unchanged. [Contract](vmdk-sparse-header.md);
[ADR-0038](adr/0038-bounded-hosted-sparse-header.md).

Validation: **456 distinct passed, one existing gated allocation test** (457 total).
Nine new tests cover geometry/regions, feature rejection, all truncated lengths,
endian/alignment handling, arithmetic/limits, overlaps and file bounds, short I/O
and 2,048 deterministic mutations. The separate 92-test CLI/VMDK run passes with
integration fixtures on Btrfs; five existing CLI unit fault tests retain temporary
fixtures. Formatting, strict all-target Clippy and portable core/datamover/VMDK
wasm32 checks pass (existing control::sum warning).

Three QEMU-generated headers are admitted (1 MiB/64 MiB monolithic and 1 MiB split),
with capacity agreement against QEMU info and unchanged file hashes. Unaligned
capacity and streamOptimized are deliberately rejected; all five fixtures remain
unsupported by the public CLI. The initial runner assumed split headers advertise
no descriptor, but QEMU reserves an empty region there. The failed assertion,
header probe and initial runner are retained; the corrected runner records the
region without claiming valid text. No parser relaxation was needed. R5.2 must
bind external descriptors explicitly. No logical sparse byte agreement, VMware
SDK source or ESXi qualification is claimed.

Performance: 24 matched runs show -1.17%, -0.65%, -0.15%, +0.25% for existing strict
text parsing, local descriptor load, FLAT/ZERO copy+verify and RAW verify. An adverse
+5.32% mixed-copy pair triggered six longer runs: -1.86% aggregate, with all pairs
between -1.86% and -0.65%. Original/adverse samples remain recorded. Twelve new
runs give 133.898 ns / 135.957 ns for valid 1 MiB/1 TiB capacity headers,
125.158 ns for reserved-byte rejection and 140.715 ns for memory-sector acquisition
plus parsing. These fixed-input costs are not sparse data throughput; no allocation
or metadata I/O scales with capacity. Prior performance follow-ups remain open.

## R5.2 — Bounded sparse metadata validation and descriptor binding

Date: 2026-09-30. Status: **Complete within the metadata-only subset**.
Baseline: `cbe2081`, initially clean. The enclosing commit records this step.
[Evidence](benchmark-results/2026-09-30-r52/README.md) retains source/harness/binary
identities, all samples, specification/reference hashes, validation and SVG/PNG plots.

Added explicit SparseDescriptor parsing without widening default Descriptor or CLI
acceptance. SparseMetadata resolves one selected extent, binds header capacity and
embedded text, admits memory/read budgets, validates all table placements before
table I/O, requires exact redundant-table agreement and checks complete grain ranges
and duplicate physical pointers. It retains the source and logical-order map,
rechecks header/endpoint observations and exposes no logical reads, native FD or writes.
Split reserved descriptor space may be empty, with external binding required.
Source quiescence remains necessary; revalidation is not a content snapshot.
[Contract](vmdk-sparse-metadata.md); [ADR-0039](adr/0039-bounded-sparse-metadata.md).

Validation: **465 distinct passed, one existing gated allocation test** (466 total).
Nine new tests cover grammar/CID widths, binding, exact budgets, placement before
reads, redundancy, invalid/duplicate grain pointers, endpoint/header changes and
short/error I/O. The separate 101-test CLI/VMDK run passes with integration fixtures
on Btrfs; five existing CLI unit fault tests retain system-temporary fixtures.
Formatting, strict all-target Clippy and portable core/datamover/VMDK wasm32 checks
pass (existing control::sum warning).

QEMU-generated 1 MiB/64 MiB monolithic and 1 MiB split metadata maps reconstruct
exact RAW oracle bytes and agree with QEMU compare. Source hashes stay unchanged;
all three remain rejected by the public CLI. The 64 MiB case spans two grain tables.
The initial attempt exposed an unpadded seven-digit CID. Its failed invocation,
probe and runner are retained; only the explicit sparse parser now accepts one to
eight hex digits as a 32-bit value. Existing eight-digit FLAT/ZERO behavior remains
covered. Final generated images need no fixture rewriting. This qualifies maps,
not a production sparse reader, multi-file producer split disks or ESXi behavior.
No producer implementation source or VMware SDK was used.

Performance: 24 matched runs show **-3.68%, +0.68%, -3.04%, +0.97%** for strict
text parsing, local descriptor load, FLAT/ZERO copy+verify and RAW verify. No adverse
aggregate or individual pair exceeds +5%, so no longer repeats trigger. Twelve new
memory-backed runs establish **5.437 µs monolithic**, **5.236 µs split**,
**4.842 µs redundant disagreement rejection** and **0.470 µs sparse text parsing**.
Load timings include resolve/metadata reads, validation, allocation, sort and drop;
no grain payload I/O occurs. The small fixture reserves 11,488 loader payload bytes
and requests 16,384 metadata bytes. Larger-capacity tuning and earlier performance
follow-ups remain open. These are shared-host cost baselines, not storage throughput
or a causal speedup claim.

## R5.3 — Read-only base sparse logical mapping

Date: 2026-09-30. Status: **Complete within the base hosted sparse library subset**.
Baseline: `c7a8a0f`, initially clean. The enclosing commit records this step.
[Evidence](benchmark-results/2026-09-30-r53/README.md) retains source/harness/binary
identities, all samples, reference hashes, validation and SVG/PNG plots.

Added SparseDisk/SparseDiskLimits over the R5.2 metadata loader. Acquire every extent
under per-extent and aggregate memory/read budgets plus a handle limit; charge
repeated references separately and revalidate all sources after loading. Reads map
across grains and split extents with different supported grain sizes, combine
physical adjacency, propagate short/error I/O and zero-fill unallocated base grains.
Logical Data/Zero queries clip/coalesce, count before allocation and enforce an
output limit. Retained immutable maps support concurrent reads without reopening.

Portable DataMover/Verifier use the new read-only VirtualDisk with all-backing alias
checks; unknown identities fail copy/verify preflight. Retained inode identity catches
hard links and survives pathname replacement. Standalone callers must revalidate
before sessions and keep sources quiescent; there is no content snapshot or replay.
No native RAW endpoint, writes, parent resolution or public CLI sparse support.
[Contract](vmdk-sparse-disk.md); [ADR-0040](adr/0040-read-only-base-sparse-mapping.md).

Validation: **478 distinct passed, one existing gated allocation test** (479 total).
Thirteen new tests cover ranges, physical coalescing/fragmentation, zero I/O avoidance,
split/mixed-grain-size mapping, exact aggregate and output limits, release on failure,
short/error reads, concurrent ownership/copy/verify, aliases/unknown/changed identities,
final acquisition observations and real-file hard links/path replacement. Separate
114-test CLI/VMDK validation passes with integration fixtures on Btrfs; five existing
CLI unit fault tests retain system-temporary fixtures. Formatting, strict all-target
Clippy and portable core/datamover/VMDK wasm32 checks pass (existing control::sum warning).

Full production reads match patterned RAW oracles and QEMU compare for four generated
fixtures: 1 MiB and 64 MiB monolithic, 1 MiB split and a two-file 2 GiB + 64 KiB split
disk with data across the split boundary. Every logical byte is read in 65,537-byte
chunks; source VMDK hashes stay unchanged. No fixture rewriting or alternate mapping
implementation supplies the reader. Public CLI rejects all four pending R5.4.
The largest case reserves 287,352 loader/struct payload bytes and reads 290,816
metadata bytes; this is larger-geometry correctness, not performance qualification.

Performance: 24 matched runs show **-0.51%, -0.09%, +0.15%, +0.21%** for existing
metadata load, FLAT/ZERO 64 KiB logical reads, CLI mixed copy+verify and RAW verify.
No adverse aggregate or pair exceeds +5%; no longer repeats trigger. The favorable
-10.96% RAW-verify pair remains visible as host variability, not a speedup claim.
Twenty-one new memory-backed runs establish **2.594 µs contiguous / 2.391 µs fragmented
128 KiB reads**, **1.053 µs Data/Zero / 1.117 µs split-boundary 64 KiB reads**,
**0.068 µs extent queries**, **10.909 µs split opening** and **0.410 ms 2 MiB portable
copy+verify**. Contiguous-read medians vary from 2.302 to 2.981 µs: fewer backend
requests are proven by tests, not a timing advantage on this shared memory workload.
Opening, reading, queries and copying have different boundaries; no storage throughput
claim follows. Larger-capacity performance and prior tuning follow-ups remain open.

## R5.4 — Sparse CLI acquisition and lifecycle integration

Date: 2026-09-30. Status: **Complete within the supported base sparse CLI subset**.
Baseline: `81aa801`, initially clean. The enclosing commit records this step.
[Evidence](benchmark-results/2026-09-30-r54/README.md) retains source/harness/binary
identities, all samples, reference hashes, validation and SVG/PNG plots.

Explicit --format vmdk now accepts monolithic sparse containers and external split
sparse descriptors for inspect/plan/copy/verify. Bounded entry acquisition shares
its first text read with format dispatch; binary header/range checks precede embedded
text reads. Monolithic extent resolution must match the opened container's device/
inode before backing metadata reads. Different-inode redirection fails; same-inode
hard links are accepted. Source observations retain the descriptor and every loaded
backing. SparseDisk participates in portable planning/execution with existing alias,
verification, cancellation and no-replace publication behavior. Sources remain
quiescent by caller responsibility, with timestamp checks at existing boundaries.

Sparse preview reports add layout and loader memory/read counters under schema 1.
Loader budgets stay independent of copy/verification memory-budget and exclude CLI
entry acquisition/observation resources. RAW selection remains explicit; native RAW
execution, parent chains, compressed/managed variants and VMDK writes remain outside
this step. Existing flat parsing is reused once after bounded acquisition; library
Descriptor/DescriptorText contracts remain unchanged. [Contract](cli-vmdk.md);
[ADR-0041](adr/0041-sparse-cli-source-acquisition.md).

Validation: **488 distinct passed, one existing gated allocation test** (489 total).
Ten new CLI tests cover both sparse entry forms, all commands, threaded/auto copying,
overwrite tails, mismatch offsets, original bytes, same-inode binding, all aliases,
malformed headers/tables, confinement, limits, native/budget rejection, source changes,
cancellation and publication collisions. Separate 124-test CLI/VMDK validation passes
with integration fixtures on Btrfs; five existing CLI unit fault tests retain system-
temporary fixtures. Formatting, strict all-target Clippy and portable core/datamover/
VMDK wasm32 checks pass (existing control::sum warning). A fresh validation target
avoids stale pre-existing local artifacts; both release variants also build fresh.

All four public commands pass on four QEMU fixtures: 1 MiB and 64 MiB monolithic,
1 MiB split, and two-file 2 GiB + 64 KiB split with data across the boundary. Plan
creates nothing. Copy uses auto/threaded, four workers, 65,537-byte blocks and full
verification, then reports file/directory sync. Output SHA-256 equals the RAW oracle;
QEMU compare agrees and source hashes stay unchanged. Earlier staged reference
runners retain historical CLI-rejection expectations; the new sparse CLI runner
qualifies current behavior. No SDK/producer implementation source or ESXi was used.

Performance: 24 matched runs show **-15.58%, -17.96%, +1.19%, +2.83%** for existing
FLAT/ZERO inspect, plan, copy+verify and RAW verify. No adverse aggregate or pair
exceeds +5%, so no longer repeats trigger. The favorable -43.54% plan pair remains
visible; shared-host variability prevents a precise causal speedup claim. The code
removes duplicate flat parsing and adds no separate text probe syscall, but this
does not close R4.4 or PERF.0 qualification. Fifteen first sparse CLI runs establish
**55.584 µs inspect**, **82.264 µs split plan**, **19.718 ms monolithic copy+verify**,
**20.212 ms split copy+verify**, and **0.243 ms split verify**. Copy timings include
verification, flush, file/directory sync and publication. Sizes/boundaries differ;
these are first warm-Btrfs CLI baselines, not cold-storage or engine speedup results.

## R5.5 — Adversarial sparse validation and scaling

Date: 2026-09-30. Status: **Complete within deterministic corpus and synthetic scaling scope**.
Baseline: `1d745b8`, initially clean. The enclosing commit records this step.
[Evidence](benchmark-results/2026-09-30-r55/README.md) retains the exact patch,
all 69 measured runs, counters, QEMU checks, validation and reproducible SVG/PNG.
[Qualification contract](vmdk-sparse-qualification.md) records architecture and limits.
Production Rust source, dependencies, supported formats and budgets are unchanged.

Eight new tests cover 60 generated layouts × 24 read ranges, independent byte
oracles, zero/contiguous/reversed/alternating/permuted mapping, exact backing request
counts, table boundaries, invalid offsets/duplicates and truncation. A fixed-seed
4,096-case corpus rejects 4,031 and admits 65 (64 positive controls), checking source
bounds/read ceilings and admitted maps against decoded entries. This is finite
mutation testing; coverage-guided fuzzing remains future work. One-header rejection
protects the default aggregate budget for 1 TiB/64 KiB-grain input. 65,536 query
outputs succeed; 65,537 fails without preventing a small tail query. Capacity
profiles retain 1 MiB/1 GiB/64 GiB loader reservation and actual metadata reads.

Validation: **496 distinct passed, one existing gated allocation test** (497 total),
plus 132-test CLI/VMDK storage-configured run; five existing CLI unit fault tests
still use system temporary files. Formatting, strict all-target Clippy and portable
core/datamover/VMDK wasm32 checks pass (existing control::sum warning). All four
public commands pass on four QEMU fixtures, including the two-file 2 GiB + 64 KiB
split disk; RAW SHA-256/QEMU agree and source hashes remain unchanged. No live ESXi,
VMware SDK or producer implementation source is used.

Performance: 24 unchanged paired controls show **+0.65%, +0.10%, -0.51%, +3.62%**
for monolithic metadata, contiguous 128 KiB reads, FLAT/ZERO copy+verify and RAW
verify. The +10.85% RAW verify pair triggers six longer runs: +4.47% aggregate,
with **+27.90%, -1.79%, -0.02%** pairs. That investigation stays open; no adverse
run is discarded or pooled. Independent builds/shared-host variability prevent
causal attribution with unchanged runtime source.

Thirty-nine first synthetic scaling runs establish **5.592 µs / 32.134 µs /
4.738 ms** opening zero maps at 1 MiB / 1 GiB / 64 GiB. A full 64 GiB zero query
takes **1.303 ms** despite one output, versus **38.163 ns** for a reused-buffer
4 KiB zero tail read. Alternating 32,768/65,536-output queries take **0.436 /
0.903 ms**; rejecting 65,537 takes **0.263 ms**. Permuted 1 GiB opening is
**0.221 ms**. Generated 1 MiB read medians are **16.417 / 13.593 / 14.475 µs**
(contiguous/reversed/permuted); the contiguous second run is **40.115 µs**, retained
as observed variation. Payload generation and cache effects prevent interpreting
these as physical fragmentation penalties. Fixture setup/descriptor parsing are
outside opening timing; validated map allocation/drop is inside. Virtual capacity
requires metadata only; counters exclude fixture memory and are not RSS.

Retain eager mapping/two-pass query admission. Separate measured proposals can
address query scanning or sorted pointer validation; do not raise limits or infer
an optimization from these first costs. PERF.0/R4.4 and earlier adverse results
remain open. Historical evidence remains unchanged.

## R5.6 — Bounded sparse parent metadata admission

Date: 2026-09-30. Status: **Complete within metadata-only sparse-chain scope**.
Baseline: `e3ce2e9`, initially clean. The enclosing commit records this step.
[Contract](vmdk-parent-chain.md), [ADR-0042](adr/0042-bounded-parent-chain-metadata.md)
and [evidence](benchmark-results/2026-09-30-r56/README.md) retain implementation
policy, all 48 timing runs, source/harness/binary identities, tests and SVG/PNG.

`SparseLayerDescriptor` parses explicit parent syntax without widening base APIs.
`SparseChain` retains leaf-to-base descriptors and grain maps through an explicit
caller-provided parent resolver and per-layer backing namespace. Adjacent CIDs and
capacities must agree; known readable identities drive cycle/shared-backing checks.
Embedded monolithic entry must resolve the same container. Defaults admit 16 layers
including base, 128 extents, 8 MiB outer descriptor bytes, 128 MiB conservative
reservation and 256 MiB acquisition payload, in addition to per-layer limits.
Final endpoint observations cover every retained source. Source quiescence remains
a caller obligation; CIDs/identity checks are neither snapshots nor content hashes.

The chain is intentionally not a VirtualDisk. Unallocated child grains remain
unresolved metadata; the public CLI and base disk APIs still reject parents.
Logical fallback and CLI lifecycle integration remain separate increments.
The shared metadata loader now binds embedded parentCID/hint as well as CID/layout.
No dependencies or copy execution defaults change. Specification guidance, not
SDK or third-party parser source, defines the format relationships.

Validation: **512 distinct passed, one existing gated allocation test** (513 total),
plus 148-test CLI/VMDK run with integration fixtures on Btrfs; five existing CLI
unit fault tests retain system temporary files. Formatting, strict all-target
Clippy and portable core/datamover/VMDK wasm32 checks pass (existing control::sum
warning). Sixteen new tests cover syntax/bounds, both entry forms, retained handles,
missing/denied parents, mismatches before backing reads, cycles, equal-CID distinct
objects, aliases, unknown/changed identities, embedded binding, differing extent
geometry, I/O failures, exact budgets and depth 16/17. Actual read counters match
reported totals. Three external layers reserve/read 37,187/49,515 bytes; embedded
layers reserve/read 67,544/81,408 bytes. Conservative counters exclude parser,
resolver, allocator, source storage and stack overhead; they are not RSS.

QEMU-created base/monolithic/split/mixed chains pass helper metadata checks against
descriptor CIDs and QEMU backing-chain capacities. Public CLI accepts the base and
rejects the three parent chains. The confined helper uses one selected directory
and basename hints. Existing all-command base CLI reference checks also pass,
including the two-file 2 GiB + 64 KiB split case, full RAW hashes, QEMU compare and
unchanged source hashes. Chain reference work checks metadata, not parent bytes.
No ESXi is needed or used.

Performance: 24 matched controls show **+0.99%, -1.08%, +3.85%, -3.34%** for base
metadata, contiguous 128 KiB reads, FLAT/ZERO copy+verify and RAW verify. The sparse
read pair **+5.0069%** triggers six longer runs: **-0.53%** aggregate, with
**-1.10%, -0.45%, -4.03%** pairs. The original remains visible. No repeat pair is
adverse above 5%; this does not close earlier shared-host investigations.
Eighteen first chain runs establish **0.607 µs child parse**, **10.110 / 41.523 /
178.604 µs embedded open** for 1/4/16 layers, **0.154 µs 16-layer endpoint recheck**
and **109.494 µs depth rejection** using external entries. All run medians remain
visible, including the 253.378 µs third 16-layer-open run. These memory-backed costs
include validation/allocation/drop, not fixture generation or logical parent reads.
No storage throughput or causal speedup is claimed. All plots reproduce identically;
prior evidence remains unchanged. PERF.0/R4.4 and previous adverse follow-ups stay open.

## R5.7 — Read-only logical sparse parent fallback

Date: 2026-09-30. Status: **Complete within the admitted sparse-chain library scope**.
Baseline: `5436dbc`, initially clean. The enclosing commit records this step.
[Contract](vmdk-chain-disk.md), [ADR-0043](adr/0043-read-only-sparse-parent-fallback.md)
and [evidence](benchmark-results/2026-09-30-r57/README.md) retain implementation
policy, all 72 measured runs, source/harness/binary identities, validation and plots.

`SparseChainDisk` wraps owned validated metadata with read-only VirtualDisk. An
iterative walk clips at every consulted grain/extent boundary and resolves nearest
allocated ancestor data or base-confirmed Zero. Allocated zeros remain Data; errors
never fall back to older contents. Physical coalescing stays inside one retained
backing map. Reads use constant auxiliary space and reuse caller buffers; two-pass
logical extent queries enforce a separate default 65,536 output limit. Whole-chain
destination alias checks include every descriptor and backing, including hidden
ancestors. Copy preflight revalidates; standalone callers keep sources quiescent.
Metadata admission/base APIs remain unchanged. CLI parent exposure is still pending.

Validation: **524 distinct passed, one existing gated allocation test** (525 total),
plus a 160-test CLI/VMDK run with integration fixtures on Btrfs; five existing CLI
unit fault tests retain system temporary files. Formatting, strict all-target
Clippy and portable core/datamover/VMDK wasm32 checks pass (existing control::sum
warning). Twelve new tests cover nine grain combinations × 38 ranges, independent
byte/kind oracles, split boundaries, allocated zero overrides, 1/4/16-layer fallback,
coalescing, range/output limits, all source aliases, changed endpoints, short reads,
EOF/errors, completed prefixes, retained handles and four-worker copy/verify.

Qualified QEMU monolithic/split/mixed three-layer chains match independently edited
RAW overlays and QEMU full decoding/compare, including two-file 2 GiB + 64 KiB.
The helper reads every byte in 65,537-byte chunks; sources remain unchanged and
public CLI still rejects parents. Metadata reference and all four base CLI commands
also pass their prior qualification. Example acquisition now shares one confined
fixture helper, separate from future public CLI policy. No SDK/producer source or
ESXi is used.

The original large-case partial cross-split write **failed** its independent oracle:
rvddk and QEMU agree, but both return 65,024 bytes of 0x31 instead of retained 0x61
at offset 2,147,484,160. Post-failure QEMU reads show base/intermediate still match;
only the final layer differs. The original generator, failure output, invocation,
source/output hashes and diagnostic commands are preserved. A separate qualified
case fully overwrites that second-extent grain. The original producer partial-write
path remains unqualified; smaller cases retain partial writes. No reader adjustment
or claim about the producer's internal cause is made.

Performance: 24 matched controls show **-0.88%, +1.20%, +0.86%, +1.20%** for base metadata,
contiguous 128 KiB reads, FLAT/ZERO copy+verify and RAW verify. Individual adverse
pairs **+5.81%, +24.27%, +13.71%** trigger 18 longer alternating runs:

- `sparse_metadata/monolithic`: -0.62% aggregate; pairs -6.68%, +7.75%, -0.62%.
- `sparse_disk/read_contiguous_128k`: +1.51% aggregate; pairs +1.51%, -4.59%, +13.94%.
- `cli_transfer/verify_only`: +1.49% aggregate; pairs +2.01%, +0.20%, -0.72%.

All original and adverse runs remain visible; no pooling or causal speedup claim.
Earlier PERF.0/R4.4 and adverse investigations remain open. Thirty new synthetic
runs establish 1 MiB inherited reads at depths 1/4/16 of **52.005 / 54.608 / 58.974 µs**,
and full queries of **0.292 / 3.713 / 14.947 µs**. Two-layer leaf/alternating/
fragmented/zero reads take **51.462 / 55.163 /
56.304 / 16.787 µs**. Metadata acquisition, fixture
creation and buffers are outside timing. These memory-backed mapping/copy costs
are not storage throughput. Plot regeneration is byte-identical; earlier evidence
is unchanged. Depth/boundary scanning remains visible for future measured tuning.

## R5.8 — Confined CLI sparse parent chains (2026-09-30)

Completed in this step's commit; baseline `6f8f317`.
[Contract](cli-vmdk-parents.md), [ADR-0044](adr/0044-confined-cli-parent-chains.md),
[evidence and plots](benchmark-results/2026-09-30-r58/README.md).

All four commands accept `--format vmdk --allow-parents`, loading sparse-only
chains through the bounded library. Parent hints must be basenames in the pinned
source directory. Every descriptor/backing retains timestamp/identity/size
observations and participates in destination alias protection. The existing
portable copy, logical verification, cancellation and publication rules apply.
Default base-only acquisition is unchanged. Preview reports expose layer/CID/
identity provenance and loader budgets separately from entry probes. External
monolithic mirrors are accepted in chain mode under library binding checks.

Validation: **537 passed, one existing gated test** (538 distinct), clean Clippy
and formatting, portable core/datamover/VMDK check with its existing warning.
Thirteen new CLI tests cover inherited data/allocated zeros, all commands and
source families, ancestor alias/mutation protection, confined paths, depth 16/17,
malformed chains, mirrors/container binding, budgets, cancellation and publication.
A separate **173-test** CLI/VMDK run passes with integration fixtures on Btrfs;
five existing CLI unit fault tests retain system-temporary fixtures. Independent
release builds and validation targets are retained in the evidence.

QEMU-generated mono/split/mixed/multi-file split three-layer chains pass all four
commands. Full copied bytes match independent RAW overlays and QEMU decoding;
QEMU compare passes; source hashes are unchanged. The largest disk is 2 GiB +
64 KiB. Default-mode inspection still rejects all four parent leaves. The prior
partial second-extent producer discrepancy and failed evidence remain unchanged;
this step uses the separately qualified full-grain replacement for that large
case and does not qualify the original producer path. No SDK/producer source or
ESXi is used.

Performance: **30 matched runs**, **6 longer follow-up runs**, and
**15 initial parent CLI runs**. Main changes for FLAT/ZERO inspect, plan, copy,
RAW verify and base sparse copy are **+6.10%, -1.02%, +1.24%, +2.34%, -0.50%**. Every adverse aggregate
or individual pair above +5% triggers longer alternating pairs; all runs remain
visible, without pooling or causal speedup claims.

- `cli_vmdk/inspect_mixed`: -2.16% aggregate; pairs -2.16%, -3.07%, -0.82%.

Initial end-to-end parent costs (1 MiB, three layers; admission included):

- `cli_parents/inspect_mono`: 0.176 ms median across three run medians.
- `cli_parents/plan_split`: 0.164 ms median across three run medians.
- `cli_parents/copy_mono_verify`: 19.930 ms median across three run medians.
- `cli_parents/copy_split_verify`: 19.930 ms median across three run medians.
- `cli_parents/verify_split`: 0.259 ms median across three run medians.

SVG/PNG regeneration is byte-identical. Source patch reconstruction, binary and
harness identities, sample medians, repeat triggers, reference hashes and local
links pass audit. PERF.0, R4.4 and earlier adverse investigations remain open for
a controlled runner; warm local timings are not physical storage throughput.

## R5.9 — Bounded admission fuzz qualification (2026-09-30)

Completed in this step's commit; baseline `aa09755`.
[Harness](../fuzz/README.md), [ADR-0045](adr/0045-bounded-admission-fuzzing.md),
[evidence and plots](benchmark-results/2026-09-30-r59/README.md).

Four isolated libFuzzer targets exercise descriptors, sparse headers, metadata
prefixes and structured parent admission. Authored seeds and memory-only sources
bound fixture/input growth; independent read/resolver counters check failure paths
as well as success. Small fuzz budgets complement deterministic production-limit,
split-layout and logical-read/reference tests. Shipping source, dependencies and
features are unchanged. No producer source, SDK or ESXi is used.

Validation: **537 workspace tests pass, one existing gated test**, plus **two
standalone harness tests**. Workspace and harness formatting/Clippy pass; portable
core/datamover/VMDK passes with its existing control::sum warning. Pinned fuzz
compiler/tool identities, lockfile, source and seed hashes are retained separately
from the unchanged stable production workspace.

Four five-second smoke runs and twelve independent 30-second ASan campaigns pass.
Formal campaigns execute **62,446,303 inputs**, with maximum reported RSS **186 MiB**,
under fixed input/read/reservation/depth and process guards. No crash, timeout,
OOM, sanitizer or invariant failure was reported. Every run, learned input and
feedback/resource log is retained; no run was discarded. Clean finite campaigns
are not proof of exhaustive safety or source coverage percentages.

Performance disposition: no production runtime changes, so no matched disk-runtime
comparison is attributed to this step. Three runs per target establish sanitized
harness execution/RSS/feedback baselines, with all values in reproducible SVG/PNG
plots. These rates include scaffolding and input-dependent rejection, and are not
storage throughput or comparable-workload speedups. Source/archive/sample audits
and byte-identical seed/plot regeneration pass. Prior adverse timing evidence,
PERF.0 and R4.4 controlled-runner work remain open.

## V0.1 — Independent export design and live discovery (2026-10-01)

Completed in this step's commit; baseline `134ffdf`.
[Design](vmware-access-plan.md),
[ADR-0046](adr/0046-independent-export-feasibility.md),
[evidence and plots](benchmark-results/2026-10-01-v01/README.md).

The authorized read-only probe confirms standalone ESXi 8.0.3 build 24677879,
VMFS 6 and two running Fedora VMs with 30/60 GiB persistent disks. Tools run on
both guests; no snapshots, backing parents or encryption keys are reported.
Available free-license metadata does not establish active assignment or export
permission. No guest power, files, snapshots, leases or licensing were changed.
Credentials/cookies remain in process memory; private host and inventory identities
are omitted from artifacts. The observed first-contact certificate pin provides
TOFU, not independently verified host identity.

Seven probe tests pass, covering response bounds/encoding, secret-safe faults,
pin-before-HTTP, forbidden methods/redirects, pagination/missing properties and
logout after discovery failure. Three recorded final-source sessions make 42
successful requests and all log out. An earlier exploratory discovery session
also logged out; it is not part of the recorded timing dataset.

ADR-0046 selects powered-off HTTP NFC export as the first candidate. It separates
sequential encoded artifacts from random logical-block access and keeps compressed
streamOptimized decoding, snapshots, CBT, restore and vCenter as distinct gates.
V0.2 implements the Rust session foundation; V0.3 proves export bytes and cleanup.
The user authorized VM shutdown if needed, but V0.1 required none.

Performance disposition: production Rust code/dependencies are unchanged. Retained
request timings include fresh TLS per request and parsing; they are not disk
throughput or a matched speedup. No before/after regression threshold applies to
these initial observations. All runs are plotted and reproducibly regenerated.
No new workspace Rust test run is claimed for this documentation/probe step.
PERF.0, prior adverse timings and the QEMU partial second-extent discrepancy remain open.

## V0.2 — Bounded independent Rust vSphere discovery (2026-10-01)

Completed in this step's commit; baseline `b5c5cc9`.
[Contract](../crates/rvvdk-vsphere/README.md),
[ADR-0047](adr/0047-bounded-rust-vsphere-discovery.md),
[evidence and plots](benchmark-results/2026-10-01-v02/README.md).

The new `rvvdk-vsphere` crate authenticates to direct ESXi 8.0.3 / HostAgent API
8.0.3.0 and reads typed, bounded inventory. Pinned TLS verifies certificate and
handshake signatures before HTTP. No redirects, proxies, compression or request
retry policy; no SDK/VDDK or Python runtime. Existing local disk crate source and
package versions are unchanged. The isolated qualification example keeps password
input in the terminal and operational identities out of serialized evidence.

The first live session exposed roxmltree's namespace-insensitive attribute shorthand:
`attribute("type")` also matched schema `xsi:type` on an array. Discovery failed
closed and logged out. The retained failed-source archive/result identifies that
attempt; exact unqualified-attribute matching and realistic array/reference fixtures
fix it. No raw server XML or credentials were retained.

Validation: **558 workspace tests pass**, one existing ignored test; formatting and
workspace Clippy with warnings denied pass. **21 new tests** include actual local
TLS, wrong pin before HTTP, redaction, escaping/namespaces, XML/response/object/disk
limits, forbidden redirects, truncation, authentication and cleanup errors,
continuation cancellation, deadlines and failed Logout after successful inventory.
The seven doctest suites contain no examples executed by cargo; no doctest count
is added. Child-process test summaries are excluded from the unique test total.

Final-source live qualification records six complete discovery sessions (84 calls)
and one deliberate inventory-limit failure (5 calls), all with confirmed Logout.
Including the retained initial failure, all eight attempted sessions confirmed
Logout. Both Fedora VMs remain running with Tools active and unchanged 30/60 GiB
disk topology. License availability is consistent with the free edition; active
assignment and export eligibility remain unresolved. No power, snapshot, export,
guest-file or license changes occurred, and no trial was requested.

Performance disposition: three alternating same-binary fresh/reuse pairs measure
identical complete discovery work. Median elapsed time falls from **7.696 s to
2.424 s (-68.51%)**; individual changes are -75.07%, -66.51%, -69.68%. No adverse
elapsed pair/aggregate exceeds +5%, so longer repeats are not triggered. Certificate
checks fall from 14 to one per successful session. CPU and process lifetime RSS
are retained; peak paired-process RSS is 5,152 KiB. This supports connection reuse
for this control workload, not disk throughput or a Python/Rust speedup. Network
and host/guest load are uncontrolled; all samples remain visible. The failed first
contact is correctness evidence, not a comparable completed benchmark.

## V0.3.1 — Rust export foundation and license gate (2026-10-01)

Completed in this step's commit; baseline `192489b`.
[Contract](../crates/rvvdk-vsphere/README.md),
[ADR-0048](adr/0048-bounded-export-lease-proof.md),
[evidence and plots](benchmark-results/2026-10-01-v031/README.md).

The Linux qualification executable implements bounded single-disk lease acquisition,
progress, manifest-verified streaming, Complete/Abort, Logout and durable no-replace
artifact publication. A shared session driver retains the discovery report schema.
VM/disk identities are revalidated; optional shutdown is graceful only. No SDK/VDDK,
new Python VMware code, local disk engine changes or public CLI integration.

Validation: **574 unique workspace tests pass**, one existing ignored test; format
and workspace Clippy with warnings denied pass. Sixteen new tests cover eligibility,
owned-lease abort, streamed bytes, manifest and endpoint rejection, shutdown identity
changes/failure, transfer bounds/truncation/redirects, cancellation, heartbeat renewal,
legacy SHA-1 with SHA-256 TLS trust, ambiguous acquisition, cleanup failures and
publication collision/race. The crate totals 37 tests. Final review also restores `Send` after shared-driver
type erasure, with a compile-contract test. Timings retain the preceding build;
final machine instructions/read-only data/relocations match byte for byte, with
only diagnostic line values and build ID differing in allocated sections. Fixture bytes are authored
synthetic data, not a valid VM image or logical-byte equivalence proof.

The live probe selects/revalidates the single 30 GiB VM and calls ExportVm without
changing power. The server returns a license restriction; Logout succeeds. No lease
was granted, no guest was shut down, and no disk bytes were transferred. The user
was told that supported trial/commercial access is needed now that the executable
and fixtures are ready. We did not change licensing. Both VMs remain powered on in
subsequent discovery observations. Active license assignment/expiry remains unresolved.

Performance evidence compares the previous committed release discovery binary with
the new shared-session implementation, using identical 14-call reused-connection
sessions. The initial aggregate improves by 7.14%, but one individual pair worsens
by 10.41%, triggering three longer alternating pairs. All raw sessions, including
adverse observations, remain in the evidence directory. Longer aggregate change is
+1.96%; pair medians +4.36%, +1.62%, -0.47%, but individual matches reach +18.35%.
The individual adverse gate remains open; no regression clearance or speedup is
claimed. All 54 discovery sessions (756 calls), plus the 16-call license probe,
confirm Logout. A controlled TLS workload is the next performance diagnostic.
This is a control-plane regression check. Live transfer throughput, CPU/RSS and
sync-bound performance remain unqualified until licensed access is available.
PERF.0/R4.4, prior investigations and the QEMU partial second-extent discrepancy stay open.

## V0.3.2a — Powered-off probes and task diagnostics (2026-10-01)

Completed in this step's commit; baseline `5e5e91b`.
[ADR-0049](adr/0049-powered-off-export-probes.md),
[evidence](benchmark-results/2026-10-01-v032a/README.md).

The user reported updating licensing on the existing ESXi 8.0.3 host. Discovery
now lists an Enterprise edition; active assignment and successful export remain
unresolved. The replacement/vSphere 9 deployment proposal is superseded for this
proof. We made no license changes.

Two powered-on eligibility probes returned faults and correlated with two running
export tasks. The first report classified a generic SOAP fault; the second typed
TaskInProgress but incorrectly marked acquisition rejected. Both original reports
are retained. Final code treats TaskInProgress cleanup as unconfirmed and refuses
to call ExportVm from a powered-on probe. Read-only, bounded recent-task inspection
helps diagnose uncertainty without changing power or canceling other tasks.

The user canceled both tasks. Subsequent inspection confirmed both terminal and
canceled. A final-source powered-on probe emitted invalid_power_state/not_acquired,
made no ExportVm or shutdown call, and confirmed Logout; the following inspection
found the same two canceled tasks. Final discovery observes both VMs powered on.
No VM power, guest data or artifact changed in this step. No CancelTask call was
made by rvddk. All retained live sessions confirmed Logout.

Validation: **580 unique workspace tests pass**, one existing ignored test;
formatting, workspace Clippy with warnings denied and release example builds pass.
Six new tests cover the guard, task inspection/identity, bounds/types/duplicates,
redaction and timestamp calendar limits. The crate totals 43 tests. One initial
new assertion incorrectly matched a datastore info query as a task query; it was
corrected to match the Task object type. The failed test log is retained alongside
the passing full run.

No formal performance comparison: these are differing diagnostic operations, and
some calls overlapped builds/tests. Preserve all timings without interpreting them
as transfer throughput or an improvement. The transfer path is unchanged; the
ordinary discovery property sets remain unchanged. PERF.0/R4.4, V0.3.1's adverse
individual pairs and the QEMU producer discrepancy remain open.

## V0.3.2b — Live export compatibility and partial-transfer cleanup (2026-10-01)

Completed in this step's commit; baseline `dedce58`.
[ADR-0050](adr/0050-live-export-compatibility.md),
[evidence and plots](benchmark-results/2026-10-01-v032b/README.md).

Guest SSH access established an independent 8 MiB known-byte oracle on the selected
30 GiB VM: local bytes, guest file bytes and independently mapped raw guest sectors
agree. The selected guest was gracefully shut down under existing authorization.
The 60 GiB control received only read-only inspection and stays powered on.
Private inputs and credentials remain outside commits.

Live failures exposed bracketed opaque lease references and auxiliary non-disk
lease URLs. Bounded escaped references and explicit single-disk selection correct
both assumptions, including manifest selection. Reports now retain accepted body
bytes on failure; the Linux example measures CPU and peak RSS. Initial ambiguous
acquisition remains visible; its task reached terminal error before another probe.
Later acquired leases all acknowledged Abort and Logout.

Real Ctrl-C cancellation received 272,684,467 bytes. The final measured build then
hit its one-hour deadline after 2,544,547,134 bytes (3,603.675 seconds including
cleanup, 33.624 CPU seconds, 7,868 KiB peak RSS). Both discarded staging and
published nothing. All five full-mode attempts are plotted, including failures.
Final task inspection returned only a terminal error; three final discovery
sessions confirm the selected VM off and control on, all with Logout acknowledged.
No completed-transfer throughput or tuning improvement is claimed. The provisional
prefix is compressed version-3 streamOptimized, not full format validation.

Validation: **584 unique workspace tests pass**, one existing ignored; formatting,
workspace Clippy with warnings denied and release examples pass. Four new Rust
tests cover opaque references, escaped lease cleanup, auxiliary-file exclusion and
bounded disk selection. Three offline oracle tests pass. No Python VMware API
access, SDK/VDDK dependency or public CLI decoder expansion was introduced.

## V0.3.2c — Repeated LAN exports and independent byte qualification (2026-10-02)

Completed in this step's commit; baseline `4e57c87`.
[ADR-0051](adr/0051-qualified-powered-off-export.md),
[evidence and plots](benchmark-results/2026-10-02-v032c/README.md).

The user approved using VM02, formerly the untouched 60 GiB control, as the LAN
runner including qemu-img installation/private files. VM01 stays powered off.
The unchanged Rust binaries completed three sequential exports, each receiving
2,727,380,992 encoded bytes for the 30 GiB disk, verifying its manifest, acknowledging
Complete/Logout and publishing the artifact. No LAN export attempt failed or was
omitted. Prior remote-connection cancellation/deadline evidence remains intact.

Whole-operation times: 238.611, 230.813 and 231.192 seconds. Encoded throughput:
10.910, 11.280 and 11.262 MiB/s, median 11.262. CPU: 46.266, 44.463 and 44.447 seconds.
Peak RSS: 8.188, 7.688 and 8.078 MiB. The one-vCPU runner shares ESXi host/storage
with the source. These are setup observations, not a code speedup or regression
clearance. Decode/hash work occurs outside measured exports.

QEMU fully decoded run 1 to a 30 GiB sparse RAW; every independently mapped 8 MiB
guest-fixture byte matched. The native CLI explicitly rejects version 3. Encoded
digests differ on repeats; the run-2 identity shortcut was rejected and retained.
QEMU compared complete decoded logical contents of runs 2/3 against run 1 and
confirmed equality. This carries the known-range proof across runs; it does not
prove independent whole-source equivalence. Private images/digests/credentials
remain outside Git; public reports redact only per-image digests.

Final task inspection returns a successful export task, no running/queued task in
the returned history, and Logout. Three final discovery sessions confirm source
off / runner on and Logout. No staging residue or new snapshots remain. The
verified temporary RAW and run-2 disk were removed for space; private encoded
runs 1/3, reports, fixture and runner tools remain for continuation.

No production Rust source changed; this step reuses the 584-pass/one-ignored
baseline and its passing Clippy result. CLI release build, three offline oracle
tests, incomplete-export rejection, full-decode/reference-comparison helper modes,
plot validation and visual inspection pass. V0 closes for this bounded export-only
workflow, combining complete/known-byte evidence with V0.3.2b's real failure cleanup.
Native compressed decoding, R6 integration/recovery and all prior performance
follow-ups remain open.

## R5.10 — Bounded streamOptimized metadata envelope (2026-10-02)

Completed in this step's commit; baseline `f756fb1`.
[ADR-0052](adr/0052-bounded-stream-envelope.md),
[contract](vmdk-stream-admission.md),
[evidence and plots](benchmark-results/2026-10-02-r510/README.md).

Separate Rust descriptor/header/marker/envelope types admit the specified clean
version-3, 64 KiB-grain base subset with QEMU front directories or VMware footer
markers. Bounds cover arithmetic, geometry, descriptor acquisition and exact read
budgets. Existing public format enum/parsers/CLI preserve their supported subset.
No grain directory entries, tables, global record order or payloads are validated;
these APIs provide no logical reads. Header/length rechecks require quiescence.

The retained live ESXi export passes with just 3584 metadata bytes read. Initial
admission rejected its informational `ddb.toolsInstallType`; the final grammar
accepts that key only for stream descriptors. The failure and regression test are
retained. No new vSphere calls, power transitions or export leases were needed.
Private guest artifacts and image digests stay outside Git.

Workspace validation: 594 passed, one ignored; ten new stream tests; Clippy with
`-D warnings` and release builds pass. Six QEMU/authored front/footer fixtures
independently decode to the original RAW bytes and pass Rust envelope admission.
Unaligned capacity fails; the current CLI rejects all seven compressed fixtures.

Performance: three alternating paired runs per design followed by three longer
pairs, all samples retained. Initial public enum expansion was removed; the final
separate descriptor stores only its required fields. Final longer small-descriptor
pairs still regress +8.45%, +10.05%, +8.83%. This remains **needs investigation**,
not a cleared gate or an accepted correctness tradeoff. The shared host limits
attribution. New in-memory front/footer envelope medians are 3.953/4.150 µs;
header checks about 129–130 ns. No storage/decompression throughput or RSS claim.

## R5.10p — Descriptor comparison specialization and timing controls (2026-10-02)

Completed in this step's commit; baseline `05b5fba`, earlier reference `f756fb1`.
[All measurements, controls and plots](benchmark-results/2026-10-02-r510p/README.md).

The old/current flat parser has the same normalized instruction sequence, with
changed placement. An inline hint on fixed-key comparisons lets the optimizer
specialize constants without changing grammar, errors or resource limits. The
measured parser grows from 5628 to 8554 bytes; this code-size cost is explicit.

Three primary rounds and three longer rounds compare all four descriptor cases
against both prior builds. Longer small-descriptor changes against R5.10 are
−7.83%, −18.87%, −12.38%; against the earlier baseline +0.60%, −11.23%, −0.43%.
The repeated small-descriptor cost is recovered in this build. Initial oversize
adverse observations did not persist in longer repeats.

Stream measurements remain mixed: CPU0 longer medians exceed +5% for grain
markers, 1 TiB headers and footer headers. Header/marker instruction shapes remain
unchanged, but that does not rule out placement effects. Three further rounds on
CPU4 include an identical-binary control, which itself varies by more than 5% on
some cases. Candidate per-case medians there are below +5%, with individual adverse
pairs retained. Accept the scoped descriptor optimization; **no blanket stream
performance clearance**. PERF.0 retains controlled timing and layout investigation.
All 219 runs / 6570 samples and both core conditions remain committed.

Final workspace rerun: 594 passed, one ignored; Clippy across all targets, fmt,
release builds and seven QEMU/public-CLI cases pass. Initial workspace validation
failed one unchanged export-cleanup fixture (`Unconfirmed` versus `Aborted`). Its
exact focused rerun and full rerun passed. The failure is retained; cause remains
unproven. No cleanup timeout/policy was changed. No live VMware access was needed.

## R5.11a — Bounded stream grain index (2026-10-02)

Completed in this step's commit; baseline `64d81a7`.
[Contract](vmdk-stream-map.md), [ADR-0053](adr/0053-bounded-stream-grain-index.md),
[measurements and plots](benchmark-results/2026-10-02-r511a/README.md).

Split R5.11 into map validation (a) and bounded decompression/logical reads (b).
`StreamMap` reads bounded directories, validates every followed table region
before table I/O, then uses two table passes to count and fill a sparse index.
Optional front redundancy agrees byte-for-byte. Footer GT markers, physical
record sequence and exact LBA binding reject aliases, gaps and orphan records.
Payloads and grain padding are skipped. Public compressed CLI rejection persists.

Explicit limits cover requested map slot bytes, metadata reads, table work and
populated grains. Fourteen new tests exercise those bounds, read traces and
malformed inputs. An empty 1 TiB footer map needs 131,136 requested slot bytes
and no table work. These counters exclude envelope/stack/allocator overhead and
are not process RSS. Source quiescence is required; rechecks are not a snapshot.

Six synthetic QEMU/authored maps match independent enumeration, and QEMU decoding
matches authored RAW. Unaligned input and all seven public CLI inputs reject.
The retained 30 GiB ESXi export validates 57,295 grains, requesting 1,205,620
metadata bytes and 693,684 map slot bytes. No new export, lease, VM power operation
or VMware API access was needed. A second retained export also passes; its first
supplemental probe was interrupted by the driver closing SSH and was retried
successfully. All attempts are recorded; only aggregate live counters are saved.

Performance evidence retains three alternating stream-admission pairs against
`64d81a7`, three longer repeats triggered by adverse observations, and three runs
of nine new map/lookup cases. No decompression, storage-throughput or RSS claim.
New median admission costs are 6.768/6.920 µs for sparse footer/front maps and
188.810 µs for an empty 1 TiB footer map; sparse lookup is about 7 ns. Longer
grain-marker pairs remain +5.74%, approximately 0%, +9.29%. Normalized header/marker
instruction shapes match the baseline, with changed placement; this is not a
performance proof. Existing stream timing/PERF.0 follow-ups remain open.

Final workspace validation: 608 passed, one ignored; 14 new map tests included.
Clippy across all targets with `-D warnings`, formatting and release builds pass.
An initial Clippy duplicate-fixture-module error was corrected and retained.
The earlier export-cleanup test failure did not recur.

## R5.11b — Owned bounded native stream reads (2026-10-02)

Completed in this step's commit; baseline `ea94b1d`.
[Contract](vmdk-stream-reads.md), [ADR-0054](adr/0054-bounded-native-stream-reads.md),
[reference checks and plots](benchmark-results/2026-10-02-r511b/README.md).

`StreamDisk` retains the exact admitted physical source and implements read-only
logical reads and coalesced Data/Zero extents. It rechecks uncached grain prefixes,
requires exactly one checksummed zlib stream with exact input consumption and
64 KiB output, and rejects invalid framing/lengths/checksums. Native Rust
`miniz_oxide` core uses fixed state with default features disabled; no C/vendor
SDK or growing output buffer. One shared scratch slot and last-grain cache bound
memory across callers. Data reads serialize. Sparse holes need no decode I/O.
Explicit limits cover decode storage, requests, encoded input, grain work and
output extents. Cache replacement/revalidation invalidate keys before failures.

Nine focused tests cover independent stored/zlib fixtures, malformed and high-ratio
streams, resource bounds, cross-grain/zero reads, cache failure, partial backend
reads, alias/endpoint checks, large holes and concurrent callers. Final workspace:
617 passed, one ignored; Clippy all targets and formatting pass. The first
concatenation test accidentally exceeded the map's compressed-size bound; a small
second valid zlib stream now reaches the intended decoder check. The initial test
failure is retained. The prior export-cleanup flake did not recur.

Six complete synthetic images match authored RAW. Public CLI rejection of all
seven compressed fixtures remains. On the authorized runner, every logical byte
of the retained 30 GiB export matches QEMU, and all 8 MiB of the independently
mapped guest fixture matches. The temporary sparse RAW reference was removed
after comparison; no VMware API, export, lease or power operation was needed.
Only sanitized counters and the helper binary hash are retained in Git.

Three live runs per workload give median read-phase throughput of 152.957 MiB/s
for allocated-grain sequential reads and 6.009 MiB/s for 4 KiB random requests.
Per-miss random reads decode a full grain. Read-workload peak RSS is 3.11–3.36 MiB;
requested decode storage is 141,679 bytes. CPU/RSS include process startup and
map acquisition. Read throughput excludes admission, which varies 4.76–20.95 s
across these runs. Preserve the cache/order sensitivity; do not claim cold-cache,
whole-job or parallel-decode performance from these numbers.

Three paired rounds for four existing map cases, longer repeats on adverse >5%
observations, and three runs of seven new in-memory read cases establish controls
and baselines. All 69 runs / 2070 samples remain visible. Longer empty-1-TiB
map and lookup medians remain adverse at +5.17% and +6.70%; instruction shapes
match with changed placement, which is not performance proof. These and prior
stream timing/PERF.0 concerns remain open; read correctness does not clear them.

## R5.12 — Local CLI streamOptimized conversion (2026-10-02)

Completed in this step's commit; baseline `d65b098`.
[CLI contract](cli-vmdk.md), [ADR-0055](adr/0055-local-cli-stream-conversion.md),
[tests, references and plots](benchmark-results/2026-10-02-r512/README.md).

Explicit VMDK version-3 routing now admits only the native reader's bounded base
subset. The confined opened container is retained as the sole physical backing;
embedded filenames are never followed. Inspect/plan expose logical topology,
map/decode reservations and lazy payload validation. Copy/verify use logical reads,
portable execution and the established durable publication path. Explicit native
execution and stream blocks above 64 MiB reject before destination preparation.
Parent opt-in remains specific to hosted sparse chains.

Six new integration tests qualify both layouts, cross-grain reads, multiworker
copy, overwrite tails, source/destination aliases, symlinks, mutation, malformed
payloads, unsupported variants, cancellation and publication races. Workspace:
623 passed, one ignored. Final Clippy across all targets and formatting pass.
Initial Clippy rejected duplicate fixture `allow(dead_code)` attributes; removing
the redundant outer attributes fixed it. The failed check is retained. The earlier
export-cleanup test flake did not recur. Six full synthetic CLI conversions match
authored RAW and QEMU; unsupported unaligned capacity rejects.

The first retained-export conversion exhausted the runner's small XFS root during
Zero output. Cleanup removed private output; no publication occurred. The retained
export/oracle were copied to ignored development-host storage with enough space.
Three full CLI conversions and readback pass there. Every logical byte (30 GiB)
agrees with QEMU; every independent guest-oracle byte (8 MiB) agrees, and standalone
CLI verification passes. No new export, lease, VM power action or storage resize
was needed. Temporary RAW outputs were removed. Only sanitized aggregate results
are committed. Local Btrfs allocated 30 GiB in each round; R5.12p now precedes R6.1.

Median local whole-command time is 23.219 s, CPU 21.90 s and peak RSS about
7.3 MiB. The 1,323.03 logical MiB/s includes zero ranges and is not decoder or
network throughput; these local results are not matched against prior VM timings.
The matrix retains 78 runs / 2,340 samples and all adverse longer repeats. Existing
monolithic sparse copy/readback remains +6.48% by median, with round variation;
causality is unresolved. Six new stream CLI cases establish complete-command
baselines on tmpfs. No blanket performance clearance is claimed. Source quiescence and existing
PERF.0/R4.4/discovery follow-ups remain required.

## R5.12p — Space-efficient local zero output (2026-10-02)

Completed in this step's commit; baseline `0ed0a09`.
[Contract](local-sparse-output.md), [ADR-0056](adr/0056-space-efficient-local-zero-output.md),
[qualification, samples and plots](benchmark-results/2026-10-02-r512p/README.md).

Linux regular-file zero output now tries punching first, then zero-range when
punching is explicitly unsupported, then bounded writes when both kernel modes
are unsupported. The punch support bit is shared with discard; one access guard
covers every attempt. EINTR retries the same mode; real errors propagate, including
ENOSPC and partial effects. Source Zero/Hole labels and copy counters are unchanged.
No new-file write omission, general device-discard assumption or extra worker
scratch state is introduced.

Workspace: 625 passed, two filesystem-specific tests ignored by default. All four
storage integration tests pass explicitly on Btrfs and the unchanged XFS runner.
Both show 8 MiB populated files falling to 2 MiB allocation after zeroing the middle
6 MiB, and fresh sparse files retaining 8 KiB with partial-block sentinels intact.
Final Clippy across all targets and formatting pass. Six synthetic CLI conversions
match authored RAW and QEMU. The earlier export-cleanup flake did not recur.

The matrix retains 72 runs / 2,160 samples across eight unchanged RAW/FLAT/sparse/
stream cases, longer adverse repeats, historical and identical-binary controls.
Sparse/mixed-zero tmpfs copy/readback medians improve 12.80–54.74%; dense/RAW cases
remain close to baseline. Two initially adverse data-only rounds trigger longer
measurements; all longer pairs remain below +5%. Prior monolithic sparse +6.48%
regression does not reproduce: historical pairs −0.44%, −1.51%, +0.80%; identical
executable pairs −2.63%, +1.87%, −0.69%. Preserve the older result without claiming
that this proves a cause. Frequency/load snapshots and SMT limitations are retained.

The full retained-export before/candidate measurements and oracle checks are in
the linked report. Local Btrfs output falls from 30 GiB to 3.497 GiB. Initial
copy-only pairs remain +11.33% by median; three extra pairs remain +8.28%.
Copy/readback is +2.78% by median. A separate strace diagnostic finds 22 successful
fallocate calls per binary, with lower candidate syscall CPU in that observation;
it does not explain end-to-end latency or delayed filesystem work. Do not infer
a cause. R5.12q now precedes R6.1a to investigate the full-copy tradeoff.
On the unchanged XFS runner, all six candidate conversions and full readbacks
pass; the first output also matches all 30 GiB against QEMU and all 8 MiB against
the independent guest oracle. Every XFS output allocates 3,754,889,216 bytes;
median copy/copy-readback elapsed is 60.628/117.311 s. This resolves the prior
ENOSPC outcome for the qualified image without a filesystem resize. Local and
runner timings are separate. Temporary RAW outputs were removed and SSH closed;
no VMware API, new export, lease or VM power change was needed. Private inputs
stay ignored and only aggregate observations are committed.
Allocation and runtime have separate meanings; fallback still provides no
universal output-space guarantee.

## R5.12q — Retained-export full-copy latency investigation

Completed with no production Rust changes. [Report, raw measurements and plots](benchmark-results/2026-10-02-r512q/README.md)
retain 28 full copies/readbacks, six native source-only controls, CPU/RSS/allocation,
physical extent counts, phase events, host observations and executable hashes.
The fixed private image and exact R5.12p executables are unchanged; all 28 full
native readbacks pass. No VMware/guest action or XFS change was needed. Temporary
RAWs are removed. Existing independent QEMU/guest and workspace qualification is
retained, not presented as newly executed tests.

Six alternating untraced policy pairs on CPU 0 give +7.98%, +1.68%, +4.68%,
+12.64%, +8.15%, +6.50% copy time (median +7.24%). Initial median +4.68%; extra
three-pair median +8.15%. Keep the earlier unpinned +8.28% evidence separately.
Median engine flush increases 0.202 → 0.961 s, while transfer is 6.198 → 6.014 s
and CPU 6.400 → 6.375 s. Subsequent file/directory sync remains small. Sparse
output retains 3,754,885,120-byte allocation versus 30 GiB; physical extent count
is 10,047–10,079 versus 116–133. This associates the policy/layout with extra
flush cost; it does not establish a Btrfs cause or a portable fix.

Same-executable pairs −0.20%, −2.49%, +1.51% and progress on/off pairs +4.88%,
−1.03%, +0.61% preserve observed variation. Two separately traced pairs show
similar destination pwrite wall time and longer sparse fdatasync, with 22 cheap
successful fallocate calls. Native probe median map admission is 0.0418 s and
allocated-grain read/decode 5.630 s. No subtraction is claimed as pure decoder CPU.
The shared host retains powersave and an unisolated SMT sibling; all cache/storage
conditions and tracing limitations are recorded. Builds/tests/plotting do not
interleave the timed matrix. Plot generation initially found no matplotlib in
system Python, then passed in the existing plotting environment. Audit validates
all 28 outcomes, phase sums, allocations, six read controls and four syscall logs;
three figures are saved in SVG/PNG. Only sanitized aggregates are committed.

Retain punch-first zero semantics, bounded memory, fallback/error handling,
cancellation, exact-range overwrite and all publication/durability barriers.
PERF.0 now has an explicit controlled-storage layout/writeback follow-up, with
bounded Data-range reservation as an experiment requiring independent correctness,
allocation and Btrfs/XFS qualification. No allocating Zero default or omitted sync
is accepted as a latency fix. R5.12q closes the bounded investigation, not the
remaining performance tradeoff.

## R6.1a — Explicit source identity and export artifact contract

Completed [ADR-0057](adr/0057-source-identity-and-artifact-contract.md) and the
[contract](export-artifact-contract.md). `rvvdk_vsphere::contract` provides bounded
runtime endpoint/pin provenance and VM reference/UUID/disk key/backing selection,
reusing the existing single-disk scope gate and requiring powered-off state.
Equal capacity, display names and inventory position cannot select the source.
Private disk identity accessors preserve existing diagnostic redaction.

The strict 4 KiB v1 artifact schema separates logical capacity from encoded length,
records completeness/validation claims and uses a stable domain-separated source
binding. No paths, credentials, URLs or lease handles are persisted. Explicit
serialization retains private content/source hashes; Debug/errors redact them.
Parsing and comparisons grant no ownership, authentication, resume or cleanup
capability. The existing capacity-selected qualification harness stays unchanged
until R6.1c replaces that selection in the production workflow.

[Evidence and plots](benchmark-results/2026-10-02-r61a/README.md): 635 workspace tests
pass, two storage tests ignored by default; focused vSphere tests total 57. Ten new
contract tests cover mismatch, ambiguity, scope, malformed/bounded input, state
consistency, redaction and an independent synthetic golden binding. Clippy across
all targets and formatting pass. No live host or disk payload qualification was
needed because this package adds only the pure contract and explicit accessors.

Thirty synthetic Criterion case runs retain 900 samples, CPU/RSS and host snapshots.
Complete parsing is 0.654 µs, worst-sized identity construction 4.839 µs, encoding
3.476 µs by median of initial run medians. The two cases with over-5% spread receive
three longer repeats; 4 KiB parse spread becomes 2.83%, while precomputed binding
comparison retains 8.62% spread at 1.208–1.312 ns. No prior implementation exists,
so these are a new baseline, not a speedup or regression claim. Plot audit and
source/artifact hashes bind the evidence. No timed run overlaps builds/tests/plots.
Keep the older Btrfs flush and other PERF.0 work separate and open.

## R6.1b — Durable ownership and conservative recovery foundation

Completed [ADR-0058](adr/0058-durable-job-ownership.md) and the
[Linux ownership contract](durable-job-ownership.md). A private, exclusively locked
job store binds a random operation ID, artifact ID, expected source and store inode.
Bounded checksummed records use exclusive temporary creation, full write, file sync,
atomic rename and parent sync. Persistence uncertainty poisons the writer; surviving
transactions are retained, never replayed. Terminal records prevent artifact reuse.

The store creates and stamps a private stage and three fixed members before claiming
ownership. Cleanup persists intent, freshly checks inode/mode/link count and marker,
then removes only owned names nonrecursively. Unknown/replaced entries are preserved.
Reopening yields an assessment, not a live Job or lease capability. Remote request /
release and publication uncertainties remain unresolved. No credential, ticket URL,
usable lease reference, automatic resume or network action is introduced. Production
export/conversion wiring is R6.1c; the legacy qualification exporter is unchanged.

[Evidence and plots](benchmark-results/2026-10-02-r61b/README.md): 649 workspace tests
pass, two filesystem-specific tests ignored. Fourteen ownership tests pass on tmpfs
and Btrfs, including sixteen real child-process exits and staged I/O failures.
Clippy across all targets and formatting pass. An initial test exposed a transient
fork-inherited lock lifetime; explicit Drop unlock fixes it. Ten repeated suites
(140 executions) pass afterward. Store handles must not be shared across fork;
private trusted ancestors and cooperating writers remain explicit prerequisites.
These are process-loss tests, not physical power-cut qualification.

Twelve synthetic runs retain 576 jobs (ten journal commits each), all phase samples,
CPU/RSS, record allocation and host observations. Both filesystems receive longer
64-job repeats after initial over-5% spreads. Longer Btrfs phase medians: record
creation 12.925 ms, stage preparation 57.793 ms, five lease-observation updates
65.044 ms, reopen/recovery 77.65 µs, cleanup 39.080 ms. Btrfs recovery spread remains
12.1%; tmpfs spreads remain 3.8–10.0%. Largest record 743 bytes; terminal file allocation
4 KiB per job excludes filesystem metadata. Preserve all durability barriers and
measure this overhead during integration. No prior implementation exists, so these
are new baselines. No benchmark overlaps builds/tests/standalone plotting. No ESXi
or guest access was needed; synthetic fixtures are removed and private data excluded.

## R6.1c.1 — Explicit selection in the export proof

Completed the first bounded R6.1c package. `export_selected_vm` derives its pinned
connection from `SourceSelection` and selects by reference, UUID, disk key/backing
and expected capacity. Equal-capacity VMs are supported without fallback selection.
The Rust export example has an all-or-nothing explicit identity mode. Invalid
capacity/shutdown options fail before connecting; the new path requires powered-off
scope and preserves the existing capacity proof as a separate entry point.

Fresh observations after output admission, before download and before completion
check identity, topology, power and disabled ExportVm. Ready-lease reads follow a
progress refresh and are bounded by one third of the lease timeout, capped at ten
seconds and the operation deadline. Cancellation prevents the next action after
a property RPC; pagination cleanup is awaited. These observations do not exclude
remote races or make the current artifact writer durably owned.

[Contract](explicit-export-selection.md), [ADR-0059](adr/0059-explicit-export-selection.md),
[tests and matched benchmark plots](benchmark-results/2026-10-02-r61c1/README.md).
Nine new integration tests cover equal-capacity disambiguation, exact operation
ordering, 30 identity/state mutations at five boundaries, missing selection,
invalid options, cancellation at all three added checks, slow live-lease reads,
pagination cleanup, certificate rejection and probe cleanup. Workspace: **658
passed, three ignored** (two storage tests and one opt-in performance matrix).
All-target clippy and formatting pass. The final mock-server adjustment also
passes all 48 discovery/export integration tests.

Synthetic timing compares both paths in the same release executable, with
alternating per-pair order, byte checks, CPU/RSS/allocation and longer repeats for
adverse pairs. The original mock's TCP delay is retained as evidence alongside a
TCP_NODELAY control; see the report for measured overhead and remaining variation.
No VMware host, guest, power operation or private image is used for this package.
R6.1c as a whole remains open: this is still the qualification artifact lifecycle,
not production durable export, recovery, verified R6 metadata or conversion.

## R6.1c.2a — Durable acquire/abort integration

Completed a bounded part of R6.1c.2: `probe_owned_export` consumes a locked private
store and binds a real acquire/abort lease to an owned empty stage. One blocking
worker owns Job and all stage handles, with a one-slot serial command channel.
AcquireIntent is durable before fresh source revalidation and ExportVm. The live
lease remains process-local; LeaseHeld and AbortIntent must be durable before one
abort attempt. The abort response and its journal acknowledgment remain distinct.

Ready-lease heartbeats run during journal waits. Cancellation and progress failure
never abandon an accepted write; the worker is drained before return. Journal
failure blocks further remote mutation, while uncertain acquisition/abort and
post-intent cancellation preserve conservative records. No file writer escapes,
no cleanup occurs on Drop, and no recovery record creates a lease capability. The
empty stage requires explicit checked cleanup after the store is released.

[Contract](durable-export-probe.md), [ADR-0060](adr/0060-durable-export-lease-probe.md),
[tests and performance plots](benchmark-results/2026-10-02-r61c2a/README.md).
Workspace **676 passed, four ignored**; all-target clippy and formatting pass.
Eighteen new tests include RPC ordering observations, journal failures, lost replies,
SIGKILL at acquisition/abort boundaries, cancelled waiters, worker failure/panic,
and heartbeats through injected 1.3-second journal stalls. All 15 owned-probe tests
also pass with Btrfs fixtures. No physical power-cut qualification is implied.

The matched release matrix retains **576 probes / 288 pairs** on Btrfs and tmpfs,
including longer repeats after adverse observations. Seven durable commits per
owned probe add a substantial fixed cost. Longer median wall is 124.832 ms on
Btrfs versus 1.590 ms for the explicit proof; tmpfs is 4.087 versus 1.338 ms. Separate
cleanup costs 38.231 ms / 0.186 ms. Every owned payload/metadata file is empty,
file allocation is 8,192 bytes per stage+record, and checked cleanup reaches Cleaned.
No data-throughput claim or sync-barrier removal follows from this comparison.

No ESXi or guest operation was needed. This is an API-level durable probe, not full
payload export, completion, conversion, publication or resumable production recovery.
R6.1c.2 and R6.1c remain open; legacy exporters retain their separate lifecycle.

## R6.1c.2b — Owned transfer and conservative completion

Completed `transfer_owned_export`: bounded network batches are written through the
journal owner's retained payload handle. Accepted writes drain before return or
abort; cancellation cannot leave a detached writer. Manifest checks, file sync and
independent full digest/length readback precede TransferComplete. Reports separate
received, written and durable bytes, payload errors and remote/journal outcomes.

CompleteIntent is submitted only after source revalidation and byte verification.
The conservative guard starts before that submission: persistence errors, source
drift, cancellation and lost completion responses never cause an automatic abort
or retry. Completion acknowledgment and durable CompletedLease remain separate.
Successful output stays private; its metadata file is empty. Container-byte checks
do not establish VMDK structure or logical-disk validity.

[Contract](durable-owned-transfer.md), [ADR-0061](adr/0061-owned-transfer-and-conservative-completion.md),
[tests, raw timing and plots](benchmark-results/2026-10-02-r61c2b/README.md).
Workspace **690 passed, five ignored**; all-target clippy and formatting pass.
Fourteen added tests include one inert child-process helper. Coverage includes
ordered completion, bounded writes, independent readback tampering, cancellation,
source drift, slow worker operations, payload/journal failures, uncertain completion
and actual SIGKILL at the completion boundary. Nine transfer integration tests also
pass with Btrfs fixtures. An existing slow-journal test exposed a 300 ms mock idle
race; only slow unit fixtures now allow three seconds. Five exact repeats pass.
Production network timeouts and retry behavior are unchanged.

The matched release matrix retains **288 transfers / 144 pairs**, 8 MiB each, on
Btrfs and tmpfs, including longer repeats after adverse observations. Longer median
wall is **219.680 ms owned / 93.275 ms proof on Btrfs** and **78.489 / 67.040 ms on
tmpfs**. CPU is 41.896 / 24.541 ms and 34.479 / 23.455 ms respectively. The added
readback, eight journal commits and other lifecycle differences have substantial
combined cost; no durability barrier is removed. All bytes compare, each payload
allocates 8 MiB, and checked cleanup reaches Cleaned. These synthetic prefix/body
fixtures are not valid VMDKs or evidence of ESXi throughput.

No ESXi or guest operation was needed. R6.1c.2 is complete at this API/container-byte
scope; R6.1c remains open for metadata, native admission, conversion, publication
and integrated live qualification. Legacy exporters retain their separate lifecycle.

## R6.1c.3 — Private artifact metadata and native VMDK admission

Completed the explicit `transfer_owned_artifact` API. After container seal/readback
and before TransferComplete, the journal owner adopts a freshly checked read-only
payload into the existing native StreamDisk. It validates bounded stream metadata,
checks source logical capacity and decodes every present grain, then rechecks the
container SHA-256 against the sealed observation. Structurally absent zero regions
need no scan proportional to logical capacity.

The existing private ExportArtifact schema is written into the owned empty metadata
member during LeaseHeld, synced and independently reread/parsed. It records
ContainerDigestVerified, never LogicalReadbackVerified. No stage handle escapes,
metadata path is followed or remote authority recreated. Heartbeats/cancellation,
writer drain and conservative completion remain in force. The container-byte API
retains its separate scope and empty metadata behavior.

[Contract](owned-artifact-admission.md), [ADR-0062](adr/0062-private-native-artifact-admission.md),
[tests, raw samples and plots](benchmark-results/2026-10-02-r61c3/README.md).
Workspace **701 passed, six ignored**; strict all-target clippy and formatting pass.
Eleven added tests include an inert crash helper. They cover both stream layouts,
empty/present grains, malformed metadata/payloads, source capacity mismatch, native
limits, changed sealed bytes/member/marker, existing metadata preservation,
partial-write/sync/readback failures, cancellation/deadlines, slow native work and
SIGKILL during metadata persistence. Btrfs reruns pass seven unit and three new
integration tests. Completion observes metadata first; lost completion still stays
pending. Mock TLS now carries arbitrary binary bodies for valid authored VMDKs.

The matched release matrix retains **288 transfers / 144 pairs**, with 128 authored
64 KiB stored-DEFLATE grains, 30 GiB logical capacity and 8,528,384 encoded bytes.
Both paths use owned transfer; the artifact path adds native admission, another hash
pass and metadata durability. Longer wall medians are **218.729 → 230.586 ms on
Btrfs** (+5.42%) and **73.888 → 82.769 ms on tmpfs** (+12.02%). CPU rises 21.50% and
27.62%. All container bytes and decoded authored grains compare; trailing hole
samples are zero. The 441-byte metadata remains private. Adverse initial comparisons
and tmpfs variation triggered longer repeats; all raw observations are retained.
No timing, logical equivalence or production throughput claim follows beyond these
fixtures. Preserve all durability barriers and PERF.0 costs.

No ESXi/guest operation was needed. Metadata persistence was chosen during the live
LeaseHeld phase; the separately checked read-only retained-artifact capability is
explicitly assigned to R6.1c.4. Conversion, publication, composed live qualification
and production remote recovery remain open.

## R6.1c.4 — Retained artifact admission and confined local conversion

Completed `ownership::RetainedArtifact`: it consumes JobStore, admits only
CompletedLease/no transaction, checks source/store/artifact and stage/member/marker
identity, reads bounded private metadata and freshly validates native structure,
every present grain and current container digest. No stored validation claim skips
checks, and no Job, remote lease or source handle escapes the capability.

The capability retains its lock through local conversion. It drops its previous map
before re-admission, rejects aliases to all owned members/journal, and invokes the
existing controlled portable DataMover into an exact-sized caller-owned buffered
RAW file. Payload budgets, sparse-zero semantics, worker drain, progress/cancellation
and destination flush are preserved. Source content/identity and destination facts
are checked again before wrapper success. Source journal bytes never change.
Process loss leaves CompletedLease plus potentially partial caller-owned output;
there is no publication, checkpoint or inferred resume authority.

[Contract](retained-artifact-conversion.md), [ADR-0063](adr/0063-retained-artifact-local-conversion.md),
[tests, raw timing and plots](benchmark-results/2026-10-02-r61c4/README.md).
Workspace **711 passed, seven ignored**; all-target clippy and formatting pass.
Ten added tests include an inert process-loss helper. They cover lock lifetimes,
concurrent copy workers, dirty destination zeros, pending/stale/corrupt/foreign
resources, forged claims, changed source/metadata, aliases, budget/size failures,
cancellation/deadlines, post-flush drift and SIGKILL during conversion. A synthetic
TLS integration test exercises transfer through native metadata and retained RAW
conversion with full byte comparison. Btrfs reruns pass all ten new tests.

The matched release matrix retains **288 conversions / 144 pairs**: 64 MiB logical
capacity, 8 MiB authored present data, two table groups and 56 MiB zeros. All logical
bytes (18 GiB total), encoded source bytes, source journal state and cleanup compare.
Outputs allocate 8 MiB each. Longer wall medians are **11.643 → 31.354 ms on Btrfs**
and **4.973 → 26.625 ms on tmpfs**; retained CPU is 24.690 / 26.487 ms. Repeated
native admission and three full hash passes add substantial cost; keep this PERF.0
follow-up. Both adverse comparisons triggered longer repeats. Baseline Btrfs CPU
spread remains 5.44% afterward; all observations are retained.

No ESXi or guest operation was needed. This local consumer and converter is complete
within its scope; the borrowed RAW destination is not journal-owned or published.
Actual output ownership/publication and composed live qualification remain open.

## R6.1c.5 plan recorded after R6.1c.4

Start **R6.1c.5 — Output ownership and durable no-replace publication**:

1. Define the durable output contract and resource owner before wiring publication.
   Keep export-container identity, metadata claims and converted RAW output distinct.
   A borrowed conversion destination from R6.1c.4 is not automatically owned by the
   journal. Choose explicit artifact/output IDs, binding, format and verification
   evidence; never derive paths or authority from untrusted metadata.
2. Design versioned journal transitions/capabilities for local output preparation,
   conversion, validation and publication. Retained CompletedLease admission does
   not reopen mutable Job or a lease. Preserve source locking and consume/release
   retained capabilities explicitly; do not reconstruct remote ownership from records.
3. Create/retain confined private output handles, reject source/destination aliases,
   preserve bounded memory, cancellation, worker drain and sparse zero behavior.
   Persist only checkpoints supported by actual data/metadata durability. Define
   independent logical verification separately from decodability and copy flush.
4. Implement actual descriptor-bound no-replace publication with file and directory
   durability, intent before external effects and acknowledgment afterward. Define
   source/destination filesystem and ancestor constraints, collision behavior and
   conservative assessment of uncertain rename/sync/acknowledgment outcomes. Never
   label an uncertain publication failed-and-safe-to-delete by assumption.
5. Fault-inject partial I/O, directory sync, collision, replacement, cancellation and
   process loss across output/publication boundaries. Keep explicit cleanup bounded
   to freshly checked owned resources. A completed engine callback is not wrapper
   success, publication or remote-recovery authority.
6. Benchmark complete phase/CPU/RSS/allocation and byte correctness with plots. Track
   cumulative admission/decode/hash/journal/flush cost, repeat adverse observations
   above 5%, and preserve the remaining baseline variation. Optimize only with an
   explicit lifetime/trust argument and matching failure tests.
7. Once the composed path is ready, use the authorized ESXi lab for new integrated
   qualification. Preserve existing power/guest controls and credential privacy;
   never replay uncertain remote completion/abort requests from journal claims.

Keep PERF.0 visible: stream timing, CPU/SMT/frequency/layout controls, map/cache
sensitivity, whole-grain amplification, Btrfs sparse-output layout/flush cost,
bounded parallel decoding and repeated retained admission/hash cost. Retain R4.4
and discovery/TLS work. VMFS sparse, seSparse, online snapshots, CBT, restore,
vCenter and vSphere 9 remain later qualified work. Commit every completed package
with tests, benchmark plots, architecture decisions and updated continuation notes.

## R6.1c.5a — Private owned RAW output and full logical verification

Split R6.1c.5 at its first durable prerequisite: implement separate output ownership
before actual publication. Consume the locked retained source into a private RAW
stage and a version-1 output journal; leave the source CompletedLease record intact.
Seven intent/acknowledgment states reserve, create, convert and verify. Member
identities, private permissions, operation marker and explicit bindings constrain
all accesses. Conversion uses the existing admission/alias/budget/zero/flush checks;
all RAW writers close before Converted. Full logical source/output readback includes
zeros, hashes RAW and checks the source again. Bounded private metadata is synced
and read back before Verified. Two fixed verification buffers add 2 MiB outside
copy/native-map budgets. No output writer or mutable source Job escapes.

Output assessment is read-only and returns no cleanup, resume, publication or remote
authority. Unknown journal versions/fields and inconsistent bindings fail closed.
Uncertain transactions and every stage are retained. A visible Verified after an
interrupted rename/acknowledgment remains a record observation; future admission
must check actual content and durability. Same-user hostile writes/offline rollback
remain outside the existing cooperating private-store model.

[Contract](owned-raw-output.md), [ADR-0064](adr/0064-private-owned-raw-output.md),
[tests, raw timings and plots](benchmark-results/2026-10-02-r61c5a/README.md).
Workspace **719 passed, eight ignored**; all-target clippy and formatting pass.
Eight added tests (including the inert crash helper) cover 35 journal boundary
fault combinations, cancellation/deadline, engine completion versus wrapper success,
changed RAW bytes/length/inode/marker/metadata, record corruption/binding/privacy,
collisions, stage/metadata interruption and actual SIGKILL at four boundaries.
All eight pass on Btrfs as well. The existing synthetic TLS transfer/conversion
test now also completes owned RAW conversion with an independent full-byte oracle.

The matched release matrix retains **288 conversions / 144 pairs** across Btrfs and
tmpfs, including longer repeats on both. All 64 MiB logical oracle comparisons pass;
outputs allocate 8 MiB. Longer wall medians are **31.421 → 221.464 ms on Btrfs** and
**26.909 → 80.366 ms on tmpfs**; owned CPU is **91.129 / 79.921 ms**. Seven output
journal commits, full logical readback and extra hashing add substantial cost; this
matrix measures their aggregate. Keep a phase-profiling PERF.0 follow-up. Longer
tmpfs owned wall/CPU spread remains **6.00% / 6.05%** after repeats. Private output
metadata is 607 bytes; process peak RSS is 24,204–32,900 KiB. All samples and plots
are retained; no barriers were removed.

No ESXi or guest operation was needed. RAW output remains private; R6.1c.5 as a
whole and production integration/recovery remain incomplete.

## Next session

Start **R6.1c.5b — Fresh output admission, durable no-replace publication and checked recovery actions**:

1. Admit an explicit artifact/output/source binding into a confined local capability.
   Freshly validate output record/stage/member/marker, canonical container contract,
   metadata, exact capacity and RAW digest/content evidence. An assessed Verified
   record alone grants no authority. Define whether source must still be retained;
   reject missing prerequisites rather than assuming ownership or rollback safety.
2. Define publication destination and ancestor constraints, same-filesystem policy,
   pinned parent descriptors, fixed/generated staging names and output bundle versus
   single-file contract. Keep RAW identity and private metadata coherent. Decide a
   journal schema/version transition explicitly; never reinterpret unknown states.
3. Persist publication intent before actual no-replace rename/link effects; sync
   required files and both affected directories, then acknowledge. Preserve existing
   destination content on collision. Track observed rename separately from durable
   acknowledgment, including process loss and cancellation after publication starts.
4. Implement read-only conservative assessment and explicit checked local cleanup.
   Revalidate resource identities immediately before mutation. Never automatically
   delete uncertain published destinations, foreign replacements or unstamped
   StageIntent members; define a bounded outcome for these cases. Do not infer a
   remote lease, mutable Job, resume point or replayable request from local records.
5. Exercise collision, foreign replacement, partial journal/metadata I/O, file/dir
   sync boundaries, cancellation and SIGKILL. Ensure every accepted worker/writer
   drains before publication, cleanup or lock release. Repeat filesystem checks.
6. Benchmark full publication cost, phase/CPU/RSS/allocation and byte correctness with
   standalone plots. Keep all samples and repeat adverse observations/spreads above
   5%. Profile the accumulated readback/hash/journal cost before proposing changes;
   no barriers may disappear without an explicit trust/lifetime argument and tests.
7. Continue **R6.1c.5c** with the authorized ESXi lab only after the composed path is
   ready: fresh export, owned conversion, verified publication and explicit cleanup.
   Preserve guest/power controls and credential privacy; uncertain remote complete/
   abort calls must never be replayed merely because a local record suggests them.

Keep PERF.0 visible: stream timing, CPU/SMT/frequency/layout controls, map/cache
sensitivity, whole-grain amplification, Btrfs sparse-output layout/flush cost,
bounded parallel decoding, repeated retained admission/hash cost, full logical RAW
readback and output journal durability cost. Retain R4.4 and discovery/TLS work.
VMFS sparse, seSparse, online snapshots, CBT, restore, vCenter and vSphere 9 remain
later qualified work. Commit every completed package with evidence and updated plans.
