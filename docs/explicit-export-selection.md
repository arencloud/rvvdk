# Explicit export selection (R6.1c.1)

`export_selected_vm(source, credentials, options)` connects the R6.1a source
identity to the existing bounded Linux export proof. It selects by VM reference,
UUID, disk key and backing, with expected capacity and endpoint trust. Two VMs
with equal capacities no longer make an explicit request ambiguous. Missing or
changed identity never falls back to capacity selection.

The connection policy comes exclusively from `source.endpoint()`: there is no
second policy parameter that could route this operation elsewhere. TLS still
checks the supplied certificate pin. Pin provenance remains a caller assertion.
The all-or-nothing explicit mode in the Rust `export` example requires
`--vm-reference`, `--vm-uuid`, `--disk-key`, `--disk-backing` and
`--pin-provenance tofu|externally-verified`. `--capacity-bytes` supplies expected
capacity. Obtain identities through private authenticated inventory; keep arguments,
reports and export artifacts out of public logs and source control.

## Scope and ordering

The original `export_vm` API and default example mode remain the capacity-selected
qualification proof. The explicit API reuses `ExportOptions`, rejects a capacity
that differs from the source, and rejects `allow_graceful_shutdown` before opening
a connection. It requires an already powered-off VM, one persistent unencrypted
FlatVer2 disk, no parent or snapshots, and no disabled ExportVm method. It does
not silently power down or expand the previously admitted disk subset.

For a full export, identity and scope are checked:

1. In bounded inventory obtained on the admitted connection.
2. In a fresh property read before local output admission.
3. After local staging admission and immediately before ExportVm.
4. After the acquired lease is ready, before the data GET.
5. After transfer, manifest verification and metadata sync, before lease completion.

The last three are additional reads relative to the capacity proof. Before each
of the two reads with a ready lease, progress is refreshed. The read deadline is
the earlier of the overall deadline and one third of the advertised lease timeout,
capped at ten seconds. A read that misses its budget fails the operation; it is
not retried. Unexpected pagination still attempts bounded cursor cancellation.

Cooperative cancellation is checked before and after fresh reads and before the
next export, download or completion operation. A property RPC is awaited through
its bounded request/cursor cleanup, rather than dropped mid-cleanup. Cancellation
latency therefore includes that in-flight request and any separate cleanup budget.
Initial discovery and pre-acquisition requests retain the existing session limits.
This is not a guarantee of 100 ms cancellation for SOAP requests.

Identity failure before acquisition performs session cleanup without an export.
After acquisition it follows the existing live lease abort and session cleanup
path; partial local output is discarded. A probe acquires and aborts without data
or output. Read-only inspection remains available within the explicit powered-off
scope; the older inspection mode retains its broader power-state behavior.

## Boundaries and continuation

These are observations, not an atomic remote VM lock. Concurrent changes between
reads remain possible. This step supplies explicit selection to the proof and
establishes the checks that the durable production workflow must preserve.

The current artifact writer and legacy manifest remain in use. This path does
not wire the R6.1b journal, establish restart recovery, create R6 artifact metadata,
validate the entire VMDK structure, or run local conversion. The proof's existing
live cleanup semantics also remain: a completion RPC error can lead to an abort
attempt. Production integration must journal intent and preserve uncertain remote
outcomes instead of treating this proof as a recovery protocol. Published proof
artifacts are not automatically promoted to durable jobs.

Next is R6.1c.2: join actual lease handling and owned resources to durable intents,
await filesystem work without blocking heartbeats, and qualify cancellation,
dropped responses, process loss and writer lifetime. Verified artifact metadata,
confined conversion and actual journal-bound publication then complete R6.1c.

[ADR-0059](adr/0059-explicit-export-selection.md),
[tests, matched timing and plots](benchmark-results/2026-10-02-r61c1/README.md).
