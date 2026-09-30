# Sparse bounds and scaling qualification (R5.5)

R5.5 qualifies the existing base sparse reader; runtime behavior and admission
limits are unchanged. [Evidence and plots](benchmark-results/2026-09-30-r55/README.md)
retain the exact test/benchmark patch, all runs and validation. This step does not
extend the supported formats or qualify parent chains.

## Deterministic correctness coverage

Eight additional tests use an authored virtual block device, with metadata in
memory and payload generated from physical grain positions. Sixty layouts span
8 KiB, 64 KiB and 1 MiB grains, 17/511/512/513/1,024 grains and zero, contiguous,
reversed and alternating allocations. Twenty-four ranges per layout compare bytes
against an independent logical oracle, including grain and table boundaries.
A separate permutation fixture checks fragmented reads. Source counters prove that
one contiguous 1 MiB read coalesces, reversed/permuted grains require 16 requests,
alternating allocation requires eight and zero allocation requires none.

A fixed-seed 4,096-case corpus mutates header, descriptor, directory and both table
copies. It includes 64 semantic-preserving positive controls. Rejections must stay
within a 128 KiB read ceiling and observed backing bounds. Admitted maps must match
independently decoded table words and produce covering logical extents. This is a
finite deterministic mutation corpus, not coverage-guided fuzzing or proof that
all corruptions are rejected. Valid-layout byte oracles are a separate test.
Directory/entry sentinels, cross-table duplicate grains and metadata/payload
truncations add explicit negative cases. Invalid directory pointers fail before
any table reads.

## Resource and performance boundaries

Capacity profiles cover 1 MiB, 1 GiB and 64 GiB virtual disks with 64 KiB grains.
Only fixture metadata is allocated; no corresponding payload disk or giant RAM
buffer is created. Loader reservation and metadata-read counters are checked
against observed requests. Fixture storage, allocator overhead, caller buffers
and output vectors remain outside loader payload budgets; counters are not RSS.

A 1 TiB header at 64 KiB grains passes capacity admission but its eager-map
reservation exceeds the default 128 MiB aggregate budget. Loading rejects after
one header read. Header capacity is an upper bound, not a promise every geometry
fits the default memory budget. Aggregate admission conservatively includes
scratch reservations, even when temporaries do not overlap in lifetime.

Extent queries first count, then allocate and fill. They scan logical grains even
when the result is one zero extent. Alternating 8 KiB grains exercise 32,768 and
65,536 outputs; 65,537 fails before output allocation. A small tail query can
succeed on that same disk. The output limit does not cap virtual capacity or block
ordinary range reads. Large full-range planning can fail this output limit.

Thirteen new timing cases isolate opening, whole-map queries, a 4 KiB zero tail
read, fragmented 1 MiB reads and permuted-map validation. Opening excludes initial
descriptor parsing and fixture creation, and includes map validation/allocation/
drop. Queries include count/allocation/fill/drop on success; rejection times only
the failure path. Read buffers are reused. Generated payload filling is timed, so
fragmentation timings describe this synthetic backend, not disk seeks, CLI or
storage throughput. There is no prior matched scaling result or optimization
speedup claim. Existing unchanged harnesses provide paired regression controls.

## Architectural consequence and next work

Keep the bounded eager map and two-pass query policy for now. R5.5 establishes
costs and guards without changing production code. Consider indexes/caching or
chunked extent consumption only as separate measured changes with an explicit
memory and consistency contract. Do not raise budgets to hide scaling costs.
PERF.0 and earlier shared-host performance investigations remain open.

Next **R5.6** defines and implements bounded parent-chain metadata admission:
explicit resolver policy, CID/identity validation, missing-parent errors, cycle
and depth limits, and aggregate resource accounting. Keep it separate from base
loading and from logical fallback reads; zero child entries cannot mean logical
zero once a parent is involved. Parent reads and CLI exposure follow as separately
qualified steps. Coverage-guided fuzzing remains planned. No ESXi is needed until
the V0 live-access proof is ready for the user's 60-day trial.
