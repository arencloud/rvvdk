# R0.3 — io_uring resource lifetime repair

Date: 2026-09-28. Baseline: `52ddb14`. Candidate: baseline plus
[candidate.patch](candidate.patch). The baseline receives only the identical new
benchmark manifest/target, using the unchanged high-level `copy_native` API.
Separate build directories prevent stale-artifact reuse. Exact source and binary
hashes are in [environment.json](environment.json).

## Correctness and safety disposition

All 231 workspace tests pass, including 18 new bounded fault/lifetime tests.
Formatting and strict all-target Clippy pass. The existing positional read/write
tests now exercise the owned API; the three original engine tests remain.

The old descriptor behavior was reproduced without invalid memory access:
[the exact old-API test](fd-regression-before.rs) queues a read, closes the caller's
file, and opens `/dev/zero` using the same numeric descriptor. The
[baseline result](fd-regression-before.txt) fails because it reads the replacement.
The equivalent candidate lifetime test passes after dropping both caller handles.
The original borrowed-buffer hazard was established by control-flow review;
we did not execute a use-after-free experiment or benchmark that unsafe error path.

New tests use real rings and test-only fault injection. They cover interrupted
enter calls, errors before/after kernel acceptance, actual one-SQE partial
submission (with and without an injected subsequent error), failed waits,
unknown/duplicate/nonfinal/missing CQEs, malformed results, negative operation
CQEs, panic unwinding, FD reuse, queue capacity, identifier wraparound, and
idempotent shutdown. Each potentially hanging case runs in a child process with
a ten-second kill/reap deadline. Fault hooks do not exist in release builds.

The engine no longer accepts borrowed slices. Each operation owns its guard and
shared owned descriptor before SQE publication. Only a matching final CQE permits
resource release. Normal and negative completions return buffers; unconfirmed
cleanup permanently retains affected resources rather than permitting unsafe
reuse. Explicit shutdown reports that count, and range-copy errors preserve the
original cause alongside cleanup failure. See the
[safety argument and migration](../../adr/0013-io-uring-buffer-ownership.md).

This exceptional retention can keep entire pools and descriptor references alive
until process exit. Tests verify retention deliberately; it is not presented as
successful resource reclamation. Draining can still block on a backend that does
not complete, and may submit previously queued writes. No rollback or deadline is
provided. Normal-copy benchmarks exercise neither injected errors nor retention.

## Workloads and measurements

- 16 MiB dense deterministic incompressible data. Buffered and O_DIRECT file
  descriptors on the same NVMe/Btrfs/dm-crypt filesystem are separate profiles.
- Native cases: queue 1 / 64 KiB blocks; queue 8 / 64 KiB, 4 KiB, and 1 MiB blocks.
  One 4096-aligned buffer per queue slot; default read windows are 1 and 4 for
  depths 1 and 8 respectively. Threaded controls use four workers,
  64 KiB blocks, and the default queue capacity four.
- Timed native interval includes ring/pool/endpoint setup, copying, explicit
  cleanup, and destination flush. Threaded control includes its own flush.
  Destination reset with different incompressible bytes, reset flush, byte-for-byte
  read-back, byte-count checks, and FD-count verification happen outside the timer
  on every iteration. Endpoint files are opened before benchmarking each profile.
- 20 flat samples, 300 ms warmup, two-second target. Three sequential runs per
  variant, in baseline/candidate, candidate/baseline, baseline/candidate order.
  Both run with `taskset -c 2-6`, selecting five physical cores on this machine.
- GNU `time -v` records process user/system CPU time, peak RSS, context switches,
  page faults, and file-system counters. These include setup, verification,
  reset/flush, and Criterion analysis; they are not per-copy resource costs.
- `/proc/self/fd` counts match before and after every successful copy. Existing
  and new tests separately verify buffers return after confirmed completions.

The machine remains a development laptop with the powersave governor and other
applications active. Buffered input is warm. Direct I/O does not imply cold device
or controller caches. The small dataset does not qualify sustained bandwidth.
No per-I/O latency histogram or isolated copy CPU/RSS instrumentation was added.

## Results

Values are medians of the three run medians. Percentages describe elapsed time;
negative means lower time. GiB/s is logical bytes divided by call time, not a
sustained device rating. Raw samples, confidence intervals, and all runs are kept.

### Initial comparison

| Profile / queue / block | Baseline ms | Candidate ms | Time change | Candidate GiB/s |
|---|---:|---:|---:|---:|
| `buffered/q1_b65536` | 21.8501 | 22.3701 | +2.38% | 0.698 |
| `buffered/q8_b1048576` | 22.6964 | 22.9871 | +1.28% | 0.680 |
| `buffered/q8_b4096` | 33.6956 | 33.9376 | +0.72% | 0.460 |
| `buffered/q8_b65536` | 19.7160 | 19.8706 | +0.78% | 0.786 |
| `buffered/threaded_control` | 17.9371 | 18.1533 | +1.21% | 0.861 |
| `direct/q1_b65536` | 52.4220 | 50.4731 | -3.72% | 0.310 |
| `direct/q8_b1048576` | 26.9911 | 26.5864 | -1.50% | 0.588 |
| `direct/q8_b4096` | 188.9032 | 177.3860 | -6.10% | 0.088 |
| `direct/q8_b65536` | 36.5393 | 36.4608 | -0.21% | 0.429 |
| `direct/threaded_control` | 27.1730 | 31.3163 | +15.25% | 0.499 |

Native aggregate changes range from −6.10% to +2.38%, but the direct threaded
control changed by +15.25% despite its runtime being unchanged. Run order also
shows considerable absolute drift. This rules out treating the apparent native
improvements as established speedups and prompted a longer direct comparison.

### Longer direct follow-up

The native queue-8 / 64 KiB case and threaded control were repeated with 40 flat
samples, 500 ms warmup, and a four-second measurement target, using the same
executables and affinity. Order: candidate/baseline, baseline/candidate, then
candidate/baseline. Bracketed values span the three run medians, not individual
I/O latency bounds.

| Case | Baseline ms [range] | Candidate ms [range] | Time change |
|---|---:|---:|---:|
| `q8_b65536` | 25.8693 [25.6705–30.7414] | 29.1384 [26.1133–31.2608] | +12.64% |
| `threaded_control` | 21.5784 [20.8655–31.5221] | 29.8624 [20.6802–30.6287] | +38.39% |

The native aggregate increased 12.64%; the unchanged control increased 38.39%.
For the native case, the three paired changes were approximately +0.94%, +13.51%,
and +1.69%. These observations suggest substantial environment/order effects,
but do not identify the cause or exclude a code-level regression. Repeated
measurements on this busy host have not established a reliable regression bound.
See [followup-measurements.json](followup-measurements.json).

### Process resources

These initial-run figures cover complete benchmark processes, including untimed
setup, reset, verification, FD-count checks, and Criterion analysis. Adaptive
iteration counts differ; CPU totals must not be interpreted as equal-work copy
comparisons. Peak RSS includes payload buffers and the benchmark framework.

| Run | User CPU seconds | System CPU seconds | Peak RSS KiB |
|---|---:|---:|---:|
| baseline-1 | 2.42 | 16.14 | 72916 |
| candidate-1 | 2.56 | 17.33 | 73440 |
| candidate-2 | 2.07 | 14.10 | 74436 |
| baseline-2 | 2.04 | 14.02 | 73908 |
| baseline-3 | 2.16 | 14.67 | 73968 |
| candidate-3 | 2.20 | 13.90 | 73956 |

All accepted iterations passed output, byte-count, and FD-count checks. No FD
growth was observed across successful calls. Normal/error-completion buffer return
is asserted by tests; these process statistics alone do not prove absence of
memory leaks. The deliberately retained resources in terminal fault tests are
reported separately as a safety limitation, not hidden in these normal-run data.

### Performance disposition

Accept the required memory/descriptor safety repair, with **performance
qualification provisional**. Both initial and follow-up results remain recorded,
including native changes above the 5% investigation threshold. A dedicated/quieter
runner, stable workload conditions, and larger sustained workloads are required
under PERF.0 before claiming a general regression bound or tuning defaults. No
queue, buffer, block-size, or read-window defaults were changed.

## Reproduction and artifacts

Build `52ddb14` with the candidate's `Cargo.toml` and `benches/native_lifetime.rs`.
Build another checkout of `52ddb14` with `candidate.patch` applied. Use different
absolute target paths. No low-level API shim is needed: this harness calls the
high-level native-copy methods shared by both revisions.

```bash
cargo bench -p rvvdk-datamover --bench native_lifetime --no-run \
  --target-dir /absolute/path/to/variant-build --message-format=json

# Take the executable path from Cargo's compiler-artifact JSON.
CRITERION_HOME=/absolute/path/to/run-results \
RVVDK_BENCH_DIR=/absolute/path/to/storage \
/usr/bin/time -v -o /absolute/path/to/resources.txt \
taskset -c 2-6 /absolute/path/to/variant-executable --bench --noplot
```

Use a fresh Criterion directory per run and the ordering above. The recorded
`taskset` selection is specific to this host's topology. The original runtime is
measured only on successful copying paths; do not benchmark its borrowed-buffer
error paths. To reproduce the historical FD regression, add
`fd-regression-before.rs` to its datamover integration tests and run that test in
isolation against `52ddb14`; an assertion failure is the expected baseline result.

[measurements.json](measurements.json) stores exact commands, Criterion estimates,
and raw iteration/time samples. Console and process resource logs accompany each
run. The patch and fingerprints identify the candidate measured before commit;
the commit containing this report records implementation and evidence together.
Two integration tests and a stronger live-ID wraparound case were added after
benchmarking; the candidate benchmark was rebuilt and its executable hash matched
the measured binary exactly.
