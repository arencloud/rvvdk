# R1.6 — Copy memory budget

Baseline: `67d245f` (R1.5). Candidate: baseline plus
[candidate.patch](candidate.patch). [Environment/source/binary fingerprints](environment.json)
identify the measured implementation. All eleven profiles use unchanged
harnesses. The default budget is 256 MiB; buffer/queue/worker/native settings and
host governor were not tuned. Concurrent scheduling now borrows extent metadata.

## Correctness and scope

**299 workspace tests passed** (five new), formatting, strict all-target Clippy,
and core/datamover library compilation for wasm32-unknown-unknown. The portable
result is a compile check, not runtime qualification.
[Commands](validation.json), [test output](tests.txt), and
[test inventory](test-inventory.txt) retain the validation evidence.

New tests exercise unused extent capacity, exact-limit acceptance, budget
rejection before callbacks/I/O/flush across portable and native entry points,
retained-plan plus live-query capacity, overflow without large allocation,
executing native queue depth, and successful native readback at the exact limit.
Existing worker shutdown, native lifetime, sparse semantics, and partial-failure
checks pass unchanged.

The [budget contract](../../copy-memory.md) defines accounted payload storage.
This is not a process RSS cap: backend extent queries allocate before their
returned capacity is checked; opaque allocator/container overhead, stacks,
backend/observer memory, kernel resources, and prior quarantine are external.
Fragmentation still determines metadata size. No per-block budget accounting
was introduced. Planning admits metadata alone; execution may require more.

## Method

Separate absolute Cargo targets produce optimized baseline/candidate binaries.
Each of eleven profiles runs as adjacent candidate/baseline, baseline/candidate,
and candidate/baseline pairs. Affinity: CPUs 2–6. Each process requests 30 flat
samples, 300 ms warmup, and a 2 s measurement target. Independent Criterion
homes preserve each run; no stale result reuse.

File workloads use warm deterministic incompressible sources on the recorded
Btrfs/NVMe mount, not /tmp. No global cache drops. Powersave governor and
uncontrolled background load limit qualification.

- Native RAW planning: buffered 1 MiB files, 64 KiB block, QD8 selection;
  destination remains unchanged.
- Fragmented portable planning: 16 MiB, 50% Data/Hole, 64 KiB blocks.
- Complete RAW copies: buffered 1 MiB, 64 KiB blocks, one-worker Threaded or
  IoUring/QD8/read-window4; public planning/preparation, payload, cleanup, and
  final flush are timed.
- Dynamic memory: 16 MiB, 64 KiB blocks, one/four workers, prebuilt plans.
- Sparse memory policy: 1 MiB, sixteen 64 KiB extents; one-worker Zero/Hole
  fallback and one/four-worker accelerated mixed Data/Zero/Hole.
- Fragmented no-op observation: portable 16 MiB, 50% Data/Hole, 64 KiB blocks,
  one worker, prebuilt plan.

Timed copies include flush. Destination reset/flush and full readback are untimed;
byte equality and statistics are checked each iteration. Process CPU/RSS logs
include setup/readback and varying adaptive iteration counts. They are **not**
per-copy memory measurements or evidence that the payload budget bounds RSS.
Boundary tests establish accounting admission, separately from timing results.

## Comparison

Aggregates are medians of three run medians. Positive changes mean slower;
individual pairs show variability rather than hiding it in the aggregate.

| Workload | Baseline | Candidate | Change | Paired changes |
|---|---:|---:|---:|---|
| `preflight/plan_native` | 1.681 µs | 1.688 µs | +0.39% | -1.33%, +0.39%, +2.06% |
| `progress_plan/fragmented50/plan` | 193.281 µs | 195.575 µs | +1.19% | +0.27%, +1.28%, -0.96% |
| `preflight_copy/threaded` | 6.854 ms | 6.955 ms | +1.48% | -0.69%, +1.48%, +0.88% |
| `preflight_copy/native` | 6.868 ms | 6.933 ms | +0.95% | +1.85%, +0.95%, -3.76% |
| `portable_memory/workers1/dyn` | 1.782 ms | 1.781 ms | -0.03% | +0.66%, -0.54%, -2.70% |
| `portable_memory/workers4/dyn` | 1.247 ms | 1.268 ms | +1.73% | +0.54%, +1.73%, +0.18% |
| `semantic_policy/zero_fallback/workers1` | 46.189 µs | 47.068 µs | +1.90% | +4.05%, -4.26%, +2.50% |
| `semantic_policy/hole_fallback/workers1` | 45.828 µs | 46.107 µs | +0.61% | +1.01%, -8.67%, +0.61% |
| `semantic_policy/mixed_accelerated/workers1` | 45.685 µs | 46.278 µs | +1.30% | +0.65%, +3.83%, -2.79% |
| `semantic_policy/mixed_accelerated/workers4` | 128.083 µs | 121.796 µs | -4.91% | -20.24%, -23.52%, +1.71% |
| `progress_copy/fragmented50/noop` | 13.039 ms | 12.611 ms | -3.28% | +2.99%, -3.28%, +39.16% |

## Performance disposition

All main aggregate changes lie between −4.91% and +1.90%. Complete RAW copying
is +1.48% Threaded and +0.95% native; native planning +0.39%, fragmented planning
+1.19%. Dynamic memory execution is −0.03%/+1.73% for one/four workers. These
small aggregate costs are accepted for the admission contract on this host.

Four-worker mixed sparse results remain variable: −20.24%, −23.52%, and +1.71%
pairs, with an aggregate −4.91%. Removing the metadata clone reduces accounted
storage by one extent Vec; these timings do not establish a general throughput
improvement or close R1.5's controlled-runner investigation.

Fragmented no-op observation has a **+39.16% slow pair**, despite aggregate
−3.28%. A longer focused repeat (500 ms warmup, 4 s target, reversed starting
order) gives aggregate +0.78%, with pairs −0.06%, +0.78%, and **+16.94%**.
[Follow-up samples](followup-measurements.json) and [summary](followup-summary.json)
retain the recurrence. No implementation changed between the main and repeated
runs. Accept functionality provisionally; do not call this a clean performance
qualification or discard the slow pairs.

The identical-baseline-binary control produces pairs **+19.85%, −0.22%, +3.15%**
(aggregate +0.06%). Both ordering slots run the same hashed baseline executable;
`candidate` in those control filenames/JSON is only an ordering label, and each
record includes `binary_variant: baseline`. [Control samples](control-measurements.json)
and [summary](control-summary.json) demonstrate substantial run-to-run variation
independently of this change. They do not prove that every candidate slowdown is
host noise or rule out a candidate-specific effect.

Carry this fragmented-observer issue and the earlier R1.2/R1.3/R1.5 findings into
PERF.0 controlled-runner qualification. The accounting checks are outside the
block loop, but that alone does not prove an absence of compiler/layout,
scheduling, or I/O effects. No defaults were tuned from these development-host
measurements.

## Retained artifacts

- [Raw Criterion samples and estimates](measurements.json), [summary](summary.json).
- `pair*-*.txt`: command output, outliers, and GNU time process resource logs.
- [Baseline build](baseline-build.txt), [candidate build](candidate-build.txt).
- [Validation](validation.json), [environment and hashes](environment.json),
  [exact source patch](candidate.patch).

The patch applies to the recorded baseline and reverses from the measured
candidate. Source and binary identities were checked after measurement. No
failed measurement run or alternate measured candidate was discarded. Source
validation was repeated after tightening the native clone-capacity check, before
all timings; only that final source state was measured.
