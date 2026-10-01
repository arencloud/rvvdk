# ADR-0046: First independent VMware access candidate

- Status: Accepted for feasibility scope; transport qualification pending
- Date: 2026-10-01
- Scope: V0.1

## Context

Local VMDK and parent-chain readers are qualified within their documented subset.
They do not establish a VMware wire transport. An authorized standalone ESXi lab
now provides two disposable Fedora VMs. Independent read-only SOAP discovery works;
export licensing, data representation and lease cleanup remain unproven.

## Decision

Select powered-off HTTP NFC export as the first candidate for an independent Rust
proof. Implement bounded control-plane session/inventory support first (V0.2), then
qualify export, known bytes and cleanup (V0.3). Preserve random-block/NBDSSL research,
online snapshots, CBT, import and vCenter as separate scopes. No VDDK FFI fallback.

Represent export data as sequential container streams and artifacts, not seekable
logical disks. Compressed streamOptimized output needs separate Rust decoder work
before production RAW conversion. A reference decoder may validate lab artifacts
but is not a shipping implementation substitute.

Use exact endpoint/certificate policy, in-memory secrets, bounded SOAP and payloads,
explicit remote-resource ownership and complete/abort/logout results. Do not infer
license permission from login, available-license inventory or powered-on disabled
methods. No licensing changes or guest shutdown are required for V0.1.

## Consequences

The first proof has a documented API and a small resource lifecycle to test, but
its success qualifies export only. API discovery alone cannot close V0 or enable
R6 production disk access. VM/container identity, licensing, representation, known
bytes and failure cleanup remain acceptance gates.

[Design, primary sources and acceptance matrix](../vmware-access-plan.md).
[Sanitized live observations and plots](../benchmark-results/2026-10-01-v01/README.md).
