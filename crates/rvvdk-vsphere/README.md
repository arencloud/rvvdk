# rvvdk-vsphere: bounded Rust discovery and export proof

V0.2 implements independent SOAP authentication and inventory for direct ESXi
8.0.3 / HostAgent API 8.0.3.0. It uses general-purpose HTTP, TLS and XML crates,
not a VMware SDK or VDDK. V0.3.1 adds a Linux export-lease proof, including explicit
graceful shutdown when requested. Live export remains license-blocked. Local disk crates
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
Scoped operation futures retain `Send` compatibility for Tokio tasks. The
qualification examples use a current-thread runtime; concurrent live sessions
are not qualified by this proof.

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

V0.3.1's live `ExportVm` probe returned `license_restricted`, then confirmed Logout.
Neither VM was shut down. Trial/commercial access is now needed for V0.3.2's live
export proof; compressed streamOptimized decoding remains a separate format gate.
[V0.3.1 evidence and comparison](../../docs/benchmark-results/2026-10-01-v031/README.md).
[Architecture and acceptance](../../docs/vmware-access-plan.md).

## Linux export qualification API

`export_vm(policy, credentials, ExportOptions::probe(capacity))` selects exactly one
VM with a single disk of that capacity. It rechecks the private managed reference,
BIOS UUID, disk key and backing path. It admits only a non-template VM with one
persistent, unencrypted FlatVer2 disk, no parent and no snapshots. Ambiguous selection
fails closed. This proof is not a general VM selection or export product API.

Probe mode calls `ExportVm` without changing power and immediately aborts any granted
lease. License restriction and invalid power state are distinct errors. Probe success
does not satisfy `ExportReport::is_success()`, which requires a verified artifact,
completed lease and confirmed Logout.

For a transfer set `probe_only = false` and a new output directory. If the VM is on,
`allow_graceful_shutdown` must be true and Tools must be running. Output admission
precedes shutdown. The original discovery deadline also bounds shutdown polling;
there is no hard power-off fallback. Disk identity is rechecked after power-off.
The VM is left in its last observed state; no automatic power-on or license change.

One owned lease is polled to ready. Its entity, capacity, one disk URL and timeout
are validated. Only HTTPS URLs on the pinned control host and effective port,
under `/nfc/` or `/ha-nfc/`, are admitted; `*` substitutes that host. No API cookie
or password is sent on GET. A SHA-256 or legacy SHA-1 lease thumbprint must match
the certificate already admitted by the SHA-256 pin; SHA-1 never replaces that
trust anchor. Lease keys, URL tickets and private identities are not serialized.

Progress calls renew the lease at min(timeout / 3, 10 s), including while a data
response or asynchronous file sync is pending. Percent is conservatively zero;
the encoded total is not known in advance. This behavior still needs live-server
qualification. Control and data calls have bounded deadlines. The default transfer
deadline is one hour, configurable to at most six hours. Control request records
are capped at 4,096, with reserved abort/logout attempts after exhaustion.

The stream uses a 1 MiB application buffer and rejects chunks over 1 MiB. Received
and advertised byte limits are checked (default 40 GiB, maximum 1 TiB). File data
is hashed incrementally with SHA-256 and SHA-1 and synced. The server manifest's
device key, size, optional capacity and declared digest are checked. Only a minimum
512-byte body with sparse VMDK magic is accepted: **this is not VMDK decoding or
full format validation**. The synthetic fixture is deliberately not a guest disk.

The output uses a pinned Linux parent directory descriptor, exclusive private
staging (0700 directory, 0600 files), a free-space preflight, and `renameat2` with
`RENAME_NOREPLACE`. Data and metadata are synced before Complete; confirmed Complete
and Logout precede directory publication and parent sync. Existing destinations
are never overwritten, including publication races. A parent-sync failure after
rename reports an error with `artifact_published = true`; the artifact is retained.
Space preflight is not a reservation. Filesystem I/O can fail later. Directory sync
is synchronous and has no hard latency guarantee; it occurs after lease release.

Normal failures discard owned staging files, abort an identifiable lease and log
out. Primary, lease, session, pagination and local cleanup outcomes remain separate.
Malformed/lost acquisition replies can leave an unidentified lease; they are
reported unconfirmed and never retried. `Cancellation::cancel()` (Ctrl-C in the
example) requests cooperative cleanup. Await the future: dropping it, process death,
panic or runtime loss cannot guarantee cleanup. Export publication is Linux-only;
other platforms return `export_scope` and are not qualified by these tests.

```bash
cargo build --release -p rvvdk-vsphere --example export
# No power change; abort immediately if the server grants a lease.
target/release/examples/export --endpoint HTTPS_URL --certificate-sha256 SHA256 --user USER
# Authorized disposable VM only; requires eligible licensing and new local output.
target/release/examples/export --endpoint HTTPS_URL --certificate-sha256 SHA256 --user USER --capacity-bytes 32212254720 --output NEW_DIRECTORY --allow-shutdown
```

`--cancel-after-ms`, `--max-bytes` and `--timeout-seconds` support qualification.
Encoded-file elapsed time includes HTTP receive, hashing, writes and file sync;
whole-operation elapsed time additionally includes discovery, shutdown if requested,
lease/session cleanup and publication. Neither includes terminal password entry.
Live throughput, CPU/RSS, logical-byte equivalence and server lease release remain
V0.3.2 acceptance work.

Infrastructure contracts: [reqwest timeouts and policy](https://docs.rs/reqwest/0.13.5/reqwest/struct.ClientBuilder.html),
[Rustls verifier and handshake signatures](https://docs.rs/rustls/0.23.44/rustls/client/danger/trait.ServerCertVerifier.html),
[roxmltree parsing limits](https://docs.rs/roxmltree/0.21.1/roxmltree/struct.ParsingOptions.html).
