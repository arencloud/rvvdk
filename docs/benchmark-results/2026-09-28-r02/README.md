# R0.2 — Concurrent worker shutdown

Date: 2026-09-28. Baseline: `3302c1b`. Candidate: baseline plus
[candidate.patch](candidate.patch). The baseline uses the same candidate manifest
and `scheduler.rs` benchmark, with its original runtime. Source and executable
fingerprints are in [environment.json](environment.json).

## Correctness and contract

The coordinator no longer retains the queue receiver while producing work.
Workers record the first error before publishing cancellation, stop between work
items, and release their receivers. Channel disconnection wakes a blocked
producer; dropping the sender wakes idle workers. Scope exit joins all workers.
Successful production still drains the queue. The original backend error survives
the resulting `WorkQueueClosed` error.

The first seven regression cases ran against the original runtime: six exceeded
their five-second subprocess deadlines, and the idle-worker producer-error case
passed. [The complete red run](regressions-before.txt) is retained. The final suite
adds healthy-peer stopping, public-copy error/flush behavior, first-recorded-error
ordering, and successful queue draining: eleven new tests, 213 workspace tests
passing. Strict Clippy across all targets and formatting also pass.

The full-queue tests gate every worker inside its failing operation, fill the
queue, and verify `try_send` reports Full before releasing the workers. They cover
read, write, zero, and discard, with 2/4 workers and capacities 1/4. Tests also
verify buffer return with fewer buffers than workers, and that all-worker panics
unblock the producer and still propagate. Potentially hanging tests run in child
processes which are killed and reaped on timeout.

This is cooperative error shutdown, not public cancellation or backend deadlines.
Already-dispatched work may finish; synchronous backend calls must return before
joining can finish. Concurrent errors are ordered by recording under the error
mutex, not by an externally observable wall-clock ordering. See
[ADR-0008](../../adr/0008-streaming-work-scheduler.md#shutdown-contract--r02-2026-09-28).

## Workloads and timing

- Dense, deterministic incompressible 16 MiB input. Memory devices and buffered
  local files are measured separately. Files reside on the development machine's
  NVMe, on Btrfs with compression and dm-crypt; `/tmp` is not used for storage data.
- 64 KiB blocks, 2/4 workers, queue capacities 1/16, one 4096-aligned buffer per
  worker. A 4-worker, 16-item, 4 KiB memory case increases scheduling pressure.
- Time includes the `copy` call: planning, buffers, threads, scheduling, copying,
  joining, and destination flush. Destination reset with different incompressible
  bytes, its flush, and full byte-for-byte read-back are outside every iteration's
  timer. Byte and block counters are verified too.
- Flat sampling, 20 samples, 300 ms warmup, 2 s target measurement. Three runs per
  source version, in baseline/candidate, candidate/baseline, baseline/candidate
  order. Executables use isolated Cargo target directories and distinct hashes.
- Error workloads use synthetic immediate-failure backends, four workers, queue
  capacity one, four 4 KiB buffers, and a 16 MiB logical extent. Read/write use Data,
  zero uses Zero, and discard uses Hole. Each iteration checks the original error.
  Complete API-call time and first-injected-fault-to-return time are measured in
  separate cases, with three candidate runs. No successful-copy bandwidth is
  inferred from an error workload.

These are warm, small, development-laptop workloads. No CPU affinity, frequency
control, dedicated runner, CPU/RSS counters, or operation-level I/O latency
instrumentation was used. The memory backend has its own synchronization. This
does not qualify sustained storage bandwidth or isolate the cost of atomic loads.

## Results

All values below are medians of three run medians. Changes describe elapsed
time, not statistical proof of a regression or speedup. GiB/s is logical data
size divided by measured call time; these warm results are not sustained device
bandwidth. Raw samples and Criterion confidence intervals are preserved in JSON.

### Initial successful-copy comparison

| Backend / workers / queue / block | Baseline ms | Candidate ms | Time change | Candidate GiB/s |
|---|---:|---:|---:|---:|
| `buffered_file/w2_q1_b65536` | 11.7527 | 11.9222 | +1.44% | 1.311 |
| `buffered_file/w2_q16_b65536` | 12.1270 | 12.0800 | -0.39% | 1.293 |
| `buffered_file/w4_q1_b65536` | 11.3174 | 11.3374 | +0.18% | 1.378 |
| `buffered_file/w4_q16_b65536` | 10.9570 | 11.3495 | +3.58% | 1.377 |
| `memory/w2_q1_b65536` | 1.2184 | 1.4142 | +16.07% | 11.049 |
| `memory/w2_q16_b65536` | 1.1295 | 1.3896 | +23.03% | 11.244 |
| `memory/w4_q1_b65536` | 1.2146 | 1.2990 | +6.95% | 12.029 |
| `memory/w4_q16_b65536` | 1.1378 | 1.1401 | +0.20% | 13.705 |
| `memory/w4_q16_b4096` | 3.5084 | 4.1304 | +17.73% | 3.783 |

Several memory cases crossed the 5% investigation threshold (up to +23.03%).
Run-to-run variation was substantial: the initial 4 KiB memory candidate medians
ranged from 3.092 to 4.824 ms. The file-copy aggregate changes stayed below 4%.
The complete memory matrix was therefore repeated with longer sampling and CPU
affinity, using the same executable files.

### Memory follow-up with CPU affinity

`taskset -c 2-6` restricts both variants to five distinct physical cores on this
host. Sampling used 40 flat samples, 500 ms warmup, and a 4 s target, with
candidate/baseline, baseline/candidate, candidate/baseline order. The powersave
governor and other applications remained active; this is not a dedicated runner.
Bracketed values are the minimum–maximum of the three run medians, not individual
copy latency bounds.

| Workers / queue / block | Baseline ms [range] | Candidate ms [range] | Time change |
|---|---:|---:|---:|
| `w2_q1_b65536` | 1.4794 [1.4347–1.5389] | 1.4839 [1.4256–1.4841] | +0.30% |
| `w2_q16_b65536` | 1.4655 [1.4467–1.4668] | 1.4123 [1.4031–1.4630] | -3.63% |
| `w4_q1_b65536` | 1.2743 [1.2578–1.2769] | 1.2594 [1.2465–1.2764] | -1.17% |
| `w4_q16_b65536` | 1.2451 [1.2327–1.2477] | 1.2364 [1.2300–1.2597] | -0.70% |
| `w4_q16_b4096` | 3.7208 [3.6887–3.8232] | 3.7217 [3.7160–3.8983] | +0.03% |

The initial increases did not persist in this longer, affinity-constrained
comparison: aggregate changes ranged from −3.63% to +0.30%. This suggests
scheduling/environment variation contributed to the initial results; it does
not establish the exact cause or prove that every workload is regression-free.
Both result sets are retained. See [followup-measurements.json](followup-measurements.json).

### Candidate failure termination

| Injected operation | Complete call µs | First fault to return µs | Fault-to-return run-median range µs |
|---|---:|---:|---:|
| read | 68.960 | 17.151 | 16.393–17.925 |
| write | 71.443 | 17.732 | 15.971–21.131 |
| zero | 70.187 | 17.036 | 16.537–18.459 |
| discard | 71.400 | 18.288 | 16.966–21.331 |

The old failure path deadlocks, so no numeric before/after error-latency ratio is
reported. These measurements include returning every scoped worker, but the
synthetic backends themselves return immediately. They cannot bound a blocked
real device operation.

### Performance disposition

Accept the required correctness fix with this targeted performance evidence.
No aggregate slowdown exceeds 5% in the file matrix or the memory follow-up; the
initial unpinned memory increases remain visible and investigated. No execution
defaults were tuned. PERF.0 still requires dedicated-runner, larger/direct-I/O,
and broader workload qualification; this step does not close those tasks.

## Failed harness attempt

[harness-attempt-1.txt](harness-attempt-1.txt) contains an excluded partial run.
The first harness called `LocalFileBlockDevice::flush` on a read-only source,
which correctly returned `Unsupported` before the file measurements. The harness
now syncs the source fixture while creating it, before opening it read-only.
Both variants were rebuilt with this identical correction, and the comparison
restarted with fresh Criterion output directories. None of that partial run's
memory measurements contributes to the results.

## Reproduction and evidence

Build the recorded baseline with the candidate's `Cargo.toml` and
`benches/scheduler.rs` copied into it. Build the candidate by applying
`candidate.patch` to a separate checkout of `3302c1b`. Keep build directories
separate, including when both checkouts have identical package names.

```bash
# Run from each respective checkout, using different absolute output paths.
cargo bench -p rvvdk-datamover --bench scheduler --no-run \
  --target-dir /absolute/path/to/variant-build --message-format=json

# Use the executable from Cargo's compiler-artifact JSON, not an ambiguous glob.
CRITERION_HOME=/absolute/path/to/run-results \
RVVDK_BENCH_DIR=/absolute/path/to/storage \
/absolute/path/to/variant-executable --bench scheduler_copy --noplot

# Candidate only: the original runtime deadlocks on failure workloads.
CRITERION_HOME=/absolute/path/to/failure-results \
RVVDK_BENCH_DIR=/absolute/path/to/storage \
/absolute/path/to/candidate-executable --bench scheduler_failure --noplot
```

Run each comparison sequentially in the order above, using fresh output
directories. [measurements.json](measurements.json) preserves the exact commands,
Criterion estimates, and raw iteration/time samples for every accepted run.
`baseline-*.txt`, `candidate-*.txt`, and `failure-candidate-*.txt` retain console
output. Source patch and harness hashes identify the measured source even though
measurements were taken before committing. The commit containing this report
records R0.2 and its evidence together.
