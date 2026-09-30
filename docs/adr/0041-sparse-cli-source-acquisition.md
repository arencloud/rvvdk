# ADR-0041: Sparse CLI source acquisition

Status: Accepted. Date: 2026-09-30. Implements R5.4.

## Decision

Extend explicit `--format vmdk` to clean version-1 base hosted sparse inputs.
Dispatch the opened regular file's bounded first chunk to text acquisition or
hosted sparse header admission. Keep RAW selection explicit. Text is bounded to
1 MiB including padding, plus one oversize probe; an embedded descriptor region
must pass SparseHeader limits and file-range validation before allocation/read.

External sparse descriptors must declare split sparse layout. An embedded entry
must declare monolithicSparse and its extent name must resolve to the same device/
inode as the already opened container, before metadata reads from that reference.
All lookup stays beneath the confined LocalResolver. This rejects redirection to
a different container; same-inode hard links are valid.

Retain descriptor/container and backing observations using the existing Source
lifecycle. Add SparseDisk to portable plan/execute dispatch; native RAW execution
remains unavailable. Reuse destination alias checks, timestamp observations,
verification, cancellation and private no-replace publication.

## Consequences

No new command/format flag or output schema version is needed. Sparse previews add
layout and metadata reservation/read counters; existing FLAT/ZERO report fields
remain unchanged. Sparse aggregate limits stay independent of --memory-budget.
Text acquisition shares its first read with dispatch, then parses FLAT/ZERO once
or explicitly tries the split sparse subset. Library Descriptor/DescriptorText
contracts remain unchanged.

The CLI still requires quiescent sources; timestamp observations are not a snapshot
or external-writer exclusion. Parents, compressed/managed variants and VMDK writes
remain unsupported. [CLI contract](../cli-vmdk.md) and
[R5.4 evidence](../benchmark-results/2026-09-30-r54/README.md) record limits and tests.
