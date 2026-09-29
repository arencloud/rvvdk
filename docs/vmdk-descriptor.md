# VMDK descriptor subset (R4.1)

`rvvdk-vmdk::Descriptor::parse(&[u8])` validates text and returns borrowed metadata.
It opens no files and implements no `VirtualDisk` yet. The [logical reader](vmdk-logical.md) consumes resolved metadata; the CLI
also accepts explicit [VMDK sources](cli-vmdk.md). This is the first format layer, independently implemented in Rust.

The format reference is VMware's [Virtual Disk Format 5.0, pages 3–5](https://github.com/vmware/open-vmdk/blob/master/vmdk_50_technote.pdf).
The specification describes headers, ordered extents, 512-byte sector units,
access modes, and informational disk database records. Broadcom's
[disk-type overview](https://developer.broadcom.com/xapis/virtual-disk-api/latest/vddkDataStruct.5.3.html)
distinguishes hosted, managed, sparse and streaming variants. The restrictions
below are **rvvdk's initial policy**, not a claim about everything VMware accepts.

## Accepted layouts

| createType (case insensitive) | rvvdk rules |
|---|---|
| `monolithicFlat` | Exactly one FLAT extent with offset zero |
| `twoGbMaxExtentFlat` / `2GbMaxExtentFlat` | One or more FLAT extents, each at most 2 GiB, offset zero |
| `custom` | One or more FLAT/ZERO extents; FLAT may have a nonzero offset |

Every extent is positive length. `RW` and `RDONLY` describe source permissions;
this crate offers no write API. `NOACCESS` fails explicitly. FLAT requires a quoted
nonempty filename and an unsigned decimal sector offset. ZERO accepts exactly
access, sector count and type; it has no backing reference. Split extents need not
be full sized. Metadata capacity is the sum of extent lengths, never CHS geometry.

Required header keys: version (1), CID (exactly eight hex digits), parentCID
(`ffffffff`), createType (quoted). Optional encoding must be quoted UTF-8.
Both spellings of the split create type are accepted explicitly. Structural
keywords and duplicate detection ignore ASCII case; filenames and values preserve
case and Unicode bytes. Parent hints/chains, sparse/streaming, managed VMFS,
physical/device mappings, other versions/encodings and unknown keys are rejected.

The informational DDB allowlist is `adapterType`, `geometry.cylinders`,
`geometry.heads`, `geometry.sectors`, `virtualHWVersion`, `toolsVersion`, `uuid`,
and `longContentID`, each prefixed with `ddb.` and containing a quoted value.
Values are retained without semantic interpretation. Unknown DDB keys fail;
this intentionally sacrifices compatibility rather than accidentally ignoring
sector-size or encryption metadata. Acceptance does not prove a referenced file
is plaintext or matches its description.

## Grammar and bounds

Header, extents, then optional DDB must appear in that order. Blank lines and
comments are permitted. ASCII spaces/tabs separate fields; LF and CRLF are accepted.
A `#` outside quotes starts a comment. Quoted filenames can contain spaces, `#`,
`=`, and UTF-8 text. Quotes/backslashes/escape sequences and controls inside quoted
values are unsupported. Bare CR, NUL, other control characters, invalid UTF-8,
BOMs, trailing fields and duplicate keys fail. No permissive recovery is attempted.

| Limit | Default |
|---|---:|
| Entire input | 1 MiB |
| Physical line excluding LF, including CR if present | 8 KiB |
| Extents | 1,024 |
| DDB entries | 128 (the current allowlist has eight unique keys) |
| Filename bytes | 4 KiB |

`parse_with_limits` lets the caller choose ceilings, including zero. Limits are
checked before scanning oversized input or adding entries. Line tokens use a fixed
six-element stack array. Extents and metadata use vectors; strings borrow input,
without filename copies or a descriptor-sized secondary buffer. Vector capacity
may exceed entry count due to normal growth; this is a bounded parser, not an exact
allocation-byte budget. Work is bounded by input/line/entry limits. DDB duplicate
checks scan the small allowlisted metadata collection.

The caller already owns input bytes. A future descriptor reader must enforce an
acquisition limit before reading/allocating the complete file. Raising limits is
an explicit caller decision. Arithmetic checks cover each sector-to-byte product,
FLAT backing offset plus length, and cumulative logical capacity. Every successful
extent has a nonempty, contiguous, sector-aligned logical byte range. Typed errors
include one-based source lines; zero denotes input-wide/final validation. Messages
do not reproduce descriptor contents or paths.

## Boundary with backing storage

R4.2 now provides [bounded acquisition and backing resolution](vmdk-backing.md).
The parser itself retains the metadata-only guarantees below.

A parsed filename is **untrusted lexical metadata**. Parsing can accept `../name`,
absolute names or URI-like strings without accessing them. No file length,
existence, confinement, symlink, alias or identity guarantee exists at this stage.
Never open a parsed name directly in application code.

R4.2 provides caller-supplied `BackingResolver` contracts and a local resolver
that defaults to descriptor-relative confined regular files. It rejects
absolute/traversal escapes and symlinks, bounds resource use, validates backing
offset/end against live file length, and retains identities.
Transport-neutral references remain supported. [R4.3](vmdk-logical.md) provides
read-only FLAT/ZERO logical mapping, cross-extent reads and scoped reference-byte
comparisons. [R4.4 CLI integration](cli-vmdk.md) is available. Container descriptors must not be
offered as native RAW endpoints.

## Fixtures and qualification

[Fixture provenance](../crates/rvvdk-vmdk/tests/fixtures/README.md) records the small,
project-authored examples. No VMware-generated guest image, third-party parser
source or format PDF is committed. Tests exercise supported shapes, grammar,
limits, arithmetic boundaries, unsupported features and deterministic mutations.
These are local parser tests, not a VMware compatibility qualification or fuzzing
campaign. [Logical-reader qualification](vmdk-logical.md#reference-qualification)
records hosted reference comparisons and the custom/NUL-padding limitations. ESXi
is not needed; the 60-day trial remains reserved for V0's live-access proof.


R4.5 keeps these text parser rules unchanged. The separate
[DescriptorText acquisition API](vmdk-padding.md) accepts bounded terminal NUL
padding, preserves original bytes and passes only the text prefix to this parser.
LocalResolver and the CLI can therefore open supported padded descriptors without
file normalization; direct Descriptor::parse calls still reject any NUL.
