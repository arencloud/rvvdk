# ADR-0053: Separate stream grain-index validation from decompression

Status: Accepted for metadata indexing, 2026-10-02.

## Context

R5.10 admits streamOptimized envelopes without following map pointers. Native
reads must first establish bounded record ownership: a structurally valid marker
alone does not establish which logical grain owns it or whether it overlaps
metadata. R5.11 is split into independently reviewable map validation (R5.11a)
and decompression/logical reads (R5.11b).

## Decision

Add a separate `StreamMap` with private record fields, explicit aggregate memory,
read, table-work and allocated-grain limits. Acquire the envelope from the same
source, validate all table locations before table I/O, count populated grains
before index allocation, and cross-check the second table pass against physical
record order. Compare redundant tables and bind each record's LBA to its GTE.
Only grain prefixes are read; payloads and padding are skipped.

Accept the observed front-directory profile and the ordered footer subset.
Require contiguous sector-rounded records and zero unused slots; reject aliases,
orphans, gaps and unsupported layouts. Keep this subset explicit rather than
claiming universal format compatibility. Retain sparse records (12 bytes each)
in logical order for binary-search lookup instead of allocating an entry for every
virtual grain. Do not enable the public CLI or reuse uncompressed sparse mapping.

## Consequences

The [contract](../vmdk-stream-map.md) defines exclusions and source quiescence.
Two table passes cost additional metadata reads but allow exact sparse allocation
without unbounded vector growth. This layer cannot prove compressed integrity,
provide logical bytes or bind a later caller-supplied source. R5.11b owns framing,
source ownership, bounded decompression and differential byte qualification.

Authored adversarial tests and QEMU/authored reference maps pass. A metadata-only
probe of the retained ESXi export validates the observed live layout without
new exports or VM state changes. [Measurements and plots](../benchmark-results/2026-10-02-r511a/README.md)
retain every timing pair, longer adverse repeats, and explicit performance
limitations. Prior stream timing/PERF.0 follow-ups stay open.
