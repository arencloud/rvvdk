# ADR-0048: Bounded independent Rust export-lease proof

- Status: accepted for V0.3.1; live byte/cleanup qualification pending licensing
- Date: 2026-10-01
- Baseline: `192489b`

## Decision

Extend the isolated `rvvdk-vsphere` crate with a single-disk, powered-off HTTP NFC
export proof. Share the bounded authentication/Logout driver with discovery while
preserving the discovery report schema. General HTTP/TLS/XML crates remain allowed;
no VMware SDK, VDDK bindings or Python implementation participates.

Keep eligibility probing separate from transfer. A probe makes one `ExportVm`
request without changing power and aborts any granted lease immediately. The real
host returned `RestrictedVersionFault`; Logout succeeded. This establishes a license
gate for this operation/account, not the precise active license assignment or the
eligibility of all VMware APIs. No VM power, guest files or license was changed.

The transfer subset selects one unambiguous disk capacity and revalidates private
VM/disk identities. Explicit Tools shutdown is graceful only. One lease is owned,
renewed, then completed or aborted. No acquisition/completion retries hide ambiguous
remote outcomes. A caller must await cooperative cancellation/cleanup.

Data access stays on the pinned HTTPS authority. Lease thumbprints are checked in
addition to the caller's SHA-256 pin, including ESXi's legacy SHA-1 representation.
API passwords/cookies never go to GET. Buffers, bytes, XML, request records and
deadlines are bounded. Errors/reports contain closed codes and allowlisted fields.

Linux descriptor-relative private staging is published atomically without replacement
only after manifest verification, file sync, lease completion and Logout. Separate
cleanup results preserve uncertainty. Failed parent sync after rename is an error
with an explicitly published artifact, not rollback or success.

## Consequences and acceptance

Local TLS fixtures exercise success, renewal, rejection, shutdown/revalidation,
cancellation, truncation, byte limits, redirect/pin mismatch, manifest mismatch,
ambiguous acquisition and remote cleanup failures. Filesystem tests cover collision
and no-replace publication races. Fixture bytes are authored synthetic data; they
do not qualify VMware VMDK encoding or decoded guest bytes.

V0.3.1 is the executable foundation and observed license gate. V0.3.2 requires
supported trial/commercial access, a live transfer, actual format identification,
an independent logical-byte oracle, live complete/abort evidence and repeated
transfer performance measurements. Neither V0 nor R6 is declared complete.
Keep prior performance investigations and the QEMU producer discrepancy open.

[Contract](../../crates/rvvdk-vsphere/README.md),
[evidence](../benchmark-results/2026-10-01-v031/README.md),
[continuation plan](../vmware-access-plan.md#v032--licensed-live-qualification).

Protocol contracts: [VirtualMachine](https://developer.broadcom.com/xapis/vsphere-web-services-api/latest/vim.VirtualMachine.html),
[HttpNfcLease](https://developer.broadcom.com/xapis/vsphere-web-services-api/latest/vim.HttpNfcLease.html),
[lease info](https://developer.broadcom.com/xapis/vsphere-web-services-api/latest/vim.HttpNfcLease.Info.html),
[device URL](https://developer.broadcom.com/xapis/vsphere-web-services-api/latest/vim.HttpNfcLease.DeviceUrl.html),
[manifest entry](https://developer.broadcom.com/xapis/vsphere-web-services-api/latest/vim.HttpNfcLease.ManifestEntry.html).
