# R0.1 — Structural plan validation and observer overhead

Date: 2026-09-28. Baseline: `b579dfb`. Candidate: baseline plus
[candidate.patch](candidate.patch), including the identical comparison harness.
The runtime change is shared structural validation before observed execution.

## Correctness

The first seven regression tests failed on the old implementation; evidence is
in [regressions-before.txt](regressions-before.txt). The candidate adds an eighth
case covering native observer dispatch. All 202 workspace tests pass, including
all eight new cases; formatting and strict Clippy across all targets pass.

Tests check error parity, untouched destination bytes, no callbacks on structural
validation failures, and rejection before the former larger-buffer panic. Existing
successful copy and observer tests remain green.

## Comparable workload

- Identical new `progress` benchmark in isolated baseline/candidate builds.
- 16 MiB dense or 50% sparse source; sparse Data/Hole runs alternate every 64 KiB
  (256 extents). One worker, one 64 KiB buffer, 4096-byte buffer alignment.
- Deterministic incompressible payload. Source and destination on the same NVMe,
  under Btrfs with compression, backed by dm-crypt; buffered warm source reads.
- Destination reset with deterministic incompressible bytes and flushed outside
  every timed iteration. Copy timing includes structural validation and flush;
  full output read-back verification follows outside the timer.
- Planning measured separately. Callback workload counts snapshots and consumes
  completed logical bytes. Dense emits 2 snapshots; fragmented emits 257.
- Initial matrix: 20 samples, 0.3 s warmup, 1 s measurement target, 3 independent
  runs per version. Order: baseline/candidate, candidate/baseline, baseline/candidate.
- Follow-up fragmented matrix: 40 samples, 0.5 s warmup, 3 s target, 3 runs per
  version. Order reversed: candidate/baseline, baseline/candidate, candidate/baseline.

This is a focused API/observer comparison, not sustained hardware qualification.
CPU frequency scaling, background workload, device caches, and filesystem behavior
were not controlled. CPU/RSS and operation-latency counters were not instrumented.
See [environment.json](environment.json) for environment, build and source hashes.

## Initial results

Values are medians of the three run medians. Negative change means lower elapsed
time. They are descriptive summaries, not statistical proof of a speedup/regression.

| Workload | Baseline ms | Candidate ms | Change in time |
|---|---:|---:|---:|
| `progress_copy_dense/counter` | 13.450988 | 13.371806 | -0.59% |
| `progress_copy_dense/noop` | 13.183852 | 13.424536 | +1.83% |
| `progress_copy_dense/unobserved` | 13.194744 | 13.440562 | +1.86% |
| `progress_copy_fragmented50/counter` | 12.007995 | 13.278321 | +10.58% |
| `progress_copy_fragmented50/noop` | 12.246439 | 12.694335 | +3.66% |
| `progress_copy_fragmented50/unobserved` | 12.655078 | 12.733005 | +0.62% |
| `progress_plan_dense/plan` | 0.000882 | 0.000872 | -1.24% |
| `progress_plan_fragmented50/plan` | 0.216921 | 0.212884 | -1.86% |

The fragmented callback result crossed the 5% investigation threshold. Rather
than drop that result, the entire fragmented copy matrix was repeated with longer
sampling and the opposite baseline/candidate order.

## Follow-up results

| Workload | Baseline ms | Candidate ms | Change in time |
|---|---:|---:|---:|
| `progress_copy_fragmented50/counter` | 16.037748 | 17.008008 | +6.05% |
| `progress_copy_fragmented50/noop` | 16.269010 | 16.853054 | +3.59% |
| `progress_copy_fragmented50/unobserved` | 16.357710 | 16.849200 | +3.00% |

Time varied substantially between independent runs: the fragmented callback
baseline ranged from about 14.5 to 16.5 ms, and the candidate from 12.6 to 17.1 ms.
The unobserved control also moved, despite having the same required validation
work before and after. Raw medians alone cannot isolate the cause.

For transparency, these are candidate-versus-baseline changes paired by repeat,
which preserve the nearby comparisons rather than selecting different runs through
aggregate medians:

| Workload | Pair 1 | Pair 2 | Pair 3 | Median paired change |
|---|---:|---:|---:|---:|
| `progress_copy_fragmented50/counter` | -13.38% | +3.14% | +6.68% | +3.14% |
| `progress_copy_fragmented50/noop` | -1.87% | +5.05% | +3.11% | +3.11% |
| `progress_copy_fragmented50/unobserved` | -2.84% | +2.62% | +3.00% | +2.62% |

## Performance disposition

**Accept the required correctness fix; performance qualification remains
provisional.** The additional source extent scan is necessary to reject stale
plans. In the initial matrix, fragmented planning itself took about 0.21 ms.
This is useful context, not an attribution of every measured timing difference.

The callback's aggregate follow-up change remains above 5%, while its paired
comparisons do not show a consistent increase above that threshold. All outputs
verified. The evidence is too variable to claim either a general regression-free
bound or an optimization win. Preserve both result sets and track a controlled-runner
repeat under PERF.0; do not omit validation to regain the unchecked baseline's
behavior. No executor defaults or tuning knobs were changed.

R0.1's structural correctness scope is complete. PERF.0 remains in progress for
stable storage qualification, the broader workload matrix, and sustained direct-I/O
results using the corrected flush boundary.

## Discarded harness attempts

Two tmpfs attempts failed when the initially sparse source map became dense before
copy execution. The cause was not established; Data-only expected-content reads
were insufficient to prevent it. These runs are excluded from accepted performance
comparisons. See [attempt 1](harness-attempt-1.txt) and
[attempt 2](harness-attempt-2.txt).

Attempt 1 also used a shared Cargo target directory, which permitted stale artifact
reuse between same-named packages in two worktrees. Accepted comparisons use
independent build directories and distinct executable hashes. All comparisons use
the final identical harness; no partial failed timing is mixed into the tables.

## Evidence

- [Initial estimates and raw samples](measurements.json).
- [Follow-up estimates and raw samples](followup-measurements.json).
- Raw console logs: `btrfs-{baseline,candidate}-{1,2,3}.txt` and
  `followup-btrfs-{baseline,candidate}-{1,2,3}.txt` alongside this report.
- [Source patch](candidate.patch) and [source/binary hashes](environment.json).

## Reproduction

Create two disposable worktrees at `b579dfb`. Apply `candidate.patch` to the
candidate worktree. Copy only these candidate files into the baseline worktree:

```text
crates/rvvdk-datamover/Cargo.toml
crates/rvvdk-datamover/benches/progress.rs
crates/rvvdk-datamover/benches/io_uring_direct.rs
```

The first two install the common new target; the third keeps the independent
direct-I/O harness correction identical. Do not copy `src/mover.rs` to baseline.
Compile each worktree using a different `--target-dir`:

```bash
cargo bench --offline -p rvvdk-datamover --bench progress --no-run \
  --message-format=json --target-dir /path/to/separate/build-directory
```

The `compiler-artifact` JSON identifies its executable. Run each executable with
a distinct `CRITERION_HOME` and the same dedicated Btrfs storage directory:

```bash
RVVDK_BENCH_DIR=/path/to/storage CRITERION_HOME=/path/to/run-results \
  /path/to/progress-executable --bench --noplot
```

For the follow-up, append:

```text
progress_copy/fragmented50 --warm-up-time 0.5 --measurement-time 3 --sample-size 40
```

Repeat in the orders above. Preserve `new/estimates.json` and `new/sample.json`
for each benchmark; the committed measurement files collect those records verbatim
inside run metadata. Recheck source hashes and workload metadata before comparing.
