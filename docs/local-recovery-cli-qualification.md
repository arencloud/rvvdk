# Local recovery CLI implementation and qualification (R6.1d.4b)

Status: plan established by **R6.1d.4a** against `f764b4d`, executed in **R6.1d.4b**,
2026-10-14. [Qualification and limitations](benchmark-results/2026-10-14-r61d4b/README.md) map the plan to actual evidence;
any unqualified cases are listed there. The [command contract](local-recovery-cli.md)
and [ADR-0071](adr/0071-explicit-local-recovery-commands.md) define the implemented interface.
The original design step changed documentation only.

## Implementation sequence

1. Freeze the existing release CLI and publication/reconciliation/attestation/
   acknowledgment test executables before Rust/dependency changes. Save source,
   Cargo.lock and binary hashes, toolchain/build flags, environment and test counts.
   Preserve [R6.1d.3b evidence and unresolved variance](benchmark-results/2026-10-14-r61d3b/README.md).
2. Add a Linux-target dependency on `rvvdk-vsphere` in the CLI. Keep networking out
   of dispatch; record added dependency/build/binary-size effects. Add deferred
   recover subcommands and a private strict config parser. No credentials, defaults
   for selection, automatic format detection or duplicate library record parsing.
3. Add the synchronous adapter: bounded config acquisition, constructor validation,
   fixed lock order, one selected API, single deadline, scoped cancellation relay,
   full report retention and sanitized classification. Assessment independently
   gathers source and publication observations without reusing it as admission.
4. Integrate signal routing before recovery work and retain the embeddable entry
   point. Join the relay on success, error and unwind; catch dispatch unwind without
   claiming rollback. Keep all filesystem work on the invoking thread. Extend only
   recovery diagnostics with fixed usage/panic/error text; preserve existing command
   behavior and help/version. Add explicit write/flush handling and report retention.
5. Qualify config/control/report logic through authored local fixtures, subprocess
   signal/process-loss tests, independent bytes/identity oracles and syscall audit.
   Test public executable dispatch separately from private injection seams.
6. Run the release measurement matrix below without concurrent tests/builds/plots.
   Retain raw samples, failures, repeats, controls and SVG/PNG charts. Only after
   qualification mark the command contract implemented, save remaining findings and
   next bounded work, scan staged files for private data and commit locally.

Production changes should be confined to the CLI and its dependency declaration.
If library control/report changes prove necessary, document that departure, review
the underlying contract and broaden regression/performance checks before proceeding.
Do not expose production crash hooks, skip-verification switches or profiling flags.

## Authored fixtures and success matrix

Use independent stored/deflated streamOptimized fixtures and a known RAW oracle,
covering dense data, sparse data, holes and trailing zeros. Reuse library test-only
helpers or a dedicated test fixture builder; valid states must originate through real
transitions and test-only interruption hooks. Never use edited journal bytes as a
positive recovery history. Adversarial edits are negative cases only. Keep the actual
artifact/store/destination/selection private; commit sanitized results and hashes.

| Operation | Required histories and assertions |
|---|---|
| Assess | Earlier, CompletedLease/PublishIntent, Published and terminal journals; pending source/output transactions; missing source record with valid output, and converse. Preserve both errors/observations. Compare record and payload bytes/inodes/names before/after; no sync or mutation. Assessment does not hash payload or admit a candidate. |
| Reconcile intent | Owned stage/absent final and absent stage/owned final. Native complete success predicate, expected rename/acknowledgment, independent full source/RAW and sparse allocation preservation. |
| Attest Published | Previously acknowledged and uncertain/lost acknowledgment histories. Fresh verification, eight syncs, original journal bytes/inode preserved, no rename or write. |
| Acknowledge Published | Normal publication plus both reconciliation origins, interrupted at complete-written and complete-synced candidate (six cases). Exact candidate bytes/inode become journal, ten syncs and one journal replacement, no bundle rename or recopy. |
| Repeated explicit calls | Assessment stays observational; repeated attestation repeats full work; second reconciliation/recovery rejects Published-at-entry. A separately requested attestation can qualify it. No automatic fallback. |

For each successful effectful call, compare its native report with the direct-library
operation on an equivalent fresh fixture. The CLI must not add an extra source/RAW
verification pass. Keep parent locks through operation completion and release them
before reporting. Cleanup is a separate test teardown using existing checked APIs;
the CLI under test never invokes cleanup. Failure fixtures remain available for
independent inspection until test teardown.

## Input, dispatch and exclusion matrix

| ID | Cases | Required result |
|---|---|---|
| C01 | Missing/unknown verb or config, extra positional input, unsupported flags, global --json placement, -- delimiter, help/version | Fixed safe usage and exit 2 for invalid syntax; no config/store access. Help/version retain text/zero and no I/O. Never interpolate a private argument in diagnostics. |
| C02 | Missing/unknown/duplicate fields at every object level, wrong version/type, null optional timeout, overflow, trailing JSON/garbage, malformed UTF-8, empty input | Rejection with fixed error, bounded allocation, no directory opening. Include duplicate equal-valued keys. |
| C03 | Exact 16384-byte file, 16385 bytes, metadata/read size mismatch, growing/shrinking file, oversized strings and depth | Correct byte limit including whitespace, bounded read, post-read rechecks; constructors enforce endpoint/selection bounds. Depth remains bounded by the JSON parser. |
| C04 | Symlink/dangling link, directory, FIFO, socket/device, hardlink, wrong owner, group/other mode, unreadable config | Reject before parsing without blocking on FIFO. Run available real file-type/owner cases; record unavailable privileged cases explicitly. Trusted ancestors remain an assumption. |
| C05 | Invalid/zero/short/long IDs; invalid pin, provenance, endpoint, source field; relative/empty/NUL parent paths; timeout 0/21601/overflow | No inferred identity/default, no network, no parent mutation. Case normalization must agree with library constructors. |
| C06 | Copied pipeline config with user/export fields; omitted or changed provenance | Raw old config rejects. Explicit private projection of original TOFU selection works; changed provenance fails original source binding. Test externally_verified separately. |
| C07 | Wrong source/artifact/output/original store/destination, cross-filesystem or nested destination, replaced parents, same directory | Preserve library rejection and descriptor confinement; no fallback canonicalization, search, parent creation or reopening after work starts. |
| C08 | Busy store; busy destination after store opens; nonprivate or missing parents | Nonblocking failure, no retry; first lock released if second open fails. Opposite-directory concurrent requests must not wait in a lock cycle. |
| C09 | All output states, partial/foreign/orphan/equal transaction, source transaction, cleaned/missing source, unknown bundle members, collisions | Selected API only; existing exclusion tables preserved, no unstamped adoption, transaction deletion or cleanup. |
| C10 | Data/hole/zero corruption, wrong RAW length, forged matching RAW metadata digest, encoded source corruption | Native fresh full verification detects corruption; no success from journal state or metadata alone. |
| C11 | No configured credentials and disabled networking | All four verbs work on authored local fixtures without DNS/socket connection, credential prompt or remote API call. Audit actual executable. |

## Control, reporting and process-loss matrix

| ID | Case | Required assertions |
|---|---|---|
| L01 | Pre-cancel before config; cancellation after parse/first lock; timeout exhausted before dispatch | No selected effectful API call. Keep partial assessment observations if already acquired; release capabilities and join relay. |
| L02 | Pure library Cancelled, Deadline, Busy/content/I/O failures; simultaneous non-cancellation/assessment errors | Exact status/exit precedence from the contract; preserve all native errors/flags. Deadline is exit 1, not 130. |
| L03 | Every report success flag removed, wrong final state/transaction/namespace, primary or assessment error, missing recovery, inconsistent no-error report | Never classify as completed. Uncertain/missing assessment/inconsistent report takes exit 3; valid success requires native is_success. |
| L04 | Mutating report has visible Published after failure; late signal after complete success | Keep original failure in the first case; retain success in the second. Never issue a second operation from result classification. |
| L05 | Real SIGINT/SIGTERM during long verification, paired sync boundaries and post-rename sync | Cooperative request, awaited synchronous completion, first-signal exit mapping only for pure cancellation. Repeated signals do not destroy resources; both locks stay held until library return. |
| L06 | Relay thread creation failure, stop-channel disconnect, immediate completion, dispatch unwind and relay unwind | No leaked thread or scope-join deadlock. Dispatch unwind is unknown outcome; relay failure cannot be silently reported as success. Use test-only injection, never a production crash flag. |
| L07 | Source assessment failure and successful output assessment; inverse; cancellation/deadline between calls | Preserve independent observations and errors; no dispatch or freshness/durability inference. No post-cancellation second read required. |
| L08 | Config/read/parse/lock/native errors containing private sentinels; malicious command values, serde diagnostics, panic payload | Sentinels absent from stdout, stderr, Debug and committed logs. CLI recovery panic hook uses fixed text; embedded-hook limitations documented. |
| L09 | Output writer fails at each document stage, newline and flush; stderr also fails | Nonzero exit, no duplicate invocation/cleanup; retain operation result in best-effort output error. Success is not returned on buffered flush failure. |
| L10 | Library report error followed by output failure; BrokenPipe after successful mutation | Exit 1 for delivery, preserved original outcome and native flags. No claim that missing/partial JSON means no effects. |
| L11 | Actual executable help/usage and all four valid verbs, both output modes; existing inspect/plan/copy/verify regressions | Library embedding and binary routing both covered. Existing schemas/signals/output behavior unchanged outside recovery. |

The relay samples before entry and at a target 10 ms interval with an interruptible
stop wait. Measure observed cancellation latency as diagnostics, not a hard deadline;
host scheduling and in-flight blocking calls can dominate. Test that completed work
wakes the relay promptly instead of paying an unconditional sampling sleep. A panic
seam should not emit secrets via the default hook before classification.

Use bounded, reaped child processes for process loss. Seed real intent and candidate
histories at the existing library interruption boundaries, then run the executable
as a separate invocation. Kill recovery at pre-dispatch, verification, rename and
post-rename/pre-report boundaries using test-only wrappers or marker barriers. A
timeout that never reached its intended marker is a test failure, not a passing kill.
Inspect surviving journal bytes/inodes, namespaces and independent RAW after each
kill; use only a subsequent explicitly selected operation. Before promotion the
matched pair remains eligible; after promotion use separate attestation. SIGKILL is
not a simulation of power loss or storage faults. Preserve existing library fault
and crash coverage; do not duplicate every low-level case solely through the CLI.

## Syscall and work audit

Trace actual release commands on authored fixtures with raw paths/traces retained
privately. Separate process startup/config/locks, selected operation and report
delivery, using test-only equivalent entry points if markers are required. Report
both actual-binary whole traces and any instrumented interval provenance.

Require zero network activity and no credential input. Assessment has zero payload
scan, fsync, write/create/rename/unlink of artifact resources. Effectful operations
preserve library sync/rename/write counts and full encoded/native/RAW verification;
report writes and relay synchronization are distinct from artifact writes. Verify
attestation's eight syncs and acknowledgment recovery's ten syncs/one same-store
promotion. Allow documented libc flags-zero renameat2-to-renameat translation, while
checking names, descriptors and actual effects. Retain reconciliation's two existing
branch-specific protocols, with no extra transaction or payload pass from the CLI.

## Performance acceptance

Freeze the reference CLI before adding the vSphere dependency. Freeze reference
library test executables too, even if their source will be unchanged. Do not present
a new CLI operation against an old unrelated command as an optimization comparison.

| Measurement | Comparison and purpose |
|---|---|
| Existing root help, inspect, plan and representative RAW/VMDK copy/verify | Frozen reference/candidate command pairing, same request/output mode and fixture. Detect command-builder, linkage, startup or dispatch regressions. |
| Assessment | New baseline only: normal and pending-transaction observations, human/JSON, separate startup/config/lock/report costs from bounded journal reads. |
| Reconciliation | Both branches, direct native API versus command on equivalent fresh fixtures; compare whole process and selected operation separately. |
| Attestation | Both originating acknowledgment histories, direct versus command; preserve fresh full verification and no journal mutation. |
| Acknowledgment recovery | Complete-written and complete-synced candidates; timed normal-origin fixtures, correctness/work coverage of all six origins. Add timed origins if work differs. |
| Cancellation relay | Idle/active sampling, immediate completion wakeup, disabled versus enabled test-only measurement controls. Never disable production cancellation or verification to claim a speedup. |
| Resource/delivery | End-to-end wall, process CPU (including relay), lifetime peak RSS, binary size, sparse allocated bytes and output mode/bytes. Record build cost separately; independent oracle and cleanup outside timing. |

Use 64 MiB stored and 256 MiB deflated logical fixtures with 8 MiB data, plus dense
correctness fixtures. Use recorded persistent Btrfs and isolated private tmpfs with
matching limits/mount options. Fix and record CPU affinity, compiler/flags, cache
policy, filesystem/mount, CPU features, SMT/sibling load, governor/frequency and
competing work. A new process is not necessarily a cold page-cache sample; do not
claim cold measurements unless actually controlled. No physical persistence claim
from tmpfs. Larger/colder working sets remain separate qualification.

Run one workload at a time; no builds/tests/audits/plot rendering concurrently.
Alternate binary and equivalent-fixture order. Begin with three rounds of at least
four pairs; require three longer rounds of at least eight pairs for >5% spread,
any positive matched wall/CPU delta or zero-resolution short phases. Positive delta
is a repeat trigger, not statistical significance. Keep all adverse and failed
observations; do not replace runs until they look stable. Record bounded repeat
decisions and unresolved variation. Do not close prior PERF.0 variance because one
new control is stable.

Whole process time includes startup, parsing, config, relay, locks, operation, report
flush and exit. Selected-operation time excludes setup/independent oracle/cleanup.
Use disjoint test-only spans for adapter stages; clocks/counters stay absent from
production library loops. Keep raw zero-duration CPU phases and flag unresolved
relative spreads; whole valid workload times must be positive. Separate process
peak RSS (includes startup) from any controlled memory accounting. RAW allocation
must match initial/post-operation/post-independent-oracle checkpoints.

Keep normal publication, both reconciliation branches, both attestation histories
and exact-pair library baselines. If only CLI changes, justify a bounded regression
comparison rather than repeating an unrelated entire campaign. Any library change
requires its affected frozen matrix and fault coverage. No causal speedup claim from
different histories, output modes, cache state, CPU features or skipped full checks.

Retain machine-readable samples/summary/repeat decisions, source/binary/fixture
hashes, commands/environment, independent validation and syscall audit. Plot existing
CLI deltas, direct-library/command timing with startup separated, CPU, peak RSS,
allocation, cancellation observations and repeated spreads as PNG and SVG. Validate
plot inputs and inspect every chart. Record limitations and unavailable test cases.

## Completion and continuation

R6.1d.4b is complete only when all four commands are implemented, local correctness,
control/report/privacy checks pass, measured evidence is saved, and docs distinguish
implemented behavior from remaining limits. Run formatting, workspace tests and
all-target Clippy; qualify actual release binaries in addition to unit seams.

This is local work: no ESXi, license, VM, vCenter, SSH or credentials are required.
Live retained-artifact integration is a later explicit qualification package. Keep
source transaction repair, unstamped-stage adoption, cleanup expansion, remote replay,
general production recovery and PERF.0 open. Record the next bounded step based on
the implementation evidence, then commit the completed package locally.
