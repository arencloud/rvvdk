# SHA-256 backend qualification

Historical PERF.1a decision. [PERF.1b](sha256-fallback-qualification.md) subsequently
qualifies a conditional fallback on native non-SHA hardware, while preserving the
accepted accelerated backend. The results and rejected unconditional patch
below remain unchanged.

PERF.1a, completed 2026-10-15; measurements began 2026-10-14. Baseline:
`95ca5245a60438965bec71a461687d41b6ce292d`. This is a bounded performance
investigation with a rejected experimental backend. Production continues to use
`sha2 0.11.0`; no production dependency, digest, verification pass or persistence
boundary changes.

## Decision and architecture

The current Intel i7-11850H exposes SHA, SSE2, SSSE3 and SSE4.1. The locked
`sha2` implementation already selects SHA-NI at runtime on this configuration.
Switching libraries would not newly enable hardware hashing. Its software fallback
on the earlier runner without SHA-NI is a separate performance question.

`ring 0.17.14` is already a transitive TLS dependency. That makes it an economical
candidate to investigate, but does not establish lower digest or command costs.
The standalone Rust probe uses the same locked versions as the workspace. It
compares complete streaming SHA-256 operations, including initialization and
finalization, rather than only compression primitives.

On this host, `ring` loses every repeated microbenchmark: bulk wall time is
4.26–8.64% higher and 64-byte record wall time is 33.40% higher. Keep the existing
backend. Full-operation results and variance are retained in the
[report and plots](benchmark-results/2026-10-14-perf1a/README.md); they do not
override the failed microbenchmark promotion gate or establish a generic result
for other CPUs.

The isolated candidate changes exactly three bulk hash contexts:

| Path | Contract preserved |
|---|---|
| Retained artifact admission | Read and SHA-256 every encoded container byte; check length and original metadata |
| Owned RAW verification | Compare every logical source/output byte and hash the entire RAW before Verified |
| Fresh publication/recovery verification | Repeat full source/output comparison and RAW SHA-256 before publication or recovered acknowledgment |

Buffers remain 1 MiB. Cancellation/deadline checks, source binding, sparse output,
file identity, journal checks, directory checks, synchronization and promotion
ordering are unchanged. No skipped zero range, cached prior digest, combined
verification boundary, extra worker or changed chunk size enters the candidate.
Identity, journal, contract and TLS hashes remain unchanged. Export transfer and
worker admission/payload hashes are audited but excluded from this bounded
candidate; no network workflow is measured.

The private facade and direct dependency exist only in the archived
[rejected patch](benchmark-results/2026-10-14-perf1a/rejected-candidate.patch).
They are not added to the production architecture. Keeping an unused backend or
new public selector would add maintenance without a supported benefit.

## Qualification boundary

Known SHA-256 vectors, padding/block/chunk boundaries, dense and sparse-pattern
bytes pass both backends. Independent Python `hashlib` checks verify the fixture
digests. The same probe passes under QEMU with SHA-NI masked. This is fallback
correctness evidence, not physical non-SHA CPU performance evidence.

Candidate workspace, existing fault/process-loss/cancellation tests, actual CLI
fixtures and frozen paired publication/recovery measurements qualify the bounded
experiment. Private tmpfs is a timing/allocation control, not durable storage.
64/256 MiB normally cached fixtures do not establish cold or disk-scale throughput.
CPU affinity is fixed; host activity, frequency and SMT are not isolated. All
slower results and initial/repeat spread remain recorded. PERF.0 stays open.

## Recorded continuation: PERF.1b (now complete)

Qualify the earlier runner without SHA-NI before selecting any alternate fallback:

1. Reconfirm its CPU features, compiler, kernel, storage, memory and available
   affinity. Freeze reference source/dependencies/binaries on that runner. Never
   compare current-host timings directly against the earlier runner as a speedup.
2. Run the standalone Rust probe's correctness and paired dense/sparse, small/bulk,
   chunk-size and larger-working-set matrix natively. Repeat every group under the
   same retention policy. QEMU timings cannot substitute for this run.
3. Only if microbenchmarks justify it, evaluate a private feature-aware candidate
   that keeps the accepted SHA-NI behavior. Repeat full publication/recovery and
   CLI comparisons on each affected CPU configuration; retain every verification,
   cancellation, allocation and durability gate.
4. Accept only a supported bounded gain, or save another negative result. Export
   plots, update the roadmap/log, scan staged evidence and commit locally.

This can use the existing runner if its recorded configuration is still available;
it does not require a new ESXi trial, vCenter or a new VM. If that CPU configuration
is unavailable, record the limitation instead of treating emulation or a changed
VM as equivalent. Larger cold workloads, storage/layout tuning, native decoding,
map/cache work and live retained-artifact recovery remain separate packages.
