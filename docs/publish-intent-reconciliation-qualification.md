# PublishIntent reconciliation qualification plan (R6.1d.1b)

This is the implementation acceptance plan established by the
[R6.1d.1a design](publish-intent-reconciliation.md). The
[R6.1d.1b report](benchmark-results/2026-10-14-r61d1b/README.md) maps it to executed
checks, measurements and remaining limits. Existing regression anchors were reviewed
at `b717207`. The sections below preserve the required procedure; they are not raw
test results.

## Implementation sequence

1. Freeze the pre-change release test executable and source/fixture manifest. Keep
   its existing normal-publication benchmark available for a matched regression
   comparison. Record toolchain, build flags, CPU features and filesystem.
2. Extract checked bundle validation into a private helper without broadening
   `VerifiedOutput::open`. Reuse existing normal publication tests before adding
   reconciliation. Preserve all source, metadata, length, byte, digest and identity
   checks, including post-read checks.
3. Add the consuming `JobStore::reconcile_publication` operation and separate report.
   Admission must retain enough borrowed access to the store for final assessment
   even if source admission fails. Do not reopen a replaceable pathname to assess.
4. Implement both branches and the common sync/acknowledgment sequence. Add private
   test hooks before/after meaningful I/O boundaries; hooks must not be production
   options or weaken the unconditional persistence sequence.
5. Qualify failures/process loss and independent bytes, then run release benchmarks
   without overlapping builds, tests or plot generation. Generate plots from all
   retained samples; document adverse results and any unresolved variation.
6. Update current API documentation, architecture, roadmap and implementation log.
   Commit the completed bounded implementation/qualification package locally.

## Existing regression anchors

The existing [publication tests](../crates/rvvdk-vsphere/src/ownership/retained/output/publication/tests.rs)
exercise these foundations. Keep them passing; they do not yet test reconciliation.

| Existing test | Behavior to preserve / extend |
|---|---|
| `v1_admission_atomic_bundle_publication_and_explicit_cleanup` | Version-1 conversion compatibility, normal publication, source journal preservation, explicit cleanup. |
| `fresh_admission_rejects_forged_claims_metadata_changes_and_unknown_members` | Full content checks despite matching forged claims; no symlink/hardlink/unknown-entry adoption. |
| `publication_journal_faults_never_turn_uncertainty_into_success` | Five commit boundaries for both intent and acknowledgment; visible new state is not success. |
| `publication_boundaries_cancellation_and_namespace_substitution_are_conservative` | Late collisions/substitutions, cancellation and parent-sync ordering. |
| `destination_constraints_and_binding_are_explicit` | Original destination, privacy, containment and filesystem constraints. |
| `process_loss_observes_rename_without_replay_and_cleanup_can_finish_absent_directory` | Real SIGKILL around rename/sync; existing assessment never replays publication. |
| Cleanup tests in the same module | PublishIntent remains ineligible for deletion; only separately requested checked cleanup can remove eligible resources. |

## New correctness and exclusion matrix

Use local authored streamOptimized fixtures and the independently generated expected
RAW bytes already used by the publication tests. Include stored/deflated present
grains, holes, data at boundaries and trailing logical zeros. Full RAW equality must
be checked independently of the native decoder used inside reconciliation. Include
a dense fixture for corruption checks; sparse allocation requires a sparse fixture.

| ID | Case | Required assertion |
|---|---|---|
| R01 | Staged PublishIntent produced by interrupting real normal publication | Success, exactly one rename, original bundle/member inodes at final name, sequence 8, both locks held until return. |
| R02 | Already-renamed PublishIntent from interrupted normal publication | Success, no rename syscall, repeated member/bundle/intent/parent syncs, durable acknowledgment. |
| R03 | All nine stage/final namespace combinations | Exactly the two eligible pairs can progress; every other pair preserves resources and journal. Test the pure decision table as well as realizable filesystem fixtures; an Owned/Owned pair may require an alias and must still be rejected. |
| R04 | Every other OutputState, unknown version/fields, invalid sequence/stamp/checksum | Reject with no publication/repair; normal Verified publication and existing cleanup remain separate. |
| R05 | Either source or output transaction, including truncated/full/foreign/symlink entries | Reject without opening as a recovery instruction, promoting, deleting or replaying it; retain transaction bytes/entry. |
| R06 | Wrong artifact/output/source/store/destination; replaced journal with identical bytes | Reject binding or inode mismatch. Recreating the same path never supplies the original identity. |
| R07 | Source cleaned, missing, changed, malformed native stream or incomplete metadata | Reject before rename/acknowledgment. A matching RAW digest alone is insufficient. |
| R08 | RAW corruption in data and zero ranges, wrong length, metadata/digest changes | Reject; include jointly forged RAW hash claims to prove source comparison remains authoritative. |
| R09 | Foreign/missing member, wrong owner marker, unknown entry, symlink, hardlink, permissions/type change | Reject without deleting or adopting any entry; test both eligible starting locations. |
| R10 | Non-private, nested/store, wrong-device or replaced destination | Reject. For equal-device unsupported rename, retain an uncertain outcome without copy fallback. Record unavailable filesystem cases as unqualified. |
| R11 | Stage/final/journal/source substitution after admission and before acknowledgment | Fail on the relevant fresh checks; no branch switching. Foreign entries survive. |
| R12 | Collision immediately before staged rename: file, directory and dangling symlink | No overwrite; original staged bundle and foreign final entry preserved. |
| R13 | Replace store or destination pathname while its descriptor remains pinned | No redirection to replacement directory. A later reopen must validate the original recorded identities. |
| R14 | Pre-cancellation, invalid/zero/over-limit timeout, deterministic mid-call deadline | No mutation before admission; one total deadline, no per-phase budget reset. |
| R15 | Source and output cleanup requested separately | Source unchanged through reconciliation; PublishIntent cleanup still refused; successful Published cleanup retains the terminal journal. |
| R16 | Serialization and Debug output | No paths, IDs, fingerprints, digests or fixture bytes; success requires all report flags and the correct origin/rename combination. |
| R17 | Final assessment error or control error after durable Published commit | Retain completed observations, report failure, never infer success from state alone. Subsequent Published-at-entry reconciliation is rejected. |

For R01/R02, verify full output bytes, RAW length, metadata, sparse allocated bytes,
unchanged encoded source bytes and exact source journal bytes. Verify unchanged RAW
and member identities, no second RAW, no additional staging directory and no leftover
output transaction. Check reacquisition fails while locks are held and succeeds
after return. No live credentials or real VM identities belong in fixtures/logs.

## Fault, cancellation and process-loss matrix

Test both Staged and AlreadyRenamed branches unless a boundary is branch-specific.
Record whether a test injects a hook failure, simulates an actual I/O return, or kills
the process; these are different evidence. Hook errors around successful syncs do
not prove behavior under a filesystem/controller sync failure. Use a test-only I/O
seam where necessary to exercise real error propagation without modifying production
control flow or issuing dangerous filesystem actions.

| Boundary | Fault / cancellation expectation | After child SIGKILL and reopen |
|---|---|---|
| Source/native admission and logical verification | Partial byte count allowed; no rename or ack; both locks released on return. | Original intent/location remains; fresh full admission required. |
| Each member and bundle sync | Failure stops; control is checked before progressing to rename. | No namespace change; no skipped sync on retry. |
| Existing intent journal file sync and store sync | Failure stops before staged rename; never rewrite intent. | Recheck exact intent and redo both syncs. |
| Just before staged rename | Cancellation/deadline blocks rename; collision/substitution stays intact. | Staged-only retry can remain eligible. |
| Immediately after staged rename | Cancellation is deferred through both parent syncs; report rename separately. | Final-only intent retry can remain eligible. |
| Before/between parent syncs on either branch | Control deferred through this phase; actual I/O failure stops without ack. | Final-only intent must repeat verification and both syncs. |
| After both parent syncs, before acknowledgment | Cancellation/deadline returns failure with sync observations; journal stays intent. | Final-only intent retry can remain eligible. |
| Published `record_partial`, `record_written`, `record_synced` | Uncertain, no in-call retry; retain transaction. | Intent plus pending transaction blocks reconciliation. |
| Published `record_renamed` before directory sync | Uncertain even if assessment observes Published. | Published-at-entry remains outside this operation. |
| Published `record_durable`, then final control/assessment | No false success on later error; durable flag reflects observed commit completion. | Do not relabel a fresh read of Published as a witnessed commit. |

Construct initial intent via the existing publication protocol, not only by editing
JSON. In particular, kill normal publication at PublishIntent `record_renamed`
**before its store sync**, then show that reconciliation explicitly syncs the journal
and store before its rename. Also cover intent after its durable commit, after bundle
rename, after each parent sync, and process loss inside reconciliation itself.

Use bounded child timeouts and always reap children. Confirm SIGKILL reached the
intended hook; a timeout kill is a failure, not passing crash evidence. Preserve
the source journal across every child and inspect namespace/transaction state before
attempting another explicit action. Do not modify records to make a failed case
eligible. Tests qualify process loss, not physical power interruption.

## Performance campaign

Keep the [R6.1c.5p baseline and profiling results](benchmark-results/2026-10-07-r61c5p/README.md)
unchanged. They prioritize hashing and retain variance; they are not matched recovery
timings. The design-only commit generated no timings. The implementation report now retains
its own frozen-reference comparison and recovery measurements.

Measure two distinct comparisons:

| Campaign | Timed operation | Interpretation |
|---|---|---|
| Frozen reference vs candidate normal publication | Identical existing matched-publication benchmark, same fixtures, flags and phase boundaries in both executables | Detect regressions from shared-validation refactoring. Confirm semantic work/counters match before calculating deltas. |
| Candidate recovery branches | Fresh source admission through final assessment for Staged and AlreadyRenamed intents, with the same authored logical contents | Measure new capability costs. Branch differences are not an implementation speedup over normal publication. |

Use one persistent local filesystem and tmpfs as a separate RAM timing control.
Record their actual types, mount options, available space and support limitations;
tmpfs does not qualify persistence. Keep one workload active, a fixed recorded CPU
affinity and comparable CPU feature/backend settings. Record governor, available
frequency observations, SMT/shared-host conditions, kernel, compiler, binary and
source/fixture hashes. Do not assume frequency or caches are controlled merely
because affinity is fixed. Never publish private live disk/fixture hashes.

For recovery timing, use a stored 64 MiB logical fixture and a deflated 256 MiB
logical fixture with 8 MiB present data each. Report encoding and logical size
separately; these two fixtures do not isolate either factor's causal effect. Prepare
Verified output and induce the selected interruption outside timing; reopen both
locks before the timer. Time only the new synchronous operation through capability
release. Independent RAW/source/journal checks run afterward, followed by separately
timed explicit output cleanup. Source fixture setup/teardown and crash process
launch are excluded. Record lifetime RSS honestly, including earlier setup.

Required measurements are whole wall/process CPU, peak RSS, source/native admission,
full logical verification, bundle syncs, intent syncs, rename (staged only), parent
syncs, acknowledgment/final assessment, cleanup wall/CPU and physical allocation.
Record logical compared bytes, source traversal counts and actual journal commit/
sync counts. Bound memory by the existing native limits and fixed verification
buffers. Check unchanged RAW allocation and contents, including after failure cases.
Use qualified test-only profiling for finer attribution; document its overhead and
avoid nesting disjoint regions. No production instrumentation is required.

For each filesystem/fixture comparison, collect three initial rounds with four
alternating pairs per round. For the old/new normal-publication comparison, launch
matched executable rounds in alternating order; preserve the existing benchmark's
internal mode order and compare like modes. Require three further rounds with eight
pairs if any matched wall/CPU median regression exceeds 5%, or any whole/phase/CPU/
cleanup round-median spread `100 * (max/min - 1)` exceeds 5%. Ignore no zero-valued
measurement silently: rename is not applicable for AlreadyRenamed; exclude only
that declared field from positive-duration comparisons. Retain all samples and
both groups. Repetition does not clear an unresolved adverse result.

Save raw per-operation samples, per-round results, environment, commands, binary/
source hashes, repeat decisions and machine-readable summaries. Export SVG and PNG
for whole/phase wall and CPU, memory/allocation and cleanup, with branches and
reference/candidate clearly labeled. Verify plot inputs against raw records and
record artifact hashes. State every metric's boundaries and use medians/spreads as
descriptive values, not confidence intervals. A missing performance gate remains
open; never call incomplete measurements qualified.

## Completion record

R6.1d.1b requires focused tests on the persistent filesystem and tmpfs, appropriate
workspace tests, formatting and all-target Clippy; independent bytes, fault and real
process-loss results; complete repeated benchmark artifacts and plots; a staged
privacy check; and updated documentation with the measured limits. Keep unsupported
cases and physical power-loss claims explicitly outside qualification.

The design-only step checks links, state-table coverage and correspondence with the
existing code. Its completion must not mark this implementation matrix, performance
qualification or overall production recovery complete. No lab access is required
for the design or initial local implementation package.

## Qualification environment adjustment

The initial release smoke found physical allocation growth on ambient `/tmp`,
although RAW inode, length and independent bytes matched. A standalone Python probe
reproduced allocation changes using read/hash/fsync after setup without rvddk code.
The cause is unresolved. Its failures, probe results and syscall-trace summary are
retained in the implementation report; they are not discarded or attributed to a
specific kernel mechanism.

The final RAM control uses a private tmpfs mount in an unprivileged user/mount
namespace, with `huge=never` and a 512 MiB limit. This is a different qualified
environment, not evidence that ambient `/tmp` is fixed. Strict initial/recovered/
post-oracle allocation equality remains asserted. No verification is skipped, no
RAW is rewritten or re-punched, and no system-wide setting changes. Ambient tmpfs
sparse-allocation behavior remains outside clearance. Persistent-filesystem and
volatile-control claims stay separate.
