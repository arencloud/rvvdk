# Benchmark plots

Render saved measurements as standalone SVG and PNG files without running a
benchmark or changing its raw evidence. Install the plotting dependency in an
ignored local environment:

```bash
python3 -m venv target/benchmark-plots
target/benchmark-plots/bin/python -m pip install -r scripts/benchmarks/requirements.txt
target/benchmark-plots/bin/python scripts/benchmarks/plot.py docs/benchmark-results/2026-09-29-r23
target/benchmark-plots/bin/python -m unittest discover -s scripts/benchmarks -p 'test_*.py'
```

The output is `REPORT/plots/`; `--output PATH` renders to another directory.
SVG is embedded in Markdown reports; PNG is available for sharing. Matplotlib
uses a headless backend and its bundled DejaVu Sans font. Fixed SVG identifiers
and omitted timestamps make repeat output stable with the same generator in the
recorded environment. Historical manifests identify the generator version used;
use that recorded source revision when reproducing older figures.
Cross-version/platform byte identity is not guaranteed. The manifest records
Python, installed package versions, generator/input hashes, and output hashes.

## Input contract

Use R2.3's [plot-config.json](../../docs/benchmark-results/2026-09-29-r23/plot-config.json)
as a template. Set the title, **measured** candidate identity, recorded conditions,
sampling description, and labels/phases for every workload. Do not use the current
checkout's commit as a substitute for the measured source. Candidate identity
can describe an uncommitted source patch; the report must link its provenance.
Baseline identity comes from `environment.json`.

Required evidence follows the compact R2.3 schema:

- `measurements.json`: a list of runs, each with a unique `tag`, `benchmark`,
  `variant` (`baseline`/`candidate`), explicit `pair`, and `criterion` object.
- `criterion["sample.json"]`: positive, finite `times` (ns) and `iters` arrays of
  equal length. Each plotted sample is **time / iterations**.
- `criterion["estimates.json"]["median"]["point_estimate"]`: run median in ns.
- `summary.json`: one row per workload with `baseline_ns`, `candidate_ns`,
  `change_percent`, and `paired_changes_percent` in ascending pair-ID order.
- `environment.json`: recorded baseline identity and environment/source evidence.

Optional `followup-measurements.json` and `followup-summary.json` use the same
comparison schema (a single-object summary is also accepted). Workloads must
exist in the main comparison. They create a separate main/repeat chart.
Optional `unsupported-measurements.json` contains candidate-only run records
without `pair`/`variant`; this specific experiment is labeled as injected EINVAL
via LD_PRELOAD. Do not reuse that filename for other experiment types. Other candidate-only
experiments use `candidate-only-measurements.json` with the same run schema and
a `candidate_only` configuration object containing `title`, `note`, and
`sampling`. Use only one candidate-only file per report. Explicitly explain why
a baseline timing comparison is unavailable or inappropriate.

The generator supports these schemas, not arbitrary historical Criterion layouts.
It checks normalized sample medians against recorded estimates and recomputes
all published aggregate/pair changes before rendering. Mismatches, missing or
duplicate pairs, and invalid samples fail rather than silently dropping evidence.
`computed.json` retains plotted aggregate values, pair IDs, and run medians.

## Reading the figures

- Latency bars use zero-based axes, median-of-run-medians heights, and individual
  run-median dots. Planning uses µs; copying uses ms. Panel scales are independent.
- Change bars use `100 × (median(candidate runs) / median(baseline runs) − 1)`.
  Dots use the corresponding ratio for each matched pair. These are different
  statistics; neither is a confidence interval. Positive means slower.
- Box plots keep runs separate: Q1–Q3, median, 1.5×IQR whiskers and every outlier.
  They show normalized Criterion sample variability, **not per-I/O latency**.
  Their axes may be zoomed and must not be interpreted as zero-based bars.
- The +5% line marks the project's investigation threshold, not significance.
- Candidate-only fallback has no successful baseline and no claimed speedup.

Keep historical trend charts for genuinely matched workloads and conditions.
Do not chain per-step percentages or combine mismatched harnesses into a trend.
A repeat within one report is shown separately and does not resolve earlier
performance investigations.

For each future measured step: record provenance and raw evidence, configure
labels/conditions, run this generator, inspect the images, embed the applicable
SVGs with PNG links, and commit the generator/config/results with the step.
Do not refresh historical measurements merely to update their presentation.

## Sparse scaling panels

R5.5 also saves capacity counters and synthetic metadata/fragmentation timings:

```bash
target/benchmark-plots/bin/python scripts/benchmarks/plot_sparse_scaling.py docs/benchmark-results/2026-09-30-r55
```

The supplemental generator validates normalized samples and reads
`capacity-profiles.json`; it writes SVG/PNG and a hash manifest under
`scaling-plots/`. Use `--output` to reproduce elsewhere. Capacity/grain axes are
logarithmic; latency and memory axes start at zero. Orange dots retain each run
median. Error-path timing is labeled separately from successful query output.
These memory-backed fixtures do not measure physical-storage throughput.

## StreamOptimized metadata admission

```sh
target/benchmark-plots/bin/python scripts/benchmarks/plot_stream_admission.py docs/benchmark-results/2026-10-02-r510
```

Validates all 4140 retained samples and plots every initial/final descriptor pair,
including longer adverse repeats, plus new in-memory metadata costs. Outputs
SVG/PNG, recomputed JSON and input/output hashes. No benchmarks, storage access,
guest data or VMware API calls occur during plotting. The small-descriptor
performance disposition remains open; these figures do not qualify decoding.

## Descriptor comparison specialization and identical-binary controls

```sh
target/benchmark-plots/bin/python scripts/benchmarks/plot_descriptor_specialization.py docs/benchmark-results/2026-10-02-r510p
```

Recomputes all 6570 saved samples, primary paired changes and separate-core
identical-binary controls. Produces descriptor, stream and control SVG/PNG figures,
computed JSON and input/output hashes. It neither runs benchmarks nor accesses
VMware. The scoped descriptor recovery does not clear all stream timing results.

R5.11a grain-index admission and stream regression pairs (including longer adverse
repeats) have a dedicated generator. It checks the complete run matrix and all
sample-derived medians before writing SVG/PNG figures and hash audits:

```sh
python scripts/benchmarks/plot_stream_map.py docs/benchmark-results/2026-10-02-r511a
```

R5.11b combines in-memory native read costs, paired map controls, and repeated
retained-export read/CPU/RSS observations. The generator verifies samples and
live comparison success and saves SVG/PNG charts with an input/output hash audit:

```sh
python scripts/benchmarks/plot_stream_reads.py docs/benchmark-results/2026-10-02-r511b
```

R5.12 plots complete local CLI conversion and verification, paired unchanged
hosted-sparse controls, and retained-export copy/readback CPU/RSS. Every adverse
pair and longer repeat remains visible; logical throughput includes sparse holes.

```sh
python scripts/benchmarks/plot_stream_cli.py docs/benchmark-results/2026-10-02-r512
```

R5.12p verifies paired RAW/FLAT/sparse/stream controls, longer adverse repeats,
historical and identical-binary comparisons, plus retained-export elapsed/CPU/RSS
and physical allocation. The XFS runner is plotted separately from local pairs:

```sh
python scripts/benchmarks/plot_zero_output.py docs/benchmark-results/2026-10-02-r512p
```

R5.12q audits every retained-export copy/readback, phase sum, policy/control pair,
allocation and selected syscall count. It plots phase deltas, identical-binary and
progress controls, physical extent counts and separate native read timings:

```sh
python scripts/benchmarks/plot_copy_phases.py docs/benchmark-results/2026-10-02-r512q
```

`measure_copy_phases.py --help` describes offline measurement inputs. Use private
source/work paths and a fresh public report directory. Native Rust binaries do all
disk operations; the harness records aggregates and keeps raw CLI paths private.

R6.1a introduces a synthetic artifact-contract baseline, including longer repeats
for cases with over-5% spread between run medians. Recompute/audit its plot without
running any benchmark or contacting VMware:

```sh
python scripts/benchmarks/measure_artifact_contract.py --plot-only --report docs/benchmark-results/2026-10-02-r61a
```

Use `--binary RELEASE_CONTRACT_BENCH --report NEW_DIRECTORY` for new observations.
The precomputed binding comparison is distinct from identity hashing; whole-process
CPU/RSS include Criterion's own warmup and analysis. Prior copy/flush follow-ups
remain separate.

R6.1b records synthetic durable-job phases separately on Btrfs and volatile tmpfs,
including longer job batches after over-5% run-median spreads:

```sh
python scripts/benchmarks/measure_ownership.py --plot-only --report docs/benchmark-results/2026-10-02-r61b
```

For new measurements, build `ownership_probe` and supply `--binary`, a new report
and new `--btrfs-parent` / `--tmpfs-parent` fixture parents. Every job performs ten
journal commits with no VMware or disk payload operations. Reopening/cleanup are
checked, and each successful synthetic fixture is removed outside timed phases.
