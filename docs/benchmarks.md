# rvvdk Benchmarks

## Performance policy — adopted 2026-09-28

Performance is a requirement throughout implementation. Every work package in
the [roadmap](roadmap.md) must record its performance impact in the
[implementation log](implementation-log.md). Performance qualification begins
with R0; R9 is final qualification, not the first time we measure.

The results below this policy are historical measurements. They are not yet an
approved regression baseline: the review identified unequal flush boundaries in
the direct io_uring comparison. The harness correction is committed, but those
historical results still need replacement measurements. Do not derive engine
defaults or performance claims from an unmatched comparison.

Latest measured step: [R4.1 bounded VMDK descriptors](benchmark-results/2026-09-29-r41/README.md)
includes SVG/PNG charts, unchanged RAW planning/copy controls, and candidate-only
parser costs for small/1,024-extent descriptors and early/late limit rejection.
All adverse aggregates/pairs trigger longer repeats; shared-host and earlier
qualification limits remain explicit.

### PERF.0 — Establish a reproducible baseline

- [x] Equalize timed durability boundaries: the low-level native direct benchmark
  now calls the local destination's flush inside the timed interval, matching
  the threaded mover. Historical direct-I/O results still require fresh runs
  before they can serve as a corrected baseline.
- [ ] Choose and record a dedicated storage directory, disk/filesystem, and
  representative workload sizes. Keep tmpfs measurements separate from storage
  throughput; `/tmp` is tmpfs in the reviewed environment.
- [ ] Identify each tested source state by commit plus dirty patch/content hashes,
  including relevant untracked source. Preserve enough evidence to reconstruct
  that state. A commit SHA alone does not identify a dirty worktree.
- [ ] Capture the environment and initial repeated baseline results using the
  same harness that will test the candidate. If the harness changes, rerun both
  baseline and candidate with the corrected harness.
- [ ] Add reproducible workload configuration and result collection incrementally.
  Record which metrics are actually instrumented; do not invent unmeasured
  latency, CPU, or memory values.

PERF.0 is in progress. Complete the relevant workload baseline before accepting
throughput comparisons for engine changes. Immediate safety fixes can proceed if the old path cannot safely run;
record the unavailable comparison and establish the first safe baseline.

### Required record for an implementation step

| Field | Required evidence |
|---|---|
| Identity | Work-package ID, baseline and candidate commit/source fingerprints |
| Workload | Disk size, data pattern/seed, Data/Zero/Hole proportions, extent count and sizes |
| Configuration | Executor, block size, workers, queue depth/read window, buffers and memory budget |
| Environment | CPU, memory, storage/controller, filesystem/mount options, encryption/compression, kernel, Rust version and build flags |
| Cache and allocation | Warm/cold policy, buffered/direct mode, fresh/preallocated destination, source/destination placement |
| Timing | Setup boundaries, plan time, copy time, flush time, and whether verification is included |
| Results | Elapsed time, logical and transferred throughput, variability, correctness result, and available CPU/RSS/I/O metrics |
| Decision | Accepted, needs investigation, or justified tradeoff; explanation and follow-up |

Use optimized builds and repeat comparable baseline/candidate runs on the same
machine. Alternate run order to help reveal thermal/cache drift. Reset destination
state consistently outside the timed region. Observe contention and device/cache
effects; one favorable run is not a tuning result. Store raw output with the
summary, not just screenshots or a selected throughput number.

For storage qualification, include a sustained workload large enough to expose
cache/burst effects within the lab's available capacity. Define a controlled
cache policy without relying on a global cache drop on a shared development host.
Use physical hardware for final throughput claims; nested ESXi remains useful
for functionality but introduces extra performance variables.

### Workload matrix

| Change area | Minimum relevant comparisons |
|---|---|
| Scheduler / native pipeline | Dense RAW, fragmented sparse RAW, workers/QD 1 and representative tuned settings |
| Sparse / zero handling | Dense, 25/50/75% holes, zero-only, hole-only, nonzero-prefilled destination; inspect allocated blocks |
| Alignment / local backend | Buffered, direct, mixed modes, aligned bulk, small disks and odd tails on real storage |
| Planning | Dense and highly fragmented extent maps; plan latency and peak memory separately from copy |
| Progress / cancellation | Observer disabled, no-op observer, realistic callback cadence; throughput and cancellation latency |
| VMDK | Flat, supported sparse layouts, fragmented grains, parent-chain depth; decoded logical-byte equality |
| VMware transport | Full-copy throughput, latency/queue-depth response, network utilization, host load, reconnect behavior |
| CBT / resume | Small/large changed sets, metadata/checkpoint overhead, time and bytes needed for recovery |

Keep separate accounting for logical disk bytes, payload read/written, zeroed and
deallocated bytes. Sparse logical throughput can exceed storage transfer rate;
report both so that removing I/O is distinguishable from moving bytes faster.

Tune block size, concurrency, queue depth, read window, and buffer count under
comparable memory limits. Prefer a small targeted sweep, then expand only when
the measurements reveal a useful direction. Maintain workload-specific results;
there may be no universal best setting.

### Acceptance and regression handling

1. Correctness and output verification must pass before timing informs a decision.
2. Every implementation package has a performance disposition. Documentation-only
   changes can say `N/A — no runtime change`; an unrun benchmark must say `Pending`
   with a reason, never `Passed`.
3. Initially investigate a repeatable throughput decrease or elapsed-time increase
   greater than 5% on a representative workload, as well as any breach of a memory,
   latency, or bounded-resource contract. This is a project investigation threshold,
   not a claim that the current environment can resolve 5% reliably.
4. Interpret changes against measured variability. Repeat noisy comparisons before
   deciding. Do not hide regressions by increasing memory, changing flush policy,
   shrinking the data set, or switching filesystems.
5. A necessary correctness/safety fix may cost performance. Record that tradeoff
   and its follow-up explicitly; do not retain incorrect behavior to win a benchmark.
6. Establish numeric workload targets after PERF.0. Until then, avoid unsupported
   promises of a fixed GiB/s rate or that io_uring is always faster.

Run affected comparisons per implementation package; run the broader matrix at
milestone boundaries. A dedicated benchmark runner should eventually execute the
stable matrix on a schedule. Shared CI timing is smoke evidence, not a substitute
for repeatable storage measurements.

### R0.4 validation measurements

The [R0.4 report](benchmark-results/2026-09-28-r04/README.md) records native
entry-point overhead, Zero fallback, fragmented Data plans, and buffered/direct
dense copies with threaded controls. The harness checks complete destination
contents after every copy; setup and verification stay outside the timer and
destination flush stays inside. Empty-call microbenchmarks report absolute
nanoseconds as well as percentage changes because their baseline is very small.

### R0.5 preflight measurements

The [R0.5 report](benchmark-results/2026-09-28-r05/README.md) measures current
endpoint inspection during planning and the full copy boundary for memory Zero
fallback, fragmented native Data, and buffered/direct dense copies. Both threaded
and native execution gained preflight; the threaded measurements are comparators,
not unchanged controls. Planning latency is reported separately from copy time.

### R1.1 portable API measurements

The [R1.1 report](benchmark-results/2026-09-28-r11/README.md) compares dense and
fragmented planning, unobserved/observed threaded copies, and buffered/direct
native controls against R0.5. The unchanged progress harness exercises the old
RAW-bound API and the new portable API with equivalent logical workloads.
`portable_plan` adds candidate-only memory measurements for static versus dynamic
dispatch with one/four workers. These are API overhead measurements, not storage
qualification or a reason to change concurrency defaults.

### R1.2 semantic execution measurements

The [R1.2 report](benchmark-results/2026-09-28-r12/README.md) compares observed,
no-op-observer, and unobserved dense/fragmented file copies, dynamic memory copies,
portable Zero/Hole fallback and accelerated mixed operations, and native Zero
fallback. The new `semantic_policy` harness runs unchanged against both runtimes,
with reset/readback untimed and flush included. It separates one-worker observed
execution from four-worker execution; these are not equivalent parallelism costs.
Final fragmented no-op observation and sequential Hole/Zero fallback qualification
remain open; the report preserves conflicting runs and does not claim a clean
performance pass.

### Result storage convention

Use `docs/benchmark-results/<run-id>/` for concise tracked summaries, environment
metadata, workload configuration, and small raw text/JSON records. Each summary
must link its baseline and candidate evidence. Record an immutable artifact
location and checksum for large traces/reports; disk images and bulky Criterion
output do not belong in the source tree. `target/criterion` alone is not durable
project history.

Keep existing `benchmark-m11.txt` as historical input until its provenance and
environment are documented; it has not been deleted or promoted to a baseline.

### Plots for recorded results

Future measured implementation steps include SVG figures in their reports and
PNG copies for sharing. Use the [reusable plot generator](../scripts/benchmarks/README.md)
and [R2.3 visual report](benchmark-results/2026-09-29-r23/README.md#visual-results)
as the initial format. Render existing evidence after validation; plotting does
not rerun benchmarks or replace raw samples.

Show planning and copying separately, identify measured source revisions and
conditions, label units, and retain paired-run variability and outliers. Include
aggregate changes and absolute timings; a large percentage on a tiny baseline
needs both. Show repeats and candidate-only experiments separately. Use historical
trends only for matched workloads/conditions, without chaining per-step changes.
Record the plot configuration, computed values, generator/input/output hashes,
and plotting dependency versions with the step. Inspect the output before commit.

### Planning and observer overhead

The `progress` target isolates structural planning and single-worker observed
execution. It compares unobserved, no-op observer, and counter observer copies
for a 16 MiB dense source and a 50% sparse source with alternating 64 KiB extents.
Every timed copy includes destination flush. Destination reset/flush and full
output read-back verification occur outside the timer, on every iteration.

```bash
RVVDK_BENCH_DIR=/path/to/benchmark-storage \
cargo bench -p rvvdk-datamover --bench progress -- --noplot
```

Source reads are buffered and warm. This is an API/observer overhead comparison,
not a sustained physical-device throughput result. A changed extent map or wrong
output fails the benchmark rather than producing a misleading timing result.
Before/after builds must use separate Cargo target directories to prevent reuse
of stale artifacts between same-named workspace packages.

First recorded comparison: [R0.1 results](benchmark-results/2026-09-28-r01/README.md).

### Concurrent scheduling and failure shutdown

The `scheduler` target measures successful 16 MiB dense copies in memory and on
buffered local files, using 2/4 workers and queue capacities 1/16. The normal block
size is 64 KiB; an additional memory case uses 4 KiB blocks to increase scheduler
pressure. Timed copies include planning, pool/thread setup, joining, and flush.
Destination reset/flush and complete output verification occur outside the timer.

Its failure group injects read/write/zero/discard errors with four workers and a
one-item queue. It separately measures complete API-call time and the interval
from the first injected backend error to API return, which includes worker joining.
These synthetic backends return promptly; results do not bound real backend stalls.

```bash
RVVDK_BENCH_DIR=/path/to/benchmark-storage \
cargo bench -p rvvdk-datamover --bench scheduler -- scheduler_copy --noplot

cargo bench -p rvvdk-datamover --bench scheduler -- scheduler_failure --noplot
```

The failure group must not run on the old runtime: it deadlocks. Baseline comparison
is restricted to successful copies, using identical harnesses and isolated build
directories. See the [R0.2 report](benchmark-results/2026-09-28-r02/README.md) for
source fingerprints, measurements, and limitations.

### Native resource lifetime overhead

The `native_lifetime` target compares unchanged high-level native-copy calls
across the R0.3 ownership change. It uses 16 MiB dense buffered/O_DIRECT workloads,
four queue/block configurations, and threaded controls. Each timed copy includes
ring/pool/endpoint setup, copy completion, cleanup, and destination flush. Reset,
full output verification, and FD-count checks are outside every iteration's timer.

```bash
RVVDK_BENCH_DIR=/path/to/benchmark-storage \
cargo bench -p rvvdk-datamover --bench native_lifetime -- --noplot
```

Only successful paths are compared against the old runtime. Lifetime/error paths
use candidate fault-injection tests, not unsafe baseline throughput runs. The
[R0.3 report](benchmark-results/2026-09-28-r03/README.md) includes executable/source
fingerprints, process CPU/RSS measurements, and the exceptional resource-retention
policy. Whole-process resource usage includes setup and verification; it is not a
measurement of per-copy resource cost.

## Purpose

rvvdk uses benchmarks to validate performance changes rather than
assuming that architectural changes improve throughput.

The initial benchmark establishes a baseline for the sequential
DataMover.

## Current benchmark path

```text
LocalFileBlockDevice
        |
        v
     RawDisk
        |
        v
    DataMover
        |
        v
     RawDisk
        |
        v
LocalFileBlockDevice
```

## Concurrent DataMover baseline

The synchronous concurrent DataMover was benchmarked with a 1 MiB
block size.

The initial Milestone 12 baseline on the development system was:

| Workers | Throughput |
|---:|---:|
| 1 | 4.94 GiB/s |
| 2 | 8.36 GiB/s |
| 4 | 8.06 GiB/s |
| 8 | 6.27 GiB/s |

For this cached local-file workload, two workers provided the highest
measured throughput.

These results are environment-specific and must not be interpreted as
universal optimal concurrency settings.

## Streaming scheduler

The Milestone 13 scheduler replaces complete work-plan materialization
with incremental planning and a bounded producer/consumer queue.

```text
Extent
  |
  v
ExtentWorkIter
  |
  v
bounded queue
  |
  +---- worker
  +---- worker
  +---- worker
```

### Milestone 13 results

The bounded streaming scheduler produced the following cached
local-file results with a 1 MiB block size:

| Workers | Throughput |
|---:|---:|
| 1 | 5.02 GiB/s |
| 2 | 8.01 GiB/s |
| 4 | 8.10 GiB/s |
| 8 | 6.12 GiB/s |

The previous Milestone 12 scheduler produced approximately:

| Workers | Throughput |
|---:|---:|
| 1 | 4.94 GiB/s |
| 2 | 8.36 GiB/s |
| 4 | 8.06 GiB/s |
| 8 | 6.27 GiB/s |

The bounded scheduler therefore preserved approximately the same
performance profile while eliminating block-level work-plan memory
growth with disk size.

Queue-capacity testing with two workers produced:

| Queue capacity | Throughput |
|---:|---:|
| 1 | 7.78 GiB/s |
| 2 | 6.91 GiB/s |
| 4 | 7.31 GiB/s |
| 8 | 7.33 GiB/s |
| 16 | 7.10 GiB/s |
| 32 | 7.09 GiB/s |
| 64 | 6.98 GiB/s |
| 128 | 7.28 GiB/s |

No throughput advantage was observed from maintaining a large pending
work queue for this workload.

The default synchronous scheduler queue capacity is therefore kept
small. Queue capacity remains configurable because other storage and
network backends may behave differently.

These measurements are page-cache-heavy local-file benchmarks and are
not measurements of physical storage throughput.

## Direct I/O benchmark

The direct-I/O benchmark exercises:

```text
O_DIRECT source
      |
      v
LocalFileBlockDevice
      |
      v
RawDisk
      |
      v
DataMover
      |
      v
RawDisk
      |
      v
LocalFileBlockDevice
      |
      v
O_DIRECT destination
```

### Milestone 14 direct-I/O results

The Linux direct-I/O backend was verified using `strace`. The local
data file was opened with:

```text
O_RDWR | O_DIRECT | O_CLOEXEC
```

### Hybrid direct-I/O regression check

After introducing buffered fallback for unaligned requests, the aligned
direct-I/O benchmark was repeated.

Aligned requests continue to use the `O_DIRECT` descriptor, while only
unaligned requests use the secondary buffered descriptor.

The aligned benchmark showed no material architectural regression
relative to the Milestone 14 direct-I/O baseline.

This confirms that hybrid tail handling does not move the aligned bulk
copy path onto buffered I/O.

### Milestone 16 storage environment

The storage-backed direct-I/O benchmark was moved from `/tmp` to a
dedicated benchmark directory on the development system.

Storage path:

```text
rvvdk
  |
  v
O_DIRECT
  |
  v
Btrfs (NOCOW benchmark directory)
  |
  v
LUKS / dm-crypt
  |
  v
Samsung MZVL21T0HCLR-00BL7 NVMe
```
## M17F — io_uring O_DIRECT comparison

The ownership-safe balanced io_uring pipeline was compared with the
existing threaded DataMover using O_DIRECT on both execution paths.

### Environment

The storage-backed benchmark used:

- 2 GiB logical copy size
- 1 MiB transfer blocks
- 4096-byte buffer alignment
- deterministic incompressible source data
- Btrfs NOCOW benchmark directory
- LUKS / dm-crypt
- Samsung MZVL21T0HCLR-00BL7 NVMe
- Criterion flat sampling
- 10 samples
- 30 second measurement target

Both implementations used the same rvvdk local direct-I/O backend for
opening source and destination files.

### Representative results

| Engine | Parallelism | Throughput |
|---|---:|---:|
| Threaded O_DIRECT | 1 worker | ~858 MiB/s |
| Threaded O_DIRECT | 2 workers | ~982 MiB/s |
| Threaded O_DIRECT | 4 workers | ~708 MiB/s |
| io_uring O_DIRECT | QD 1 | ~866 MiB/s |
| io_uring O_DIRECT | QD 2 | ~613 MiB/s |
| io_uring O_DIRECT | QD 4 | ~928 MiB/s |
| io_uring O_DIRECT | QD 8 | ~1011 MiB/s |
| io_uring O_DIRECT | QD 16 | ~1012 MiB/s |

The most stable configurations were:

```text
threaded workers=2:
~977-987 MiB/s

io_uring QD8:
~1008-1014 MiB/s

io_uring QD16:
~1010-1015 MiB/s
```
