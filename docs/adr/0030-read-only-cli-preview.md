# ADR-0030 — Read-only RAW CLI and non-executable plan previews

Status: Accepted, 2026-09-29 (R3.1).

## Problem

The next vertical slice needs inspect/plan commands before output creation and
copy execution exist. Executable RAW planning requires an actual writable
endpoint. A missing output has neither identity nor descriptors, and opening
existing output writable would make a read-only preview dependent on write access.

## Decision

Introduce the thin rvvdk-cli crate with public binary name rvddk. Require explicit
RAW format. Use existing LocalFileBlockDevice and DataMover logical planning for
extent discovery, canonical topology validation, totals, and metadata budget
admission. Keep the libraries free of CLI/serialization dependencies.

Inspect emits observed facts. Plan emits a schema-versioned preview with explicit
destination policy and requested configuration. Threaded selection is known by
request; Auto/native selection is deferred until actual destination preparation.
Do not fake writable capabilities, use /dev/null as a destination proxy, or
serialize an executable CopyPlan. All runtime and write-readiness flags stay false.
No copy, verify, cancellation, or new progress semantics are implemented here.

Inspect existing destination metadata without opening it writable. Default to
no-clobber intent for absent paths and require --overwrite for existing regular
files. Reject known aliases and destination leaf symlinks/special files. Overwrite
intent is in-place over source length with any larger tail preserved. R3.2 must
recheck live endpoints and enforce race-safe no-replace publication for new output.

## Consequences

Preview success does not establish destination permissions, native request/ring
compatibility, an admitted execution memory budget, or a stable snapshot. These
limits appear in human and JSON output rather than a fabricated backend decision.
The Threaded payload estimate is explicitly labeled even for native requests.

CLI parsing uses already-locked clap; serde/serde_json serialize report types in
the CLI crate only. No library engine implementation or tuning default changes.
JSON version 1 fixes field meanings and error codes, with additive extension
allowed. Paths retain exact non-UTF-8 bytes separately from display text.

[The CLI contract](../cli.md) defines schema, error codes, resource boundaries,
and future creation/overwrite requirements. Tests cover unchanged inputs/output,
explicit policy, aliases, sparse/odd/empty images, error streams, output failure,
and exec children denied writable opens and ring setup. Matched library controls
and separately identified new CLI measurements retain performance evidence.
