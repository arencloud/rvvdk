# ADR-0047: Bounded independent Rust vSphere discovery

- Status: Accepted for V0.2; export and production integration pending
- Date: 2026-10-01

## Decision

Implement direct ESXi 8.0.3 / HostAgent 8.0.3.0 authentication and inventory in the
new `rvvdk-vsphere` crate. Keep the local DataMover, format crates and public CLI
independent of networking. General-purpose reqwest, Rustls, roxmltree and Tokio
supply infrastructure; no VMware SDK, VDDK FFI or Python runtime is involved.

Use explicit leaf-certificate SHA-256 trust, verified handshake signatures,
HTTPS authority confinement and no redirects, proxies, compression or retries.
Default to persistent HTTP/1.1 with one idle connection. Measure fresh/reused
connections with identical Rust source and discovery work before accepting that
policy. Do not claim disk performance from control-plane timings.

Bound responses, XML nodes/depth/text, object counts, disks, request duration and
whole-discovery duration. Reject unsupported pagination and attempt to release
its continuation cursor. Separate primary failure, cursor cleanup and Logout;
never report success if required cleanup is unconfirmed. A received Login cookie
must survive response parsing errors long enough to attempt Logout. Explicit
awaited cleanup covers internal errors; dropped futures/process crashes remain
unqualified and cannot be fixed by a Rust Drop assertion.

Managed-reference `type` attributes are unqualified XML attributes; schema
`xsi:type` is distinct. Match exact namespaces rather than the parser library's
local-name attribute shorthand. Keep the first failed live discovery as evidence,
with a regression fixture and the failed source identity.

## Consequences

V0.2 qualifies discovery and authenticated error cleanup only. Available-license
metadata is explicitly unresolved for active assignment/operation eligibility.
VM identity stays private and must be revalidated on the admitted host before any
future mutation. Powered-off HTTP NFC export, disk bytes, cancellation/recovery,
least-privilege roles, additional versions and vCenter need their own qualification.
No VM power or licensing change is needed here.

[Contract](../../crates/rvvdk-vsphere/README.md),
[V0.2 evidence](../benchmark-results/2026-10-01-v02/README.md),
[export plan](../vmware-access-plan.md).
