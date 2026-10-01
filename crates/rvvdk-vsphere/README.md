# rvvdk-vsphere: bounded Rust discovery

V0.2 implements independent SOAP authentication and inventory for direct ESXi
8.0.3 / HostAgent API 8.0.3.0. It uses general-purpose HTTP, TLS and XML crates,
not a VMware SDK or VDDK. It does not export disks or mutate VMs. Local disk crates
and the public `rvddk` CLI do not depend on this crate.

## API and trust

`ConnectionPolicy::pinned` admits one HTTPS authority and `/sdk`; embedded
credentials, other paths, query strings and fragments are rejected. The caller
supplies a SHA-256 leaf-certificate pin. Rustls verifies the exact certificate
and the TLS handshake signature before application data. Pin mode replaces
CA-chain, hostname and expiry checks: obtain the pin through an independently
trusted channel. No automatic trust, certificate bypass or fallback is exposed.
The current lab's original pin was observed on first contact (TOFU).

HTTP/1.1, TLS 1.2/1.3, no client certificate, no early data or TLS resumption.
Redirects, proxies, HTTP compression and retries are disabled. The default reuses
one idle connection; `ConnectionReuse::Fresh` exists for matched qualification.
Cookie headers are marked sensitive and sent only to the admitted endpoint.

```rust,no_run
use rvvdk_vsphere::{discover, ConnectionPolicy, Credentials, InventoryLimits};

# async fn example(endpoint: &str, certificate_sha256: &str, user: String, password: String) -> Result<(), rvvdk_vsphere::Error> {
let policy = ConnectionPolicy::pinned(endpoint, certificate_sha256)?;
let credentials = Credentials::new(user, password)?;
let report = discover(policy, &credentials, InventoryLimits::default()).await?;
if report.is_success() {
    // Both discovery and explicit Logout succeeded.
    let inventory = report.inventory.as_ref().unwrap();
    assert_eq!(inventory.about.api_type, "HostAgent");
}
# Ok(())
# }
```

A Tokio runtime with I/O and time enabled is required. `Credentials`, endpoint/pin
and VM identity Debug output are redacted; VM references/UUIDs are excluded from
serde. Callers can explicitly access a VM's identity for later revalidation on the
same admitted host. BIOS UUIDs and enumeration labels alone are not sufficient
identity for a future destructive operation. Do not log credential-bearing input.

Credentials and owned cookie/response buffers use `Zeroizing`; copies inside the
HTTP/TLS/XML infrastructure are not guaranteed to be erased. Reports select
nonsecret inventory fields and closed error codes. Raw SOAP, fault messages,
license keys, names, paths, cookies, host addresses and pins are never written by
the example. `available_editions` is **not an active-license assignment or operation
eligibility check**; the report explicitly says `active_assignment: unresolved`.

## Limits and cleanup

| Resource | Default / hard admission bound |
|---|---|
| Control response | 2 MiB encoded UTF-8 body; checked advertised and received length |
| XML | No DTD/custom entities; 16,384 nodes, depth 64, text node 64 KiB |
| Inventory | 128 visited objects and queued references; lower caller limit allowed |
| VM disks | 16 per VM; lower caller limit allowed |
| License entries | 128 |
| Credentials / managed references | 1 KiB per credential field / 256 bytes per reference |
| Connect / read inactivity / request | 5 s / 5 s / 15 s |
| Discovery / each cleanup request | 120 s / 15 s |

Timeouts may be lowered or raised up to 600 seconds each. A discovery deadline
covers the whole request sequence, with separate cleanup time after expiry.
Synchronous parsing is bounded by the XML limits, not preemptively interrupted.
Body buffers and XML metadata are separate budgets; the table is not a process-RSS
cap. HTTP/TLS/runtime/DNS allocations have their own library behavior. DNS work may
outlive a canceled request even though the caller's request deadline is enforced.

RetrievePropertiesEx requests exactly one object. A continuation token is rejected
as an unsupported partial result, with CancelRetrievePropertiesEx attempted to
release the owned cursor. Missing/duplicate properties, wrong object references,
unknown object types, invalid namespaces and count limits fail closed. In particular,
managed-reference `type` must have **no namespace**, while `xsi:type` is schema type
metadata. The parser library's local-name attribute helper cannot distinguish them.

After any Login attempt, normal completion and internal failures attempt Logout.
A cookie received with malformed/truncated Login content is retained for cleanup.
`primary_error`, `pagination_cleanup_error` and `cleanup` remain separate; an
inventory result with failed Logout is not success. Lost Login headers can leave
an unidentifiable session, and a failed Logout is reported unconfirmed. No retry
can establish certainty in that case. Await `discover` to completion: dropping
its future, panicking, killing the process or losing the runtime cannot guarantee
remote cleanup. No Drop implementation claims otherwise.

## Local validation and authorized live qualification

```bash
cargo test -p rvvdk-vsphere
cargo clippy -p rvvdk-vsphere --all-targets -- -D warnings
cargo build --release -p rvvdk-vsphere --example discover
target/release/examples/discover --endpoint HTTPS_URL --certificate-sha256 SHA256 --user USER
```

The example reads the password from the terminal with echo disabled. It accepts no
password argument or environment variable. Default: three independent sessions.
`--paired` runs three alternating fresh/reuse pairs. `--fail-inventory` runs one
session with an object limit of one, expecting an inventory-limit error and
confirmed Logout. It stops after an unexpected failure to avoid repeated bad-login
attempts. Failure cases such as bad credentials are tested only against the local
TLS fixture server. All fixture certificates/keys are generated in memory.

The example emits sanitized JSON, per-request and full-discovery timings, TLS
certificate checks, and Linux process CPU/high-water RSS measurements. Peak RSS is
process lifetime high-water, not a per-session allocation maximum. Timed discovery
includes client setup, login, parsing, inventory and logout; it excludes password
entry and final JSON output. CPU measurement also includes report assembly.
Connection reuse measurements do not establish disk throughput or Python/Rust
speedups. [Evidence and plots](../../docs/benchmark-results/2026-10-01-v02/README.md).

V0.3 next qualifies powered-off export bytes, licensing and lease cleanup;
compressed streamOptimized decoding remains a separate format gate.
[Architecture and acceptance](../../docs/vmware-access-plan.md).

Infrastructure contracts: [reqwest timeouts and policy](https://docs.rs/reqwest/0.13.5/reqwest/struct.ClientBuilder.html),
[Rustls verifier and handshake signatures](https://docs.rs/rustls/0.23.44/rustls/client/danger/trait.ServerCertVerifier.html),
[roxmltree parsing limits](https://docs.rs/roxmltree/0.21.1/roxmltree/struct.ParsingOptions.html).
