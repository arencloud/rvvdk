# R0.4 — Native validation before I/O

Baseline: `2da7c8f`. Candidate: baseline plus [candidate.patch](candidate.patch).
[Environment and fingerprints](environment.json), [test evidence](validation.json),
[before-fix failures](before-tests.txt), and [summary data](summary.json) are retained.

## Correctness and scope

Seven of eight new regressions failed before the repair; the Zero fallback child
was killed/reaped after five seconds. All eight pass afterward, together with the
239-test workspace suite, formatting, and strict all-target Clippy. Regressions
cover configuration on empty/Data/Zero/Hole plans and all zero/discard capability
paths, unsupported later extents before Data writes, Zero-before-Data invalid
alignment, signed file-range limits, and invalid owned requests without SQE
publication, lost pool buffers, or engine poisoning. Existing tests exercise valid
Data/Zero/Hole output paths. Every measured copy checked full output equality;
dense copy cases also checked stable FD counts after each iteration.

## Method

Baseline and candidate use identical harnesses and separate optimized target
directories. Runs are sequential, pinned to CPUs 2–6 (five physical cores).
Initial sampling: three paired runs, order B/C, C/B, B/C; 30 flat samples for the
validation harness and 20 for dense copies, 300 ms warmup, 2 s measurement target.
Tables use the median of three run medians, not a single favorable estimate.
Positive change means slower. Raw Criterion estimates and samples are stored in
[initial measurements](measurements.json) and [final measurements](final-measurements.json).
Per-process console/resource logs are alongside them.

Validation workloads use QD8/read-window4, 64 KiB blocks, alignment4096: valid
empty calls, first-extent rejection, 1 MiB memory Zero fallback with sixteen
extents, and a buffered 1 MiB Data plan with sixteen extents. The Data plan measures
the supported-kind scan plus existing per-extent ring/pool setup and execution.
Dense workloads copy 16 MiB in buffered/direct modes; the unchanged threaded
control has four workers and 64 KiB blocks. Final direct sampling also covers
QD1/64 KiB, QD8/4 KiB, and QD8/1 MiB. No execution defaults were tuned.

Plans are constructed outside timing. Copy setup, execution, cleanup, and flush
are timed equally. Each destination is reset to different bytes and flushed
before timing; full readback verification is afterward. Sources use deterministic
incompressible bytes. Files live on the recorded Btrfs/NVMe mount, not `/tmp`
tmpfs. Memory fallback uses a nonzero-prefilled MemoryBlockDevice. These are small,
warm development workloads; direct I/O does not imply cold device caches.

## Initial results

| Workload | Baseline | Candidate | Change |
|---|---:|---:|---:|
| `native_lifetime/buffered/q8_b65536` | 13.367 ms | 13.660 ms | +2.19% |
| `native_lifetime/buffered/threaded_control` | 11.593 ms | 11.925 ms | +2.87% |
| `native_lifetime/direct/q8_b65536` | 30.404 ms | 30.868 ms | +1.53% |
| `native_lifetime/direct/threaded_control` | 24.290 ms | 24.674 ms | +1.58% |
| `native_validation/data_1mib_16extents` | 7.473 ms | 7.465 ms | -0.11% |
| `native_validation/empty_plan` | 6.666 ns | 8.285 ns | +24.29% |
| `native_validation/empty_range` | 8.281 ns | 10.371 ns | +25.24% |
| `native_validation/unsupported_first_extent` | 10.992 ns | 11.618 ns | +5.69% |
| `native_validation/zero_fallback_1mib_16extents` | 46.595 µs | 46.641 µs | +0.10% |

The initial copy aggregates stayed within 3% of baseline, but individual dense
pairs and unchanged controls drifted. Empty-call checks added about 2 ns. That
relative increase is an accepted cost of rejecting invalid inputs consistently.

## Final harness and storage follow-up

Clippy required equal digit grouping for hexadecimal benchmark seeds. Numeric
seed values were unchanged, but rebuilt validation binary hashes differed. The
original [initial patch](initial-candidate.patch), fingerprints, and results are
preserved; the complete validation harness was repeated for both final builds.
The dense benchmark binaries remained identical. Final runs reverse the starting
order (C/B, B/C, C/B). The full direct profile uses longer sampling: 30 samples,
500 ms warmup, 3 s target. Validation sampling is unchanged.

| Workload | Baseline | Candidate | Change |
|---|---:|---:|---:|
| `native_lifetime/direct/q1_b65536` | 47.242 ms | 45.821 ms | -3.01% |
| `native_lifetime/direct/q8_b1048576` | 22.493 ms | 22.440 ms | -0.23% |
| `native_lifetime/direct/q8_b4096` | 177.190 ms | 173.675 ms | -1.98% |
| `native_lifetime/direct/q8_b65536` | 33.134 ms | 33.448 ms | +0.95% |
| `native_lifetime/direct/threaded_control` | 21.887 ms | 21.721 ms | -0.76% |
| `native_validation/data_1mib_16extents` | 7.459 ms | 7.588 ms | +1.73% |
| `native_validation/empty_plan` | 6.584 ns | 7.844 ns | +19.14% |
| `native_validation/empty_range` | 8.468 ns | 9.592 ns | +13.28% |
| `native_validation/unsupported_first_extent` | 10.964 ns | 11.379 ns | +3.78% |
| `native_validation/zero_fallback_1mib_16extents` | 49.750 µs | 42.486 µs | -14.60% |

## Disposition and limits

Accept the required correctness repair and its measured nanosecond entry-point
cost. Final empty-range/plan calls add 1.12/1.26 ns; first-extent rejection adds
0.42 ns. Fragmented native copy is +1.73%; repeated direct native aggregates range
from −3.01% to +0.95%. The initial buffered native aggregate is +2.19%. No copy
aggregate exceeded the 5% investigation threshold in the final comparison.

Performance qualification remains provisional: individual direct 64 KiB pairs
range from −12.35% to +11.55%, despite a +0.95% aggregate. Initial control drift
prompted the longer repeat. Memory fallback's apparent −14.60% improvement is
not an optimization claim; it contrasts with the initial +0.10% and substantial
pair-to-pair variation. Preserve all samples and rerun on controlled PERF.0
hardware before claiming a regression bound or tuning defaults.

Whole-process peak RSS across recorded runs was 22,124–23,596 KiB for the
validation harness and 66,812–75,312 KiB for dense copy harnesses. These include
harness allocations and cannot attribute a per-copy memory change.

The laptop uses the powersave governor and has uncontrolled background work.
These results do not establish a universal regression bound or sustained-storage
throughput. PERF.0 still needs a controlled runner and representative disk sizes.
GNU time logs describe complete benchmark processes, including Criterion,
reset/readback, and unequal adaptive iteration counts; they are not per-copy
CPU/RSS costs. No per-I/O latency or hardware-counter attribution was collected.
The validation checks allocate nothing, but do not impose an aggregate memory
budget or guarantee later allocation/ring creation succeeds.

## Reproduction

Create two checkouts at the baseline commit. Apply `candidate.patch` to the
candidate. Copy its `crates/rvvdk-datamover/Cargo.toml` and
`benches/native_validation.rs` into the baseline, leaving baseline runtime code
unchanged. Build each with separate absolute target directories:

```sh
cargo bench -p rvvdk-datamover --bench native_validation --bench native_lifetime --no-run --target-dir /absolute/variant-target
```

Run the emitted executables with `--bench --noplot`, `taskset -c 2-6`, a unique
`CRITERION_HOME`, and `RVVDK_BENCH_DIR` pointing at the selected storage mount.
Use the initial dense filter `q8_b65536|threaded_control`, or final direct filter
`native_lifetime/direct/ --sample-size 30 --warm-up-time 0.5 --measurement-time 3`.
Exact argument arrays are retained in the measurement JSON. Compare hashes and
mount information before interpreting timing. Apply `initial-candidate.patch`
instead to reconstruct the initial seed spelling and its measured artifacts.
