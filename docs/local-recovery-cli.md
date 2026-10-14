# Explicit local recovery commands (R6.1d.4)

Status: design accepted in **R6.1d.4a**, 2026-10-14, against `f764b4d`.
**Not implemented.** R6.1d.4b will implement and locally qualify these commands.
Existing commands, library admission rules and journal formats are unchanged.

[ADR-0071](adr/0071-explicit-local-recovery-commands.md) ·
[Implementation and qualification](local-recovery-cli-qualification.md)

## Command contract

Add a Linux `recover` group to the existing `rvddk` binary. Each invocation selects
exactly one operation and one output. Proposed syntax, not currently executable:

```text
rvddk recover assess PRIVATE_CONFIG.json [--json]
rvddk recover reconcile-intent PRIVATE_CONFIG.json [--json]
rvddk recover attest-published PRIVATE_CONFIG.json [--json]
rvddk recover acknowledge-published PRIVATE_CONFIG.json [--json]
```

All four verbs require the same explicit original selection. No discovery, credential
prompt, connection, export, VM action, cleanup, automatic retry, transaction editing
or state-selected dispatch is part of these commands. No report can be supplied as
an executable request. `--force`, `--overwrite`, `--progress`, backend/worker/copy
options and endpoint/identity overrides are rejected. Help/version do not open input.
The existing root `--json` remains usable before or after the subcommands.

| Verb | Exact library operation | Meaning of a successful invocation |
|---|---|---|
| `assess` | `JobStore::recover`, then `JobStore::assess_publication` | Both source journal and output namespace observations were obtained; no content or durability claim. |
| `reconcile-intent` | `JobStore::reconcile_publication` | Fresh verification and durable completion of eligible PublishIntent; either owned-stage/absent-final or absent-stage/owned-final, with no pending output transaction. |
| `attest-published` | `JobStore::attest_publication` | Fresh full verification and persistence observations for Published-at-entry, with no transaction or journal rewrite. It does not prove earlier success. |
| `acknowledge-published` | `JobStore::recover_publication_acknowledgment` | Fresh full verification and promotion of the exact intent-7/Published-8 pair, preserving the candidate's bytes/inode. |

The three effectful operations consume the original store/destination capabilities.
Their own `is_success()` predicates are authoritative. The adapter must not infer
success from bytes verified, a visible Published record, an assessment, or exit of a
worker. Attestation performs syncs even though it writes no journal or bundle bytes.

### Selection by the operator

This table explains which explicit operation may apply; it is not a dispatcher.
Each operation must repeat all its own admission and full verification work.

| Observed state | Separately requested operation that may qualify |
|---|---|
| PublishIntent, no output transaction; one eligible owned namespace | `reconcile-intent` |
| Published, no output transaction; absent stage/owned final | `attest-published` |
| PublishIntent with a transaction; absent stage/owned final | `acknowledge-published` only if it independently admits the exact complete successor. Assessment cannot establish that match. |
| Partial, foreign, orphan or duplicate transaction; ambiguous namespace | No command in this group repairs it. |
| Incomplete, cleaned or unavailable retained source; source transaction | Effectful recovery remains blocked even with an apparently intact output. |

`assess` is observational, including for earlier/terminal states or surviving
transactions. It reads source and output independently under both locks: if source
assessment fails, still attempt output assessment unless control has stopped the
request, and preserve both results/errors. It must not require CompletedLease merely
to observe a journal. It neither decodes candidate bytes nor scans payload. A pending
transaction reports existence only. A successful assessment can report foreign
locations or a remote-unknown source state; exit zero means assessment completed.

## Private configuration version 1

Use a dedicated DTO with `deny_unknown_fields` on every object and required
`schema_version: 1`. Parse directly into typed structs so duplicate known fields,
unknown fields, trailing values and invalid types reject. Do not deserialize to a
generic map first. The old pipeline config is not accepted unchanged.

| Field | Rule |
|---|---|
| `schema_version` | Required integer 1; reject all other values. |
| `endpoint`, `certificate_sha256` | Original endpoint and 64-hex certificate pin; build `EndpointIdentity::pinned` locally. No TLS connection or fresh trust claim. |
| `pin_provenance` | Required `trust_on_first_use` or `externally_verified`; it participates in the source binding and must match the original request. |
| `selection` | Required `vm_reference`, `bios_uuid`, `disk_key`, `backing`, `logical_bytes`; use `SourceSelection::new` and its existing limits. |
| `store`, `destination` | Required absolute UTF-8 directory paths, nonempty, without NUL; original private parents, not journal or bundle paths. No expansion, creation, canonicalization or fallback search. |
| `artifact_id`, `output_id` | Required nonzero 32-hex strings, case-insensitive; construct the existing checked ID types. Never generate missing IDs. |
| `timeout_seconds` | Optional integer 1..21600, default 3600; reject null, zero, overflow and out-of-range values. |

All other fields, including `user`, `password`, export options, operation selectors,
and expected-state overrides, reject. The operation is chosen only on the command
line. A `PathBuf` command argument permits a non-UTF-8 config filename; paths within
the JSON are UTF-8 only in version 1. Existing native bindings are reconstructed,
not copied from metadata hashes. Do not substitute current inventory or a new pin.

The existing pipeline example constructs `TrustOnFirstUse` unconditionally. To
prepare a recovery config for its retained artifacts, privately copy the original
selection, paths and IDs; add schema version and that provenance; remove `user` and
export-only options. No automatic migration or reading of old credential material.
For other library callers, use their original provenance. Guessing provenance can
change the binding and must fail admission.

Open the config read-only with `O_NOFOLLOW | O_NONBLOCK | O_CLOEXEC`. Require a
current-effective-user regular file, exactly one link, no group/other permissions,
and metadata length at most 16384 bytes. Read at most 16385 bytes from that same
descriptor and reject excess even if the file grew. Empty/malformed input rejects;
FIFO/device/directory/symlink/hardlink and wrong-owner/mode cases reject before JSON
parsing. Recheck descriptor identity, type, owner, mode, links and length after the
bounded read; require read length to match metadata length. This is not a snapshot
against a hostile same-user writer. Config and directory ancestors must be trusted
and stable. Never log input bytes, serde errors, path values or config Debug output.

## Capability lifetime and control

1. Parse the verb/options without filesystem effects. Start the default command
   deadline before config acquisition; once an explicit timeout is parsed, derive
   its absolute deadline from that same start. Config reading cannot be interrupted
   by a timeout learned later. Check cancellation before opening input and after it.
2. Validate all config fields and construct the original source/IDs before opening
   either directory. Open `JobStore`, then `PublicationDirectory`, once each.
   Nonblocking advisory lock contention returns Busy; do not wait, poll locks, retry
   or release/reacquire them between assessment and execution. Opening creates no
   lock file or directory. A failed second open drops the first capability.
3. For assessment, call the two observational APIs with control checks before,
   between and after them. They have no internal cancellation/deadline argument;
   do not claim per-read interruption. Preserve partial observations on timeout.
4. For an effectful verb, pass the remaining positive deadline budget and the shared
   cancellation token in `RetainedOptions`, then invoke exactly its selected API.
   Do not call normal publication `preflight` (which rejects existing output),
   `VerifiedOutput::open`, or an exploratory mutating operation. Library checks
   enforce original identity, same filesystem, outside-store destination and state.
5. Await return and release capabilities before rendering. Do not reopen parents
   for a fallback assessment. Preserve the operation's own primary/assessment errors
   and partial flags. No deletion, cleanup or rollback runs on success, error or Drop.

The adapter belongs in a private `rvvdk-cli::recovery` module with config, dispatch
and rendering separated. Add the Linux-target `rvvdk-vsphere` dependency; no new
Tokio runtime is needed for the synchronous command. No recovery wire parser or
state machine is duplicated in the CLI. Scripts/runners invoke this same binary;
the existing `pipeline` example and its network/cleanup commands remain unchanged.

The CLI's generic datamover cancellation predicate and vSphere's owned cancellation
token are different types. Preserve `run_with_cancellation`: run filesystem work
synchronously on the invoking thread, with one scoped relay thread that samples the
borrowed predicate and cancels a cloned vSphere token. Sample before work starts,
then use an interruptible channel wait with a 10 ms sampling interval. Signal the
relay to stop and join it on every ordinary exit/unwind; its stop-channel sender
must drop before scope join on unwind. No busy spin, detached task or 10 ms terminal
sleep. This is a sampling target, not a scheduling or cancellation latency guarantee.
The small relay owns no artifact capability. Use fallible thread creation; failure
is a pre-dispatch operational error. A panicking cancellation predicate is outside
the existing caller contract and must never be silently ignored.

Extend binary SIGINT/SIGTERM handler selection to the parsed recovery group, including
assessment. Retain first-signal, atomic-only handlers and repeated-signal behavior.
The library entry point installs no process handlers. A late signal after a library
success must not rewrite that result; output failure can still change the exit code.
Do not place another cancellation check between final success and report rendering.
Existing library deferral across paired syncs and rename/store-sync remains intact.

Async integration must put the entire synchronous invocation in one blocking task,
signal its token and await the join; never race it against an outer timeout that
drops accepted work. No new async wrapper is required in R6.1d.4b. A caught local
unwind/join failure gives unknown outcome and no retry. Catching an unwind cannot
recover an aborted process or SIGKILL. The binary should use a fixed, non-payload
panic diagnostic for recovery; embedded callers own their panic-hook policy, since
a process-global hook cannot safely be replaced by a library. Sanitized error
conversion alone cannot redact a panic hook that has already emitted a payload.

Trusted stable ancestors and cooperating writers remain prerequisites. Descriptor
pinning prevents a replacement pathname from redirecting accepted work; it does not
grant exclusion against hostile same-user content changes. Config I/O, local syscalls,
sync, thread scheduling and output can block beyond the budget. No hard deadline,
physical-power-loss guarantee, prior writeback repair or network-filesystem support
is added by this adapter.

## Reports, exit codes and output delivery

Use a recovery-specific envelope: `schema_version: 1`, `command: "recover"`,
`operation` (the selected verb, null if syntax did not identify one), `status`, `operation_started`,
`operation_succeeded`, `assessment_only`, `data`, and `error` (null on success).
`data` is null before dispatch, otherwise the selected native report. For assessment
it contains `source`, `source_error`, `publication`, `publication_error`, with null
for unavailable values. Do not flatten away error precedence or library flags.
`operation_started` means the selected API was entered, not that mutation occurred.
`operation_succeeded` follows the selected predicate or both assessment Results plus
final control checks. `assessment_only` is true only for `assess`.

Serialize the current sanitized report fields through an explicit adapter/allowlist
so future library additions do not accidentally expose private fields. Current
source `operation` is skipped by serde and must stay absent. Never emit endpoint,
certificate, selection, paths, IDs, operation IDs, digests, journal JSON, panic
payloads, command arguments or OS/serde error text. Error codes/messages are closed
constants; native OwnershipError variants remain separate in `data`. Counts, states,
booleans, origin and sanitized errors are allowed. Human output uses the same facts.
Non-null `error` has fixed `code` and `message` fields; its code is the failure status
below. Native error variants stay in `data`, with no raw error-chain formatting.

| Priority | Status / exit | Rule |
|---|---|---|
| Output delivery | `output_failed` / 1 | Any serialization/write/flush error. Preserve the completed operation envelope in a best-effort fixed diagnostic; never rerun the operation. |
| Usage | `invalid_request` / 2 | Invalid command syntax, config structure/schema/field values. Filesystem access, private-file safety and lock failures are operational errors (1). |
| Unknown outcome | `uncertain` / 3 | Caught dispatch/relay unwind or join failure; any native Uncertain error; an effectful report with assessment_error or no recovery; or a non-success report without a primary error. These override cancellation. |
| Operational failure | `failed` / 1 | Remaining non-cancellation errors, including Busy, identity/content/record/state rejection, I/O and deadline. Preserve all reported effects; this exit does not assert that nothing changed. |
| Cancellation | `cancelled` / 130 or 143 | Pure cooperative cancellation after excluding the above conditions. Embeddable entry point returns 130; binary maps first SIGTERM to 143. |
| Completion | `assessed` or `completed` / 0 | Both assessments plus final control checks, or the selected effectful report's complete `is_success()` predicate; output flush also succeeds. |

If adapter control/join completed without error, classify native success first and
apply native failure precedence only to a failed operation. An adapter unwind/join
failure prevents overall success even if a completed native report was obtained;
retain that report and its `operation_succeeded` value in the uncertain envelope.
Assessment missing observations because of error/control are not automatically exit
3: it makes no mutation-outcome claim. A native Uncertain error still takes exit 3.
For mixed assessment errors, non-cancellation errors take precedence over Cancelled.
For effectful operations, a Cancelled primary error with an assessment failure takes
exit 3. Deadline is not a signal and never takes a cancellation exit.

These categories describe this invocation, not the historical state of every
resource. Exit 1 or 130 can follow persistence effects; exit 0 for assessment can
describe an uncertain prior operation. Report flags remain necessary to interpret
effects. Exit 3 directs explicit inspection, not automatic replay. Previously failed
invocations remain failed after a later successful attestation/recovery.

Keep the CLI convention: one success document on stdout; one failure document on
stderr containing the full available envelope, with empty stdout for operational
failure. No progress stream in this step. Help/version remain plain text. Recovery
usage errors use a fixed message and safe usage text, never clap's interpolated
argument text. Recognize recovery error routing without echoing malformed inputs;
test global-option placement and the `--` delimiter. Existing command schemas and
diagnostics keep their contracts.

Explicitly flush output before returning zero. After a failed stdout write/flush,
the best-effort stderr document retains `operation_succeeded` and the native result,
but returns 1. Stdout may be partial; stderr can also fail. No finite report can prove
its own delivery, and broken output must not become permission to repeat mutation.
Do not create a report file, rewrite the private config, or persist a new checkpoint.

## Reviewed implementation anchors

| Source at `f764b4d` | Design implication |
|---|---|
| [args](../crates/rvvdk-cli/src/args.rs), [dispatch](../crates/rvvdk-cli/src/lib.rs) | Deferred clap builders; existing generic cancellation and stdout/stderr convention. Recovery needs its own failure classification/redaction, not raw Failure::io strings. |
| [binary](../crates/rvvdk-cli/src/main.rs) | Current handlers cover only copy/verify; current positional sniff must not misclassify recover or install handlers after work starts. |
| [pipeline runner](../crates/rvvdk-vsphere/examples/pipeline.rs) | Private 16 KiB input precedent; optional network config and implicit trust provenance are unsuitable as the new schema. |
| [contract](../crates/rvvdk-vsphere/src/contract.rs) | Endpoint pin provenance is part of the original binding; constructors do no network/filesystem work. |
| [ownership](../crates/rvvdk-vsphere/src/ownership.rs), [publication](../crates/rvvdk-vsphere/src/ownership/retained/output/publication.rs) | Nonblocking directory locks; assessment is observational; normal preflight rejects existing outputs. |
| [reconciliation](../crates/rvvdk-vsphere/src/ownership/retained/output/publication/reconciliation.rs), [attestation](../crates/rvvdk-vsphere/src/ownership/retained/output/publication/attestation.rs), [acknowledgment](../crates/rvvdk-vsphere/src/ownership/retained/output/publication/acknowledgment.rs) | Consuming synchronous APIs, distinct flags/predicates and primary/assessment errors. |
| [retained control](../crates/rvvdk-vsphere/src/ownership/retained.rs), [pipeline](../crates/rvvdk-vsphere/src/pipeline.rs) | Bounded relative timeout at API entry, owned cancellation token, awaited blocking-work precedent. |

The qualification plan preserves full library baselines and separately measures
CLI startup/config/thread/rendering overhead. This documentation step runs no new
runtime benchmarks and closes no PERF.0 finding. No ESXi or credential is needed.
