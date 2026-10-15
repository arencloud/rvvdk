# Native SHA-256 fallback qualification

PERF.1b, 2026-10-15. Baseline `82c025d2571ee7fba1cf1ffbf854e0118519341e`.
This follows the [PERF.1a decision](sha256-performance-qualification.md), which
rejected an unconditional backend replacement on the SHA-NI development host.
The [native evidence and plots](benchmark-results/2026-10-15-perf1b/README.md)
qualify a narrower runtime selection for three retained payload hash contexts.
Adopt that selection: measured non-SHA bulk hashing uses 43.22–46.94% less wall time,
and complete core workflows use 34.95–44.65% less. SHA-NI core workflow changes range
from −1.42% to +3.72%; startup and phase variability remain open in PERF.0.

## Backend boundary

The private `payload_hash::Sha256` facade returns the same 32-byte SHA-256 value.
On Linux x86-64, it keeps `sha2 0.11.0` when SHA, SSE2, SSSE3 and SSE4.1 are all
available—the locked dependency's existing accelerated-path prerequisites.
Otherwise it uses `ring 0.17.14` and that library's internal CPU dispatch.
Other architectures retain `sha2`. Selection occurs when a streaming context is
created; there is no public backend option, changed digest format or persistent
backend marker. No allocation or unsafe block is added by the facade.

`ring` is already present through TLS. The target-specific direct dependency adds
no new version; its default features were already enabled by the existing TLS
edge. The facade is private to the Linux retained-output implementation.

| Changed context | Preserved contract |
|---|---|
| Retained container admission | Read and hash every encoded byte; compare sealed length/digest and metadata |
| Owned RAW verification | Read and compare all logical source/RAW bytes and hash the complete RAW |
| Fresh publication/recovery verification | Repeat full comparison and hashing before publication, attestation or recovered acknowledgment |

All 1 MiB buffers, zero-byte hashing, cancellation/deadline checks, original-source
binding, file identity, allocation checks, journal controls and sync/promotion
ordering remain intact. The change neither combines verification passes nor reuses
an earlier digest. Identity, journal, contract, TLS, export, owned-transfer and
worker admission/payload hashes keep their existing implementation.

One test-only portability change lets the cross-filesystem publication test use
`RVVDK_RETAINED_PARENT`. A frozen test executable can then exercise the check on
runner XFS and private tmpfs without needing its compile-time checkout path.

## Qualification and limits

Identical frozen reference/candidate executables run natively on a Xeon Silver
4116 guest without SHA-NI and an i7-11850H host with SHA-NI. Builds use the same
local compiler, dependencies and default flags. The guest has no Rust compiler;
this is native execution of locally built binaries, with recorded hashes.

The guest now exposes two CPUs and approximately 8 GiB, versus one CPU/2 GiB in
older evidence. Both sides of each comparison use the current configuration,
CPU 0 and the same filesystem/cache conditions. Historical measurements and
cross-machine timing ratios are not used to establish an implementation gain.

Every primary group retains three rounds of four pairs and three rounds of eight
pairs. Native hashing includes small inputs, dense/sparse patterns, chunk-size
variation and working sets up to 512 MiB. Complete workflows include normal
publication, reconciliation, attestation, acknowledgment and direct/whole-CLI
controls. Correctness includes known vectors, independent digests, streaming
boundaries, full workspace tests and the existing fault/process-loss/control tests.

Private tmpfs with `huge=never` controls allocation; it is volatile. Normally cached
64/256 MiB workflow fixtures and in-memory 512 MiB hash inputs do not qualify cold
or disk-scale performance. Frequency, SMT, hypervisor scheduling and competing
host activity are not isolated. Child high-water RSS may include launcher memory
inherited before exec; it is not an isolated CLI working-set measurement.

The report retains adverse timings, initial/repeat spreads and separately declared
assessment startup controls. PERF.0 stays open. These results cover the measured
CPU configurations and three hash contexts; they do not establish a universal
non-SHA CPU gain, other-architecture throughput or live VMware export performance.

## Next package: PERF.1c

Audit and qualify the remaining bulk hashes in `export.rs`, `owned_transfer.rs`,
`journal_worker/admission.rs` and `journal_worker/payload.rs` as a separate package:

1. Map each hash's byte domain, source/ownership binding, cancellation and
   persistence boundary. Identify representative existing fixture/fault tests.
2. Freeze this accepted implementation as the reference. Add bounded measurements
   for the complete affected operations, including small payloads, bulk/zero-rich
   data and transfer/readback costs. Separate CPU hashing from loopback/network and
   storage bottlenecks. Preserve independent full-byte digests and all gates.
3. Test a private reuse of the facade only where the byte domain and platform
   boundary permit it. Repeat same-host comparisons on both native CPU profiles,
   preserving every adverse result and checking startup/size/resource costs.
4. Promote only supported changes, or commit a negative result with plots, source
   provenance, updated planning and a local commit. Native runner access is enough
   for local/loopback qualification; a live export is a distinct explicitly scoped
   qualification and must not be inferred from these timings.

No new ESXi trial, vCenter, VM or guest resource change is required for that local
package. Live retained-artifact recovery, larger/cold workloads, stream decoding,
map/cache tuning, general production recovery, snapshots, CBT and restore remain
separate work.
