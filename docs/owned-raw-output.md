# Private owned RAW output (R6.1c.5a)

`ownership::RetainedArtifact::convert_owned` consumes a freshly admitted retained
source and creates a separate private RAW output. It retains the source store lock
until copying, worker drain, output flush, full logical verification, metadata
persistence and final assessment finish. The source CompletedLease journal remains
unchanged. This is a synchronous Linux local operation; invoke it from a blocking
context and await completion.

Supply a nonzero `OutputId`, CopyOptions, RetainedOptions and an observer. IDs are
explicit and are never inferred from filenames or artifact metadata. Public reports
contain closed errors, state/sequence and progress facts, without resource names,
source IDs, content digests or remote credentials. `is_success()` requires both
errors absent, conversion flushed, metadata durable, and Verified with no pending
transaction. Engine Completed only reports the copy engine's flush; subsequent
checks may fail. Cancellation/deadlines remain cooperative around blocking I/O.

## Distinct contract and durable state

An output is RAW, distinct from the exported streamOptimized container. A separate
version-1 journal binds the output ID to artifact ID, explicit source binding,
logical capacity, store identity, random operation ID and a SHA-256 of the canonical
admitted ExportArtifact metadata. The binding digest is local consistency evidence,
not authentication. A checksummed bounded envelope rejects unknown fields, versions,
inconsistent states and wrong identities. The source journal schema is unchanged.

| Sequence | State | Acknowledged fact / next effect |
|---:|---|---|
| 0 | Prepared | Output ID reserved by a durable record |
| 1 | StageIntent | Intent precedes private directory/file creation |
| 2 | Staged | RAW, metadata placeholder and marker identities recorded after file and directory sync |
| 3 | ConvertIntent | Intent precedes source re-admission and destination writes |
| 4 | Converted | Conversion flushed, writer dropped and stage checked |
| 5 | VerifyIntent | Intent precedes full readback and metadata writes |
| 6 | Verified | Full logical equality, RAW digest and durable metadata acknowledged |

Seven successful journal commits each write a new exclusive transaction file,
sync it, rename it and sync the store directory. Initial record creation uses
RENAME_NOREPLACE. Any uncertain journal write stops the owner; it never retries a
state change or performs a later resource action. Assessment reads surviving state
and reports a pending transaction. A renamed record can be visible even when its
directory sync or acknowledgment was interrupted; visibility alone does not prove
physical durability or grant mutation authority.

The generated private stage contains `disk.raw`, `output.json` and `owner`. Names
are fixed/generated single components under the pinned store descriptor. The stage
is 0700; regular members are 0600 with one link and the current owner. Opens reject
symlinks; every use checks recorded device/inode identities and the operation marker.
RAW starts at exact logical length. Conversion reuses the existing confined
`convert_to` checks, controlled DataMover, copy budgets and sparse-zero behavior.
All writable RAW handles close before Converted and logical readback.

## Logical verification and metadata

Read every logical source/output byte, including absent/zero grains, with two fixed
1 MiB buffers. Compare native source bytes to RAW bytes while computing SHA-256 over
the observed RAW. Recheck exact output size, stage identities and the entire encoded
source contract/content after readback. This verification is independent of the
copy execution, but shares the native source decoder. Independently authored raw
bytes qualify decoder and output correctness in tests; this is not a claim of a
second production decoder implementation.

Only then write bounded private output metadata (at most 4 KiB), sync it, reopen
read-only and compare exact bytes and parsed fields before acknowledging Verified.
Metadata records RAW format, explicit bindings, capacity, digest and
`logical_source_readback` evidence. No path, URL or remote ticket is stored there.
Two verification buffers add 2 MiB outside the CopyOptions payload budget; the native
map/decoder retains its existing separate limits. Readback is single-threaded and
bounded; the whole operation retains the six-hour maximum timeout policy.

## Retention and recovery boundary

`JobStore::assess_output` is read-only. It checks record consistency and the stamped
stage identities/marker, returning state, sequence and pending-transaction status.
It does **not** reread RAW bytes/metadata or recreate a conversion, cleanup or
publication capability. Even a returned Verified is an observed journal claim.

Every output stage and record is retained on success, cancellation, error and drop.
A StageIntent can have an unrecorded partial stage; assessment does not adopt or
remove it. Output IDs remain reserved. No output cleanup, restart/resume, final
pathname publication or remote capability is provided by this package. Source
cleanup remains a separate existing explicit operation; it does not clean RAW output.
Test fixture-tree deletion is test infrastructure, not a production cleanup API.

The private local store, trusted ancestors and cooperating writers are requirements
throughout the operation. Advisory locks do not exclude hostile same-user writers,
and checksums do not prevent offline rollback. tmpfs testing establishes behavior,
not power-loss persistence. Fault hooks and SIGKILL do not emulate storage controllers.

## Follow-up implemented in R6.1c.5b

[Durable output publication](durable-output-publication.md) now adds a separate
VerifiedOutput capability, fresh source/RAW verification, atomic no-replace bundle
publication, conservative namespace assessment and explicit checked cleanup. It
preserves the version-1 conversion contract and introduces version 2 only for new
publication/cleanup transitions. The conversion operation itself still retains
output on every outcome and never publishes or deletes it automatically.

Composed live qualification follows as R6.1c.5c. Pending transactions, uncertain
publication and unstamped stages remain ineligible for automatic recovery actions.

[ADR-0064](adr/0064-private-owned-raw-output.md),
[tests, timing and plots](benchmark-results/2026-10-02-r61c5a/README.md),
[retained source contract](retained-artifact-conversion.md).
