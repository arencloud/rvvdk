# Bounded streamOptimized metadata admission — R5.10

`rvvdk-vmdk` now has separate `StreamDescriptor`, `StreamHeader`, `StreamMarker`
and `StreamEnvelope` APIs. An envelope proves only the bounded structural checks
listed here. It is not a grain map, `BlockDevice`, logical reader or decompressor.
The public CLI and existing sparse parsers continue to reject compressed disks.
[Decision](adr/0052-bounded-stream-envelope.md),
[validation and performance](benchmark-results/2026-10-02-r510/README.md).

## Supported structural subset

| Field | Admission rule |
|---|---|
| Sparse header | Exactly 512 bytes, magic KDMV, version 3, clean shutdown, reserved bytes zero |
| Flags | Compressed grains and markers required; only newline-check and redundant-directory bits additionally allowed |
| Geometry | 64 KiB grains, 512 entries per grain table; nonzero capacity divisible by grain size |
| Compression | Algorithm identifier 1 only; no compressed framing or content is checked yet |
| Descriptor | Embedded bounded text, descriptor version 1, `streamOptimized`, one SPARSE base extent, no parent |
| Front profile | Primary and optional redundant directory regions before aligned overhead; no overlap with descriptor or each other |
| Footer profile | Initial `gdOffset = u64::MAX`, no redundant directory; trailing footer marker/header/EOS and adjacent directory marker/region |
| Header/footer agreement | All meaningful common fields agree; footer resolves GD offset, RGD is ignored when its flag is absent |
| Markers | Bounded grain LBA, payload size and sector-rounded position; exact GT/GD/footer counts; terminal positions checked |

The front profile is independently observed in QEMU output. The footer profile
follows VMware's published layout and is observed in the retained ESXi export.
This is a strict subset, not acceptance of every legal streamOptimized variant.
In particular, unaligned virtual capacity, alternate grain sizes, unknown flags,
dirty disks, parents and nonzero reserved header bytes fail explicitly.

The stream descriptor parser shares the bounded lexer/layout checks with the
existing parsers through a compile-time mode. Its public type cannot be passed to
`SparseDisk`; the existing `CreateType` enum is unchanged. The additional
`ddb.toolsInstallType` key is admitted only in this mode as bounded quoted,
informational text. Duplicates still fail; its value has no operational effect.
Embedded filenames remain untrusted lexical references and are never followed.

## Limits and acquisition

| Default limit | Value |
|---|---:|
| Virtual capacity | 1 TiB |
| Encoded extent length | 2 TiB |
| Pre-data overhead | 128 MiB |
| Directory entries | 32,768 |
| Compressed bytes per grain marker | 128 KiB |
| Envelope read budget | 2 MiB |
| Descriptor bytes, including terminal NUL padding | 1 MiB |
| Descriptor line / filename | 8 KiB / 4 KiB |
| Descriptor metadata entries | 128 |

`StreamEnvelope::read_from` accepts a caller-owned `Read + Seek` source and its
observed encoded length. It checks length, reads the initial header, validates
geometry and the complete read budget before descriptor allocation or following
offsets, parses the descriptor, checks the footer envelope when applicable, and
rechecks the header and length. Arithmetic and region ends are checked.

The front profile requests `1024 + descriptor_bytes`; footer acquisition adds
2048 bytes for the three terminal sectors and directory-marker sector. It does
not read directory entries or tables. The retained 30 GiB ESXi image requests
3584 bytes; the synthetic 1 TiB footer case requests 13,312 bytes because its
descriptor occupies 20 sectors. These are requested-byte counters, not process
RSS, disk traffic or cache measurements.

The caller must keep the source quiescent. Header/length rechecks detect some
changes but provide no snapshot or lock and cannot prove descriptor/tail stability.
Envelope acquisition uses bounded descriptor storage and fixed-size stack buffers;
it does not allocate in proportion to virtual disk capacity.

## Deliberate validation boundary

Directory pointers, table contents, redundancy agreement, global marker ordering,
physical aliases and compressed payloads are unvalidated. A valid marker prefix
does not prove that a scan can reach it. Marker padding is ignored as specified.
Tests intentionally admit an envelope containing a bad directory pointer to make
this boundary explicit. No result from these APIs authorizes logical reads yet.

R5.10p [recovers the measured old descriptor cost](benchmark-results/2026-10-02-r510p/README.md),
while retaining unresolved stream timing observations. The separate
[R5.11a map validator](vmdk-stream-map.md) now checks followed pointers, ownership,
aliasing, redundancy, record ordering and LBA binding under aggregate limits.
The envelope API itself retains the validation boundary above. R5.11b must bound
compressed input/output and specify framing, truncation, trailing input and sparse
zero semantics. Differentially compare logical bytes with authored RAW and QEMU,
then use the private guest oracle. Measure sequential/random reads, CPU/RSS and
throughput. R5.12 separately qualifies public CLI integration.

## Reproduce offline qualification

```sh
cargo build --release -p rvvdk-vmdk --example inspect_stream_envelope
cargo build --release -p rvvdk-cli
python3 scripts/vmdk/compare_stream_envelope.py \
  --inspect target/release/examples/inspect_stream_envelope \
  --cli target/release/rvddk \
  --directory target/stream-reference-new \
  --report target/stream-reference.json
cargo bench -p rvvdk-vmdk --bench stream
```

Use a new fixture directory. Python only generates local fixtures and orchestrates
QEMU/Rust reference checks; production decoding remains Rust. Three QEMU front
images and three authored footer layouts using QEMU compressed records decode to
their original RAW bytes. Rust admits their envelopes; it rejects an unaligned
capacity and the public CLI rejects all seven compressed images. Guest content,
credentials and exported image digests are excluded from committed evidence.

Format reference: VMware's [Virtual Disk Format 5.0 technical note](https://github.com/vmware/open-vmdk/blob/master/vmdk_50_technote.pdf),
header/table definitions on pages 6–9 and stream records on pages 11–13.
The [QEMU image utility](https://www.qemu.org/docs/master/tools/qemu-img.html)
is an offline reference only; no SDK or QEMU implementation code is incorporated.
