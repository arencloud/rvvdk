# R1.3 — Shared local RAW endpoint inspections

Baseline: `28221ec` (R1.2). Final candidate: baseline plus
[candidate.patch](candidate.patch). [Environment/source/binary fingerprints](environment.json)
identify the measured source. No executor defaults, buffering, or queue settings changed.

## Correctness and API compatibility

**277 workspace tests pass** (four new). Formatting, strict all-target Clippy,
and core/datamover library compilation for wasm32-unknown-unknown pass. The latter
is a portability compile check, not runtime qualification.
[Validation commands](validation.json), [test output](tests.txt), and
[test inventory](test-inventory.txt) preserve the result.

New cases cover descriptor binding even for another handle to the same inode,
logical restrictions on writable descriptors, current size, repeated custom
endpoint checks, unknown native identity, and preserved FD trait object use.
The append-flag test now explicitly inspects before and after a flag change.
The custom-backend contract test also passes on the baseline runtime
([record](baseline-contract.txt)); no permission behavior is intentionally relaxed.
Existing source/destination resize, hard-link alias, mismatched backing FD, and
rejection-before-observation tests remain in the workspace suite.

Final API review found the initial optional hook made LinuxFdBackend
non-dyn-compatible. Adding a Sized bound to that hook preserves the existing
trait object API and FD-only implementors. All checks and measurements were
repeated for that final source. The initial patch, fingerprints, full raw
measurements, summary, syscall traces, and console/resource logs remain under
the `initial-` prefix; their timings are not attributed to the final source.

## Method

Each variant uses its own absolute Cargo target directory and identical
benchmark harnesses. Five cases run as adjacent candidate/baseline,
baseline/candidate, candidate/baseline pairs. Each process uses CPUs 2–6,
30 flat samples, 300 ms warmup, and a 2 s measurement target. Source creation
uses deterministic incompressible bytes. Storage is the recorded Btrfs/NVMe
mount, not /tmp. Caches are warm; no global cache drop or host tuning.

- RAW planning: buffered 1 MiB source/destination, 64 KiB blocks, Threaded or
  IoUring/QD8 selection. Includes endpoint checks, extent query, and plan creation.
  No payload writes; the harness checks destination contents remain unchanged.
- Portable planning control: existing 16 MiB dense progress harness.
- Complete RAW copies: buffered 1 MiB, same strategy/block/QD settings. Timing
  includes public copy_raw_with_report planning, execution validation, allocation,
  payload I/O, and final flush. Destination reset/flush and full byte readback
  are untimed; byte counters and output equality are checked every iteration.
  Threaded uses the default one worker. The io_uring read window remains its
  existing default; no parameter search was performed.

Results report the median of three run medians; positive changes mean slower.
[Raw estimates/samples/outliers](measurements.json), [summary](summary.json), and
per-process console/resource logs are retained. The /usr/bin/time records cover
the entire process, including untimed reset/readback and adaptive iteration
counts; they are not per-copy CPU or memory cost measurements.

## Final results

| Workload | Baseline | Candidate | Change | Paired changes |
|---|---:|---:|---:|---|
| `preflight/plan_threaded` | 2.580 µs | 2.215 µs | -14.16% | -27.97%, -14.16%, -4.72% |
| `preflight/plan_native` | 2.520 µs | 1.806 µs | -28.34% | -27.28%, -34.76%, -28.34% |
| `progress_plan/dense/plan` | 2.014 µs | 1.977 µs | -1.82% | -0.03%, -5.84%, +0.77% |
| `preflight_copy/threaded` | 6.818 ms | 6.815 ms | -0.05% | +1.08%, +1.59%, -0.71% |
| `preflight_copy/native` | 6.964 ms | 6.981 ms | +0.25% | -4.79%, +9.92%, -0.21% |

Local RAW inspection syscall counts are deterministic, independent of latency:

| 100 plans, either strategy | Baseline | Candidate |
|---|---:|---:|
| fstat | 400 | 200 |
| fcntl(F_GETFL) | 400 | 200 |
| Total per plan | 8 | 4 |

The probe emits markers around exactly 100 calls for each strategy. Startup
metadata and file opening lie outside those markers.
[Counts](syscall-summary.json), [baseline trace](baseline-syscalls.txt),
[candidate trace](candidate-syscalls.txt), and [probe source](preflight_probe.rs)
are preserved. Strace timings are not used for latency claims.

## Performance disposition and remaining work

Accept the bounded optimization: final RAW planning medians improve **14.16%**
(Threaded) and **28.34%** (IoUring), and descriptor inspection calls halve.
All final matched RAW planning pairs improve, although threaded improvement
varies from 4.72% to 27.97%. Portable planning and complete-copy aggregate medians
remain within 2% of baseline. This does not establish a universal copy speedup.

One native complete-copy pair is **+9.92%** slower, while the other two are
−4.79% and −0.21%; its aggregate is +0.25%. Preserve this variability for
controlled-runner qualification instead of calling this a clean performance
pass. The host governor is powersave and background load is uncontrolled.
PERF.0 and the prior R1.2 fragmented observer/Hole/Zero follow-ups remain open.

The matched baseline retains R0.5's duplicate RAW descriptor inspections.
The historical [R0.5 final run](../2026-09-28-r05/README.md) recorded roughly
3.197/3.342 µs for Threaded/native planning; this final candidate measures
2.215/1.806 µs. Those older timings are context only: differing run conditions
preclude treating them as a matched speedup. The current paired comparison
isolates the effect against the immediately preceding implementation.

Fresh inspections are taken again for execution. They are not atomic snapshots
or locks, and are not persisted in CopyPlan. The default custom backend hook
may still inspect independently. Native lower layers retain their own preflight;
prepared native rings and complete native preparation remain R2 work.
See [ADR-0028](../../adr/0028-endpoint-inspection.md).

## Reproduction

Use a detached baseline worktree at 28221ec. Copy the current
crates/rvvdk-datamover/benches/preflight.rs into it so both variants include the
new complete-copy controls; leave baseline runtime code untouched. Apply
candidate.patch only when reconstructing the candidate from baseline.

Build both with separate absolute target directories:

```sh
cargo build --release -p rvvdk-datamover --benches --target-dir /absolute/variant-target --message-format=json
```

Select the preflight/progress executables from Cargo's artifact output.
Run each exact case from the table with a fresh CRITERION_HOME and the same
RVVDK_BENCH_DIR on the chosen storage mount:

```sh
taskset -c 2-6 /absolute/benchmark-executable --bench --noplot --sample-size 30 --warm-up-time 0.3 --measurement-time 2 'preflight/plan_threaded$'
```

The exact per-run commands are also stored in measurements.json. Alternate
adjacent variant order as described above and preserve all results.

For syscall verification, temporarily copy preflight_probe.rs to the datamover
examples directory, build it with --release --example preflight_probe, and supply
existing distinct regular 1 MiB source/destination paths:

```sh
strace -yy -e trace=fstat,newfstatat,statx,fcntl,write -o syscalls.txt /absolute/preflight_probe source.raw destination.raw
```

Count metadata/access calls only between BEGIN/END markers. Remove the temporary
example after building; do not commit generated binaries or disk images.
