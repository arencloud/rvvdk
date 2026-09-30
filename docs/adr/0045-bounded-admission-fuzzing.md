# ADR-0045: Isolated, bounded admission fuzz qualification

- Status: Accepted
- Date: 2026-09-30
- Scope: R5.9

## Context

Deterministic tests cover supported descriptor/header/metadata and parent-chain
behavior, including production resource boundaries. Mutation-based coverage
feedback can explore additional malformed combinations, but must not widen
production limits, hide failure artifacts or introduce tooling into shipped crates.

## Decision

Use a standalone `fuzz` workspace with its own committed lockfile and pinned
qualification toolchain. Four libFuzzer targets exercise the unchanged production
parsers and loaders using authored, read-only in-memory devices. Raw parser/header/
metadata bytes and structured chain mutations provide complementary admission
coverage. No production `cfg(fuzzing)` override or dependency is introduced.

Bound input, fixture creation, parser collections, geometry, metadata read/reservation
budgets, chain depth and resolver work. Independently meter reads and parent
callbacks; verify invariants on accepted results and resource bounds after failures.
Apply ASan, checked arithmetic and process time/memory guards around the harness.

Retain every qualification run, fixed seed, command, executable/source identity,
raw feedback, resource log and learned-input archive. Stop on failures and preserve
original artifacts before minimization. Fuzz feedback is not proof of exhaustive
coverage, source coverage percentage or a logical-byte oracle.

## Consequences

Production builds remain unchanged. Smaller fuzz budgets permit fast exploration;
deterministic tests continue to cover full default limits and split/read semantics.
These targets do not qualify filesystem confinement, concurrency, live VMware
access, logical-byte equivalence or producer write behavior. Those require their
own tests and references. A finite clean campaign supports only its recorded scope.

Because this step changes no shipping code or dependencies, its performance evidence
measures sanitizer harness execution and RSS, with no matched disk-runtime claim.
Any production fix found later requires the normal matched performance checks and
adverse-pair repeat policy. See [harness contract](../../fuzz/README.md) and
[dated qualification and plots](../benchmark-results/2026-09-30-r59/README.md).
