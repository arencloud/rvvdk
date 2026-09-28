# R1.4 — Logical intent and invocation preparation

Baseline: `5e75d51` (R1.3). Candidate: baseline plus
[candidate.patch](candidate.patch). [Environment and source/binary fingerprints](environment.json)
identify the measured implementation. Existing benchmark harnesses are unchanged;
no executor defaults, buffer sizes, or queue settings were tuned.

## Correctness and scope

**284 workspace tests pass** (seven new), with formatting, strict all-target
Clippy, and core/datamover library compilation for wasm32-unknown-unknown.
The portability check is compilation, not runtime qualification.
[Validation commands](validation.json), [test output](tests.txt), and
[test inventory](test-inventory.txt) preserve these results.

Three preparation regressions fail on baseline and pass on candidate:
native strategy rejection, native mover alignment mismatch, and live backend
alignment mismatch. Baseline emits one observer callback before returning the
error; candidate emits none. Both retain rejection before payload mutation.
The live alignment test changes both source and destination capabilities, then
restores them and verifies a successful native copy with initial/final callbacks.
[Before results](before-regressions.txt) retain the expected baseline failure.

Four additional tests cover planning selection/provenance, unchanged logical
extent accounting/fingerprints across backend choices, portable Auto behavior,
and explicit-native rejection versus Auto fallback selection policy. Existing
endpoint identity/capability, stale plan, translated disk, trait object, worker,
native lifetime, sparse policy, and flush regressions continue to pass.

CopyPlan now separates private logical intent from planning selection/settings.
Each planned execution builds private invocation-scoped dispatch state bound to
the validated plan and endpoint borrows. Native strategy/options, native extent
plan construction, and current capability alignment checks precede observation.
No new payload mechanism or extra extent-vector copy is introduced. Existing
native payload timer/flush boundaries remain; the complete call is benchmarked.

Planning selection is historical. A different executing mover can use its current
native queue/read-window settings; reports retain the actual backend. Preparation
does not create a ring, reserve buffers, or establish snapshot consistency. The
descriptor compatibility evaluator still lacks complete request/runtime checks.
See [ADR-0027](../../adr/0027-portable-planning.md) for exact scope and limits.

## Method

Baseline and candidate use separate absolute Cargo target directories and
identical optimized harnesses. Eight cases run in adjacent candidate/baseline,
baseline/candidate, candidate/baseline pairs, with affinity to CPUs 2–6.
Each process uses 30 flat samples, 300 ms warmup, and a 2 s measurement target.
The longer fragmented-planning follow-up reverses the starting order and uses
30 samples, 500 ms warmup, and a 4 s measurement target.

Sources are warm deterministic data. File workloads use the recorded Btrfs/NVMe
storage mount, not /tmp. No global cache drops or host governor changes.
The governor remains powersave; background load is uncontrolled.

- RAW planning: buffered 1 MiB source/destination, 64 KiB blocks; Threaded and
  IoUring/QD8 choices. Includes current endpoint checks, extent query, selection,
  and logical plan construction. Destination bytes are checked unchanged.
- Portable planning: 16 MiB dense or 50% fragmented local source, 64 KiB blocks.
  The fragmented map has alternating Data/Hole blocks.
- Complete RAW copies: buffered 1 MiB, 64 KiB blocks, one-worker Threaded or
  IoUring/QD8/read-window4. Public copy_raw_with_report includes planning,
  preparation, payload execution, cleanup, and final flush in timing.
- Fragmented no-op observation: existing 16 MiB/50% sparse portable execution,
  one worker, 64 KiB blocks, prebuilt plan and no-op observer. Execution includes
  live preparation, callbacks, and final flush.
- Dynamic memory: 16 MiB, one worker, 64 KiB blocks, prebuilt portable plan,
  source/destination accessed through dyn VirtualDisk.

Copy destination reset/flush and full readback are outside the timed intervals.
Counters and byte equality are checked on every iteration. File sources use
incompressible data; the memory harness uses its existing deterministic pattern.
Process resource logs include setup/readback and adaptive iteration counts:
they are not per-copy CPU or memory-cost measurements.

## Results

Each aggregate is the median of three run medians; positive changes mean slower.
Individual paired changes are retained because changing host conditions can
make the aggregate differ from the median paired change.

| Workload | Baseline | Candidate | Change | Paired changes |
|---|---:|---:|---:|---|
| `preflight/plan_threaded` | 1.713 µs | 1.770 µs | +3.35% | +2.02%, +2.96%, +3.35% |
| `preflight/plan_native` | 1.793 µs | 1.821 µs | +1.55% | +3.29%, +0.98%, +4.18% |
| `progress_plan/dense/plan` | 1.741 µs | 1.764 µs | +1.33% | -5.26%, +1.75%, -5.66% |
| `progress_plan/fragmented50/plan` | 203.359 µs | 213.555 µs | +5.01% | +0.30%, +5.01%, -2.20% |
| `preflight_copy/threaded` | 6.713 ms | 6.710 ms | -0.04% | -0.18%, -0.04%, +2.69% |
| `preflight_copy/native` | 6.811 ms | 6.864 ms | +0.78% | -0.08%, -0.82%, +0.78% |
| `progress_copy/fragmented50/noop` | 12.106 ms | 12.640 ms | +4.41% | +3.95%, +4.41%, -7.61% |
| `portable_memory/workers1/dyn` | 1.680 ms | 1.650 ms | -1.79% | +8.32%, -1.79%, +2.61% |

The fragmented planning aggregate initially exceeded the 5% review threshold
(+5.01%). A longer unchanged-binary follow-up measured **210.402 → 210.292 µs
(−0.05%)**, with paired changes **−0.37%, −0.05%, +3.44%**. It did not reproduce
the initial aggregate increase. Both experiments are retained:
[main samples/outliers](measurements.json), [main summary](summary.json),
[follow-up samples/outliers](followup-measurements.json), and
[follow-up summary](followup-summary.json), plus every console/resource log.

## Performance disposition

Accept the bounded implementation and measured RAW planning cost:
**+57 ns/+3.35%** for Threaded and **+28 ns/+1.55%** for native planning.
All three RAW planning pairs show small positive costs, consistent with the
additional planning metadata/refactored dispatch boundary; this is an accepted
tradeoff, not a claimed speedup.

Complete RAW copy medians change **−0.04%/+0.78%**. Fragmented no-op observation
is **+4.41%** by aggregate with mixed paired changes (+3.95%, +4.41%, −7.61%).
Dynamic memory is **−1.79%** by aggregate, but one pair is +8.32% and the others
are −1.79%/+2.61%. These variable results do not qualify all workloads or resolve
the earlier R1.2/R1.3 follow-ups. PERF.0 remains open for controlled-runner
qualification; no universal performance claim or clean qualification pass is made.

R1.3's endpoint inspection implementation is unchanged. R2 still owns runtime
io_uring setup, complete native request alignment/fallback, and ring reuse.
This change does not close contextual errors, partial statistics, memory budgets,
snapshot consistency, or durability guarantees.

## Reproduction

Check out detached worktrees for baseline 5e75d51 and candidate reconstructed
with candidate.patch. Use unchanged harnesses and separate absolute targets:

```sh
cargo build --release -p rvvdk-datamover --bench preflight --bench progress --bench portable_plan --target-dir /absolute/variant-target --message-format=json
```

Select executable paths from Cargo's artifact output. With a fresh CRITERION_HOME
for each run and RVVDK_BENCH_DIR on the chosen storage mount, run the exact case:

```sh
taskset -c 2-6 /absolute/benchmark-executable --bench --noplot --sample-size 30 --warm-up-time 0.3 --measurement-time 2 'preflight/plan_threaded$'
```

Substitute each case in the table and alternate adjacent variant order as above.
Use 0.5 s warmup/4 s measurement for the targeted follow-up. Exact commands are
also recorded per measurement. Preserve every result and the final source/binary
fingerprints.

To demonstrate the callback regressions, copy only
crates/rvvdk-datamover/tests/execution_preparation.rs into the baseline worktree and
run its integration test. Expected result: three failures showing one unexpected
callback. Remove that temporary file afterward. Do not apply candidate runtime
changes to this baseline test.
