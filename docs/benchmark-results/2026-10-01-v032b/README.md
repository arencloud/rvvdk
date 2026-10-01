# V0.3.2b — Guest oracle and live export compatibility

Date: 2026-10-01. Baseline: `dedce58`.
[Contract](../../../crates/rvvdk-vsphere/README.md),
[ADR-0050](../../adr/0050-live-export-compatibility.md),
[continuation plan](../../vmware-access-plan.md#v032--licensed-live-qualification).

## Scope and private inputs

The existing ESXi 8.0.3 / HostAgent API 8.0.3.0 host is used, with one selected
30 GiB disk and a 60 GiB control VM. Guest SSH access was confirmed for both;
only the selected guest received a new deterministic 8 MiB fixture file. Its local
reference, digest, guest extent map and disk images remain in ignored private
storage. Guest passwords, host password/pin, private endpoints and identities are
excluded from this directory. No license changes, SDK/VDDK calls or new Python
VMware access were used.

The selected guest uses XFS on one linear LVM segment. The FIEMAP extent (reported
by filefrag in 512-byte units), LVM segment offset and partition start independently
identify the fixture's disk range. A direct read of that range inside the guest
matched the locally generated reference before graceful shutdown. The user had
already authorized shutdown; no hard power operation was used. The control VM
received only read-only SSH inspection.

The offline comparator checks mapped bytes in a QEMU-decoded RAW file. Its result
is scoped to the 8 MiB fixture, not whole-source disk equivalence. QEMU remains an
independent lab decoder, not a production dependency or automatic fallback.

## Failures and cleanup retained

| Report | Observation |
|---|---|
| `pre-export-inspection.json` | Selected VM powered on; no running/queued tasks in the returned recent-task history. |
| `initial-export.json` | Graceful shutdown reached poweredOff. ExportVm returned a lease reference rejected by the old identifier filter; cleanup unconfirmed, Logout acknowledged, zero bytes. |
| `after-initial-failure.json`, `lease-timeout-inspection.json` | The resulting task was still running; no acquisition retry occurred. |
| `lease-timeout-inspection-2.json` | The task reached a terminal error state without manual cancellation. The exact server error was not retained; the filename does not prove a particular timeout fault. |
| `opaque-reference-probe.json` | Corrected bounded opaque-reference parsing; acquire-and-abort and Logout both acknowledged. |
| `export-1.json` | Lease acquired, but the one-URL assumption rejected auxiliary entries. Abort and Logout acknowledged; zero bytes. |
| `device-diagnostic.json` | Same rejection, with sanitized diagnostics establishing disk flags false/true/false. Abort and Logout acknowledged. |
| `completed-1.json` | One-hour deadline after 2,544,547,134 accepted encoded bytes; Abort and Logout acknowledged, no publication or staging residue. The filename records intent, not success. |
| `export-2.json` | Corrected disk selection streamed 272,684,467 accepted encoded bytes before deliberate Ctrl-C cancellation. Abort and Logout acknowledged; no publication or remaining staging directory. |

The reference-shape diagnostic reports only length and character classes, never
the reference. It observed an 81-byte lease reference containing square brackets.
The device diagnostic reports only field presence/counts and booleans, never URLs
or keys. Both temporary diagnostics were removed from the measured build.

All attempts remain visible. The original unconfirmed acquisition report is not
rewritten as successful cleanup simply because the task later became terminal.
No CancelTask was issued. Explicit Abort is attempted only for a parsed lease owned
by the current invocation.

## Build and validation provenance

Source patches and binary hashes accompany the diagnostic, reference-fixed,
disk-selection and measured variants. `measured-source.patch` contains the Rust
source and tests used for completed-run attempts. Later documentation changes do
not alter that executable. The first instrumented build, used for initial shutdown
and reference rejection, did not have a separate binary snapshot retained; those
reports are correctness observations, not comparable performance samples.

The measured code passes **584 unique workspace tests**, one existing ignored,
and workspace Clippy with warnings denied. Four added Rust tests cover opaque
reference serialization, lease abort using escaped opaque references, explicit
auxiliary-file exclusion, and bounded unambiguous disk selection. Existing transfer
and manifest-failure tests also check retained received-byte accounting.
Three additional offline oracle tests exercise fragmentation, corrupted bytes,
invalid/overlapping/out-of-range maps, wrong digests and truncated disks.

## Transfer qualification status

Completed-transfer measurements and decoded format/oracle results are pending.
[SVG plot](attempts.svg), [PNG plot](attempts.png) and
[computed data](attempts-computed.json) retain all five full-mode attempts.
A cooperative cancellation and a real deadline with received bytes are established;
completed artifacts and independent decoded-byte verification are not yet claimed.
The live source remains powered off during measurement. Builds/tests are kept out
of timed completed-run attempts. Network and ESXi host/storage load are uncontrolled;
client-side RSS excludes kernel socket buffers and filesystem page cache.

Throughput must use encoded bytes over receive/hash/write/file-sync elapsed time.
Logical capacity is a separate quantity. Whole-operation time includes discovery,
lease lifecycle, Logout, metadata/directory sync and publication. CPU measurement
excludes terminal entry and final JSON serialization; RSS is process-lifetime peak.
The cancellation sample is not a completed-transfer throughput benchmark.

V0.3.1's adverse individual discovery pairs, PERF.0/R4.4 and the historical QEMU
partial second-extent discrepancy remain open. No tuning improvement or regression
clearance is claimed from these diagnostic attempts.


The deadline run lasted 3,603.675 seconds including cleanup, used 33.624 CPU
seconds and reported 7,868 KiB process-lifetime peak RSS. Low client CPU does not
identify the network/host/storage bottleneck. `completed-1-progress.jsonl` samples
staging size and process usage every ten seconds after monitoring began several
minutes into the attempt; its time origin is not the operation start.
`provisional-container-header.json` records only allowlisted prefix fields:
version 3, 64 KiB grains, compression algorithm 1, streamOptimized, 30 GiB logical
capacity. The full container was never validated. Native compressed decoding is
still unsupported. No QEMU operation ran against a growing partial image.

VM02's read-only prerequisite check found x86_64/glibc 2.43, about 12.7 GiB free,
and no qemu-img. A local private runner package is prepared, but using VM02 would
change its untouched-control role. The user was asked to choose that setup, a
separate LAN runner, or a longer timeout on this connection. No guest installation
or runner files were written to VM02 in this step.


Final read-only inspection found one returned export task in terminal error, no
running/queued task in that bounded history, and confirmed Logout. Three final
discovery sessions confirmed the selected VM powered off and control powered on;
each acknowledged Logout. The selected guest fixture remains for the next run.
[Validation summary](validation-summary.json) preserves every session's cleanup
classification. The final product source and binary hashes match the measured
snapshot. Source format/diff checks and offline comparator tests pass. Raw test logs and
source snapshots preserve their original whitespace and are excluded from the
whitespace-only diff check. Plot smoke checks
reject the actual deadline report as a completed sample and validate synthetic
rate calculation/rendering only in discarded temporary output.

Reproduce the actual diagnostic plot from the repository root:

```bash
target/benchmark-plots/bin/python scripts/vsphere/plot_export_attempts.py \
  docs/benchmark-results/2026-10-01-v032b \
  docs/benchmark-results/2026-10-01-v032b/{initial-export,export-1,device-diagnostic,export-2,completed-1}.json
```
