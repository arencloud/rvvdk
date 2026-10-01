# ADR-0050: Opaque references and single-disk selection in live exports

- Status: accepted; live qualification results are tracked separately
- Date: 2026-10-01
- Baseline: `dedce58`
- Extends: [ADR-0048](0048-bounded-export-lease-proof.md), [ADR-0049](0049-powered-off-export-probes.md)

## Observed interoperability gaps

The existing licensed host accepts ExportVm on the powered-off selected VM. Its
lease reference contains square brackets. The former ASCII identifier filter
rejected it before ownership could be recorded. The original attempt retained
unconfirmed cleanup; subsequent read-only inspection observed its task become
terminal before another acquisition was attempted. No blind retry or manual
cancellation hid the initial uncertainty.

After correcting references, the host returned three device URLs, with disk flags
false/true/false. Requiring exactly one URL incorrectly rejected a lease containing
one disk and auxiliary files. Those attempts did identify their leases, so Abort
and Logout were acknowledged.

## Decision

Treat managed-reference values as opaque XML text bounded to 256 UTF-8 bytes.
Retain the exact decoded value for identity and separately escape it when building
requests, including a numeric reference for carriage return to preserve XML round
trips. Keep the closed managed-type allowlist, unqualified type-attribute check,
namespace/identity checks and redacted diagnostics.

Select exactly one explicit disk entry from at most 32 lease devices, with unique,
nonempty, bounded keys and required boolean disk flags. Ignore and count explicit
non-disk entries; never fetch their URLs. Apply the same bounds and unambiguous disk
selection to the download manifest, and match the selected lease key before checking
size/capacity/digest. This produces a disk container, not an OVF or complete VM
package. Multiple disks and absent/ambiguous flags remain unsupported.

Expose accepted encoded-body bytes even on failure/cancellation. The counter is
not durable or published bytes and excludes a chunk rejected by the transfer limit.
Add CPU seconds and process-lifetime peak RSS to the qualification example. Keep
whole-operation, encoded-transfer and logical-capacity quantities separate.

## Independent validation

Prepare an 8 MiB deterministic guest file before graceful shutdown. Obtain its
XFS extent map and the linear-LVM/partition translation independently, then verify
the translated sectors through a read-only guest disk read. Keep fixture contents,
digests, maps and VM images outside Git. After a completed export, QEMU can decode
it offline solely for lab validation; a bounded local comparator checks every mapped
fixture byte against the independent source. This proves the known fixture range,
not whole-source disk equivalence or Rust support for compressed VMDK decoding.

Python helpers operate only on local files for reference comparison and plots.
VMware access and the shipping transfer implementation remain Rust. No SDK or
production QEMU fallback is introduced. Full completion, cancellation, failure,
cleanup and repeated performance outcomes must be recorded before closing V0.3.2.

## References

- [ManagedObjectReference](https://developer.broadcom.com/xapis/vsphere-web-services-api/latest/vmodl.ManagedObjectReference.html)
- [Lease information](https://developer.broadcom.com/xapis/vsphere-web-services-api/latest/vim.HttpNfcLease.Info.html)
- [Device URLs](https://developer.broadcom.com/xapis/vsphere-web-services-api/latest/vim.HttpNfcLease.DeviceUrl.html)
- [Manifest entries](https://developer.broadcom.com/xapis/vsphere-web-services-api/latest/vim.HttpNfcLease.ManifestEntry.html)
- [Linux FIEMAP](https://docs.kernel.org/filesystems/fiemap.html)
- [Linux linear device mapper](https://docs.kernel.org/admin-guide/device-mapper/linear.html)
- [QEMU image tools](https://www.qemu.org/docs/master/tools/qemu-img.html)
