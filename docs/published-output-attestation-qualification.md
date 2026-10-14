# Published-output attestation implementation and qualification (R6.1d.2b)

Status: **acceptance plan**, established by R6.1d.2a on 2026-10-14. None of the new
runtime acceptance items below has been executed. Existing anchors were reviewed
at `b569060`. [Contract](published-output-attestation.md) and
[ADR-0069](adr/0069-fresh-published-output-attestation.md) define the proposed API.

## Implementation sequence

1. Freeze the unchanged release test executable, tracked source/fixture hashes,
   build flags/toolchain and existing normal-publication/reconciliation benchmarks.
   Preserve the [R6.1d.1b evidence](benchmark-results/2026-10-14-r61d1b/README.md).
2. Add a private attestation module and consuming public API/report. Reuse retained
   admission, `CheckedOutput::validate_at`, pinned descriptor checks and sanitized
   assessment. Borrow the store until source admission succeeds so failed admission
   still permits assessment. Keep public Verified and PublishIntent gates unchanged.
3. Add narrowly scoped final-source/output/namespace checks. A small existing helper
   may move from reconciliation to the parent module if needed; avoid a generalized
   recovery dispatcher. Use a private sync-only syscall-error seam and private hooks;
   production always calls the OS and gains no instrumentation/skip options.
4. Implement the eight-sync protocol with the same record/inode, zero commits,
   no namespace mutations, one deadline and paired-phase control behavior. Final
   post-assessment checks must include source transaction absence and original journal
   identities, since `PublicationRecovery` alone does not contain that evidence.
5. Execute correctness/failure/process-loss cases, then release comparisons and
   independent work accounting. Keep benchmarks separate from builds/tests/plotting.
6. Update implementation docs, record all observations and limits, validate/redact
   artifacts and commit the bounded package locally. No lab/VM action is required.

## Existing regression anchors

| Test / helper | Preserve or extend |
|---|---|
| `publication_journal_faults_never_turn_uncertainty_into_success` | Normal publication still fails when acknowledgment is uncertain, even with visible Published. |
| `acknowledgment_faults_and_assessment_failures_never_become_success` | Reconciliation reports its own errors; a later attestation never changes that earlier result. |
| `reconciliation_process_loss_preserves_retry_or_blocked_acknowledgment` | Existing intent reconciliation must continue rejecting Published after process loss. Add separate explicit attestation tests. |
| `both_branches_verify_preserve_sync_and_acknowledge_without_recopy` | Preserve intent paths, original-byte checks, lock custody and no-recopy behavior. |
| `fresh_content_and_member_checks_reject_corruption_and_forged_claims` | Extend full checks to Published-at-entry rather than trusting recorded digest claims. |
| `destination_constraints_and_binding_are_explicit` | Original private same-filesystem destination, containment and binding checks stay in force. |
| `cleanup_accepts_stamped_partial_states_but_refuses_unstamped_and_uncertain_ones` | Explicit Published cleanup remains independently eligible, including after source cleanup; attestation never invokes it. |

Normal anchors are in [publication/tests.rs](../crates/rvvdk-vsphere/src/ownership/retained/output/publication/tests.rs),
reconciliation anchors in [reconciliation/tests.rs](../crates/rvvdk-vsphere/src/ownership/retained/output/publication/reconciliation/tests.rs)
and [tests/crash.rs](../crates/rvvdk-vsphere/src/ownership/retained/output/publication/reconciliation/tests/crash.rs).
These are existing evidence, not tests of the proposed operation.

## New correctness and exclusion matrix

Use authored stored/deflated streamOptimized fixtures, dense and sparse layouts,
known data at boundaries, holes and trailing zeros. The expected RAW must come from
independently authored bytes, not the native decoder under test. Construct Published
through actual publication/reconciliation before faulting; record edits are useful
only for negative corruption tests.

| ID | Case | Required assertion |
|---|---|---|
| A01 | Fully successful normal publication and repeated explicit attestation | Fresh full verification on every call; exactly eight syncs, zero commits/renames/writes, unchanged output record bytes/inode and sequence 8. |
| A02 | Visible Published after interrupted acknowledgment | Admit separately after `record_renamed` and `record_durable` failures from normal publication and both reconciliation origins. Original failed reports remain failures; new success observes only new barriers. |
| A03 | All nine namespace pairs | Only Absent/Owned eligible. Exercise the complete decision table and realizable invalid/ambiguous filesystem states; preserve all resources. |
| A04 | All eleven output states and malformed records | Only valid v2 Published sequence 8 eligible. Unknown fields/version, checksum, sequence, operation, stamps, digest/contract and cleanup marker corruption fail. |
| A05 | Source/output transaction at entry or late | Partial/full/matching-valid/foreign/symlink/directory entries all block. Preserve bytes/type/inode; no promotion or deletion. Include insertion before/after final assessment. |
| A06 | Missing, cleaned, incomplete, malformed or changed retained source | Reject even with intact RAW/digest; no new export, source recreation or source cleanup. |
| A07 | Wrong output/artifact/source/store/destination and replaced journals | Reject explicit binding mismatch. Same-byte journal replacement after admission is invalid for both source and output. |
| A08 | RAW data/zero corruption, wrong length, forged matching digest/metadata | Reject on independent full source comparison; no persistence phase begins after verification failure. |
| A09 | Missing/extra/foreign/symlink/hardlink/wrong-mode/member/owner entries | Reject without adopting or removing them, at initial and late boundaries. |
| A10 | Non-private/nested/store/wrong-device destination; replaced parent paths | Reject invalid destination. Pinned descriptors never redirect to replacement paths; original recorded identity remains required on reopen. |
| A11 | Late namespace/source/output substitution around every phase and assessment | Fresh binding checks catch drift, preserve foreign entries and never switch to staged recovery. Include identical-byte output inode replacement after the final journal sync. |
| A12 | Pre-cancellation, zero/over-limit timeout and shared mid-call deadline | Fail before sync for invalid/pre-cancelled requests; no phase budget reset. Expiry inside admission/verification and final assessment prevents success. |
| A13 | Lock custody and bounded resources | Both locks unavailable through final checks, reacquirable on return; release source handles before store. No writable capability escapes. |
| A14 | Preservation and independent byte oracle | Same original source/output journal bytes/inodes, source bytes, RAW member inodes/length, independent full bytes, sparse allocation at initial/post-call/post-oracle checkpoints. No new stage/transaction/second RAW. Ignore normal read-atime effects. |
| A15 | Report semantics, redaction and failure precedence | Each missing flag, primary/assessment error, wrong final state/sequence/pair or pending transaction prevents success. Debug/JSON contain no private bindings/digests/paths. Preserve primary error if assessment also fails. |
| A16 | Cleanup/source-retention boundary | No automatic deletion on success/failure/drop. Explicit cleanup remains separately checked; after source cleanup attestation rejects but eligible output cleanup still works. After output cleanup attestation rejects. |
| A17 | Final control and source checks | Cancellation/expiry during or after assessment returns failure; a late source transaction/identity change fails despite an otherwise valid `PublicationRecovery`. Retain sync flags. |

For A01/A02 instrument exact sync order and current journal descriptor identity.
Verify final journal/store sync happens after both namespace parents, despite the
already-visible record. Capture source/output journal bytes/inodes before/after,
prove zero transaction creations/commits and no rename through the private seam or
syscall audit. Code review alone does not replace these assertions. Required no-write
checks refer to application payload/journal mutations, not filesystem metadata such
as access times or sync-induced writeback.

## Fault, cancellation and process loss

| Boundary | Injected result/control expectation | SIGKILL and explicit retry expectation |
|---|---|---|
| Entry/admission/first and later verification chunks | Failure or cancellation, partial comparison count allowed; no sync before full verification. | Resources remain Published; new call starts admission again. |
| Each member and bundle sync | Simulate EIO/EINTR/unsupported-sync result through test-only seam; stop at failing call. Check control before each; record bundle flag only after all four succeed. | No journal/state mutation; repeat all verification/syncs. |
| Before/between store and destination namespace syncs | Cancellation before pair prevents entry; inside pair defers until both complete. I/O failure stops immediately and prevents terminal record phase. | Published visibility does not skip either parent on retry. |
| Before/between existing Published file and final store syncs | Check control after binding revalidation, just before entry. Cancellation within pair waits for both; I/O failure stops immediately. No transaction created. | Same record/inode remains; retry repeats all eight syncs. |
| After final store sync and around final binding/assessment checks | Completed flags survive late error; no success from state alone. Check original source/output journals and both transactions after assessment. | A fresh call repeats verification and syncs; no cached success grant. |

Simulated syscall failures, hook errors around successful operations and actual
SIGKILL are different evidence; label them separately. No destructive filesystem or
physical power-cut test is authorized by this local package. Test both late control
errors and failed independent assessment with an already-existing primary failure.
Keep bounded/reaped child processes; a timeout kill is a failure, not a reached hook.

Seed crash cases from normal publication and each intent-reconciliation origin:
kill at Published `record_partial`, `record_written`, `record_synced`, `record_renamed`,
`record_durable` and after acknowledgment before return (using an appropriate hook).
The first three retain PublishIntent plus transaction and must remain blocked;
Published after the last three is eligible only after fresh admission with no
transaction. Test no automatic transition from blocked intent into attestation.
Then kill inside attestation at admission/verification, each of eight sync boundaries,
and final binding/assessment boundaries. Inspect both journals, namespace, member
identities and independent bytes before explicitly retrying; do not edit records to
manufacture eligibility. Repeated successes leave the same Published journal inode.

## Performance acceptance

Freeze the pre-implementation release binary before any Rust change. Keep both
existing benchmark entry points unchanged and use the same compiler flags and fixture
sources for matched candidate/reference comparisons:

- `ownership::retained::output::publication::tests::benchmark::matched_publication`
- `ownership::retained::output::publication::reconciliation::tests::benchmark::recovery_branches`

Measure the new API separately; there is no old attestation implementation to claim
a speedup against. Compare normal publication and **both** existing reconciliation
branches to the frozen executable, because shared private helper changes may affect
any of them. Do not present different operations, logical sizes, encodings or CPUs as
a matched optimization. New attestation histories must include acknowledged Published
and visible Published from an uncertain acknowledgment; histories are controls, not
inferred fields in the public report.

Use the same 64 MiB stored / 256 MiB deflated logical fixtures with 8 MiB data as the
previous bounded campaign, plus dense correctness fixtures. Measure Btrfs and an
isolated private tmpfs control with recorded mount options. Ambient `/tmp` allocation
changes remain unresolved: retain any recurrence and do not weaken allocation gates,
rewrite/punch RAW to repair a sample or silently replace a failed run. Tmpfs establishes
no persistence guarantee. Larger working sets and other filesystems remain separately
qualified work; report unavailable cases explicitly.

Preopen locks, create fixtures and induce acknowledgment outcomes outside timing.
Main timing spans fresh admission through final assessment/binding/control checks and
capability release. Keep disjoint wall/CPU phases for admission, full verification,
bundle sync, parent sync, Published-record sync, and final checks/assessment/release.
Include whole wall/process CPU, lifetime peak RSS (including fixture setup), original/
post-operation/post-oracle RAW allocation and separately measured explicit cleanup.
Run independent complete RAW/source/journal checks outside the main timing, retaining
all pass/fail results. Final fixture teardown is excluded and must leave no children.

A separate counter smoke must demonstrate one native admission with present-grain
reads, two complete encoded-source read/hash traversals and one full logical RAW/source
comparison/hash using the existing validator. Require eight explicit syncs, **zero
journal commits**, zero renames and preserved original journal bytes/inodes. Confirm
actual measured work before claiming equivalence; explain any justified difference
without removing checks to fit the counts. Output cleanup still commits separately.

Use identical-binary disabled/enabled phase-recording controls, keeping all byte and
failure gates equal. Production adds no clocks/counters. Inactive hook/counter costs
remain in disabled mode; negative deltas do not establish a speedup or zero overhead.
Alternate pair order and reference/candidate execution order, record affinity,
CPU/SMT/frequency/cache/storage conditions and competing-work limits. Run one timed
workload at a time, with no overlapping builds/tests/plots.

Start with three rounds of four pairs per comparison. Any >5% round-median spread,
matched adverse wall/CPU observation or other unexplained regression requires three
longer rounds of eight pairs. Retain initial samples and repeat decisions before
collecting repeats. Report unresolved variation and absolute small-phase values;
round-median spread is descriptive, not a confidence interval. Every failure stops
acceptance until investigated and recorded; a later pass cannot erase it.

Keep raw numeric samples, commands, environment, source/fixture/binary hashes,
summary/validator results, negative validator smoke cases and plot input hashes.
Export SVG/PNG phase, normal/reconciliation comparison, memory/allocation/cleanup and
recording-control figures; visually inspect them. Keep real fixture identities,
credentials, endpoints and disk digests private. Python may orchestrate local Rust
executables and plot results; production VMware/data/recovery logic stays Rust.

Complete focused release and appropriate workspace test/fmt/Clippy checks. Record
performance disposition without declaring PERF.0 or general recovery complete. The
R6.1d.2a design-only commit does not rerun Rust tests or benchmarks and claims no new
runtime timing; implementation acceptance remains entirely open here.
