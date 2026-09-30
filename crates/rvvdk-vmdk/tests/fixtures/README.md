# Descriptor fixture provenance

These three text files were authored for rvvdk on 2026-09-29 under the repository's
Apache-2.0 license. They contain invented CIDs/names and small sector counts; they
are not copied from a guest, a VMware installation, or a specification example.
No backing data is bundled. Tests only parse text.

| File | Intended result |
|---|---|
| monolithic.vmdk | 8,192 bytes, one FLAT source, four informational DDB records |
| split.vmdk | 5,632 bytes across two FLAT sources, including RDONLY |
| custom.vmdk | 3,584 bytes across FLAT/ZERO/FLAT, nonzero backing offset and UTF-8 name |

Format facts were checked against VMware's [Virtual Disk Format 5.0, pp. 3–5](https://github.com/vmware/open-vmdk/blob/master/vmdk_50_technote.pdf),
downloaded from the VMware-owned repository on 2026-09-29. Only the PDF was read;
no implementation code was used. The inspected PDF SHA-256 is recorded below.
The original vmware.com URL linked by Broadcom returned 404; its VMware-owned
GitHub copy supplied the reference. The PDF has its own license and is not vendored.

The [subset contract](../../../../docs/vmdk-descriptor.md) documents intentional
restrictions. Negative fixtures and deterministic byte mutations are built in test
memory. They cover invalid input, not reference-decoder agreement. Once logical
reads exist, generate deterministic backing bytes, compare decoded RAW with a
recorded trusted tool/version, and retain commands and hashes. Do not label these
synthetic parser examples as ESXi-tested or VMware-generated images.
Inspected PDF SHA-256: `88ce1615a703d1d4e3df3c227846bb9ecda4d929129c191ea4ffe7df3294ad59`.

R4.3 adds `scripts/vmdk/compare_reference.py`, which generates deterministic
backing bytes and disposable descriptors in a caller-selected new directory.
It checks hosted monolithic/split fixtures against QEMU, preserves
QEMU-generated hosted descriptors and records any removal of trailing NUL padding
in fixture-only text copies, and separately checks custom FLAT/ZERO against
an independently assembled byte oracle. QEMU rejects createType custom; this is
recorded as a reference-tool limitation, never counted as reference agreement.
Version, commands, complete descriptor text and file hashes are saved in the
R4.3 evidence report. Generated disk data stays outside version control.


R4.5 updates the same runner to require success on the original generated hosted
descriptors: no fixture normalization or fallback. It records terminal padding
lengths and verifies descriptor bytes remain unchanged. Optional `--cli` also runs
inspect/plan/copy/verify on these originals. The R4.5 report records tool/executable
hashes and bytes; R4.3's earlier normalization evidence remains immutable. Unit
padding fixtures are synthesized in memory, including hostile suffixes and limits.

R5.1 adds an authored 512-byte hosted sparse header factory in
`tests/support/sparse.rs`, shared by parser tests and benchmarks. Field offsets and
units were checked against the same VMware Virtual Disk Format 5.0 PDF, pp. 6–9,
on 2026-09-30; the SHA-256 above is unchanged. No implementation code was read or
copied. `scripts/vmdk/compare_sparse_header.py` separately generates disposable
QEMU monolithic/split sparse files, records full header hex and tool/file hashes,
and compares admitted capacity with QEMU info. It also records deliberate subset
rejections and continued public CLI rejection. These are header-only checks, not
logical sparse byte comparisons or VMware-produced/ESXi-tested images.

R5.2 adds authored sparse descriptor/table fixtures in `tests/support/sparse_metadata.rs`.
They exercise a version-1 base map with two allocated grains, unallocated entries,
redundancy and embedded/external binding. The same VMware PDF pp. 3–9 supplies
field semantics; no producer implementation source was read. The separate
`compare_sparse_metadata.py` runner creates monolithic/split sparse images with
QEMU, reconstructs bytes from the validated map only in a test utility, and compares
with the original RAW oracle and QEMU. Production sparse reads remain disabled.

R5.3 reuses these authored maps for production SparseDisk boundary, alias and
budget tests and benchmarks. `compare_sparse.py` generates known patterned RAW
oracles and QEMU monolithic/split sources, then compares production logical reads
against both. The multi-file case crosses the 2 GiB split boundary. The dump helper
reads every logical byte in 65,537-byte chunks and leaves zero-only output chunks
unallocated; it is a qualification utility, not the public CLI publication path.

R5.5 adds `support/sparse_scale.rs`: authored metadata layouts for multiple grain
sizes, table boundaries, allocation patterns and virtual capacities. The synthetic
BlockDevice stores only metadata and generates physical-grain payload bytes from an
arithmetic pattern; it is not a real filesystem or allocated multi-GiB image.
Expected logical bytes come from the authored allocation rule, independently of
SparseDisk's map. Tests use fixed-seed mutations and bounded-read instrumentation.
Benchmarks disable instrumentation and include generated payload costs. No producer
implementation code or new disk-format acceptance is introduced.
