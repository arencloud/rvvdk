# Composed export pipeline (R6.1c.5c)

Linux `run_export_pipeline` composes the existing owned export, retained admission,
RAW conversion and no-replace publication capabilities. Success retains both the
source artifact and published RAW bundle. Cleanup is a separate explicit action.
This is a bounded qualification entry point, not general production recovery.

## Lifetime and failure contract

The caller supplies an explicit `SourceSelection`, runtime credentials, open private
`JobStore` and `PublicationDirectory`, nonzero artifact/output IDs and bounded options.
Preflight rejects an existing output record/final bundle and invalid destination
before contacting the host. The destination lock remains held across all phases.
A descriptor anchor pins the original store inode; subsequent local phases reacquire
its advisory lock through that descriptor rather than resolving its pathname again.
Path replacement cannot redirect conversion into a foreign store. Other cooperating
users can acquire the store between phases; each admission rechecks durable state.

```mermaid
flowchart LR
    A[Destination / ID preflight] --> B[Owned export and artifact validation]
    B --> C[Complete acknowledgment / Logout / worker drain]
    C --> D[Fresh retained admission / owned RAW conversion]
    D --> E[Fresh output admission]
    E --> F[Revalidate / publish bundle / durable acknowledgment]
    F --> G[Retain source and output]
    G -. separate explicit action .-> H[Checked output cleanup then source cleanup]
```

Export must pass every artifact-success gate before local work starts. An uncertain
completion remains `CompleteIntent`: no conversion, remote replay or automatic abort.
Local work runs in one awaited blocking task. Conversion admission, conversion,
output admission and publication share one absolute local deadline after export.
The export cancellation token is shared by local operations. Ctrl-C in the runner
requests cooperative cancellation and then waits for the report and worker drain.
Dropping the library future or killing the process cannot guarantee remote cleanup
or stop accepted blocking work; callers must await it. A local worker panic produces
`LocalOutcomeUnknown`, retains the transfer report and grants no cleanup authority.

Reports retain nested transfer/conversion/publication errors and boundaries, plus
outer admission errors and phase timings. Success requires all three nested success
predicates and `Completed`. Failure never automatically deletes, rolls back or repairs
resources. Partial/uncertain states remain subject to each underlying capability's
rules. The library does not power VMs, infer source selection or retry remote requests.

## Internal Rust runner

Build with `cargo build --release -p rvvdk-vsphere --example pipeline` and use
`target/release/examples/pipeline COMMAND PRIVATE_CONFIG.json`. The input is a
current-user regular file, mode 0600, one link, at most 16 KiB; symlinks and unknown
fields are rejected. Keep its directory and every report private (0700 / 0600).
Network commands prompt for a password through the terminal, without echo. No
password field, environment variable or command-line password is supported.

Commands: `inventory`, `inspect`, `run`, `cleanup`, `cleanup-aborted`.
Inventory includes private selection identities; never commit its output. Inspect
performs fresh selected-source checks without acquiring a lease. Run requires
existing private store/destination directories, explicit selection and IDs.

Config fields:

| Field | Meaning |
|---|---|
| `endpoint`, `certificate_sha256`, `user` | Explicit host and exact trust pin; private |
| `selection` | `vm_reference`, `bios_uuid`, `disk_key`, `backing`, `logical_bytes` |
| `store`, `destination` | Private directories on one filesystem, destination outside store |
| `artifact_id`, `output_id` | Caller-selected nonzero 32-hex identifiers |
| `max_encoded_bytes` | Optional encoded export bound |
| `timeout_seconds` | Optional export budget and separate shared local budget |

`cleanup` checks eligible source state, then calls checked output cleanup followed
by source cleanup. If output cleanup fails, source cleanup does not run. It retains
both terminal journals. `cleanup-aborted` requires an acknowledged `AbortedLease`
(or already `Cleaned`) and no pending transaction, and requests source cleanup only.
Neither command authenticates or contacts VMware. The aborted helper does not handle
interrupted `CleanupIntent`; use the separately designed recovery API/qualification.

The runner reports process CPU and lifetime peak RSS. Its outer wall clock includes
password entry; benchmark **`data.elapsed_ms`** for the pipeline, with `export_ms`,
`conversion_ms`, `publication_admission_ms` and `publication_ms` for phases. Export
includes network, journal, manifest, container readback and native grain validation;
it is not a pure socket-throughput measurement. Conversion includes retained
admission and owned RAW readback. Publication includes its full revalidation and
sync barriers. Independent QEMU/guest comparisons and explicit cleanup follow timing.

## Qualification and limits

Five composed synthetic tests cover success and full authored RAW bytes, separate
cleanup/tombstones, initial and late collision, precancel and post-export cancellation,
local budget/deadline failure, uncertain completion and store-path substitution.
Existing publication crash/fault tests remain applicable to the composed capability.
[Live evidence and plots](benchmark-results/2026-10-02-r61c5c/README.md) record the
actual scope and conditions. QEMU provides independent full logical comparison;
the previously guest-mapped fixture separately verifies known guest bytes.

Pending journal transactions, unstamped-stage adoption and uncertain-publication
reconciliation remain blocked. No remote resume/cleanup authority is inferred from
journals. No online snapshot/CBT/restore, vCenter or vSphere 9 qualification is added.
Repeated full source/RAW checks remain deliberate costs tracked in PERF.0.

[ADR-0066](adr/0066-awaited-export-pipeline.md),
[owned transfer](durable-owned-transfer.md),
[publication/cleanup](durable-output-publication.md).
