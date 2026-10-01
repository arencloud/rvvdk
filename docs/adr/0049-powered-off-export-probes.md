# ADR-0049: Powered-off export probes and read-only task inspection

- Status: accepted for V0.3.2a; full V0.3.2 qualification remains open
- Date: 2026-10-01
- Baseline: `5e5e91b`
- Amends: [ADR-0048](0048-bounded-export-lease-proof.md) probe precondition

## Observation

After the user's license update, two powered-on ExportVm probes returned faults.
Two running export tasks had creation times corresponding to those probes. This is
strong correlation, not definitive task ownership: raw fault/task references were
not retained in diagnostics. The user canceled both, and subsequent read-only
inspection confirmed terminal error states with `cancelled: true`.

## Decision

Require a revalidated powered-off VM before the acquire-and-abort probe. Do not use
ExportVm as a powered-on licensing check. Preserve TaskInProgress as a typed error
with unconfirmed acquisition cleanup; receiving a fault is insufficient evidence
that no remote work started. Do not retry ambiguous acquisition automatically.
Full transfer keeps its existing explicit graceful-shutdown path.

Add a distinct inspection option, incompatible with transfer/shutdown options.
Read only recentTask and each referenced TaskInfo, bounded to 32 unique Task
references. Validate both requested task identity and optional VM entity identity.
Emit closed state/operation codes, strict UTC timestamps and cancellation booleans;
exclude IDs, raw descriptions, results and fault messages. Keep ordinary inventory
property queries unchanged. A recent-task list can be incomplete and is not an
ownership or lock oracle. Do not implement CancelTask in this change.

## Validation and limits

Local TLS fixtures verify no ExportVm/ShutdownGuest on a powered-on probe, typed
faults and unconfirmed TaskInProgress cleanup, successful read-only inspection,
identity rejection, task reference limits/duplicates/types, timestamp validation
and diagnostic redaction. All paths retain explicit Logout. Live final-source
inspection confirms the user's cancellation; the guarded live probe sends no
ExportVm. No disk bytes or performance improvements are claimed.

Guest access is still needed to prepare independent deterministic content before
shutdown. Live format/bytes, Complete/Abort cleanup, transfer CPU/RSS and repeated
throughput plots remain V0.3.2. The host now lists an Enterprise edition, but active
assignment and actual export eligibility remain unresolved.

[Evidence](../benchmark-results/2026-10-01-v032a/README.md),
[continuation plan](../vmware-access-plan.md#v032--licensed-live-qualification).
Protocol references: [VirtualMachine.ExportVm](https://developer.broadcom.com/xapis/vsphere-web-services-api/latest/vim.VirtualMachine.html),
[TaskInProgress](https://developer.broadcom.com/xapis/vsphere-web-services-api/latest/vim.fault.TaskInProgress.html),
[TaskInfo](https://developer.broadcom.com/xapis/vsphere-web-services-api/latest/vim.TaskInfo.html).
