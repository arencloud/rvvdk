# R1.1 — Portable planning and execution

## Decision

Accept the portable API change. No repeatable copy slowdown above 5% was found
in this bounded comparison. No defaults were tuned. Performance qualification
remains **provisional**: small warm workloads, powersave policy, and uncontrolled
background load do not establish sustained storage throughput or resolve small
regressions reliably. Preserve the outliers and follow-up measurements below.

## Source identity and correctness

- Baseline: `92f0350` (R0.5), unmodified detached worktree.
- Candidate: that commit plus [candidate.patch](candidate.patch), including new
  tests and the memory benchmark. The patch passes application checks on baseline
  and reverse checks on candidate. Full hashes are in [environment.json](environment.json).
- Separate absolute Cargo target directories were used. The unchanged `progress`
  and `native_lifetime` sources are identical in both variants; executable hashes
  are recorded. The progress harness exercises the former RAW-bound Threaded API
  versus the newly portable Threaded API with the same logical workload.
- All output comparisons, statistics assertions, and native descriptor-count
  checks passed. The [same external consumer](portable-api-probe.rs) fails to
  compile on baseline ([output](baseline-probe.txt)) and executes on candidate
  ([output](candidate-probe.txt)). This is API availability evidence, not a
  baseline runtime failure.
- 268 workspace tests passed, including ten new tests; formatting, all-target
  Clippy with warnings denied, and core/datamover library compilation for
  `wasm32-unknown-unknown` passed. The latter does not qualify WebAssembly execution.

## Method

Intel i7-11850H, approximately 64 GB RAM, Linux 7.2.5, Rust 1.95.0. File data lives
under `target/r11-benchmark-storage` on Btrfs (`compress=zstd:1`), through LUKS/dm-crypt
on Samsung NVMe, not `/tmp` tmpfs. No NOCOW change or global cache drop was made.
See the captured mount, CPU, memory, device, and governor metadata.

Pin to CPUs 2–6. Build both variants before timing; run sequentially in three
pairs: baseline/candidate, candidate/baseline, baseline/candidate. Criterion uses
20 flat samples, 300 ms warmup, and a 2 s measurement target. Each process has a
fresh Criterion output directory. Reset/prefill and full readback are untimed;
copy validation, buffer/worker setup, payload operations, and flush are timed.
Planning is measured separately. Sources are intentionally warm.

| Harness | Workload and configuration |
|---|---|
| `progress` | 16 MiB; dense or alternating 64 KiB Data/Hole (50% holes, 256 extents); buffered; 64 KiB blocks, 4096 alignment, one worker/buffer, queue capacity 4; unobserved or counting callback |
| `native_lifetime` | 16 MiB dense; QD8/read window4, 64 KiB blocks; buffered and direct; caller flush timed; descriptor counts checked after every copy |
| `portable_plan` | Candidate-only 16 MiB dense memory disk; 64 KiB blocks, 4096 alignment; one/four workers and buffers, queue capacity 4; concrete and trait-object execution of a prepared plan |

Patterns/seeds are fixed in the harness sources. Fragmented copy reads 8 MiB of
Data and writes 16 MiB through the existing Hole fallback; logical throughput is
not payload-read throughput. Extent counts and byte totals are printed per run.
No allocated-block or cold-storage claim is made by this API comparison.

## Initial paired results

Times are medians of the three run means. Changes are medians of corresponding
candidate/baseline ratios, so they need not equal the ratio of displayed medians.
Positive means slower. Raw samples/estimates are under [samples](samples/), and
all per-run values are in [summary.json](summary.json).

| Case | Baseline | Candidate | Median paired change | Individual paired range |
|---|---:|---:|---:|---:|
| Dense planning | 2.605 µs | 1.817 µs | -33.35% | -45.09% to -17.48% |
| Fragmented planning | 229.031 µs | 218.603 µs | -5.91% | -17.42% to +2.63% |
| Dense unobserved | 13.475 ms | 13.459 ms | +2.22% | -0.35% to +3.11% |
| Dense observer | 14.493 ms | 13.653 ms | -3.51% | -5.98% to +1.53% |
| Fragmented unobserved | 12.944 ms | 12.900 ms | -0.34% | -6.43% to +0.51% |
| Fragmented observer | 13.145 ms | 13.044 ms | -0.77% | -3.77% to +12.56% |
| Native buffered control | 13.240 ms | 13.690 ms | +1.79% | +1.05% to +5.73% |
| Native direct control | 30.761 ms | 30.427 ms | -0.52% | -1.42% to +1.64% |

Portable dense planning avoids RAW descriptor inspection because it does not
execute via descriptors. Its lower cost is a result of that API boundary, not
an optimization of native preparation. RAW planning still performs the R0.5
checks; sharing its descriptor snapshot remains follow-up work. Fragmented
planning is dominated by filesystem extent discovery and varies across pairs.

## Investigation of outliers

Initial pair3 showed +12.56% for fragmented observed copy and +5.73% for buffered
native. Repeated those exact cases with 30 flat samples, 500 ms warmup, a 4 s
target, and candidate/baseline then baseline/candidate order. Fragmented observed
copy was −1.50% and −1.48%; buffered native was −6.53% and +0.01%. The slowdowns
did not repeat. Absolute times also drifted during follow-up, so retain a
provisional disposition rather than claiming improvements. Direct native initial
pairs ranged from −1.42% to +1.64%; no follow-up was needed there.

## First memory API baseline

No old plan-execution measurement exists for memory disks: that API did not
accept them. The following candidate-only values establish a starting point.
Concrete calls ran before dynamic calls within each process; the compiler is
prevented from recovering the concrete type through `black_box` at the trait
object boundary, but differences also include ordering and scheduling noise.

| Workers | Concrete median | Trait-object median | Median paired dyn/concrete change |
|---|---:|---:|---:|
| 1 | 2.107 ms | 1.992 ms | −3.28% |
| 4 | 1.221 ms | 1.257 ms | +2.96% |

Four-worker dynamic pair2 was +26.64%; pairs1/3 were +2.96%/+0.99%. This isolated
outlier prevents tight dispatch-overhead claims. Memory locking, worker startup,
and scheduling are included; these results do not justify changing worker defaults
or predicting the cost of translated format decoding. Translation safety is
covered by guard-byte/FD-panic tests, not a format-performance claim.

Whole-process peak RSS was 66,700–68,876 KiB for initial progress runs,
65,944–68,116 KiB for initial native runs, and 100,008–101,040 KiB for memory runs.
Process CPU/RSS logs include reset/readback and adaptive iteration counts. They
are retained for diagnosis and must not be interpreted as per-copy memory/CPU
budgets. Source/destination/verification buffers dominate the memory harness.

## Reproduction

Build the baseline and candidate in separate absolute target directories:

```bash
cargo bench -p rvvdk-datamover --bench progress --bench native_lifetime \
  --no-run --message-format=json --target-dir /absolute/variant-build
# Candidate only:
cargo bench -p rvvdk-datamover --bench portable_plan --no-run \
  --message-format=json --target-dir /absolute/candidate-build
```

Extract each executable from Cargo's JSON artifact output. Set a fresh
`CRITERION_HOME` and a real-storage `RVVDK_BENCH_DIR` for each invocation:

```bash
taskset -c 2-6 /absolute/progress-executable \
  'progress_plan/|progress_copy/.*/(unobserved|counter)$' \
  --bench --noplot --sample-size 20 --warm-up-time 0.3 --measurement-time 2
```

Native filter: `q8_b65536$`; candidate-only memory filter: `portable_memory/`.
Follow-up filters: `progress_copy/fragmented50/counter$` and
`native_lifetime/buffered/q8_b65536$`, with the longer settings above. Wrap in
`/usr/bin/time -v` to capture process resources. Alternate variant order as stated,
verify output on every iteration, and preserve each `new/sample.json`,
`new/estimates.json`, and `new/benchmark.json`. Recompute the saved summary with
`python3 summarize.py` from this directory. Do not mix these results with older
unequal-flush benchmarks or use them as a sustained-storage qualification.
