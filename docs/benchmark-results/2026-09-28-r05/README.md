# R0.5 — Copy endpoint preflight

Baseline: `eb10da1`. Final candidate: baseline plus
[candidate.patch](candidate.patch). [Environment and source/binary fingerprints](environment.json)
identify each comparison. The [initial patch](initial-candidate.patch) records the
implementation before the final access-mode refinement. No defaults were tuned.

## Correctness and behavior

The [baseline regressions](before-tests.txt) failed thirteen of fourteen tests;
the valid partial-range copy passed. The final candidate passes 258 workspace
tests (nineteen new), formatting, and strict all-target Clippy. Five added tests
exercise the new endpoint contract, adversarial RAW descriptor binding, paired
local handles, metadata errno preservation, and actual access-mode inspection.
[Validation record](validation.json) preserves commands and counts.

Checks now precede writes and observer notifications: source READ, destination
WRITE/FLUSH for full copies, current capacity/source size, known backing aliases,
append/unsupported FD modes, and native backend/descriptor binding. Full native
extent plans are checked once before execution; Data extents use a private
validated range dispatcher instead of re-querying FDs for each extent. Local
direct/buffered handles must name one inode. Low-level native copies still do
not flush and do not extend short files. Unknown custom identities cannot prove
absence of aliases; native Data execution requires known FD binding. See the
[architecture contract](../../architecture.md#copy-endpoint-preflight-r05).

Existing worker failure coverage still reaches the worker: its synthetic backend
now advertises FLUSH. Two public native lifetime tests now expect contextual
preflight rejection without leaked descriptors; the owned-engine fault-injection
cleanup tests are unchanged. The new flag regression rejects ioctl-only access
instead of treating every non-read-only/non-write-only mode as data access.

## Method

Each variant uses an isolated optimized Cargo target directory. Identical
harnesses run sequentially with affinity to CPUs 2–6. Each comparison has three
pairs in baseline/candidate, candidate/baseline, baseline/candidate order. Planning
and native validation use 30 flat samples; dense copies use 20. Warmup is 300 ms,
measurement target 2 s. Tables show the median of three run medians; positive
change means slower. [Initial raw estimates/samples](measurements.json),
[final raw estimates/samples](final-measurements.json), and console/resource logs
are retained, including all outliers.

- Planning: 1 MiB buffered file, Threaded/IoUring selection, 64 KiB blocks. Timing
  includes endpoint inspection, extent query, and plan construction. Destination
  contents are checked unchanged after the benchmark.
- Zero fallback: nonzero-prefilled 1 MiB memory destination, sixteen extents,
  64 KiB blocks. Both source variants now use a regular file source to satisfy
  the native endpoint contract; the unused destination FD remains irrelevant.
- Fragmented Data: buffered 1 MiB file, sixteen extents, 64 KiB blocks,
  QD8/read-window4, alignment4096, eight buffers (512 KiB).
- Dense: 16 MiB buffered/direct copies, QD8/read-window4/64 KiB native and
  four-worker threaded comparators. Both executors gained preflight; the legacy
  `threaded_control` benchmark name does not imply an unchanged control here.

For copies, preflight/setup/execution/cleanup/flush are timed. Nonzero destination
reset/flush and full output readback are outside timing. Dense cases also check
FD counts after every copy. Sources use deterministic incompressible bytes;
storage is the recorded Btrfs/NVMe mount, not `/tmp` tmpfs. No global cache drop.
The reported CopyStats elapsed field is not used for these timing comparisons;
the harness measures the complete public-call boundary plus native flush.

## Initial comparison

| Workload | Baseline | Candidate | Change |
|---|---:|---:|---:|
| `native_lifetime/buffered/q8_b65536` | 13.594 ms | 13.045 ms | -4.04% |
| `native_lifetime/buffered/threaded_control` | 11.422 ms | 11.488 ms | +0.58% |
| `native_lifetime/direct/q8_b65536` | 29.993 ms | 26.894 ms | -10.33% |
| `native_lifetime/direct/threaded_control` | 24.177 ms | 23.205 ms | -4.02% |
| `native_validation/data_1mib_16extents` | 7.637 ms | 7.460 ms | -2.32% |
| `native_validation/zero_fallback_1mib_16extents` | 41.577 µs | 42.209 µs | +1.52% |
| `preflight/plan_native` | 0.924 µs | 2.574 µs | +178.72% |
| `preflight/plan_threaded` | 0.887 µs | 2.472 µs | +178.58% |

Planning added 1.58–1.65 µs (about 179%). The initial copy aggregates showed no
increase above 5%, but direct native paired changes ranged from −12.45% to
+16.84%. A longer direct-only repeat used reverse starting order, 30 samples,
500 ms warmup, and a 3 s target. It measured 26.694 ms → 26.403 ms (−1.09%), with
paired changes −2.37% to +4.83%. Its [raw samples](followup-measurements.json) and
[summary](followup-summary.json) are preserved.

## Final comparison

Final review tightened access flags to accept only the named read/write modes
and added a real-FD regression. The complete benchmark matrix was repeated after
rebuilding the candidate; baseline binaries and harnesses are unchanged. The
final source patch and binary hashes identify these measurements separately.

| Workload | Baseline | Candidate | Change |
|---|---:|---:|---:|
| `native_lifetime/buffered/q8_b65536` | 14.747 ms | 14.683 ms | -0.43% |
| `native_lifetime/buffered/threaded_control` | 12.279 ms | 12.255 ms | -0.20% |
| `native_lifetime/direct/q8_b65536` | 36.033 ms | 35.668 ms | -1.01% |
| `native_lifetime/direct/threaded_control` | 28.785 ms | 29.003 ms | +0.76% |
| `native_validation/data_1mib_16extents` | 8.141 ms | 8.333 ms | +2.37% |
| `native_validation/zero_fallback_1mib_16extents` | 60.298 µs | 59.133 µs | -1.93% |
| `preflight/plan_native` | 1.190 µs | 3.342 µs | +180.84% |
| `preflight/plan_threaded` | 1.120 µs | 3.197 µs | +185.40% |

## Disposition

Accept the required preflight repair and its planning cost. Final planning adds
2.08–2.15 µs (+181–185%). Inspection of the implementation accounts for eight
additional metadata/flag queries per RAW planning call: logical backend checks
and actual descriptor checks each inspect both endpoints. These independent
checks establish current capabilities/capacity and correct descriptor binding.
They do not run per block or per Data extent. R1 preparation should investigate
combining descriptor snapshots without weakening logical capability or binding
checks; the cost is recorded rather than hidden in copy throughput.

Final copy aggregate changes range from −1.93% to +2.37%. General performance
qualification remains provisional: individual direct-native pairs still range
from −5.23% to +17.55%, while absolute baseline timings changed materially between
phases. No broad speedup or 5% regression bound is established on this shared
host. Preserve all results and repeat representative workloads on controlled
PERF.0 hardware. Every measured output and dense FD-count check passed.

## Limits and reproduction

The host uses the powersave governor with uncontrolled background work. These
small, warm workloads do not qualify sustained disk throughput or cold-device
behavior. There is no per-I/O latency or hardware-counter attribution. GNU time
logs include the harness, reset/readback, Criterion analysis, and unequal adaptive
iteration counts; they are not per-copy CPU or RSS measurements. Across complete
processes, peak RSS ranged from 16,632 to 70,116 KiB. There is still no aggregate
copy memory budget, and preflight does not prevent concurrent mutation or make
execution transactional.

Create two checkouts of the baseline. Apply `candidate.patch` to the candidate;
copy its datamover `Cargo.toml`, `benches/preflight.rs`, and
`benches/native_validation.rs` to the baseline without changing baseline runtime.
Build each with separate absolute target directories:

```sh
cargo bench -p rvvdk-datamover --bench preflight --bench native_validation --bench native_lifetime --no-run --target-dir /absolute/variant-target
```

Run emitted binaries with `--bench --noplot`, `taskset -c 2-6`, a unique
`CRITERION_HOME`, and `RVVDK_BENCH_DIR` on the selected storage. Filters are none
for preflight, `zero_fallback|data_1mib` for native_validation, and
`q8_b65536|threaded_control` for native_lifetime. Exact commands are in the
measurement JSON. The initial comparison/follow-up uses `initial-candidate.patch`
instead. Compare source/binary fingerprints and mount metadata before timing.
