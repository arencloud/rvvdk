# ADR-0071: Expose local recovery through explicit, awaited commands

Status: accepted in **R6.1d.4a**, implemented and locally qualified in **R6.1d.4b**,
2026-10-14. Original design reviewed against `f764b4d`; implementation reference
`852b0e8`. [Evidence and limits](../benchmark-results/2026-10-14-r61d4b/README.md) retain qualification. The original
design step changed documentation only.

## Context

Local assessment, intent reconciliation, Published attestation and exact Published
transaction recovery now have separate library contracts. The public CLI has local
disk commands and generic cancellation; the pipeline example has a private network
config and an owned vSphere cancellation token. Reusing its network dispatch or
selecting an operation from observed state would hide authority and failure boundaries.
Visible Published state is not evidence that the requested operation succeeded.

## Decision

Add `rvddk recover assess|reconcile-intent|attest-published|acknowledge-published`
with exactly one private config path. Each verb maps to its explicit library API;
assessment calls the source and publication observational APIs and never dispatches
effects. Scripts use this command group; no parallel recovery implementation is added
to the pipeline example. Do not add networking, cleanup, retry or generic repair.

Use a separate strict, bounded version-1 config with original source selection,
endpoint pin **and provenance**, artifact/output IDs, absolute store/destination paths
and a bounded timeout. No credentials or identity overrides. Reconstruct bindings
through existing constructors without contacting VMware. Existing pipeline configs
require explicit private projection; their implicit TrustOnFirstUse must be preserved.

Open and lock the original parents once, in store/destination order. Dispatch one
synchronous operation, preserve its complete report and release capabilities before
rendering. Assessment reports observations without content/durability authority.
Effectful success is exactly the selected report's `is_success()` predicate.

Bridge the CLI cancellation predicate to the owned library token with a joined,
scoped sampling relay while filesystem work runs synchronously. Keep one command
deadline and pass remaining budget at dispatch. Extend first-signal handling to all
recovery verbs, retain paired-sync deferral, and never abandon accepted work. Async
callers must await the whole blocking invocation. No new Tokio runtime is needed.

Preserve distinct primary and assessment errors and partial flags in a sanitized,
command-specific envelope. Use exit 0 for completed assessment/operation, 1 for
operational/output failure, 2 for invalid requests, 3 for explicit unknown outcome,
and 130/143 for pure cancellation. Error precedence and delivery failures are defined
in the contract. No failure exit implies untouched resources; no assessment exit
proves a durable publication. Do not replay on missing or failed report delivery.

Use fixed error messages and allowlisted fields, including for parsing and panic
handling. No paths, identities, hashes, raw config or credential material in reports.
Keep existing CLI command schemas unchanged. Config admission, signal routing,
thread ownership and final flush require their own qualification.

## Alternatives and consequences

A state-driven `resume` command would merge distinct authority and success rules.
Accepting saved reports or hashes as requests would bypass fresh source binding.
Extending the pipeline example alone would leave scripts with another command/error
contract and duplicate control logic. Those alternatives are not selected.

Changing the library's cancellation type solely for this adapter would affect all
existing callers and measured paths. A small joined relay preserves those APIs but
adds startup/thread and sampling costs that must be measured. It is not a hard
cancellation timer. The CLI dependency can also affect binary size/linking/startup;
retain frozen existing-command comparisons before implementation.

Caller-controlled private directories, trusted ancestors and cooperating writers
remain required. No journal format, success gate, durability order or cleanup policy
changes. Larger working sets, physical power loss, remote lease recovery and general
production integration remain separate. PERF.0 stays open.

[Command/config/report contract](../local-recovery-cli.md),
[implementation and qualification matrix](../local-recovery-cli-qualification.md),
[awaited pipeline precedent](0066-awaited-export-pipeline.md).
