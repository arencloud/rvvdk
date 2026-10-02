# ADR-0051: Powered-off HTTP NFC as the first qualified transport

- Status: accepted for the bounded powered-off single-disk export workflow
- Date: 2026-10-02
- Baseline: `4e57c87`
- Extends: [ADR-0046](0046-independent-export-feasibility.md), [ADR-0050](0050-live-export-compatibility.md)

## Decision

Use the independent Rust SOAP/HTTP NFC export implementation as the first transport
for the bounded powered-off, single-disk, direct ESXi 8.0.3 workflow. It yields a
sequential VMDK container artifact. It does not supply random logical reads and must
not implement `BlockDevice::read_at` by pretending that container offsets are guest
sectors. Public logical conversion needs a separate native format decoder.

Preserve exact endpoint/certificate trust, private source identity revalidation,
explicit Complete/Abort and Logout, manifest validation, bounded private staging
and durable no-replace publication. No production QEMU or SDK/VDDK fallback.

## Qualification boundaries

The user approved using the former 60 GiB control VM as a LAN runner. This shares
ESXi host/storage resources with the powered-off 30 GiB source, so measurements are
observations of this setup, not an isolated physical-host benchmark or a code
optimization comparison. CPU/RSS belong to the Rust export process; offline QEMU
and hashing run outside timed transfers. Preserve prior remote-connection failures.

The guest oracle identifies an independently mapped 8 MiB known range. Independent
QEMU decoding and matching that range establish the scoped byte proof. Repeated
encoded-file equality can reuse the independent result only after the published
artifact's size and digest are checked against the previously verified image.
If encoded digests differ, QEMU must compare their complete decoded logical
content against the independently verified reference before carrying forward the
known-range result. Neither repeated image equality nor a server manifest proves
whole-source identity.

## Follow-on architecture

1. Specify and implement bounded native version-3 streamOptimized admission and
   decompression in separate R5 increments: header/descriptor/marker bounds first,
   then indexed grain reads, integrity/resource failures, synthetic and QEMU byte
   oracles, benchmarks, and public local CLI integration. Keep the existing reader's
   rejection until each supported subset is proven. The live export is private lab
   evidence; author safe synthetic regression fixtures rather than committing it.
2. Integrate the qualified container-export workflow under R6 with explicit source
   selection and trust inputs, secret-safe reporting and typed artifact semantics.
   The current capacity-based experimental selector is not a production identifier.
3. Define durable resource ownership and crash/restart reconciliation before claiming
   recoverable backup jobs. No retries of ambiguous resource-creating calls. Existing
   cooperative cancellation does not prove cleanup after process termination.
4. Keep online snapshots, multi-disk consistency, random access, CBT, import/restore,
   vCenter and vSphere 9 as independent qualification packages. Least-privilege roles
   and active-license assignment/expiry remain separate from observed operation success.

Three completed exports, independent known-byte verification and prior real failure
cleanup close V0 for this export-only scope. [Evidence and plots](../benchmark-results/2026-10-02-v032c/README.md).
R6 and native
compressed decoding remain open. PERF.0/R4.4, V0.3.1's adverse individual discovery
pairs and the historical QEMU partial second-extent discrepancy remain open.

[Saved acceptance plan](../vmware-access-plan.md#v03--powered-off-export-and-cleanup-proof).
