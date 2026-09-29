# ADR-0037: Bounded terminal NUL padding during descriptor acquisition

Status: Accepted. Date: 2026-09-29. Implements R4.5.

## Context

The R4.3 reference tool generated hosted flat descriptors with terminal NUL bytes.
The strict text parser rejected these originals, so only explicitly normalized
fixture copies were qualified. CLI users need to open those original files while
retaining strict syntax, resource bounds and complete input provenance.

## Decision

Handle a contiguous terminal NUL run in `DescriptorText::read_from`, after reading
to EOF within `Limits::descriptor_bytes`. The limit includes text and padding;
one additional probe byte still rejects oversize input. An error after padding
is an acquisition error, not successful end of text. Do not stop reading at the
first NUL, trust a size hint, or ignore arbitrary bytes after a terminator.

Retain the entire original byte vector. Cache the index after its final nonzero
byte once during acquisition. `as_bytes()` returns original bytes unchanged;
`text_bytes()` returns the strict text prefix and `padding_bytes()` reports the
suffix length. `parse()` borrows the cached prefix, avoiding a padding scan on
each call. No second text allocation or per-block work is introduced.

`Descriptor::parse` and `parse_with_limits` remain strict text APIs: any NUL still
rejects. Acquisition accepts a NUL suffix only when its remaining prefix passes
all existing syntax, encoding, create-type, line and resource checks. Interior
NULs, nonzero content after NULs, empty/all-zero files and invalid prefixes reject.
No sector alignment or final newline is required; the total-byte ceiling provides
the padding bound. This is an explicit acquisition policy for the supported
subset, not a claim about every producer or VMDK container format.

## Consequences and evidence

LocalResolver, the CLI and dump helper use DescriptorText and therefore gain this
behavior without reopening or rewriting files. Existing identity, source-change,
confinement, publication and execution policies remain. Repeated parsing has the
same text work regardless of padding size; first acquisition scans the suffix
once. The unpadded boundary lookup examines only the final byte.

Tests cover suffix lengths, chunk boundaries, Interrupted reads, I/O failure after
padding, exact/over-limit input, hidden suffixes, unchanged original bytes, strict
parser behavior and all four CLI commands. The reference runner feeds unaltered
QEMU-generated hosted descriptors to the dump helper and public CLI and verifies
original input hashes, expected RAW bytes and QEMU comparison. Custom-layout
independent-decoder qualification remains open.

See [padding contract](../vmdk-padding.md) and
[measured evidence](../benchmark-results/2026-09-29-r45/README.md).
