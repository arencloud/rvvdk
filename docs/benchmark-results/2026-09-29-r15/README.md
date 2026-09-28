# R1.5 — Contextual failures and partial progress

Baseline: `8adfabd` (R1.4). Candidate: baseline plus
[candidate.patch](candidate.patch). [Environment/source/binary fingerprints](environment.json)
identify the measured implementation. All harnesses are unchanged. No execution
defaults, pool sizes, queue settings, or host governor settings were tuned.

## Correctness, migration, and limits

**294 workspace tests pass** (ten new), plus formatting, strict all-target
Clippy, and core/datamover library compilation for wasm32-unknown-unknown.
The latter is compilation, not runtime qualification.
[Validation commands](validation.json), [test output](tests.txt), and
[test inventory](test-inventory.txt) preserve the checks.

New tests cover sequential operation/range/errno and lower bounds, writes that
partially change the destination before failing, threaded/native flush failures,
worker counters aggregated after joining, uncertainty from a later failed worker,
native runtime EOF/short-write accounting and buffer return, sparse/fallback
failure counters, native cleanup chains, and typed plan rejection.

Existing worker-failure and sparse-policy assertions now inspect the retained
cause inside execution context. They still enforce first-cause preservation,
bounded shutdown, buffer return, no flush after failed payload execution, and
no retry of an advertised sparse operation via another capability. Engine
ownership and shutdown implementation are unchanged.

CopyFailure is a pre-release error-shape change within the existing Result API.
Original core and OS errors remain in the source chain. Success signatures,
counters, flush ownership, and callback cadence remain unchanged. Error counters
are confirmed lower bounds, not durable or contiguous progress. Failed opaque
backend calls and native cleanup may perform additional uncounted I/O. Native
completion errors without request identity retain attempted Data-range context.
Panics/process aborts are not converted to CopyFailure. See
[the complete contract](../../copy-errors.md).

Error boxing and worker uncertainty publication happen only on failure; workers
retain their local counters and merge them after joining. No per-block statistics
atomics were introduced. The benchmark measures successful public-call paths,
not failure allocation latency or worst-case blocking I/O shutdown.

## Method

Separate absolute Cargo targets produce optimized baseline/candidate binaries.
Ten profiles run as adjacent candidate/baseline, baseline/candidate,
candidate/baseline pairs on CPUs 2–6. Each process uses 30 flat samples, 300 ms
warmup, and a 2 s measurement target. Follow-up/control runs use 500 ms warmup
and a 4 s target, with reversed starting order.

File workloads use warm deterministic incompressible sources on the recorded
Btrfs/NVMe mount, not /tmp. No global cache drops. The powersave governor and
uncontrolled background load limit qualification.

- Native planning control: buffered 1 MiB files, 64 KiB block, QD8 selection;
  destination contents are checked unchanged.
- Complete RAW copies: buffered 1 MiB, 64 KiB blocks, one-worker Threaded or
  IoUring/QD8/read-window4. Timing includes public planning/preparation, payload
  execution, cleanup, and final flush.
- Dynamic memory: 16 MiB, 64 KiB blocks, one/four workers, prebuilt plan and
  dyn VirtualDisk endpoints using the existing deterministic memory pattern.
- Sparse memory policy: 1 MiB, sixteen 64 KiB extents. One-worker Zero/Hole
  fallback and one/four-worker accelerated mixed Data/Zero/Hole plans.
- Fragmented no-op observer: portable 16 MiB, 50% Data/Hole, 64 KiB blocks,
  one worker, prebuilt plan.

Every timed copy includes its flush boundary. Reset/flush and full readback
remain untimed, and counters/byte equality are checked each iteration. Process
CPU/RSS logs include setup, readback, and differing adaptive iteration counts;
they are not per-copy CPU or memory-cost measurements.

## Main comparison

Aggregates are medians of three run medians. Positive changes mean slower.
Individual pairs are shown because host drift can make the aggregate differ
from the median paired change.

| Workload | Baseline | Candidate | Change | Paired changes |
|---|---:|---:|---:|---|
| `preflight/plan_native` | 1.737 µs | 1.720 µs | -0.94% | +0.45%, -27.55%, -0.94% |
| `preflight_copy/threaded` | 6.735 ms | 6.745 ms | +0.14% | +0.14%, +0.77%, -1.74% |
| `preflight_copy/native` | 6.861 ms | 6.880 ms | +0.28% | +0.28%, +0.15%, +0.79% |
| `portable_memory/workers1/dyn` | 1.773 ms | 1.812 ms | +2.18% | +0.58%, -0.39%, +2.18% |
| `portable_memory/workers4/dyn` | 1.273 ms | 1.234 ms | -3.04% | +4.29%, -24.22%, -6.53% |
| `semantic_policy/zero_fallback/workers1` | 47.866 µs | 47.256 µs | -1.28% | -0.33%, -1.15%, -3.17% |
| `semantic_policy/hole_fallback/workers1` | 45.946 µs | 45.812 µs | -0.29% | -0.95%, -7.97%, -0.29% |
| `semantic_policy/mixed_accelerated/workers1` | 46.176 µs | 45.708 µs | -1.01% | +2.73%, +3.99%, -1.43% |
| `semantic_policy/mixed_accelerated/workers4` | 120.710 µs | 120.831 µs | +0.10% | -0.59%, +0.10%, +19.60% |
| `progress_copy/fragmented50/noop` | 12.464 ms | 12.525 ms | +0.49% | -2.14%, +4.41%, +1.17% |

[Raw samples/estimates/outliers](measurements.json), [summary](summary.json), and
every console/resource log are retained.

Complete RAW copy medians are **+0.14% threaded / +0.28% native**. All aggregate
changes are between −3.05% and +2.18%. These observations do not imply universal
performance equivalence or repair earlier qualification issues.

## Sparse worker follow-up and control

The four-worker mixed sparse profile has one **+19.60%** main pair despite an
aggregate of +0.10%. A longer unchanged-binary repeat measured
**100.621 → 97.670 µs (−2.93%)**, with pairs **−11.43%, −2.86%, +16.44%**.
The variability persists; it is not a clean pass.
[Follow-up samples](followup-measurements.json) and
[summary](followup-summary.json) retain it.

A subsequent same-baseline-binary control used the identical baseline executable
in both nominal slots. Its paired differences were **−2.59%, −0.02%, −0.53%**,
with an aggregate of −0.69%. The control was tighter. It does not establish the
cause of the candidate variation because runs were not simultaneous, but it
prevents attributing all of the variation to host noise without further evidence.
In the [control samples](control-measurements.json), the nominal "candidate" slot
also runs the baseline binary; each command and binary_variant field records
that fact. [Control identity and summary](control-summary.json) identify it.

## Performance disposition

Accept the functionality and small aggregate success-path costs provisionally.
Keep the recurring four-worker sparse slow pairs as an explicit PERF.0 issue:
investigate scheduling/code-generation effects as well as host conditions on a
controlled runner. Do not declare this profile qualified or claim the variation
is solely noise. Prior R1.2/R1.3 qualification issues remain open as well.

No production tuning was introduced to chase these samples. The measurement
covers this implementation's success path; failure semantics have deterministic
regression coverage. Total memory budgets remain R1.6, and full native runtime
preparation remains R2.

## Reproduction

Use detached baseline 8adfabd and candidate worktrees, applying candidate.patch
only to the candidate. The same committed harnesses run on both variants:

```sh
cargo build --release -p rvvdk-datamover --bench preflight --bench progress --bench semantic_policy --bench portable_plan --target-dir /absolute/variant-target --message-format=json
```

Select executable paths from Cargo's artifact output. Use a fresh CRITERION_HOME
per run and RVVDK_BENCH_DIR on the selected storage mount:

```sh
taskset -c 2-6 /absolute/benchmark-executable --bench --noplot --sample-size 30 --warm-up-time 0.3 --measurement-time 2 'preflight_copy/native$'
```

Substitute the exact case in the table and alternate adjacent variant order.
For the sparse follow-up/control, use 0.5 s warmup and 4 s measurement. The
control runs the baseline executable in both slots. Exact commands, source/binary
fingerprints, samples, and resource logs are recorded; retain every result.
