# R2.2 — Local zeroing and hole punching

Baseline: `a4a04fb` (R2.1), plus [matched benchmark harnesses](matched-harness.patch).
Candidate: that commit plus [candidate.patch](candidate.patch).
[Environment/source/binary hashes](environment.json) identify the measured builds.
No worker, queue, block-size, memory-budget, native read-window, or host-governor
defaults were tuned.

## Validation and actual storage

**314 distinct tests covered**: the workspace run passes 313 and explicitly
ignores one allocation test requiring a supporting storage filesystem. That test
passes separately, together with both other local sparse-output integration
tests, on the recorded Btrfs storage mount. Formatting, strict all-target Clippy,
and core/datamover wasm32 library compilation pass. The portability result is
compilation, not runtime qualification.

[Commands](validation.json), [test output](tests.txt), [inventory](test-inventory.txt),
and [storage test output](storage-tests.txt) retain the checks. The storage command
used RVVDK_TEST_DIR set to the storage path in environment.json and:

```text
cargo test -p rvvdk-local --test sparse_write -- --include-ignored --nocapture
```

The allocation test writes an 8 MiB incompressible file and punches the middle
6 MiB. Allocated 512-byte blocks fell **16,384 → 4,096**. Complete readback verified
zeroes in the punched range, unchanged surrounding bytes, and unchanged size.
This measures one supported filesystem/run; fallback does not promise reclamation.

New tests cover unsupported-mode injection, independent per-file caches, EINTR,
real errors with partial effects and no fallback, empty/overflow/truncated ranges,
primary and alias append flags, read-only rejection, odd boundaries and byte
readback through buffered/direct opens, and mixed native/threaded copies with
aligned Data ranges. Fault injection establishes fallback behavior even where
the development filesystem supports the operations.

The initial [first](first-workspace.txt) and [second](second-workspace.txt)
workspace failures are retained. Four native-extent and two unified-dispatch
assertions still expected local Zero/Hole output to count as ordinary writes.
Those expected counters were migrated; final tests pass. No readback failure
was hidden or benchmark result discarded.

[The local contract](../../local-sparse-output.md) documents logical capabilities,
bounded internal fallback, errno handling, fresh inspection, counters, and limits.
No transactional rollback, process RSS cap, external mutation lock, or general
mixed buffered/direct qualification is added.

## Harness corrections and smoke checks

Both builds use identical harnesses. Progress counters now account separately
for actual advertised discard; the new local benchmark checks exact Data/Zero/
Hole counters. The native zero-destination benchmark uses capability-aware
counters and a clearer name; it still targets memory without WRITE_ZERO and is
an **unchanged fallback control**, not the new local range-zeroing path.
The local mixed benchmark measures actual local Zero handling.

The known scheduler_failure fixture now advertises FLUSH and inspects the
underlying Io cause inside CopyExecution. All eight failure profiles smoke-run
successfully on both builds. This repairs the fixture but does not renew earlier
failure-latency qualification. [Smoke commands](smoke.json) also cover the new
local benchmark, fragmented observed/unobserved/counter paths, and native zero
control. Their output is saved as `*-smoke.txt`.

## Method

Isolated absolute Cargo targets build optimized baseline/candidate binaries.
Eleven profiles run as adjacent candidate/baseline, baseline/candidate, and
candidate/baseline pairs on CPUs 2–6. Each process requests 30 flat samples,
300 ms warmup, and a 2 s measurement target in its own Criterion directory.

File workloads use warm deterministic incompressible Data and prefill on the
recorded Btrfs/NVMe mount. No global cache drops. Powersave and uncontrolled host
load limit performance qualification. Btrfs compression is recorded; zero writes,
unwritten ranges, and punched holes can have different allocation costs.

- Native planning and complete RAW copy controls: 1 MiB, 64 KiB blocks,
  one-worker Threaded or native QD8/read-window4.
- Fragmented planning and observed copying: 16 MiB, alternating 64 KiB Data/Hole
  extents. Dense unobserved copying uses the same size/block with all Data.
- New local mixed copy: 32 MiB, four extents, Data 4 MiB / Zero 4 MiB /
  Hole 20 MiB / Data 4 MiB. 64 KiB blocks, queue capacity4, one/four-worker
  Threaded or native QD8/read-window4, prebuilt RAW plan.
- Native zero fallback control: 1 MiB memory destination, sixteen Zero extents,
  no advertised WRITE_ZERO, explicit final flush.
- Four-worker dynamic memory control: 16 MiB Data, 64 KiB blocks, prebuilt plan.

Copies include their final flush. Reset/flush and full readback are untimed;
byte equality and operation counters are asserted each iteration. New local
workload logs include allocated bytes; only the line for the selected Criterion
profile describes the executed copy. Lines for filtered profiles are ignored.
These are observations, not universal reclamation assertions. The dedicated storage test establishes the
bounded physical-allocation result above.

Sparse candidate output performs different physical operations than baseline
zero writes while preserving identical logical bytes. These are end-to-end
feature comparisons, not pure syscall/observer overhead comparisons. Concurrent
execution still splits sparse extents into block-sized work items, whereas
sequential/native execution calls the sparse backend once per whole extent.

GNU time CPU/RSS logs include setup/readback and varying adaptive iteration
counts; they are not per-copy resource or strict memory-cap measurements.

## Main comparison

Aggregates are medians of three run medians. Positive changes mean slower.

| Workload | Baseline | Candidate | Change | Paired changes |
|---|---:|---:|---:|---|
| `preflight/plan_native` | 1.805 µs | 1.816 µs | +0.64% | -0.52%, +1.90%, +0.17% |
| `preflight_copy/threaded` | 7.031 ms | 6.998 ms | -0.47% | +0.45%, -0.47%, +0.87% |
| `preflight_copy/native` | 7.198 ms | 7.097 ms | -1.40% | +0.18%, -1.40%, -0.45% |
| `progress_plan/fragmented50/plan` | 203.047 µs | 205.514 µs | +1.21% | -1.72%, +0.69%, +1.21% |
| `progress_copy/dense/unobserved` | 13.814 ms | 13.572 ms | -1.75% | -1.39%, -1.75%, -28.60% |
| `progress_copy/fragmented50/noop` | 12.789 ms | 16.200 ms | +26.67% | +31.03%, +13.90%, +10.99% |
| `local_sparse/threaded1` | 17.821 ms | 11.643 ms | -34.67% | -38.62%, -33.50%, -42.96% |
| `local_sparse/threaded4` | 26.695 ms | 19.346 ms | -27.53% | -4.10%, -29.08%, -27.53% |
| `local_sparse/native` | 28.901 ms | 15.956 ms | -44.79% | -35.31%, -45.79%, -44.79% |
| `native_validation/zero_destination_1mib_16extents` | 42.931 µs | 42.616 µs | -0.73% | +0.00%, -9.45%, +9.53% |
| `portable_memory/workers4/dyn` | 1.265 ms | 1.256 ms | -0.72% | -0.23%, -1.91%, -0.47% |

## Performance disposition

The 32 MiB mixed local workload improves by **34.67% sequentially, 27.53% with
four workers, and 44.79% natively** by median-of-medians. These are results for
the recorded filesystem and workload, not universal throughput claims.
Dense RAW controls change −0.47% Threaded / −1.40% native; native planning
+0.64%, fragmented planning +1.21%, and four-worker memory −0.72%.

The **fragmented observer workload regresses +26.67%** in the main aggregate,
with all three pairs slower (+31.03%, +13.90%, +10.99%). A longer focused repeat
(500 ms warmup, 4 s target, reversed starting order) confirms **+15.39% aggregate**
and +15.07%, +13.52%, +16.10% pairs. [Repeat samples](followup-measurements.json)
and [summary](followup-summary.json) retain this evidence. No code changed
between measurements.

Treat this as an explicit feature cost on this host, not merely noise. This
profile changes 128 small Hole ranges from zero writes to checked hole punches;
large extents can amortize sparse-operation costs, while fragmented ranges incur
repeated descriptor inspections and filesystem metadata operations. That is an
implementation-based explanation, not a syscall-level attribution study.
Future tuning should investigate batching/prepared range operations and backend
costs while preserving exact boundaries, fresh validation, and logical guarantees.
Do not silently disable sparse preservation to claim throughput parity.

Accept the functionality with this recorded tradeoff; **performance qualification
remains open**. Keep this new consistent regression distinct from earlier
observer variability. The unchanged memory native-zero control also has opposing
−9.45%/+9.53% pairs despite aggregate −0.73%; dense copying has a −28.60% pair.
Retain these variations and prior PERF.0 issues rather than claiming a generally
clean or universally faster implementation. No extra same-binary control was
needed to dismiss a regression: the repeated fragmented cost is explicitly kept.

## Artifacts

- [Raw Criterion samples/estimates](measurements.json), [summary](summary.json).
- `pair*-*.txt`: benchmark output/outliers and GNU time process resource logs.
- [Baseline build](baseline-build.txt), [candidate build](candidate-build.txt).
- [Validation](validation.json), [smoke checks](smoke.json), storage/initial failure
  logs above, [environment/hashes](environment.json), source/harness patches.

Only the final validated candidate was timed. Source, binaries, and identical
harnesses were checked against the saved fingerprints. The baseline patch is
limited to benchmark registration/harness changes; its production code is intact.
