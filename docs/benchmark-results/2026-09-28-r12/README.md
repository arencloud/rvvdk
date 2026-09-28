# R1.2 — Shared semantic policy and sequential execution

## Decision and remaining qualification

Accept the shared execution implementation and its two measured refinements.
Correctness contracts are preserved, and no defaults or resource budgets changed.
**General performance qualification remains open.** This development host does not
provide a controlled storage/cache/scheduling baseline, and the final matrix must
not be described as a clean performance pass.

Final case-by-case median changes for dense copies are −2.11% to +0.83%; dynamic
memory is −0.40%/+0.26% for one/four workers; accelerated mixed is −2.87%/+1.30%.
Three profiles exceed the 5% investigation threshold by median: fragmented no-op
observer +7.66%, one-worker Hole fallback +8.29%, and one-worker Zero fallback
+14.12%. Individual sparse pairs conflict (Hole −2.98%/+19.55%; Zero
+30.64%/−2.41%). The longer final-source Zero repeats were −2.81%/+0.09%/−17.31%.
The fragmented no-op pair was +4.50%/+10.82%; neighboring unobserved/counter cases
were closer to baseline, but that does not disprove callback-path overhead.

Retain these as explicit PERF.0 follow-ups on a controlled runner before making
throughput claims or selecting defaults. Do not average them away, claim a
universal improvement, or use earlier source-state timings as final-code results.
The implementation log and roadmap retain this qualification work.

## Source identity and correctness

Baseline is `b7162fc` (R1.1). Initial candidate is that commit plus
[candidate.patch](candidate.patch). The dispatch-refined candidate adds
[tuned-candidate.patch](tuned-candidate.patch) to the baseline; the final candidate
adds [final-candidate.patch](final-candidate.patch) to the baseline. Each is a complete alternative
patch, including new execution modules, contract tests, and the sparse-policy
benchmark; do not apply them on top of one another. [environment.json](environment.json)
records full source, harness, executable, and host fingerprints. Baseline runtime
is unchanged; the new benchmark/test and benchmark registration were copied to its
isolated worktree. All measured harnesses are identical across variants. Separate
absolute Cargo target directories avoid reusing same-package build artifacts.

273 workspace tests pass. Five new contract tests also pass against baseline
([output](baseline-contract-tests.txt)), demonstrating preserved behavior. They
cover operation traces, odd tails, dirty zero buffers, sparse capability precedence,
selected-operation failure without retries, progress thresholds, and failed flush.
Existing translated-disk/native parity tests retain full content checks.
Formatting, strict all-target Clippy, and core/datamover library compilation for
`wasm32-unknown-unknown` pass. The latter is not runtime/thread qualification.

All benchmark copies check full destination contents after every iteration;
statistics are also checked. Zero/Hole fixtures must overwrite nonzero-prefilled
destinations. No public API, buffer budget, worker default, operation granularity,
flush owner, or byte accounting was changed to improve timings.

## Workloads and method

Intel i7-11850H, approximately 64 GB RAM, Linux 7.2.5, Rust 1.95.0. Storage fixtures
use `target/r12-benchmark-storage` on Btrfs with zstd compression over LUKS/dm-crypt
on Samsung NVMe. They do not use `/tmp` tmpfs. No global cache drop or filesystem
attribute change was made. Workloads are small and warm; this is not sustained
storage qualification. Powersave and uncontrolled background load remain limits.

| Harness/cases | Size and data | Execution |
|---|---|---|
| `progress` | 16 MiB dense or alternating 64 KiB Data/Hole (50% holes, 256 extents) | Buffered local files, 64 KiB blocks, one worker/buffer; unobserved, no-op observer, and counting observer |
| `portable_plan` | 16 MiB dense memory, deterministic bytes | Dynamic dispatch, one/four workers and buffers; four-worker static dispatch added as a follow-up comparator |
| `semantic_policy` | 1 MiB, sixteen 64 KiB extents; Zero-only fallback, Hole-only fallback, or repeating Data/Zero/Hole/Data with WRITE_ZERO and DISCARD | One worker with no-op observer, or four workers unobserved; memory backend |
| `native_validation` | 1 MiB in sixteen Zero extents | Destination-aware native entry point, memory destination without sparse capabilities; ordinary zero-write fallback |

All use 64 KiB transfer blocks and 4096-byte alignment. Portable queue capacity is
4; buffer counts match workers. Native options use QD8/read window4, but the
Zero-only case performs synchronous fallback writes and submits no native Data I/O.
It measures that shared-policy branch, not io_uring device throughput.

The accelerated mixed profile reads/writes 512 KiB, zeroes 256 KiB, and discards
256 KiB through the memory backend. Fallback profiles read no payload and write
1 MiB. Fragmented file copies read 8 MiB and write 16 MiB because local holes
still use fallback writes. Logical throughput must not be confused with physical
payload or allocation savings. No new discard zero-read guarantee is claimed.

Setup, reset, and complete readback are outside the timer. Plan execution,
validation, allocations, worker creation, payload operations, and flush are inside.
Planning itself is unchanged and was not remeasured. Pin all processes to CPUs
2–6 and run sequentially, with no builds/tests overlapping timing. Initial order
is baseline/candidate, candidate/baseline, baseline/candidate; 20 flat samples,
300 ms warmup, and 2 s measurement targets. Every process has a fresh Criterion
output directory. Raw samples/estimates and console/resource logs are retained.

## Initial results — before dispatch refinement

Times are medians of three run means. Change is the median of paired
candidate/baseline ratios, which need not equal the ratio of displayed medians.
Positive means slower. [summary.json](summary.json) retains each run and pair;
[summarize.py](summarize.py) recomputes it from saved [samples](samples/).

| Case | Baseline | Candidate | Median paired change | Paired range |
|---|---:|---:|---:|---:|
| `native_validation/zero_fallback_1mib_16extents` | 43.842 µs | 41.448 µs | +1.61% | -21.25% to +16.34% |
| `portable_memory_workers1/dyn` | 1.616 ms | 1.792 ms | +10.73% | -2.33% to +16.09% |
| `portable_memory_workers4/dyn` | 1.018 ms | 1.159 ms | +9.69% | +7.57% to +13.89% |
| `progress_copy_dense/counter` | 13.058 ms | 12.747 ms | +1.29% | -3.63% to +5.64% |
| `progress_copy_dense/noop` | 13.208 ms | 13.118 ms | -2.40% | -3.39% to -0.68% |
| `progress_copy_dense/unobserved` | 12.906 ms | 13.019 ms | +2.82% | +0.65% to +6.16% |
| `progress_copy_fragmented50/counter` | 11.797 ms | 12.006 ms | +1.77% | +1.31% to +10.88% |
| `progress_copy_fragmented50/noop` | 12.425 ms | 11.937 ms | +0.48% | -6.73% to +10.16% |
| `progress_copy_fragmented50/unobserved` | 11.721 ms | 11.750 ms | +0.25% | -1.02% to +5.82% |
| `semantic_policy_hole_fallback/workers1` | 43.942 µs | 44.776 µs | +6.33% | -9.83% to +15.94% |
| `semantic_policy_hole_fallback/workers4` | 121.376 µs | 136.815 µs | +21.52% | -9.63% to +29.63% |
| `semantic_policy_mixed_accelerated/workers1` | 42.874 µs | 43.166 µs | +0.68% | -3.81% to +8.60% |
| `semantic_policy_mixed_accelerated/workers4` | 109.558 µs | 115.181 µs | +5.13% | +3.65% to +7.55% |
| `semantic_policy_zero_fallback/workers1` | 46.147 µs | 43.351 µs | -6.06% | -11.68% to +16.34% |
| `semantic_policy_zero_fallback/workers4` | 121.174 µs | 133.472 µs | +10.15% | -3.57% to +43.52% |

## Longer case-by-case repeats

Initial medians exceeded 5% for dynamic memory, Hole fallback with one/four
workers, and four-worker Zero fallback/accelerated mixed operations. Repeat those
cases and add four-worker static memory as a comparator. Use 30 flat samples,
500 ms warmup, and 4 s targets. Alternate variants **for each case**, shortening
the gap between comparable measurements: candidate/baseline, baseline/candidate,
candidate/baseline. The executable and source hashes are unchanged.

| Case | Baseline median | Candidate median | Median paired change | Individual paired changes |
|---|---:|---:|---:|---|
| `portable_memory_workers1/dyn` | 1479.68 µs | 1488.52 µs | +0.95% | +1.71%, -2.30%, +0.95% |
| `portable_memory_workers4/dyn` | 1089.63 µs | 1077.32 µs | -0.12% | -0.12%, -5.22%, +1.01% |
| `portable_memory_workers4/static` | 1083.39 µs | 1168.77 µs | +2.48% | -0.86%, +11.41%, +2.48% |
| `semantic_policy_hole_fallback/workers1` | 42.56 µs | 42.70 µs | +2.36% | +2.52%, -2.64%, +2.36% |
| `semantic_policy_hole_fallback/workers4` | 124.47 µs | 120.67 µs | -2.79% | -6.44%, -2.79%, -0.19% |
| `semantic_policy_mixed_accelerated/workers4` | 115.54 µs | 124.26 µs | +10.23% | -4.71%, +14.54%, +10.23% |
| `semantic_policy_zero_fallback/workers4` | 175.59 µs | 192.07 µs | +1.96% | +9.39%, +1.96%, -1.81% |

## Same-binary control and dispatch refinement

The initial accelerated mixed four-worker case remained +10.23% slower in the
longer repeats. Three back-to-back pairs of the **same baseline executable** gave
−0.16%, −3.45%, and −0.75%. This did not justify dismissing the residual slowdown
as noise. The original source and every measurement remain saved.

Worker code generation was inspected. The central selector was then changed from
a combined sparse branch with a Hole/discard condition to an explicit extent-kind
match and a shared zero/fallback helper. This preserves policy and capability query
counts while allowing Zero to take its own branch. It is a measured dispatch
refinement, not proof of a hardware-level root cause or a universal speed claim.

All tests/checks were repeated. Three targeted pairs with the dispatch-refined executable
(30 samples, 0.5 s warmup, 4 s targets, C/B then B/C then C/B) gave
−0.62%, −2.85%, and −15.10% for accelerated mixed four-worker copies. Dispatch-refined candidate
and baseline medians were 104.27 µs and 107.33 µs; the median paired change was
−2.85%. Dispatch-refined executable hashes are under `tuned` in the environment record.
Earlier candidate hashes identify superseded binaries; their build paths were
reused during the rebuild and should not be treated as immutable artifact paths.

## Configured block-size refinement

The full matrix after dispatch refinement (raw `final*` runs) still showed
+12.42% median for one-worker dynamic memory and +9.78% for one-worker Zero fallback.
Those numbers describe the intermediate source state, not the committed code.

The shared transfer helper had used the allocated buffer's runtime length as its
loop bound, where the original code used the configured transfer block size. Both
produce the same ranges, but the latter preserves the original optimization input.
The final source restores that explicit bound. No buffer size, copy range, or
observer boundary changed. All tests/checks were repeated again.

Three targeted pairs (`block*`, 30 samples, 0.5 s warmup, 4 s targets, C/B then
B/C then C/B) gave +1.92%/+6.22%/−2.79% for one-worker dynamic memory and
−2.81%/+0.09%/−17.31% for observed Zero fallback. Median paired changes were
+1.92% and −2.81%. Code generation and host variation can both affect these numbers;
this is not a universal throughput claim. Final fingerprints are under `block`. The `source_states` mapping identifies each phase.

The complete final matrix is repeated case by case (`review*`), with each pair's
variants adjacent rather than running a whole variant's suite before its counterpart.
This reduces elapsed time between comparable runs. Two pairs use C/B then B/C,
20 samples, 0.3 s warmup, and 2 s targets. The remaining resource controls and
copy-verification boundaries are unchanged.

## Final case-by-case matrix — committed source

The `review*` rows below use `final-candidate.patch`. Times are medians of two
run means; paired changes use the corresponding ratios. Full per-run values,
including outliers, remain in the summary and raw samples.

| Case | Baseline | Candidate | Median paired change | Individual pairs |
|---|---:|---:|---:|---|
| `native_validation/zero_fallback_1mib_16extents` | 50.962 µs | 45.477 µs | -9.84% | -18.18%, -1.50% |
| `portable_memory_workers1/dyn` | 2.000 ms | 1.993 ms | -0.40% | -1.16%, +0.36% |
| `portable_memory_workers4/dyn` | 1.288 ms | 1.291 ms | +0.26% | +2.59%, -2.06% |
| `progress_copy_dense/counter` | 13.638 ms | 13.352 ms | -2.11% | -2.75%, -1.47% |
| `progress_copy_dense/noop` | 13.638 ms | 13.411 ms | -1.65% | -0.76%, -2.55% |
| `progress_copy_dense/unobserved` | 13.253 ms | 13.359 ms | +0.83% | +2.03%, -0.38% |
| `progress_copy_fragmented50/counter` | 14.276 ms | 14.599 ms | +2.03% | +0.38%, +3.69% |
| `progress_copy_fragmented50/noop` | 14.374 ms | 15.539 ms | +7.66% | +4.50%, +10.82% |
| `progress_copy_fragmented50/unobserved` | 12.648 ms | 12.770 ms | +0.95% | +0.21%, +1.68% |
| `semantic_policy_hole_fallback/workers1` | 46.893 µs | 50.634 µs | +8.29% | -2.98%, +19.55% |
| `semantic_policy_hole_fallback/workers4` | 137.185 µs | 139.017 µs | +1.34% | +1.09%, +1.58% |
| `semantic_policy_mixed_accelerated/workers1` | 49.242 µs | 47.826 µs | -2.87% | -2.54%, -3.20% |
| `semantic_policy_mixed_accelerated/workers4` | 128.260 µs | 129.929 µs | +1.30% | +2.63%, -0.03% |
| `semantic_policy_zero_fallback/workers1` | 49.141 µs | 56.188 µs | +14.12% | +30.64%, -2.41% |
| `semantic_policy_zero_fallback/workers4` | 211.400 µs | 206.766 µs | -2.22% | +0.46%, -4.90% |

## Resource interpretation and reproduction

Whole-process resource logs include setup, reset, full verification, and adaptive
iteration counts. CPU time and maximum RSS are diagnostic process measurements,
not per-copy costs or total execution memory budgets. The benchmark keeps several
full-size source/destination/verification buffers in addition to executor buffers.
No syscall/latency histogram, sustained throughput, allocation-saving, or
translated-format decoding claim is made here.

To reconstruct, check out baseline `b7162fc` in an isolated worktree; copy only
`benches/semantic_policy.rs`, `tests/semantic_policy.rs`, and the candidate datamover
`Cargo.toml` into it. Keep runtime sources unchanged. Apply `final-candidate.patch` to
a separate clean baseline for final candidate (or either earlier patch for its
corresponding experiment). Build each in a separate absolute target:

```bash
cargo bench -p rvvdk-datamover --bench progress --bench portable_plan \
  --bench semantic_policy --bench native_validation --no-run \
  --message-format=json --target-dir /absolute/variant-build
```

Use the executable paths returned by Cargo and a fresh `CRITERION_HOME` each time.
Set `RVVDK_BENCH_DIR` to a real storage directory, then run:

```bash
/usr/bin/time -v taskset -c 2-6 /absolute/progress-executable 'progress_copy/' \
  --bench --noplot --sample-size 20 --warm-up-time 0.3 --measurement-time 2
```

Other initial filters are `/dyn$` for `portable_plan`, `semantic_policy/` for the
new harness, and `zero_fallback_1mib_16extents$` for `native_validation`. Full
follow-up filters/order are in `environment.json`; use 30 samples, 0.5 s warmup,
and 4 s targets. The final `review` phase instead uses 20 samples, 0.3 s warmup,
2 s targets, and pairs each case separately. Retain console/resource logs and each Criterion `new/sample.json`,
`new/estimates.json`, and `new/benchmark.json`; recompute with `python3 summarize.py`.
All first-run outliers remain in the record; no samples were dropped manually.
