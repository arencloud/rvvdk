# Explicit source selection and export artifacts (R6.1a)

`rvvdk_vsphere::contract` supplies bounded runtime selection and private artifact
metadata without network or filesystem operations. This is the contract for the
future production workflow. The existing `export` example remains a qualification harness with its legacy
capacity-selected mode. [R6.1c.1](explicit-export-selection.md)
adds an explicit-identity export path; durable workflow integration remains open.

## Runtime identity

`EndpointIdentity::pinned` uses the existing pinned HTTPS policy: canonical `/sdk`
endpoint, exact leaf DER SHA-256 and explicit `PinProvenance`. There is no insecure
fallback. Provenance is either `TrustOnFirstUse` or `ExternallyVerified`, supplied
by the caller; this module does not independently establish it. Current pin mode
replaces CA/hostname/expiry checks and verifies handshake possession as already
specified by `ConnectionPolicy`. No new TLS behavior is introduced.

`SourceSelection::new` requires that endpoint plus a VM managed reference, BIOS
UUID, disk device key, backing identity and expected logical capacity. Callers can
obtain private values explicitly from `VmIdentity` and the new `Disk::device_key`
and `Disk::backing_identity` accessors. These accessors do not add identities to
the existing diagnostic serialization or Debug output.

`select` accepts only one matching VM reference, rejects duplicate references,
checks UUID, disk key, backing identity and expected capacity, and retains the
qualified single persistent unencrypted flat disk/no-parent/no-snapshot scope.
It requires powered-off state, no template and no disabled export method. Guest
names and equal capacity do not identify a source. `check_vm` repeats those checks
for a fresh single-VM observation. The caller must bind the supplied inventory to
the admitted connection: an `Inventory` value alone is not authenticated. These
checks do not freeze the remote VM or prevent changes after an observation.

| Bound | v1 value |
|---|---:|
| Endpoint input | existing policy, at most 2,048 bytes |
| VM reference | 1–256 UTF-8 bytes, no control characters |
| UUID | canonical hyphens and 32 hexadecimal digits; normalized lowercase |
| Disk key | positive, at most `i32::MAX` |
| Backing identity | 1–4,096 UTF-8 bytes, no control characters |
| Candidate inventory | at most 128 VMs, matching current discovery budget |
| Logical capacity | positive, 512-byte aligned, at most 64 TiB |
| Encoded container length | at most 1 TiB; zero permitted only when incomplete |
| Metadata input | at most 4,096 bytes, checked before JSON deserialization |

These are contract admission limits, not a claim that every such VMDK is supported
by the decoder or every server can export it. Existing stream admission, resource
and operation limits still apply at integration. No work scales with logical disk
capacity here. Selection scans at most 128 observations without allocating a list.

## Persisted schema

Only `ExportArtifact::to_json` and `from_json` persist/parse the type. There is no
public unchecked serde constructor. [The synthetic v1 golden fixture](../crates/rvvdk-vsphere/src/contract/artifact-v1.json)
records the schema and an independently computed identity binding.

| Field | Meaning |
|---|---|
| `schema_version` | exactly `1`; no automatic migration |
| `artifact_id` | caller-generated nonzero 16 bytes as lowercase hex |
| `source_binding` | domain-separated SHA-256 of explicit runtime identity |
| `pin_provenance` | caller's `trust_on_first_use` or `externally_verified` assertion |
| `container_format` | exactly `stream_optimized_vmdk` |
| `logical_bytes` | virtual disk capacity, independent of encoded length |
| `container_bytes` | accepted encoded body length; a claim until checked |
| `completeness` | `incomplete` or `complete` |
| `validation` | `unchecked`, `container_digest_verified`, or `logical_readback_verified` |
| `container_sha256` | lowercase SHA-256 of encoded bytes, or null when incomplete |

The container member name is fixed to `disk-1.vmdk`. No persisted path, URL,
credential, cookie, lease reference, cleanup target or guest name is admitted.
Unknown fields and enum variants, duplicate fields, overflowing/fractional sizes,
invalid hex, malformed/trailing JSON and inconsistent states are rejected with
closed, redacted error values. JSON whitespace counts toward the byte limit.
Missing optional digest is equivalent to null, and only valid when incomplete.

Incomplete records must be unchecked and have no digest, regardless of received
length. Complete records require a positive container length and a digest; they
may still be unchecked. Validation claims record the producer's assertion and are
never trusted merely because the JSON parses. Encoded bytes may be smaller or
larger than logical capacity; the contract does not conflate these quantities.

`check_source` compares identity binding, capacity and provenance against an
explicit expected selection. `check_container` compares fresh caller-supplied
length and SHA-256, rejecting incomplete records even if their received count
looks plausible. It does not read a path, validate VMDK structure or compare
logical content. There is deliberately no `is_safe_to_resume` or trusted-artifact
type produced by deserialization. The caller must retain a validated file handle
and apply the R5.12 source-change/publication rules during future conversion.

## Stable binding construction and privacy

Each SHA-256 input field is prefixed with its byte length as little-endian u64.
The endpoint binding hashes, in order: ASCII `rvddk.endpoint.v1`, canonical SDK
URL bytes, lowercase hexadecimal certificate pin text, then the lowercase schema
spelling of provenance. The source binding hashes: ASCII `rvddk.source.v1`, raw
32-byte endpoint binding, VM reference, lowercase UUID, little-endian u64 disk
key, backing identity and little-endian u64 logical capacity. Strings use UTF-8.
Length prefixes prevent concatenation ambiguity. Altering these rules requires
an explicit schema/version migration decision.

No source binding or digest is printed by Debug. Explicit serialization contains
private, correlatable identity/content digests and must be stored privately.
Hashing predictable names is not anonymization or authentication. Artifact IDs
must be assigned uniquely by a future job store; nonzero validation alone neither
proves uniqueness nor establishes resource ownership. Copying or editing a valid
record does not authorize remote or local actions.

## Remaining integration gates

[R6.1b](durable-job-ownership.md) supplies the private durable ownership journal,
state transitions and conservative process-loss assessment/cleanup. Artifact claims
never grant authority over a lease or staging directory. R6.1c.1 now connects
explicit selection and revalidation to the export proof. R6.1c.2 connects actual
lease ownership to durable intents, followed by artifact admission and conversion. Existing
capacity-only manifests are not automatically promoted. No new ESXi run is needed
for R6.1a's pure contract tests and synthetic measurements.

[ADR-0057](adr/0057-source-identity-and-artifact-contract.md),
[tests and performance evidence](benchmark-results/2026-10-02-r61a/README.md).
