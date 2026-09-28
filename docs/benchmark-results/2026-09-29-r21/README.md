# R2.1 — Logical Hole zero-read guarantee

Baseline: `25977f6` (R1.6), with the [matched harness patch](matched-harness.patch).
Candidate: that commit plus [candidate.patch](candidate.patch).
[Environment/source/binary fingerprints](environment.json) identify both builds.
Production baseline code is unchanged for timing. No buffer, worker, queue,
read-window, memory-budget, or host-governor defaults were tuned.

## Correctness and regression reproduction

**305 workspace tests passed** (six new), formatting, strict all-target Clippy,
and core/datamover library compilation for wasm32-unknown-unknown. The portable
result is compilation, not runtime qualification. [Validation commands](validation.json),
[test output](tests.txt), and [inventory](test-inventory.txt) retain the checks.

The new adversarial backend returns successful ordinary discard with nonzero
contents. Tests cover all eight combinations of DISCARD, DISCARD_ZEROES, and
WRITE_ZERO across four portable entry paths and one/four workers, sparse-only
native dispatch, odd boundaries, sentinel bytes beyond logical size, omitted
source sparse reads, operation counters, and partial guaranteed-discard failure
without retry/flush/final observation. Two mixed RAW/native parity profiles cover
ordinary discard with accelerated zeroing and with ordinary-write fallback.
Memory capabilities are explicit; existing zero-guaranteeing fixtures opt in to
the modifier so failure and accelerated-path tests retain their intended coverage.

The old policy fails both portable and native regressions when given only the
new flag declaration and the regression fixture. One failure/no-retry test still
passes. The [reproduction patch](baseline-regression.patch),
[command and expected exit status](baseline-regression.json), and
[failed baseline output](baseline-regression.txt) are retained. Baseline production
code was restored before performance builds; the regression shim was not timed.

[ADR-0026](../../adr/0026-logical-hole-guarantee.md) records the contract and
migration. Hole and Zero guarantee logical zero reads; only DISCARD together
with DISCARD_ZEROES permits copier discard. An advertised operation failure is
not retried. bytes_discarded counts logical operation bytes, not physical space
released. This step does not implement local hole punching or qualify filesystem
allocation, native runtime availability, or unaligned Data requests.

## Matched harnesses and method

The semantic-policy fixture already zeroes on discard. Both builds now advertise
that guarantee using bit 8 literally, allowing the identical harness to compile
against the old API without changing baseline production code. Added exact
bytes_zeroed/bytes_discarded assertions ensure the accelerated mixed workload
uses the same operations on both builds. The scheduler failure fixture is also
migrated; it is not one of the timed profiles in this step. Other timed harnesses
are unchanged. Harness hashes are recorded and compared across both checkouts.

These comparisons measure correct, equivalent work. Ordinary discard without a
zero-read promise intentionally falls back now; any extra writes/physical
allocation on that path are a correctness cost, not an equivalent-operation
comparison against the old unsafe fast path. Existing Hole fallback profiles
measure bounded zero writes. No speedup over incorrect output is claimed.

Separate absolute Cargo targets produce optimized baseline/candidate binaries.
Eleven profiles run as adjacent candidate/baseline, baseline/candidate, and
candidate/baseline pairs on CPUs 2–6. Each process requests 30 flat samples,
300 ms warmup, and a 2 s measurement target, with its own Criterion home.

File workloads use warm deterministic incompressible sources on the recorded
Btrfs/NVMe mount, not /tmp. No global cache drops. The powersave governor and
uncontrolled background load limit performance qualification.

- Native RAW planning: buffered 1 MiB files, 64 KiB blocks, QD8 selection;
  destination remains unchanged.
- Fragmented portable planning: 16 MiB, 50% Data/Hole, 64 KiB blocks.
- Complete RAW copies: buffered 1 MiB, 64 KiB blocks, one-worker Threaded or
  IoUring/QD8/read-window4; public planning/preparation, payload, cleanup, and
  final flush are timed.
- Dynamic memory: 16 MiB, 64 KiB blocks, one/four workers, prebuilt plans.
- Sparse memory policy: 1 MiB, sixteen 64 KiB extents; one-worker Zero/Hole
  fallback and one/four-worker accelerated mixed Data/Zero/Hole.
- Fragmented no-op observer: portable 16 MiB, 50% Data/Hole, 64 KiB blocks,
  one worker, prebuilt plan.

Timed copies include flush. Reset/flush and full readback are untimed; counters
and byte equality are asserted each iteration. GNU time CPU/RSS logs include
setup, readback, and varying adaptive iteration counts. They are not per-copy
resource measurements or evidence of a strict process memory cap.

## Comparison

Aggregates are medians of three run medians. Positive changes mean slower.
Individual pairs retain variability instead of hiding it behind the aggregate.

| Workload | Baseline | Candidate | Change | Paired changes |
|---|---:|---:|---:|---|
| `preflight/plan_native` | 1.805 µs | 1.789 µs | -0.94% | -1.56%, -1.31%, -0.93% |
| `progress_plan/fragmented50/plan` | 205.327 µs | 204.806 µs | -0.25% | -3.14%, -0.36%, +2.04% |
| `preflight_copy/threaded` | 6.755 ms | 6.811 ms | +0.82% | +4.47%, -0.49%, +0.88% |
| `preflight_copy/native` | 6.897 ms | 6.917 ms | +0.29% | -0.01%, +0.29%, +2.12% |
| `portable_memory/workers1/dyn` | 1.804 ms | 1.781 ms | -1.23% | +1.25%, -0.76%, -1.44% |
| `portable_memory/workers4/dyn` | 1.191 ms | 1.239 ms | +4.01% | +4.97%, +5.92%, -15.22% |
| `semantic_policy/zero_fallback/workers1` | 47.368 µs | 47.972 µs | +1.28% | -1.40%, -0.29%, +1.31% |
| `semantic_policy/hole_fallback/workers1` | 46.999 µs | 45.986 µs | -2.16% | +0.92%, +1.26%, -2.16% |
| `semantic_policy/mixed_accelerated/workers1` | 47.227 µs | 46.328 µs | -1.90% | +1.84%, -1.63%, -3.86% |
| `semantic_policy/mixed_accelerated/workers4` | 121.384 µs | 121.216 µs | -0.14% | -6.51%, -0.16%, +0.51% |
| `progress_copy/fragmented50/noop` | 12.633 ms | 12.556 ms | -0.61% | +0.16%, +10.39%, -0.61% |

## Performance disposition

All main aggregate changes are between −2.16% and +4.01%. Complete RAW copies
are +0.82% Threaded and +0.29% native. Native/fragmented planning are
−0.94%/−0.25%; sequential Zero/Hole fallback +1.28%/−2.16%; accelerated mixed
one/four-worker copying −1.90%/−0.14%. Accept the correctness fix with these
measured costs; do not infer general speedups from negative development-host
results.

Four-worker dynamic memory copying is +4.01% aggregate, with pairs +4.97%,
+5.92%, −15.22%. A focused repeat with 500 ms warmup and a 4 s target, starting
in the opposite order, gives +2.60% aggregate and +0.34%, +2.95%, **+7.49%**
pairs. [Repeat samples](followup-measurements.json) and [summary](followup-summary.json)
retain the results. No implementation changed between runs.

The identical-baseline-binary control has +1.92%, −1.76%, +4.95% pairs and
−0.26% aggregate. Both ordering slots run the same hashed baseline executable;
`candidate` in control filenames/JSON is an ordering label only, with each record
annotated `binary_variant: baseline`. [Control samples](control-measurements.json)
and [summary](control-summary.json) show scheduling/run-to-run variation but do
not fully explain the candidate's +7.49% pair. Keep potential code/layout effects
and controlled-runner qualification open instead of dismissing all differences
as host noise.

Fragmented no-op observation is −0.61% aggregate but includes a **+10.39%** pair.
Keep that profile open alongside R1.2/R1.6's earlier findings; this run does not
close prior variability. R1.5's mixed four-worker investigation also remains
open despite small matched aggregate costs here. Performance qualification is
provisional and needs a controlled runner; the required logical correctness
fix is not contingent on interpreting unsafe discard as a faster equivalent.

## Additional harness audit finding

The pre-existing `scheduler_failure` fixture omits FLUSH and still matches raw
Error::Io instead of CopyExecution. Under current contracts it requires those
updates before its failure-to-return timings are usable. R2.1 migrates its
discard guarantee declaration, but does not claim this separate harness is
runtime-qualified. It was excluded from this step's comparisons; record its
repair under PERF.0 before using it again. The timed semantic-policy harness
and all eleven compared profiles have their output/counter checks enabled.

## Artifacts

- [Raw Criterion samples/estimates](measurements.json), [summary](summary.json).
- `pair*-*.txt`: benchmark output/outliers and GNU time resource logs.
- [Baseline build](baseline-build.txt), [candidate build](candidate-build.txt).
- [Validation](validation.json), [environment/hashes](environment.json),
  [candidate source patch](candidate.patch), [matched harness patch](matched-harness.patch).
- Baseline regression reproduction patch/command/expected failure above.

No alternate measured candidate or failed timing was discarded. The baseline
patch is limited to matched benchmark fixtures; the candidate source patch
includes implementation, fixtures, and regression coverage. Source/binary/harness
identity checks accompany the recorded validation.
