# Pipeline performance qualification (R6.1c.5p)

This package separates repeated live observations from test-only local cost profiles.
The live series uses the frozen R6.1c.5c Rust executable. Local profiles use a separately
identified release test executable. Neither is an implementation speedup comparison.
No production verification, cancellation, identity or durability gate is removed.

## Live series

Use the authorized powered-off source and LAN runner only after fresh pinned TLS,
explicit identity, version, power, snapshot/task and storage checks. Retain at least
three complete observations. Each includes owned export, conversion, fresh admission
and durable publication, followed outside timing by full QEMU logical comparison and
the independent guest fixture. Keep one source/output pair at a time and use explicit
checked cleanup before the next pair. Keep both terminal journals privately.

Compare the published RAW digest privately between observations to detect source
content drift. Commit only a boolean confirmation, never the image digest, selection,
endpoint or credentials. Record the executable hash per completed observation.
`campaign.json` binds the plot inputs to that hash, the minimum observation count and
conditions. The older October 2 observation stays separate from this newly dated
series, even if the source contents agree.

The whole-operation, phase, CPU/RSS, allocation and cleanup definitions remain those
of the [pipeline contract](composed-export-pipeline.md). After three initial runs,
any wall/phase/CPU/cleanup spread above 5% triggers three further complete runs. Keep
both groups and unresolved spread; repeats are not confidence intervals. No more
than one timed workload runs on the guest. Workstation development/validation is on
a separate host and does not run in the guest's CPU/storage environment.

## Test-only local regions

The `profiled!` expression wrapper contains a `cfg(test)` guard. Normal library and
example builds contain no profiling state, CPU-clock calls or telemetry API from
this module. Expression results, error propagation, callback ordering and sync
operations are unchanged. There is no runtime flag to disable a production check.

An explicit thread-local profiling session enables disjoint regions. A scoped guard
records monotonic wall time, `CLOCK_THREAD_CPUTIME_ID`, byte counts and call counts.
Sessions/guards cannot move between threads. Nested regions are rejected to prevent
double counting; unwinding resets the session. Disabled controls use the same release
test executable with counters and clocks inactive. The benchmark requires serial
copy options because per-thread region CPU cannot attribute worker-thread CPU.
Disabled controls still execute the test wrapper's inactive thread-local check.
The paired ratio measures recording overhead within the test executable, not total
test-executable overhead relative to the production executable. Region timings also
include part of the clock/probe overhead; they are diagnostic estimates.

| Region | Timed operation |
|---|---|
| Container read / hash | Encoded file reads and their SHA-256 updates, separately |
| Native map | Native metadata/map admission |
| Present grain read | Native reads/decompression of present grains during admission |
| Copy engine including flush | DataMover execution through its flush boundary |
| Logical source read | Native logical read, including grain decoding and zero synthesis |
| RAW read | Buffered RAW file readback |
| Logical comparison / hash | Byte equality and RAW SHA-256 updates, separately |
| Journal commit including sync | Complete output-journal commit, including its syncs |
| Payload sync | Explicit non-journal member/directory syncs covered by the probes |

Copy and journal regions include their internal flush/sync costs; they are not pure
syscall timings. A publication member-sync region also includes its immediate
checked file open. Metadata work, uninstrumented checks, digest finalization and probe
bookkeeping remain in the residual. CPU outside the measured thread is not attributed
to a region. Whole-operation CPU uses `RUSAGE_SELF`; RSS is the process lifetime peak
at the sampling point and can include fixture setup and earlier oracle work. Region byte counts describe traversals, not unique
bytes stored or physical disk IO.

A successful local conversion plus publication measures three complete logical
source/RAW comparisons and RAW hash passes, seven complete encoded read/hash passes,
three native admissions/present-grain traversals and nine output journal commits.
The benchmark asserts those counts and unchanged source bytes/journal. Explicit
output cleanup is timed separately and retains its Cleaned record. Synthetic fixture
teardown after validation is outside timing; this is distinct from live source cleanup.

## Controls and reproduction

The three authored workloads contain 8 MiB of data in 64 or 256 MiB logical capacity:
`stored_64`, `deflated_64`, `deflated_256`. The stored stream is independently authored;
the compressed pattern comes from the existing system-zlib fixture and is checked
against its authored byte pattern, not a second native decode. Every operation is
followed by an independent full RAW byte comparison and source/journal checks.

The 64 MiB pair exposes encoding/encoded-size sensitivity; the 256 MiB case increases
logical verification work while keeping data size fixed. These warm, small working
sets do not reproduce the larger-than-RAM private live disk or its compression mix.
Do not extrapolate their timing percentages as an exact live profile.

Build `cargo test --release -p rvvdk-vsphere --lib --no-run` and freeze the reported
test executable before measuring. Run the offline harness separately per filesystem:

```sh
python3 scripts/benchmarks/measure_pipeline_profile.py \
  --binary PRIVATE_RELEASE_TEST_BINARY --report PRIVATE_PROFILE_REPORT \
  --parent PRIVATE_FIXTURE_PARENT --filesystem xfs --cpu 0
```

Three rounds each contain four alternating disabled/instrumented pairs. Instrumented
wall/CPU increases over 5%, or initial run-median wall/phase/CPU/cleanup spreads over
5%, require three rounds of eight pairs. Preserve all samples. Fixture creation,
independent comparisons and output cleanup are outside whole-operation timing;
cleanup has separate wall/CPU fields. Record CPU affinity, filesystem, CPU features,
binary/harness identity and lack of frequency/SMT/storage isolation.

Place sanitized groups in `REPORT/profile-xfs` and `REPORT/profile-tmpfs`, then render:

```sh
target/benchmark-plots/bin/python scripts/benchmarks/plot_pipeline_profile.py REPORT
target/benchmark-plots/bin/python scripts/vsphere/plot_pipeline.py REPORT
```

The profile plotter validates sample counts, repeat decisions, byte accounting and
controls. Cost panels show mean disjoint regions plus residual; overhead panels show
individual pairs and ratios of median run medians. Initial and repeated groups stay
separate. Filesystem sessions are separate observations, not a paired proof of a
filesystem's causal effect. Keep raw evidence and standalone SVG/PNG artifacts.
Tmpfs is an in-memory timing control, not a persistent-storage qualification.

[ADR-0067](adr/0067-disjoint-test-only-pipeline-profiling.md),
[measurements](benchmark-results/2026-10-07-r61c5p/README.md).
