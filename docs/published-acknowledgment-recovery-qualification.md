# Published acknowledgment recovery implementation and qualification (R6.1d.3b)

Status: acceptance plan saved by **R6.1d.3a**, 2026-10-14. **Not executed.**
The [contract](published-acknowledgment-recovery.md) and
[ADR-0070](adr/0070-matched-published-transaction-recovery.md) define the proposed API.
Existing anchors below were reviewed against `6c0134e`.

## Implementation sequence

1. Freeze the unchanged release test executable and source/fixture hashes before
   Rust edits. Record compiler/build flags, kernel/filesystem/CPU controls and retain
   the [R6.1d.2b baseline and unresolved limits](benchmark-results/2026-10-14-r61d2b/README.md).
2. Extract only the bounded output-envelope validation needed for reading both
   generated names. Existing read_output must preserve its checks and public behavior.
   Introduce a private admitted-pair value with pinned read-only descriptors, exact
   bounded bytes and identities, old record and exact expected successor. Its
   constructor enforces every pairing rule; no caller-supplied pathname or skip flag.
3. Adapt shared full-content validation to a private explicit binding guard: the
   existing no-transaction guard for all current callers, or a borrowed validated
   pair guard for this operation. Invoke it at every existing output-current gate.
   Keep encoded-source rereads and all native/RAW/metadata checks intact. Never
   temporarily remove/rename the transaction to get through existing validation.
   Keep the new guard private; a public Verified capability cannot be manufactured.
4. Add the consuming API/report in a private publication submodule. Borrow the store
   until retained admission succeeds; keep both locks for failure assessment. Use a
   private statically dispatched sync/promotion seam and boundary hooks. Production
   always calls the OS; no instrumentation, writable handles or verification bypass.
5. Implement the ten-sync, one-journal-rename order. Candidate sync is followed by
   full pair/source/namespace revalidation and control immediately before promotion.
   After successful rename, finish store sync before observing cancellation. Use the
   promoted candidate bytes/inode for final binding checks, not the replaced old one.
6. Run correctness/fault/process-loss tests and a measured-interval syscall audit.
   Then run non-overlapping release comparisons, counter smokes and recording controls.
   Preserve adverse observations and the full acceptance/measurement evidence.
7. Update contracts to implemented only after qualification, record limits and the
   next bounded step, scan staged artifacts for private data, and commit locally.
   This package needs no ESXi/SSH, license, VM or remote credential.

## Existing regression anchors

| Existing symbol | Behavior to preserve |
|---|---|
| `publication_journal_faults_never_turn_uncertainty_into_success` | Normal publication retains its own failed result even if a later explicit recovery succeeds. |
| `acknowledgment_faults_and_assessment_failures_never_become_success` | Reconciliation never converts an uncertain acknowledgment into success by assessment. |
| `both_branches_verify_preserve_sync_and_acknowledge_without_recopy` | Both intent branches retain their original gates and content work. |
| `acknowledgment_process_loss_keeps_transactions_blocked_and_attests_only_published` | Attestation continues rejecting every transaction; new recovery is a separate invocation. |
| `pending_transactions_are_retained_for_both_journals_and_branches` | Existing attestation source/output transaction exclusions remain unchanged. |
| `late_bindings_and_transactions_are_checked_even_after_assessment` | Final source/output checks cannot be replaced by assessment. |
| `repeated_attestation_preserves_original_journals_and_syncs_without_mutation` | Eight-sync attestation retains zero mutation and original journal identity. |
| `cleanup_accepts_stamped_partial_states_but_refuses_unstamped_and_uncertain_ones` | Pending transaction/intent cleanup exclusions remain in force. |

Sources: [normal publication tests](../crates/rvvdk-vsphere/src/ownership/retained/output/publication/tests.rs),
[reconciliation tests](../crates/rvvdk-vsphere/src/ownership/retained/output/publication/reconciliation/tests.rs),
[attestation tests](../crates/rvvdk-vsphere/src/ownership/retained/output/publication/attestation/tests.rs),
[attestation crash tests](../crates/rvvdk-vsphere/src/ownership/retained/output/publication/attestation/tests/crash.rs).

## Correctness and exclusion matrix

Use authored stored/deflated streamOptimized fixtures with independently specified
RAW bytes, dense and sparse layouts, boundary data, holes and trailing zeros. Seed
positive cases through actual normal publication and both reconciliation origins,
interrupted at Published record_written and record_synced. Do not manufacture valid
histories by editing journal state; edits belong in adversarial negative fixtures.

| ID | Case | Required assertions |
|---|---|---|
| T01 | Six originating histories: three origins × complete written/synced candidate | Separate recovery succeeds only after all fresh checks, ten syncs and one journal promotion; original failed reports remain failed. |
| T02 | Exact successor relation | Clone old and change only state/sequence. Mutate each remaining field independently, with recomputed valid checksums; reject even when bundle/source looks intact. Reject changed version, sequence and cleanup marker. |
| T03 | Complete state/transaction table | All eleven states, missing/invalid old journal, absent/partial/equal/earlier/later/foreign transaction. Only intent 7 plus exact Published 8 can pass. No orphan adoption or duplicate Published cleanup. |
| T04 | Record parser and file safety | Empty/partial/oversized/unknown/duplicate/trailing-invalid JSON, corrupt checksums, symlink/directory/FIFO/device/wrong-owner/mode/hardlink candidates reject. Use safe authored fixtures for available types; unavailable cases must be reported. Bounded reads must not block on special files. |
| T05 | All nine stage/final pairs | Only Absent/Owned eligible; test classifier and realizable filesystem cases. No bundle rename, stage adoption or collision removal. |
| T06 | Source transaction and source availability | Partial/full/matching/foreign/type-invalid source transactions always block. Missing/cleaned/incomplete/malformed source blocks even with intact RAW and candidate. |
| T07 | Explicit original bindings | Wrong output/artifact/source/store/destination and record operation/member stamps reject. Same-filesystem/private/outside-store constraints remain. Pinned parent paths never redirect to replacements. |
| T08 | Full content verification | Data/zero corruption, wrong length, metadata/owner mismatch and forged matching RAW digest/metadata reject against the independent source. Failed verification performs no sync/promotion. |
| T09 | Exact pair preservation before mutation | Read-only candidate and old opens; reject same-byte inode replacement and same-inode byte change. Capture bytes/inodes/names at admission and every failed pre-promotion outcome. |
| T10 | Late changes at all phase boundaries | Pair/source/namespace/member substitution, transaction removal/replacement, stage insertion and new source transaction block promotion. Include mutation after candidate sync and during/after assessment. Never replace a detected foreign old journal. |
| T11 | Successful promotion identity | Final journal bytes/inode equal the original candidate, old journal name replaced, candidate name absent, final record Published 8. No second candidate, backup, new operation ID or sequence 9. Source journal/bytes and bundle/member inodes/bytes unchanged. |
| T12 | Preservation and sparse allocation | Independent complete RAW/source oracle outside timing; equal initial/post-call/post-oracle allocation, exact length and member identity. No recopy/punch/preallocation repair; failures are retained. Ignore normal read-atime effects. |
| T13 | Control and lock custody | Invalid/pre-cancelled request has zero syncs/mutations; one deadline through admission, validation and final assessment. Both locks held and reacquirable on return, source handles released first. No returned mutation capability. |
| T14 | Report and error precedence | Every missing flag, primary/assessment error, wrong state/sequence/pair or pending transaction prevents success. Retain flags from completed phases; byte count/visible Published cannot fill them. Verify sanitized Debug/JSON. |
| T15 | Explicit retry selection | Before promotion repeat fresh recovery; after promotion reject recovery, allow separately gated attestation. No automatic dispatch or reusable report authority. Original intent without transaction still belongs to reconciliation. |
| T16 | Cleanup boundary | No automatic cleanup on success/failure/drop. Pending transaction blocks explicit output cleanup. After success, independently checked Published cleanup works; source cleanup prevents fresh recovery/attestation. |
| T17 | Shared-validator regressions | Normal Verified, both intent paths and Published attestation retain existing no-transaction gates and full byte work. No public admission widening from the private pair guard. |
| T18 | Syscall and independent work accounting | Ten syncs on the expected descriptors, one flags-0 journal rename, no bundle rename/create/unlink/application write, zero commit_output calls. Distinguish promotion from zero commit-helper invocations. |

## Fault and cancellation matrix

| Boundary | Required observation |
|---|---|
| Admission and first/later verification chunks | Partial progress allowed, no mutation; existing content/source gates and same absolute deadline. |
| Each of ten sync calls | Inject EIO, EINTR and unsupported-sync EINVAL (30 cases), stop at the failed call, retain earlier flags, no automatic retry. Reopen explicitly to inspect actual surviving names. |
| Old-intent file/store pair and namespace store/destination pair | Cancellation before entry prevents pair; cancellation within pair defers until both calls finish. I/O failure stops immediately. |
| Candidate file sync and final pre-promotion gate | Cancellation, expiry or binding drift after successful candidate sync must prevent rename. Candidate sync is not an indivisible group with promotion. |
| Journal replacement | Inject EIO/EINTR/ENOSPC/EACCES without mutation; retain both entries and report Uncertain. Also simulate real promotion followed by an error return: primary failure and false promotion/durability flags despite visible Published, no rollback. Label this as ambiguity simulation. |
| Successful replacement before store sync | Cancellation waits for store sync; store-sync error leaves promoted=true, acknowledgment_durable=false. Hook failure here is distinct from cancellation or a failed rename syscall. |
| After final store sync, before/after assessment and final control | Completed flags survive but late binding/control/assessment failure prevents success. A late source transaction fails despite valid output assessment. |

Mutation hooks must run before the ensuing binding gate. In particular,
before_promotion runs before the final full pair/source/namespace check and control
observation; leave no arbitrary callback between that gate and the rename syscall.
A hostile concurrent change inside the syscall boundary is outside the trust model.

Injected syscall results, hooks that stop after successful calls, and real process
loss are different evidence; record them separately. Callback failure is allowed to
interrupt a paired phase for testing, but production cancellation follows the above
rules. For each failed path assert no follow-on mutation, no hidden cleanup/retry,
correct independent assessment and preserved primary error if assessment also fails.

## Process-loss matrix

Retain the existing 18 originating boundaries: normal/staged-reconciliation/
already-renamed-reconciliation × Published record_partial, record_written,
record_synced, record_renamed, record_durable and after_return. Inspect current record,
transaction bytes/type/inode, both namespaces, original source and independent RAW.
Only written/synced complete pairs are candidates for the new operation; partial
stays blocked and post-rename Published requires a separate attestation invocation.
Existing intent reconciliation/attestation must still reject the candidate pair.

Kill a dedicated bounded child inside the new operation at all of these boundaries:

| Group | Required hook or equivalent boundary |
|---|---|
| Admission/content | before_admission, admitted, verified_chunk, validated |
| Bundle persistence | raw_synced, metadata_synced, marker_synced, bundle_synced |
| Old-intent persistence | intent_file_synced, intent_directory_synced |
| Namespace persistence | store_namespace_synced, destination_synced |
| Promotion preparation | transaction_checked, transaction_synced, before_promotion |
| Promotion/acknowledgment | transaction_promoted, acknowledgment_directory_synced |
| Final checks | before_assessment, after_assessment, bindings_rechecked, after_return |

Before transaction_promoted, the exact old/candidate pair must survive and a new call
repeats all work. At/after transaction_promoted, Published with absent transaction
must be handled by a separately requested attestation. Repeat at least the promotion
and post-promotion boundaries from both written and synced candidate histories.
Use fresh fixtures for each kill, assert the intended hook was reached, bound and
reap every child, and treat timeout termination as a failure. Process loss does not
simulate machine power loss, filesystem corruption or storage-controller faults.

Audit successful measured intervals from both written and synced candidates with
syscall tracing. Exclude fixture setup, oracle and cleanup via test-only markers.
Resolve file descriptors to the original old/candidate and bundle members; require
ten successful syncs, exactly one same-store journal replacement and no writable/
creating opens, application byte writes, unlink, bundle rename or allocation repair.
Keep raw paths/traces private; publish sanitized counts, trace hashes and verdicts.

## Performance acceptance and evidence

Freeze and compare these unchanged reference entry points, because parser/content
refactoring can affect all three APIs:

- `ownership::retained::output::publication::tests::benchmark::matched_publication`
- `ownership::retained::output::publication::reconciliation::tests::benchmark::recovery_branches`
- `ownership::retained::output::publication::attestation::tests::benchmark::attestation_histories`

Preserve both reconciliation branches and both attestation histories. Measure the
new operation separately using complete-written and complete-synced candidates from
normal publication; the six originating histories all receive correctness/counter
coverage. Those history labels describe fixture setup, not facts inferred by the
API. Neither different history nor different operation supplies a speedup baseline.
If an origin changes measured work, add separate timed coverage and explain it.

Use the existing 64 MiB stored and 256 MiB deflated logical fixtures with 8 MiB data,
plus dense correctness cases. Run on recorded Btrfs and isolated private tmpfs with
matching mount options, toolchain, affinity and storage/cache conditions. Record
CPU features, SMT/sibling load, frequency/governor and competing-work limits. Endpoint
frequency snapshots are not in-operation averages. Larger/colder working sets and
other filesystems remain separate qualification; tmpfs is a volatile control.

Fixture creation, source acquisition/conversion, original publication interruption
and opening the two locks are outside timing. Time fresh pair/source admission
through final assessment/binding/control and capability release. Keep disjoint
wall/process-CPU phases: admission, full verification, bundle sync, old-intent sync,
namespace sync, transaction sync/pre-promotion checks, promotion/final store sync,
and final checks/assessment/release. Record standalone rename and sync timings only
as clearly labeled nested diagnostics, not additional disjoint totals.

Retain whole wall/CPU, lifetime peak RSS including setup/oracle snapshots, exact RAW
length/allocation at initial/immediate-post/post-oracle checkpoints, source/journal/
candidate identity/byte checks and independent complete RAW comparison. Measure
explicit cleanup separately and require successful teardown with no child processes.
Do not punch or rewrite a fixture to fix allocation. Ambient tmpfs allocation drift
and prior approximately 15% 256 MiB tmpfs attestation timing variation remain PERF.0
items even if a new private control is stable.

Separate counter smokes for all six histories must confirm one native admission with
present-grain reads, two complete encoded-source read/hash traversals, one complete
logical RAW/source comparison/hash, ten syncs, one journal promotion and zero
commit_output calls during recovery. Record cleanup's two commits separately. A
changed count needs explanation and fresh review, never removal of a gate to fit a
counter. Production has no profiling clocks, counters or verification switches.

Run identical-binary phase-recording disabled/enabled controls for each filesystem/
fixture using the written-candidate history, with all verification/failure gates
identical. Disabled mode still has inactive hook costs; negative deltas establish
neither speedup nor zero overhead. State this bounded history coverage explicitly.
Alternate sample-pair and reference/candidate execution order. One timed workload
runs at a time; never overlap builds, tests, audits, plotting or another benchmark.

Begin with three rounds of four paired observations per comparison/history group.
Any >5% round-median spread, adverse matched wall/CPU observation or unexplained
regression triggers three longer rounds of eight pairs. Save initial observations
and repeat decisions before collecting repeats. Publish absolute short-phase values,
all repeats and unresolved variation; spread is not a confidence interval. New
operation results establish a baseline, not a performance improvement claim.

Carry forward the [timer-resolution and resume findings](benchmark-results/2026-10-14-r61d2b/README.md):
finite zero CPU observations in short phases/rename are valid and remain exactly
zero. Whole-operation/cleanup CPU and all wall durations retain their existing
positive gates. Reject negative/nonfinite numbers. A zero denominator yields null
relative spread plus explicit resolution flags/counts, never an epsilon or dropped
sample. Include such resolution-limited groups in longer repeats and retain their
unresolved status; they cannot silently pass a percentage gate. Retain phase-to-whole
accounting gaps from hook bookkeeping instead of assigning them to I/O.

A rejected measurement halts acceptance for investigation; later passes cannot erase
it. A resumed campaign must preserve original scripts, raw data, commands, repeat
decisions and rejection, validate unchanged frozen binaries/conditions, and document
namespace recreation or pauses. Never silently replace completed observations.

Save raw numeric rows, commands, environment, source/fixture/binary hashes, validator
and counter/audit results, summary, repeat decisions and manifest/plot input hashes.
Negative validator cases must cover pair order, missing phases, byte/work counts,
allocation, exact candidate promotion identity, incorrect sync/rename/commit counts,
cleanup failure, negative/nonfinite timing and broken hashes; include a positive
zero-CPU short-phase case. Export and inspect SVG/PNG normal/reconciliation/attestation
comparisons, new whole/phase timings, resources/cleanup and recording-control plots.
Keep private endpoint/credential/VM bindings and real image digests out of commits.
Python may orchestrate offline measurements and plots; recovery/data-path code is Rust.

Run focused release tests on both filesystems and workspace tests, fmt and all-target
Clippy with warnings denied. Record bounded completion and remaining limits without
closing PERF.0 or general R6 recovery. R6.1d.3a itself changes documentation only;
its design validation does not rerun Rust tests or produce runtime benchmark results.
