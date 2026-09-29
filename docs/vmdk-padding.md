# Bounded descriptor padding (R4.5)

`DescriptorText::read_from` accepts a valid supported descriptor followed by a
contiguous run of NUL (`0x00`) bytes. Linux descriptor loading and all four VMDK
CLI commands use this acquisition API. Input files are never rewritten.

| API / input | Behavior |
|---|---|
| `Descriptor::parse` / `parse_with_limits` | Strict text; any NUL rejects |
| `DescriptorText::read_from` | Read to EOF within the total-byte limit, then validate the prefix before the terminal NUL run |
| Valid text without padding | Existing behavior; no full input scan to locate padding |
| Valid text + only terminal NULs | Accepted; final newline and sector alignment are not required |
| NUL inside text, filename or a comment | Rejected by the strict parser |
| Nonzero bytes after a NUL, including whitespace | Rejected; no hidden second descriptor or ignored suffix |
| Empty file, all NULs, invalid prefix | Rejected by descriptor validation |
| I/O error after NULs | Acquisition fails; NUL is not treated as EOF |
| Total length exceeds `descriptor_bytes` | Rejected after at most the configured limit plus one probe byte |

The default acquisition ceiling remains **1 MiB including padding**. Line,
extent, filename and metadata limits apply to the validated text prefix. Padding
does not consume line length because it is outside that text; it always consumes
the total byte budget. This is not a wall-clock or whole-process memory limit.

`as_bytes()` preserves original bytes, including padding, for hashes/provenance.
`text_bytes()` borrows the validated prefix. `padding_bytes()` returns the suffix
length. `parse()` returns borrowed metadata from the cached prefix, without
rescanning the padding. Acquisition retains one byte vector and one boundary
index; it does not allocate a normalized text copy.

The [R4.5 reference report](benchmark-results/2026-09-29-r45/README.md) records
unaltered QEMU-generated monolithic/split flat descriptors, tool and executable
hashes, CLI inspect/plan/copy/verify results, expected RAW bytes and QEMU comparison.
The historical R4.3 normalization evidence remains unchanged. These are local
hosted format checks; custom layouts still lack independent decoder qualification,
and no ESXi or live VMware compatibility is implied.

See [ADR-0037](adr/0037-bounded-vmdk-padding.md), the [descriptor subset](vmdk-descriptor.md),
[backing acquisition](vmdk-backing.md) and [CLI contract](cli-vmdk.md).
