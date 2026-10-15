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

## Explicit local recovery commands (R6.1d.4b)

`measure_recovery_cli.py` compares the actual CLI with equivalent direct native
recovery, measures test-only adapter spans, and pairs existing commands against a
frozen reference. `measure_recovery_cli_supplement.py` adds human/JSON assessment
baselines and frozen native publication/recovery controls. Both are local authored
fixture harnesses; they never accept VMware credentials. Run them serially, with
separate private fixture parents and frozen release executables. Use `--help` for
the required paths. The profile executable is the CLI release unit-test binary;
the library executables are the vSphere release unit-test binaries.

Use a private mount/network namespace for tmpfs, with `size=512m,mode=0700,huge=never`;
keep persistent Btrfs separate. Do not overlap builds, tests, tracing or plots with
timed work. All groups retain three rounds of four pairs and three of eight pairs.
The supplemental script requires a new supplemental index; the main script refuses
an existing campaign unless explicitly resumed. Preserve failures before resuming.

Render and validate the saved report afterward:

```bash
target/benchmark-plots/bin/python scripts/benchmarks/plot_recovery_cli.py \
  docs/benchmark-results/2026-10-14-r61d4b
```

The ten PNG/SVG figures include existing CLI changes, whole/direct recovery costs,
adapter phases, recording controls, memory/allocation, assessment, frozen library
controls, variation and real signal outcomes. Hashes in `plots/audit.json` bind the
figures to their inputs. Direct test-process RSS includes fixture setup and cannot
be compared as isolated operation memory against CLI child RSS. Signed profile
clock-accounting gaps and zero-resolution CPU phases remain in the report. See the
[recorded campaign and limits](../../docs/benchmark-results/2026-10-14-r61d4b/README.md).

## SHA-256 backend qualification

[PERF.1a](../../docs/benchmark-results/2026-10-14-perf1a/README.md) records a rejected
backend experiment with native and CPU-masked correctness, paired micro/workflow
measurements and independent byte/allocation/durability checks. The standalone
Linux x86-64 Rust probe lives in `hash_probe/` with its own Cargo manifest/lock;
it is excluded from the production workspace. Python orchestrates offline local
measurements and plots; it does not implement VMware transport.

```sh
python3 scripts/benchmarks/check_hash_evidence.py docs/benchmark-results/2026-10-14-perf1a
target/benchmark-plots/bin/python scripts/benchmarks/plot_hash_backends.py docs/benchmark-results/2026-10-14-perf1a
```

`measure_hash_backends.py` and `measure_hash_workflows.py` require explicit frozen
binaries and a new report location; see `--help` and the report for the private
filesystem setup. They refuse campaign-index overwrite. The plotter validates all
recorded samples and regenerates matching summaries before writing four PNG/SVG
pairs. Positive percentage changes mean slower. Round ratios/spreads describe
variation and are not confidence intervals. Emulated correctness is never native
performance evidence. Keep native/probe lifetime RSS separate from CLI child RSS. Child high-water RSS
can include inherited launcher memory before exec; PERF.1a records a diagnostic,
so these counters must not be treated as isolated CLI working sets.

[PERF.1b](../../docs/benchmark-results/2026-10-15-perf1b/README.md) adds native
non-SHA fallback qualification, conditional-backend controls on both CPU profiles,
and seven PNG/SVG pairs. `measure_hash_fallback.py` extends the micro matrix through
512 MiB; `qualify_hash_candidate.py` exercises frozen library/CLI fixtures before
timing. `measure_hash_workflows.py` supports explicit `--disk-fs xfs|btrfs` and
`--disk-parent` (the earlier `--btrfs-parent` remains an alias).
`measure_assessment_controls.py` records separately declared A/A and A/B startup
diagnostics; these never replace the primary observations.

```sh
python3 scripts/benchmarks/check_hash_fallback.py docs/benchmark-results/2026-10-15-perf1b
target/benchmark-plots/bin/python scripts/benchmarks/plot_hash_fallback.py docs/benchmark-results/2026-10-15-perf1b
```

The checker replays 3,120 observations, independent digests, frozen binary/log hashes
and corruption rejections. Use normal Python execution: `-O` disables assertions
used by the evidence validators. SHA-NI and non-SHA results are separate same-host
comparisons. Changed guest resources are recorded; old/new or cross-host timing
ratios cannot establish an implementation gain. No VMware credentials are accepted
by these offline fixture tools. The shared tools still replay PERF.1a summaries
byte-identically.


## PERF.1c transfer and worker hashes

`measure_transfer_hashes.py` compares frozen reference/candidate library,
`discovery` integration-test and CLI binaries on one native CPU profile. Existing
opt-in Rust fixtures perform full proof/owned transfer, artifact admission and
separate seal/admit measurements, including small/bulk/stored-zero data. Pass six
binary paths, a fresh report directory, empty disk/tmpfs parents and an ignored
private scratch directory; `--help` lists flags. The tmpfs control must be mounted
in a private namespace with `size=512m,mode=0700,huge=never`, and loopback enabled.
Fixed CPU 0 and serial measurements are intentional. No VMware credentials or
production source paths are accepted by these tools.

`measure_transfer_hash_controls.py` records the separately declared A/A and adjacent
A/B controls, selecting `--profile sha-host` or `--profile non-sha-host`. Frozen
binary filenames and empty `fixtures`/`private` directories are relative to
`--root`. Original measurements are retained. `--summarize` only replays a report.

```sh
python3 scripts/benchmarks/check_transfer_hashes.py docs/benchmark-results/2026-10-15-perf1c --self-test
target/benchmark-plots/bin/python scripts/benchmarks/plot_transfer_hashes.py docs/benchmark-results/2026-10-15-perf1c
```

The checker validates 2,640 observations, complete pair coverage, independent
oracle/allocation claims, binary and raw-file hashes, logs, plots and 14 corrupt
copies. Add `--source .` only at the qualified implementation revision. Python
`-O` is unsupported because it removes assertion gates. Five PNG/SVG plot sets
show original adverse observations, round ranges, separate controls and costs;
they do not claim live export or cold/disk-scale throughput. See the
[report and reproduction notes](../../docs/benchmark-results/2026-10-15-perf1c/README.md).
