# Bounded native streamOptimized reads — R5.11b

`StreamDisk` implements read-only `VirtualDisk` over the
[R5.11a validated grain map](vmdk-stream-map.md). It retains the exact
`Arc<dyn BlockDevice>` used for admission and never opens an embedded filename.
It supports the admitted base front/footer profiles, including the retained
ESXi export. R5.12 exposes the admitted subset through [local CLI commands](cli-vmdk.md).
[Decision](adr/0054-bounded-native-stream-reads.md),
[qualification and plots](benchmark-results/2026-10-02-r511b/README.md).

## Framing and integrity

Each allocated grain must contain exactly one **zlib-wrapped DEFLATE** stream
with a valid Adler-32 checksum, consuming exactly its marker's compressed length
and producing exactly 65,536 bytes. Bare DEFLATE, gzip, preset dictionaries,
concatenated streams, trailing input, truncated input, checksum failures and
short/oversized output reject. Sector padding is outside the compressed length
and is not decoded. The grain's LBA/length prefix is reread and compared with the
map before each uncached decode.

VMware's [technical note](https://github.com/vmware/open-vmdk/blob/master/vmdk_50_technote.pdf)
identifies DEFLATE/RFC 1951; the strict zlib wrapper requirement comes from the
independently observed QEMU and retained ESXi grains. This does not claim all
possible producer framing. There is no automatic raw-DEFLATE fallback.

The decoder uses the Rust `miniz_oxide` 0.9.1 core API with default features
disabled. It uses caller-provided fixed storage and no growing decode buffer or
C/vendor SDK. [Core API documentation](https://docs.rs/miniz_oxide/0.9.1/miniz_oxide/inflate/core/fn.decompress.html)
defines the returned completion/input/output counters; all three are checked.
The complete bounded input is supplied in one call. A non-wrapping 65,537-byte
output buffer detects oversized output without permitting unbounded expansion.
The checksum remains enabled. `adler2` is the only new transitive dependency.

Admission validates metadata, not every payload. Checksums are checked lazily
when a grain is read. Successful range reads validate their touched allocated
grains; they do not establish integrity of untouched payloads. The full reference
comparison in qualification separately drives every logical byte.

## Ownership, cache and logical behavior

The disk owns the validated source handle and immutable map. A single mutex
protects one reusable decoder/input/output slot and one last-grain cache. Data
reads serialize through it; caller concurrency cannot multiply scratch memory.
Zero-only reads need no decode lock or source I/O. No parallel-decode throughput
claim is made. Any cache replacement invalidates the old key before I/O, prefix
checks or decompression, so failures cannot expose partially overwritten data
under an old cache key. Only a fully validated grain is copied to caller output.
A multi-grain failure may leave earlier successful/zero portions of the caller's
buffer modified; discard that buffer on error.

Absent base grains read zero. Reads may start/end inside a grain and cross data/
zero boundaries. `extents()` coalesces logical Data/Zero runs, iterates present
records instead of every virtual grain, and allocates only after checking the
coalesced output count. A compressed all-zero grain remains Data. Writes, discard,
zero writes and flush reject. There is no RAW/native-file execution endpoint for
logical compressed contents.

Callers must keep the source quiescent throughout use. `revalidate()` invalidates
the cache, refreshes source size/access/identity, and checks the semantic header;
call it before read sessions. Copy preflight invokes it. Alias checking compares
destination identity with the retained physical source and rejects unknown
identities. A logical endpoint does not expose container identity as logical-byte
identity. These checks provide no lock, snapshot, complete map reread, checksum
of the whole source or durable content identity. Cache contents can be stale if
the caller violates quiescence. Buffered arbitrary byte-range reads are required
from the backend; its own allocations remain outside this layer's accounting.

## Limits

| Default | Scope |
|---|---|
| Existing `StreamMapLimits` | Metadata admission; map memory/read/work/grain bounds remain unchanged |
| 512 KiB decode memory | Requested input/output slots plus `size_of(DecodeState)` |
| 64 MiB request | Maximum buffer length for a nonempty logical read |
| 256 MiB encoded reads | Sum of complete touched grain prefixes/payloads per call, conservatively including cache hits |
| 1,025 decoded grains | Touched populated grains per call, conservatively including cache hits |
| 65,536 output extents | Maximum coalesced entries returned by one extent query |

The input allocation fits the largest admitted record plus its 12-byte prefix.
Input/output vectors use fallible exact reservation; neither grows during reads.
Decoder state is inline and fixed size. Decode accounting excludes the separately
bounded map, allocator/mutex overhead, transient stack, backend memory and caller
buffers. It is not peak process RSS. Loading adds a fixed 512-byte header recheck
beyond map read accounting. Output extent storage is separately bounded by its
count. Request/work limits are checked before source I/O or caller-buffer changes;
range errors also precede mutation. A single read does not allocate in proportion
to virtual capacity or compressed expansion ratio.

## Qualification and next step

Nine focused tests cover independent stored/zlib fixtures, malformed framing,
checksum/size/ratio bounds, sparse and cross-grain reads, exact resource limits,
cache failure/revalidation, partial backend reads, alias/endpoint checks, large
holes and concurrent callers. QEMU reference images match authored RAW bytes.
The full retained 30 GiB export agrees with QEMU decoding, and every byte of the
independently mapped 8 MiB guest fixture agrees. Private images, mappings, paths,
credentials and content digests remain outside Git.

```sh
cargo test -p rvvdk-vmdk --test stream_disk
cargo bench -p rvvdk-vmdk --bench stream_reads
cargo build --release -p rvvdk-vmdk --example stream_read_probe
# Historical R5.11 checks require an archived pre-R5.12 CLI.
# Generate the corpus with compare_stream_envelope.py and that CLI first.
python3 scripts/vmdk/compare_stream_reads.py \
  --probe target/release/examples/stream_read_probe --cli target/r512/reference/rvddk-before \
  --directory target/stream-reference-new --report target/native-stream-reads.json
```

`stream_read_probe` compares against RAW or an independent range oracle and runs
allocated-grain sequential / 4 KiB random workloads. It reports aggregate counters,
never decoded content or disk identities. It is a qualification helper, not a
public conversion command. Python generates synthetic compression fixtures and
orchestrates references/plots; VMware access and logical reads remain Rust.

**R5.12 is complete:** confined local CLI acquisition and inspect/plan/copy/verify
are qualified in the [CLI contract](cli-vmdk.md). Use `compare_stream_cli.py` for
current CLI conversion checks; historical envelope/read scripts deliberately
retain their pre-integration CLI rejection gate. Retain PERF.0 and prior stream
timing follow-ups. R5.12p next qualifies space-efficient zero output; then R6.1
defines artifact identity and durable ownership before
production VMware workflow integration.
