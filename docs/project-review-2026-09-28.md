# rvddk project review — 2026-09-28

## Assessment

The project has a substantial local data-movement foundation. It is currently a Rust library workspace for copying logical RAW disks between memory and local regular files, with sparse source discovery, reusable aligned buffers, bounded threaded scheduling, Linux direct I/O, io_uring, structural copy plans, and preliminary progress reporting.

It does not yet implement VMDK parsing, VMware sessions, snapshot orchestration, remote VMware disk access, CBT, or a CLI. The next investment should be correctness and a usable local VMDK workflow, together with an early investigation of independent VMware transport feasibility. More io_uring tuning alone will not close the VMware functionality gap.

The user calls the project **rvddk**; the repository, Cargo packages, imports, and environment variables currently use **rvvdk**. This review retains existing identifiers. Decide the public spelling before publishing crates or a CLI; renaming is a separate change.

Implementation sequence: [roadmap.md](roadmap.md).

## Scope and baseline

- Reviewed workspace manifests, production modules across all four crates, integration-test coverage and fixtures, benchmark implementations, architecture documentation, and all 24 ADRs.
- Baseline commit: `e932d88` — `Add copy planning and structural validation`.
- Working tree already contained changes to `lib.rs`, `mover.rs`, `observer.rs`, `progress.rs`, and `tests/progress_observer.rs`, plus untracked `benchmark-m11.txt`. Findings include this in-progress observer work. Those files were not edited during review.
- Inventory: 72 Rust files, 12,957 Rust source lines including tests/examples/benchmarks, 24 integration-test files, six Criterion benchmark targets, four crates.
- This is an architecture and code review with targeted experiments, not a complete unsafe-code proof, production qualification, or VMware interoperability test.

### Validation performed

| Check | Result |
|---|---|
| `cargo test --workspace` | 194 passed, 0 failed, 0 ignored; doc-test targets contain no tests |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed |
| Worker-error reproduction outside repository | Two-worker copy to a read-only memory destination did not return within 5 seconds |
| Observer validation reproduction outside repository | Ordinary execution rejected mismatched block size; observer execution succeeded |
| Low-level zero-block reproduction outside repository | Zero-extent fallback with block size zero did not return within 2 seconds |
| Native direct-I/O tail probe | 4,097-byte copy succeeded on this machine's tmpfs and Btrfs; does not prove alignment handling on other filesystems |

Environment: `rustc 1.95.0 (59807616e 2026-04-14)`, Linux `7.2.5-200.fc44.x86_64`. Tests use the temporary directory, which is tmpfs here; the repository is on Btrfs. Existing direct-I/O tests therefore do not establish real storage behavior on this run. No throughput benchmarks were rerun and no VMware lab was accessed.

## What exists today

| Component | Implemented strengths | Current boundary |
|---|---|---|
| `rvvdk-core` | Checked disk ranges; geometry; capabilities; Data/Zero/Hole extents; `BlockDevice` and `VirtualDisk`; exact-I/O helpers; `RawDisk`; memory backend; aligned buffers and RAII pool guards | Sparse/discard contracts need tightening; no disk identity, consistency token, change tracking, or format parser |
| `rvvdk-local` | Positional regular-file I/O; read-only opens; Linux sparse discovery; paired direct/buffered descriptors; `statx` alignment discovery | No zero-range/hole-punch implementation; no physical block-device support; Linux assumptions are incompletely gated |
| `rvvdk-platform` | Keeps Linux FD capability requirements outside generic disk traits | Bare FD and alignment values lack lifetime, validation, access-mode, and logical-offset guarantees |
| `rvvdk-datamover` | Sequential/threaded execution; bounded work queue; lazy block subdivision; native read/write pipeline; sparse semantic parity; structural planning and backend reports | Error termination, shared validation, native preflight, portable planned execution, and observer parity need work |
| Tests | Useful coverage of ranges, partial positional I/O, sparse destinations containing old data, queue depths, buffer ownership, and execution parity | Sparse coverage of injected failures, worker shutdown, cancellation, stale observer plans, unsupported runtime capabilities, and real filesystems |
| Benchmarks | Dense/sparse/direct/buffered workloads; deterministic data; configurable storage directory | Some comparisons use different flush boundaries; fragmented extent setup costs and physical allocation need measurement |
| Documentation | Clear original layering; 24 incremental architectural decisions | README status is obsolete; architecture mixes historical stages with current behavior; several ADRs are very short/incomplete |

The separation between logical disks, backing storage, and execution policy is worth preserving. So are the bounded queue, owned buffers, checked ranges, and tests against destinations prefilled with nonzero data.

## Findings requiring action

Priorities: **P0** blocks trusting the affected path; **P1** is required before a reliable local release; **P2** is an architectural or maintainability improvement. “Confirmed” distinguishes observed behavior or directly evident implementation from an unexercised risk.

### F01 — P0: threaded copy can deadlock after worker failures

Evidence: [`concurrent.rs`](../crates/rvvdk-datamover/src/concurrent.rs), `execute` and `run_worker`; [`mover.rs`](../crates/rvvdk-datamover/src/mover.rs), `copy_concurrent`.

`execute` keeps the original `Arc<Mutex<Receiver<_>>>` alive while the producer sends work into a bounded channel. Workers return immediately on an I/O error. If every worker exits and the queue fills, the producer blocks permanently: the receiver is still alive, but nobody consumes it. The enclosing scoped-thread operation cannot finish.

Confirmed with a 1 MiB memory source, a read-only destination, 4 KiB blocks, two workers, and queue capacity one. The call timed out after five seconds. Static ownership/control-flow inspection explains the hang.

Required fix: release coordinator receiver ownership, propagate the first worker failure to the producer, stop new scheduling, wake blocked participants, join workers, and preserve the originating I/O error over secondary queue-closure errors. Preflight access checks help, but injected mid-copy failures must also terminate.

### F02 — P0: borrowed io_uring buffers may outlive their Rust borrow on error

Evidence: [`io_uring/engine.rs`](../crates/rvvdk-datamover/src/io_uring/engine.rs), `read_at`, `write_at`, `wait_borrowed_completion`, and `drain_for_drop`.

Borrowed methods enqueue pointers into caller slices, then use `?` on submit/wait operations. An error after queuing or submission can return to the caller while an operation is still pending. The caller may reuse/free its buffer before the engine is dropped; draining in `Drop` cannot retroactively extend that borrow.

This is a safety concern established by control-flow inspection, not a reproduced use-after-free. Resolve it before relying on the public safe API. Prefer removing the borrowed compatibility path, or using engine-owned storage and an explicit completion/cancellation protocol. Audit owned-operation teardown too: prove when the kernel releases all buffer references, including failed waits and ring shutdown. Keep FD resources alive through completion rather than relying on bare integer handles.

### F03 — P0: observer execution bypasses plan validation

Evidence: [`mover.rs`](../crates/rvvdk-datamover/src/mover.rs), `execute_plan_with_observer` and `execute_threaded_plan_with_observer`.

For a threaded plan with concurrency one, the observer entry point directly calls a separate copy implementation. It skips the checks in `execute_plan`: source size, destination size, block-size match, extent fingerprint, and alignment.

Confirmed: plan with 4 KiB blocks; execute through a mover configured for 2 KiB blocks. `execute_plan` returns a mismatch error; `execute_plan_with_observer` copies successfully. A larger execution block size can also exceed the buffer allocated from the plan. A stale Hole map can silently clear bytes that have become Data.

Required fix: one shared preflight and execution implementation, with optional observation. Add negative tests through both entry points, asserting no destination modifications before validation succeeds.

### F04 — P1: native compatibility and Auto fallback are incomplete

Evidence: [`io_uring/compatibility.rs`](../crates/rvvdk-datamover/src/io_uring/compatibility.rs), `evaluate_compatibility`; [`mover.rs`](../crates/rvvdk-datamover/src/mover.rs), planning/native dispatch.

Compatibility currently sets `compatible: true` for every backend pair. It does not check runtime ring availability, required operations, access permissions, or whether planned offsets/lengths satisfy direct-I/O constraints. The separate runtime probe is not integrated into selection. Consequently, Auto's incompatible-pair fallback branch is effectively unreachable.

The native path submits the primary FD directly, bypassing `LocalFileBlockDevice`'s buffered fallback. Increasing buffer alignment does not align an odd transfer length or offset. A 4,097-byte probe happened to succeed on this machine; the missing request validation remains a portability risk, not a reproduced EINVAL here.

Required fix: prepare the executor before writes, validate both devices and every native request, expose selection/fallback reasons, and make Auto choose a viable path before mutation. Explicit IoUring should return a precise unsupported error. Never restart the whole operation on another backend after partial writes as an implicit fallback.

### F05 — P1: DISCARD does not express the zero-read guarantee needed for Hole copying

Evidence: [`capabilities.rs`](../crates/rvvdk-core/src/capabilities.rs), [`block_device.rs`](../crates/rvvdk-core/src/block_device.rs), and Hole handlers in all executors.

Hole copying chooses `DISCARD` whenever advertised, assuming subsequent reads are zero. The current memory and test backends deliberately zero discarded bytes, so parity tests cannot reveal a backend whose deallocation operation lacks that guarantee.

Required fix: distinguish general discard/deallocation from an operation guaranteed to produce zero reads. Use discard for Hole copying only under the latter contract; otherwise zero explicitly. Define `VirtualDisk::Hole` as a resolved logical zero range. An unallocated child VMDK grain may inherit parent data and must not become a logical Hole merely because the child has no allocation. Standard NBD likewise distinguishes trim and write-zeroes semantics in its [protocol](https://github.com/NetworkBlockDevice/nbd/blob/master/doc/proto.md).

### F06 — P1: local sparse reads are implemented; local sparse output is not

Evidence: [`rvvdk-local/src/file.rs`](../crates/rvvdk-local/src/file.rs).

The local backend advertises sparse extent discovery but neither `WRITE_ZERO` nor `DISCARD`, and implements neither operation. Copying a Hole into a local destination therefore writes zero buffers. Logical contents are correct, but the engine does not actively preserve/reclaim destination holes.

Add zero-range and hole-punch implementations with filesystem capability handling, correct partial-block behavior, range checks, and guaranteed read semantics. Validate allocated blocks as well as bytes. If extent discovery is unsupported by a filesystem, provide a correct dense-map fallback instead of making ordinary copies unusable.

### F07 — P1: planned execution is restricted to Linux RAW disks

Evidence: [`mover.rs`](../crates/rvvdk-datamover/src/mover.rs).

`plan` accepts a generic `VirtualDisk`, but destination-aware planning, `execute_plan`, reports, and observer execution require Linux `RawDisk<S/D>` with `LinuxFdBackend`. A future VMDK or remote logical disk cannot use that workflow. The generic `copy` entry point also ignores the configured native strategy by design, which needs clear public documentation.

Make generic planning and execution the stable user-facing path. Keep native acceleration as a separately constrained implementation. A VMDK's container FD must never be treated as if its physical offsets were guest logical offsets.

### F08 — P1: public low-level entry points need complete input validation

Evidence: [`io_uring/extent_copy.rs`](../crates/rvvdk-datamover/src/io_uring/extent_copy.rs), `copy_extent_plan_with_destination` and `write_zero_fallback`.

These public functions accept a raw `block_size`. With a Zero/Hole extent, no specialized destination operation, and `block_size == 0`, fallback writes an empty slice and advances by zero forever. The validation inside `CopyOptions` does not protect callers of these exported functions.

Confirmed using a 4 KiB Zero extent and a writable memory destination without `WRITE_ZERO`: the public low-level call timed out after two seconds.

Validate configuration before any operation, including empty/Data-only/Zero-only/Hole-only plans. Prefer one validated internal configuration type. Also preflight unsupported extent kinds before earlier extents can modify the destination in the older Data-only entry point.

### F09 — P1: completion, progress, and durability need one lifecycle

Evidence: [`progress.rs`](../crates/rvvdk-datamover/src/progress.rs), [`observer.rs`](../crates/rvvdk-datamover/src/observer.rs), [`mover.rs`](../crates/rvvdk-datamover/src/mover.rs).

Intermediate observation exists only for single-worker threaded execution; concurrent/native execution emits initial/final snapshots. Errors have no terminal progress event or partial report. A byte-only emission threshold can leave slow operations silent, and a dense transfer can reach 100% byte progress before the final flush.

Define Preparing/Copying/Flushing/Verifying/Completed/Failed/Cancelled states, completion-based counters, a time-based reporting interval, and an explicit observer callback context. Keep callbacks off worker hot paths; a coordinator can support the existing non-`Sync` closure observers. Separate written bytes from durable checkpointed bytes. Add cancellation and partial error reports after the immediate deadlock fix.

### F10 — P1: source consistency and endpoint identity are not established

Evidence: [`plan.rs`](../crates/rvvdk-datamover/src/plan.rs), [`mover.rs`](../crates/rvvdk-datamover/src/mover.rs), [`file.rs`](../crates/rvvdk-local/src/file.rs).

The extent fingerprint is structural, correctly described as such by ADR-0024. It does not bind the plan to a disk identity, freeze data, or detect content changes inside Data extents. Local geometry is cached at open; an already-open handle does not refresh it. The two direct/buffered opens do not verify identical inode identity. Same-file source/destination aliases are not rejected.

Add endpoint identity and geometry refresh/preflight, verify paired descriptors refer to the same object, reject unsupported source/destination aliasing, and require an explicit source-consistency policy. Snapshot-backed VMware jobs should carry immutable snapshot/disk identity. A stronger hash alone cannot create snapshot consistency.

### F11 — P2: resource scaling and duplicated logic will complicate extension

- Extents are fully materialized and cloned even though block work is streamed. Memory is bounded in work-item count, not in extent count.
- Each native Data extent creates another ring and buffer pool through `copy_file_range_with_options`; fragmented images pay repeated setup costs.
- Zero fallback in the native extent executor allocates an ordinary `Vec`, potentially losing direct-I/O alignment.
- Extent validation appears in both `extent_validation.rs` and `io_uring/extent_plan.rs`, despite ADR-0020 describing shared validation.
- Sequential, observed, concurrent, and native paths duplicate semantic decisions. Fixing F03 should remove the newest duplication first.
- Queue depth and worker count imply memory allocation without an explicit total memory budget.
- Core `Error` contains execution-specific io_uring variants. This can remain during stabilization, but a later error layering change should keep format/network/control-plane errors contextual without turning core into an exhaustive list of every backend.

Consolidate semantics while retaining distinct execution mechanisms. Introduce paginated extent discovery and persistent per-job resources only after correctness is stable and measurements justify them.

### F12 — P2: portability, release engineering, and measurement need explicit contracts

- No repository CI workflow is present. Establish Linux jobs with real io_uring/storage coverage and a portable core check.
- `rvvdk-local` uses Unix/Linux APIs without consistent target guards. `plan.rs` and `progress.rs` unit tests refer to the Linux-only IoUring enum variant without gating. Full workspace portability is not currently established.
- `rust-toolchain.toml` follows moving stable; no `rust-version` is declared. Declare supported Rust and OS versions rather than implying broad compatibility.
- `rvvdk-platform` does not inherit workspace license/repository metadata.
- `io_uring_direct.rs` compares a threaded mover that flushes with a low-level native copy that does not flush. Align durability boundaries before using those numbers for engine selection.
- Existing tests primarily cover successful operation; add bounded-time failure cases, meaningful fuzz/property tests, and malformed-input fixtures.
- Several historical ADRs/architecture sections describe superseded stages as if current. Preserve history but mark supersession and maintain one current architecture page.

## Immediate reproduction recipes

All experiments used a temporary Cargo package with path dependencies on the current workspace; production source was unchanged.

Worker hang:

```rust
let source = RawDisk::new(MemoryBlockDevice::new(1024 * 1024)?);
let destination = RawDisk::new(MemoryBlockDevice::read_only(1024 * 1024)?);
let options = CopyOptions::with_execution(4096, 4096, 2, 1)?;
DataMover::new(options).copy(&source, &destination)?;
```

Run this as a subprocess under a short timeout; do not make a regression test capable of hanging the entire test runner.

Observer bypass: create two 16 KiB local files; plan with `CopyOptions::new(4096)`; execute with `CopyOptions::new(2048)`. Compare `execute_plan` with `execute_plan_with_observer(..., &NoopProgressObserver)`. Current results are error and success respectively. Add stale-extents and larger-block variants when implementing the fix.

## Product gap to a VMware disk toolkit

| User capability | Current state | Next architectural requirement |
|---|---|---|
| Copy/inspect local RAW | Library available | CLI, robust errors, sparse destination operations |
| Read local VMDK | Absent | Descriptor parser, flat/sparse extent mapping, resolver, format fixtures |
| Read snapshot chains | Absent | Parent resolution, generation validation, inherited-data semantics |
| Discover VMware VM disks | Absent | vSphere control-plane client and target version matrix |
| Read VMware snapshot disk remotely | Absent | Independently verified transport and access lifecycle |
| Incremental backup | Absent | Separate changed-range model, CBT pagination, baseline identity |
| Resume interrupted copies | Absent | Durable journal and immutable source/destination binding |
| Restore safely | Local RAW writes only | Destination lifecycle, verification, explicit supported target types |
| VDDK C ABI compatibility | Absent | Separate optional facade after Rust behavior stabilizes |

Broadcom documents several access methods and distinct disk variants; implementing one format or transport is not a claim of complete VDDK compatibility. See [transport methods](https://developer.broadcom.com/xapis/virtual-disk-api/latest/vddkDataStruct.5.5.html) and [disk types](https://developer.broadcom.com/xapis/virtual-disk-api/latest/vddkDataStruct.5.3.html).

The roadmap treats independent VMware network access as a feasibility gate. The available documentation does not, by itself, establish that a standard NBD implementation can substitute for VMware's managed-disk transport.
