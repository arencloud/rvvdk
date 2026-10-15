# PERF.1c predeclared qualification

Reference: `fb4ea8d` (accepted PERF.1b), with identical benchmark-only additions
in both builds. Candidate: reuse the private conditional SHA-256 facade at four
encoded-payload hash sites. Linux x86-64 without SHA-NI selects ring; accelerated
and other-platform paths retain sha2. No dependency or public API change.

## Boundary audit

| Site | Bytes and gates preserved | Representative existing checks |
| --- | --- | --- |
| `export::download` | All encoded HTTP bytes; status/encoding/length/chunk limits, KDMV prefix, SHA-1, buffered write, flush/sync; outer lease heartbeat/deadline/cancellation and server manifest | `export_tests` transport, manifest, deadline and cleanup cases |
| `owned_transfer::download` | All encoded bytes; same transport gates, SHA-1, bounded worker queue/write acknowledgment, expected digest handed to independent seal | `owned_transfer::tests` pending write cancellation, write failure, heartbeat and durable completion boundary |
| `journal_worker::payload::seal` | Sync/close sole writer, freshly checked descriptor, complete SHA-256/SHA-1 readback, length and namespace/identity checks; outer worker drains seal before abort | `journal_worker::tests` payload limits/readback; owned-transfer seal cancellation tests |
| `journal_worker::admission` | Native map admission, every present grain decode, fresh full encoded rehash with chunk cancellation/deadline checks, source/identity binding, metadata sync/readback/reparse | `admission::tests` valid layouts, corrupt grains/structure/capacity, changed sealed bytes, cancellation and metadata fault boundaries |

No earlier digest substitutes for a new pass. No format, chunk size, SHA-1,
allocation, ownership, manifest, lease, journal or persistence behavior changes.
The portable export path must retain its existing non-Linux backend.

## Measurements declared before timing

Identical frozen release binaries execute natively on the SHA-NI development
host and existing non-SHA runner. Same compiler, lockfile, default flags and
CPU 0 affinity. Compare versions only within a host/current filesystem. Keep
source/binary hashes, sanitized environment, every sample, initial/repeat spread,
CPU, wall, process-lifetime peak RSS, allocated bytes, cleanup and release size.

Primary filesystem: proof/owned transfer, owned export/artifact admission, and
worker seal/admit phases; each has `small`, `bulk`, `zeros` cases. Transfer small
is 64 KiB and bulk/zeros 8 MiB; native fixtures contain one or 128 stored 64 KiB
grains in a sparse 30 GiB logical VMDK. Zero grains are present stored records,
so the encoded-byte verification work remains. Full encoded-byte equality and
independent RustCrypto digests remain outside timers; native integration also
checks every authored logical grain and absent zero ranges. Worker seal includes
sync and fresh readback; admission includes native decoding, rehash and metadata
persistence. Those are phase measurements, not pure hash throughput. The accepted
PERF.1b native digest probe remains the isolated hash evidence.

For every group: three initial rounds of two pairs, then three repeat rounds of
four pairs, with alternating binary order by round and operation order by pair
where possible. Each binary invocation contains the same number of operations.
A private tmpfs `huge=never` bulk control repeats all three groups with the same
schedule. Do not run unrelated local builds/tests during local timing. Normal
page cache; no cold-cache or disk-scale claim. Worker admission follows seal in
its required order; fixture/oracle overhead is outside phase timers. RSS includes
fixture setup and TLS server, not an isolated production working set.

Measure CLI `--help` startup separately using three initial rounds of four pairs
and three repeat rounds of eight pairs; child peak RSS may include Python memory
inherited before exec. No inferred live-network speedup from loopback TLS.

Adoption requires all correctness gates and independent oracles to pass, repeated
bulk benefit on the non-SHA runner, and investigation of >5% repeat-median wall or
CPU regressions on the SHA-NI host/small cases. Retain adverse results and any
additional controls separately; never replace initial samples. Wider variance
remains PERF.0. A failed gain gate permits a documented negative result.

Run formatting, all-target Clippy and workspace tests, plus the frozen native
candidate library and affected integration fault groups on the runner. Inspect
plots, replay evidence checks, verify final hashes, audit staged privacy, update
planning and commit locally. No ESXi API, live export, VM changes or push.
