# Transfer and worker SHA-256 qualification

PERF.1c, 2026-10-15. Reference: `fb4ea8d` (accepted PERF.1b).

Adopt the existing private conditional SHA-256 backend in four additional
encoded-payload paths: proof export download, owned transfer download, worker
seal/readback, and worker native admission. The [report and plots](benchmark-results/2026-10-15-perf1c/README.md)
retain the frozen comparisons, adverse results, controls and limits.

## Runtime boundary

On Linux x86-64, SHA/SSE2/SSSE3/SSE4.1-capable CPUs keep RustCrypto `sha2`;
otherwise the existing facade selects `ring`. Other Linux architectures keep
`sha2`, and the portable export path explicitly keeps `sha2` outside Linux.
There is no new dependency, public API, format, journal or command.

Each site still hashes all encoded bytes. Download retains HTTP status, encoding,
size/chunk limits, KDMV prefix, SHA-1, writes, manifest and lease controls. Owned
transfer retains its bounded worker queue and acknowledgments. Seal still syncs
and closes the sole writer, opens a fresh checked descriptor, and verifies the
full length, SHA-256 and SHA-1 before accepting it. Native admission still decodes
every present grain, revalidates structure, rehashes the full container with
cancellation/deadline checks, and syncs/rereads/reparses private metadata. Source,
namespace, ownership and identity checks remain. No earlier digest replaces a pass.

The [site-by-site audit and predeclared gates](benchmark-results/2026-10-15-perf1c/plan.md)
map existing corruption, heartbeat, cancellation, failure and durable-boundary
checks to these paths. Runtime hashing and VMware operations remain Rust;
Python only drives offline fixtures, checks evidence and renders plots.

## Evidence and disposition

The same frozen reference/candidate binaries execute natively on a Xeon Silver
4116 runner without SHA-NI and an i7-11850H host with SHA-NI. Each comparison uses
one host, CPU 0, the same current filesystem/resources and compiler/lockfile.
Small, bulk and stored-zero cases cover complete loopback TLS workflows plus
separate worker seal/admit phases. Private tmpfs controls separate part of the
storage cost. PERF.1b supplies the previously accepted isolated hash evidence.

There are 2,640 timed observations: 1,728 workflow/phase samples, 144 startup
samples, and 768 separately declared A/A and interleaved A/B controls. All full
encoded-byte, independent digest, logical fixture, allocation and cleanup oracles
pass. Existing SHA-1, readback and durability work remains inside the operations.

Native bulk complete operations use 14.37–30.09% less wall time and worker phases
22.43–41.96% less. Original SHA-NI wall changes span −3.90% to +2.88%; four CPU
results cross +5% and are retained. Supplemental A/B CPU changes for those four
operations span −0.76% to +2.64%. The native small owned-export original is +6.67%
wall/+5.65% CPU; the separate A/B control is +3.63% wall/−1.25% CPU. These controls
support bounded adoption without establishing that small inputs never regress.
Both release CLIs are 3,141,384 bytes. Initial/repeat spread, cleanup, process RSS
and all original adverse values remain in the report.

## Limits and continuation

This qualifies small, normally cached local/loopback fixtures on two CPU profiles.
The 30 GiB logical native fixture has only one or 128 present 64 KiB grains; it is
not a full 30 GiB transfer or a cold/disk-scale workload. Process RSS includes
fixture construction and TLS; startup child RSS can include launcher memory.
Frequency, SMT and hypervisor/host scheduling are not isolated. No ESXi API,
live export, VM power/resource operation, other-architecture throughput test or
production recovery qualification occurred. PERF.0 variance and wider workloads
remain open.

Next: **R6.1d.5a — live retained-artifact recovery qualification plan and preflight**.
Inspect existing private lab artifacts and original source/trust provenance
read-only. Establish available space, a frozen CLI, independent byte oracles and
an explicit recovery-state/command matrix before a separately scoped live-data
qualification run. Preserve originals and use only independently owned disposable
fixtures. Do not edit live journals to manufacture authority or replay remote
leases. Record which data/ownership/capacity prerequisites are actually available;
then implement the bounded harness in R6.1d.5b. This begins the already-open live
retained-artifact command follow-up, not general source-transaction recovery.

No new trial, vCenter or VM is needed for this preflight. Any need for a fresh
export or new lab capability must be established from that audit. Continue local
commits after completed packages. Keep snapshots, CBT, restore, multi-disk
consistency, VMFS sparse, seSparse, vCenter and vSphere 9 separately qualified.
