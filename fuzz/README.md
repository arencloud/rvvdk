# Bounded VMDK admission fuzzing

This standalone cargo-fuzz workspace uses the production library unchanged.
Its lockfile isolates libFuzzer tooling from the shipping workspace. No VMware SDK,
producer implementation source, filesystem resolver or remote endpoint is used.
Authored fixtures come from the existing sparse metadata test support.

```bash
cargo install cargo-fuzz --version 0.13.2 --locked --root target/fuzz-tools
RUSTUP_TOOLCHAIN=nightly-2026-07-15 target/fuzz-tools/bin/cargo-fuzz build --target-dir target/r59-fuzz-build
python3 scripts/fuzz/qualify.py \
  --binaries target/r59-fuzz-build/x86_64-unknown-linux-gnu/release \
  --work target/my-fuzz-run --output target/my-fuzz-report --seconds 30 --repeats 3
```

Run from the repository root. Install the named nightly toolchain if unavailable.
The committed `fuzz/Cargo.lock` pins fuzz dependencies; root `Cargo.lock` is unchanged.
The qualified tool/compiler identities are in the [R5.9 report](../docs/benchmark-results/2026-09-30-r59/README.md).

## Targets and encodings

| Target | Input | Checks |
|---|---|---|
| `descriptor` | Raw text/bytes, maximum 65,536 bytes | FLAT/ZERO, base sparse and parent-capable parsers; contiguous extent arithmetic, collection limits, base/parent parser agreement |
| `header` | Eight-byte little-endian advertised extent length, then up to 512 raw header bytes | Bounded geometry and descriptor/directory ranges |
| `metadata` | Eight-byte advertised length, then raw physical prefix; maximum total input 65,536 bytes | Monolithic metadata and embedded binding against an authored base descriptor; reservations, read payload, grain-map length and endpoint revalidation |
| `chain` | Eight controls followed by up to 64 eight-byte replacement records; maximum 520 bytes | Metadata admission, identity/CID/capacity, parent resolution, depth/aggregate budgets and endpoint revalidation |

Metadata devices synthesize zeros beyond their prefix through their advertised
length, without allocating that length. This exercises malformed/huge advertised
lengths without a huge backing allocation. These are read-only in-memory devices;
a source write or flush is a harness failure. Reads and parent callbacks are
metered independently of the loader's own counters, including rejected loads.

Chain byte 0 selects 1–4 authored layers; byte 1 selects embedded/external entries
by layer bit. Byte 2 selects normal, missing, cyclic, shared-backing or unknown
identity behavior. Byte 3 selects normal, 1 KiB memory, 512-byte read or one-layer
limits; bytes 4–7 are reserved. Each record is `layer, region, offset_lo,
offset_hi, replacement[4]`. Layer and offsets are bounded by modulo. Regions
select raw header, embedded descriptor, both directories, primary table, both
tables or external descriptor. No mutation repairs format validation fields.
External entries are monolithic mirrors; the existing deterministic suite retains
split-layout and 16-layer boundary coverage. This target does not fuzz logical reads.

Loader limits are 16 MiB virtual capacity, 1 MiB grain, 64 directory entries,
64 KiB descriptor, 1 MiB metadata geometry, 4 MiB reserved/read metadata, 32 parser
extents and 16 DDB entries. Chains cap at four layers, eight total extents and
256 KiB acquired descriptors. These deliberately smaller fuzz limits complement
the production-default boundary tests; they do not qualify arbitrary resources.

## Running, replaying and failures

The runner copies committed seeds into a fresh corpus for each campaign. It sets
ASan quarantine to 64 MiB, enables leak detection, permits at most 1 GiB RSS and
64 MiB single allocations, and sets a five-second per-input timeout. Its outer
watchdog kills the process group after the requested duration plus 60 seconds.
These are process guards, separate from loader admission limits. All completed
runs retain commands, fixed random seeds, raw feedback, process resource logs,
input hashes and deterministic archives containing every learned corpus unit.
A crash, timeout, OOM, assertion or sanitizer error stops qualification and retains
its artifact. No failed run is silently replaced. Instrumentation covers the
library and harness; feedback counts are not source-line coverage percentages.

For interactive work, copy seeds to an ignored writable corpus first:

```bash
cp -r fuzz/seeds/chain target/chain-corpus
RUSTUP_TOOLCHAIN=nightly-2026-07-15 target/fuzz-tools/bin/cargo-fuzz run chain target/chain-corpus -- -max_len=520 -timeout=5 -rss_limit_mb=1024
```

Replay an artifact with the exact recorded binary, environment and flags, replacing
the corpus path with the artifact path. Minimize with `cargo-fuzz tmin TARGET
ARTIFACT` under the same nightly/build configuration, preserving the original
artifact and invocation first. Separate a harness mistake from a production bug;
add a deterministic regression before fixing the latter, then rerun affected
qualification and matched performance checks. [Rust Fuzz Book](https://rust-fuzz.github.io/book/cargo-fuzz/guide.html)
and [libFuzzer options](https://llvm.org/docs/LibFuzzer.html#options) explain the tooling.

Seed generation refuses an existing destination:

```bash
cargo run --manifest-path fuzz/Cargo.toml --example seeds -- target/fuzz-seeds
cargo test --manifest-path fuzz/Cargo.toml --lib
cargo clippy --manifest-path fuzz/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path fuzz/Cargo.toml -- --check
```

Only reviewed seeds are committed. Mutating corpora and artifacts normally stay
ignored; qualification archives are deliberately retained in the dated report.
