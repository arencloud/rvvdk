# V0.3.2a — Powered-off probe guard and task inspection

Date: 2026-10-01. Baseline: `5e5e91b`.
[Contract](../../../crates/rvvdk-vsphere/README.md),
[ADR-0049](../../adr/0049-powered-off-export-probes.md),
[continuation plan](../../vmware-access-plan.md#v032--licensed-live-qualification).

## Outcome

The existing ESXi 8.0.3 / HostAgent API 8.0.3.0 host now lists
`esx.enterprise.cpuPackage` following the user's license update. Active assignment
remains unresolved; no successful export establishes operation eligibility yet.

Two powered-on probes returned faults and correlated with two running Export VM
tasks. The user canceled them manually. Read-only inspection confirmed both in
terminal error state with `cancelled: true`. No CancelTask request was made by
rvddk. No VM shutdown, disk transfer or artifact publication occurred.

The correction requires poweredOff before a probe calls ExportVm and treats
TaskInProgress acquisition cleanup as unconfirmed. The final guarded probe sends
15 control calls, ending with Logout and no ExportVm or shutdown. Its nonzero exit
is expected: `invalid_power_state` with `not_acquired`. The following inspection
shows the same two canceled tasks; recent-task history is not a complete lock oracle.
Final discovery observes the selected 30 GiB VM and untouched 60 GiB control powered
on, with Tools running and unchanged disk topology.

## Preserved observations

| File | Source and meaning |
|---|---|
| `eligibility.json` | Baseline export binary; powered-on probe returned generic SOAP fault, cleanup unconfirmed. |
| `before.json` | Baseline discovery; three successful sessions, available Enterprise edition. Overlapped a build. |
| `state-probe.json` | Intermediate typed-fault build; TaskInProgress. Its `rejected` cleanup label was incorrect and is superseded by final `unconfirmed` handling. Original retained unchanged. |
| `task-inspection.json` | Intermediate read-only inspection; two running export tasks, no timestamps yet. |
| `task-times.json` | Intermediate inspection with timestamps; task queue times correspond to the two probes. |
| `after-user-cancellation.json` | Guard/inspection build before final reference-prevalidation refinements; both tasks canceled and terminal. |
| `guarded-probe.json` | Final product source; powered-on probe returns locally without ExportVm. |
| `final-task-inspection.json` | Final product source; same two tasks canceled and terminal, no running tasks in this returned history. |
| `final-inventory.json` | Final product source; three successful sessions, both VMs powered on. |

All **13 sessions / 203 control calls** confirmed Logout. Exactly two calls were
ExportVm, both preceding the guard. No HTTP disk GET, lease Complete or Abort was
performed. The faults returned no owned lease handle to abort. Task queue times
are strong correlation with the probes, not independently established ownership.
No raw XML, private managed references, task IDs, VM names, disk paths, URLs,
license keys, host pin or credentials are retained.

`audit.json` records final binary/source-patch hashes and per-session call/error/
cleanup summaries. Baseline binary hashes from before modification:

- discover: `e2ec5f1e45c4e8624e4d85d690199f6bfb38090991a1388429a7f056937a82f2`
- export: `7e7df6c652545cf5b0559a11755678ca52dba477f53fc090b6b0d93703418287`

Intermediate build hashes/source snapshots were not captured; their observations
are diagnostic evidence, not reproducible benchmark candidates. The final
`source.patch` applies to baseline and contains all Rust/example/test changes.
Documentation is in the containing commit. The final live binary predates only a
test-assertion correction; production/example source is identical.

## Validation and performance disposition

`final-tests.txt`: **580 unique workspace tests passed**, one existing ignored;
43 tests belong to rvvdk-vsphere. Count only unfiltered top-level summaries; child
processes repeat tests. Six new tests exercise powered-on admission, read-only task
inspection, identity binding, limits/duplicates/types, redaction and UTC calendar
validation. Existing fault coverage now includes MethodDisabled, InvalidState and
TaskInProgress, including uncertain cleanup.

`initial-tests.txt` retains one new assertion failure: it matched the ordinary
datastore `info` property as a Task query. The assertion now matches the requested
Task object type. The full rerun passes. `final-clippy.txt`, `final-fmt.txt` and
`final-build.txt` record successful workspace Clippy, formatting and release builds.

There is **no formal performance comparison or new plot** in this correction.
Operations differ, task/host state changed, and some sessions overlapped local
builds/tests. Raw timing and discovery CPU/RSS remain visible, without speedup,
regression clearance or transfer-throughput claims. Disk streaming code and normal
discovery property lists are unchanged. V0.3.1's adverse individual-pair gate,
PERF.0/R4.4 and the historical QEMU producer discrepancy remain open.

V0.3.2 still requires guest access for deterministic fixture preparation (requested,
pending), actual export/format and independent logical-byte verification, live
Complete/Abort cleanup, and at least three comparable transfer runs with CPU/RSS,
encoded/logical counts, sync boundaries and plots. A server digest alone is not an
independent logical-byte oracle. The saved plan uses this existing host; neither
vCenter nor a replacement deployment is required for the current scope.
