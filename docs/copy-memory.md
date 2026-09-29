# Copy memory budget (R1.6)

Every DataMover invocation has a **256 MiB default payload budget**. Configure
it with `CopyOptions::with_memory_budget(bytes)`; zero is allowed. The limit
covers the total accounted copy storage across buffers, queue/worker entries,
and extent metadata. It is an admission limit, not a process RSS limit or a
reservation of available RAM. A configuration that previously attempted an
arbitrarily large allocation can now be rejected before execution.

```rust
use rvvdk_datamover::{CopyOptions, DataMover};

let options = CopyOptions::with_execution(64 * 1024, 4096, 4, 8)?
    .with_memory_budget(64 * 1024 * 1024);
let mover = DataMover::new(options);
let plan = mover.plan_with_destination(&source, &destination)?;
let usage = mover.execution_memory(&plan)?;
println!("execution payload requirement: {} bytes", usage.total_bytes());
mover.execute_plan(&plan, &source, &destination)?;
```

The example assumes source/destination implement VirtualDisk and a Result-returning
caller. `execution_memory` returns CopyMemoryUsage with extent, buffer, queue,
and total byte getters. It uses the **executing mover's configuration**, including
native queue depth; it does not reserve resources or validate endpoints. Live
extent revalidation can exceed this execution requirement.

## Accounting and lifetimes

All arithmetic is checked. Exact limits succeed; required bytes greater than the
limit return `Error::MemoryBudgetExceeded { phase, required, budget }`.
Unrepresentable totals return `Error::MemoryAccountingOverflow`. These are
preparation rejections with no CopyFailure progress wrapper. Earlier endpoint
or configuration checks can still reject first.

| Storage | Charge |
|---|---|
| Extent metadata | Each retained Vec's **capacity**, including unused slots, times size_of::<Extent>() |
| Threaded buffers | buffer_count × (block_size + size_of::<AlignedBuffer>()), including idle pool buffers |
| Worker scheduling | For concurrency > 1: (queue_capacity + concurrency + 1) WorkItems, plus concurrency WorkerStats entries |
| Native Data buffers | queue_depth × (block_size + size_of::<AlignedBuffer>()) |
| Native operation entries | queue_depth × size_of::<(u64, InFlightOperation)>(), plus one CompletedOperation |
| Native sparse-only fallback | One block_size buffer; reserved conservatively even if every sparse operation is accelerated |

The worker formula includes active items and a producer blocked on send. These
logical entry charges are conservative about items on the stack, but exclude
opaque container overhead. Native submission/completion ring mappings are kernel
resources, excluded below. As of R2.5, Data jobs borrow a pool buffer for sparse
write fallback; sparse-only jobs reserve one scratch block. No separate scratch
payload overlaps a Data pool.

1. **Planning:** inspect and charge the returned extent Vec before retaining the
   plan. Planning does not reserve executor resources. A plan may fit while
   execution under the same budget does not.
2. **Preparation:** check the known execution requirement before querying fresh
   extents. Charge the retained plan **plus** the returned live revalidation Vec.
   Drop that temporary Vec before executor preparation.
3. **Execution:** charge retained plan metadata plus executor storage. Concurrent
   workers borrow the plan's extent slice, removing the previous scheduling clone.
   Native preparation retains a separate plan Vec: check the expected clone
   before allocation and its actual capacity after allocation. Both Vecs remain
   charged while native payload execution runs.
4. **Direct portable copy:** one extent Vec remains live with buffers and worker
   scheduling; there is no additional structural revalidation Vec.
5. **Runtime Auto fallback (R2.5):** after admitted native setup is unavailable,
   release the native plan clone and separately admit the Threaded buffers/queue
   before observation. Native planning admission is still required.
6. **Direct native range copy:** charge the native pool and operation entries;
   there is no extent plan. An empty native range has zero accounted storage.

Empty threaded execution still constructs its configured pool and therefore
requires that budget. All DataMover copy/report/plan/observed/RAW/native entry
points enforce the relevant checks. Budget rejection precedes payload I/O,
flush, and observer callbacks. Existing failure progress and success flush
boundaries remain unchanged. Planning can perform endpoint/metadata inspection.

## Explicit exclusions and remaining limits

- The Vec extent API allocates **inside the backend before returning**. Its actual
  capacity can only be checked after the query. An oversized result is dropped
  and rejected; the budget cannot prevent that transient allocation. Backend
  caches and working allocations are external. Fragmentation still increases
  total extent memory. A streaming or budget-aware query API remains future work.
- Allocator headers, padding/rounding, Arc/Mutex/channel control structures,
  hash-table spare buckets/control bytes, thread handles/stacks, and general
  runtime/error bookkeeping are excluded. Queue charges bound logical payload
  entries, not the exact storage used by Rust's opaque collections.
- R2.6 includes the owned admission guard in each native operation entry. Shared
  per-file admission vectors/registry capacity and coordinator allocations are
  backend bookkeeping outside the payload budget; admission is not a process
  reservation. Quarantines also retain their conflicting range admissions.
- Kernel io_uring mappings/requests, OS page cache, filesystem/device buffers,
  disk backend contents (including MemoryBlockDevice), and observer-owned
  allocations are external. Availability of RAM or native resources is not
  guaranteed by admission.
- The limit is per invocation, not a shared process reservation. Caller-retained
  plans/clones and simultaneous copies have separate lifetimes. A supplied plan's
  extent storage is charged to that invocation even though the caller owns it.
- Safety quarantine can retain native resources permanently after unconfirmed
  completion. Subsequent copies do not charge earlier quarantines. Existing
  native shutdown diagnostics remain essential; no process-wide cap is promised.
- Public low-level BufferPool/io_uring functions accept no CopyOptions and retain
  their existing validation/allocation contracts. Use DataMover for this budget.

The default is a conservative admission policy, not a benchmark-derived optimum.
Buffer sizes, worker count, queue capacity, and native queue depth/read window
remain unchanged. R2.5 [prepares and reuses native resources per job](native-runtime-preparation.md);
[its measurements](benchmark-results/2026-09-29-r25/README.md) record the effect. [R1.6 measurements](benchmark-results/2026-09-29-r16/README.md)
record admission overhead and the removed concurrent clone.


## Verification buffers (R3.2)

`Verifier::new(length, block_size, budget)` admits two Vec payloads of requested
size `min(length, block_size)`, checking expected storage before allocation and
actual capacities afterward. Empty comparisons allocate no payload. CLI
`copy --verify` retains these buffers throughout the invocation and subtracts
`Verifier::storage_bytes()` from the budget passed to every DataMover phase.
Standalone verify admits only its comparison buffers. This shares the exclusions
above, including allocator metadata and backend working allocations; it is not an
RSS limit. See [the transfer contract](cli-transfer.md).


## Lifecycle bookkeeping (R3.3)

Controlled copies retain a fixed lifecycle state and, for multiple workers, one
mutex-protected cumulative WorkerStats aggregate. No event history or extra
per-block queue is retained. These control objects and consumer callback/output
allocations are runtime metadata outside the payload budget, as with existing
worker handles and locks. Native cancellation retains the same shutdown/quarantine
ownership rules and budget limits. See [the lifecycle contract](cli-progress.md).
