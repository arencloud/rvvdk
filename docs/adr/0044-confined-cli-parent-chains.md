# ADR-0044: Explicit, confined CLI parent-chain acquisition

- Status: Accepted
- Date: 2026-09-30
- Scope: R5.8

## Context

The library validates sparse chain metadata and resolves inherited logical bytes.
Public commands previously rejected parents. Opening a parent expands the set of
files a command reads, observes and protects from destination aliasing. The CLI
must define the namespace and opt-in independently of the portable library.

## Decision

Add `--allow-parents`, requiring `--format vmdk`, to inspect/plan/copy/verify.
Use `SparseChain::load` and `SparseChainDisk` with their existing defaults. Keep
base-only acquisition unchanged without the flag. Chain mode supports sparse
bases, monolithic containers, split descriptors and validated external monolithic
mirrors, including mixed ancestry.

Pin the root source directory. Require every parent hint to be a basename in
that directory; use the existing Linux confined resolver for all descriptors
and backing references. Do not search, follow symlinks or silently widen access.
Retain timestamp/size/identity observations for all opened descriptors/backings.
Bind embedded entries to their own container identity before metadata loading.
Keep whole-chain alias checks, portable execution and existing copy lifecycle.

Report layer identities/CIDs and loader budgets separately from the four-byte
entry probes. Keep source metadata/FD costs separate from copy payload budgets.
The complete user-facing contract is [CLI parent chains](../cli-vmdk-parents.md).

## Consequences

Parent access is explicit and bounded. All layers have an unambiguous directory
namespace, but chains with relative parent subdirectories or absolute hints must
be staged into the supported layout; no descriptor rewriting is automatic.
Observation checks require quiescent sources and do not create snapshots.

[Integration tests and performance evidence](../benchmark-results/2026-09-30-r58/README.md)
qualify this policy without ESXi, VMware SDK code or producer implementation code.
Broader formats, namespace policies and live access remain separate work.
