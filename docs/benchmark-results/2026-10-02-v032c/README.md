# V0.3.2c — LAN export and independent guest-byte qualification

Date: 2026-10-02. Baseline: `4e57c87`.
[Contract](../../../crates/rvvdk-vsphere/README.md),
[transport decision](../../adr/0051-qualified-powered-off-export.md),
[acceptance plan](../../vmware-access-plan.md).

## Setup and scope

The user explicitly approved using VM02, the former 60 GiB control, as a LAN
runner, including qemu-img installation and private benchmark files. VM01, the
30 GiB source, remains powered off after the prior authorized graceful shutdown.
Both use the existing ESXi 8.0.3 / HostAgent API 8.0.3.0 host. No license changes,
new snapshots, hard power operations or VMware SDK/VDDK calls were made.

The runner has one Xeon Silver 4116 virtual CPU, approximately 2 GiB RAM, XFS and
qemu-img 10.2.2. It shares source-host CPU/storage resources. This is not an isolated
physical LAN benchmark, nor a before/after code optimization comparison. Network,
host load and caches are uncontrolled; cache dropping is not used. No compilation
or independent QEMU decoding overlaps timed export runs. A small script/metadata
copy and read-only progress checks may run during measurement.

The Rust export/discovery executables are copied unchanged from the prior measured
build. Their SHA-256 values match V0.3.2b exactly. No Rust production code changed.
Each export uses an 8 GiB encoded-byte cap and 1,800-second lease-ready/transfer deadline,
one disk/stream at a time, with identical manifest/hash/fsync/cleanup/publication
boundaries. The source is already off, so shutdown time is excluded.

The former remote-connection failures remain in
[V0.3.2b](../2026-10-01-v032b/README.md). They are not replaced or averaged into
completed-transfer performance. This evidence directory includes every LAN export
attempt; setup and read-only inspections are recorded separately.

## Verification and privacy

The first published file passes a fresh size/SHA-256 check against its Rust export
report. QEMU fully decodes its compressed version-3 streamOptimized container to a
30 GiB sparse RAW file. The independently obtained guest extent/LVM/partition map
locates the 8 MiB fixture; every byte matches the original reference. Before guest
shutdown, that reference had also matched direct guest-disk sectors.

QEMU is an offline lab oracle, not a production dependency or fallback. The native
Rust CLI rejects the image with `unsupported sparse feature: version (only 1)`.
Compressed format support remains an explicit separate implementation gate.
The known-range oracle does not prove independent whole-source equivalence.

Original reports, image digests, QEMU diagnostics, fixture data/map and VM images
stay in private ignored/local runner storage. Published export reports replace
only per-image SHA-256/SHA-1 values with a redaction marker; numeric timings, byte
counts, call history and cleanup outcomes remain unchanged. Plot input hashes
therefore identify the sanitized evidence files. No credentials, pins, addresses,
managed references, VM identities, guest contents or image digests are committed.

## Measurements and cleanup


[SVG plot](live-export.svg), [PNG plot](live-export.png), [computed data](computed.json).

| Run | Encoded bytes | Transfer seconds | Whole-operation seconds | Encoded MiB/s | CPU seconds | Peak RSS MiB |
|---|---:|---:|---:|---:|---:|---:|
| 1 | 2,727,380,992 | 238.409 | 238.611 | 10.910 | 46.266 | 8.188 |
| 2 | 2,727,380,992 | 230.584 | 230.813 | 11.280 | 44.463 | 7.688 |
| 3 | 2,727,380,992 | 230.965 | 231.192 | 11.262 | 44.447 | 8.078 |

All three exports completed with manifest verification, Complete/Logout and
publication. Median encoded throughput is **11.262 MiB/s**; median
whole-operation time is **231.192 seconds**. No adverse or failed LAN
export was omitted. These three observations are not a tuning regression test.

Run 2 has the same encoded size as run 1 but a different encoded SHA-256. The
identity shortcut was explicitly rejected (`identity-2.json`). QEMU then compared
both complete decoded logical disks and returned success. This establishes logical
equality with the independently verified reference, not independent whole-source
equivalence. Container differences are not treated as corruption merely because
the complete encoded digests differ; decoded comparison is required.

Run 1's temporary RAW and the verified run 2 disk file were removed deliberately
to leave headroom for run 3. Private original reports, metadata and verification
diagnostics remain. Run 1's encoded reference remains available. Run 3 also has a different encoded digest but passes the full decoded logical
comparison against run 1. QEMU comparison elapsed was 45.934 seconds for run 2
and 45.577 seconds for run 3, outside export timings. Both inherit the independent
known-range result through logical equivalence, not through encoded digest equality.

Final inspection returns one successful export task and no running/queued tasks
in the bounded returned history. Three final discovery sessions confirm VM01
powered off, VM02 powered on, and Logout acknowledged. No source power changes,
new snapshots or staging residue remain from this step. The retained private
encoded artifacts are runs 1 and 3; VM02 retains qemu-img and the private runner
directory for continuation. About 7.44 GiB remains available on its root filesystem.


## Reproduction

Use the same release export binary and explicit 8 GiB / 1,800-second limits on
the authorized LAN runner, with terminal-only credentials. Keep source identity,
certificate policy and powered-off eligibility checks. Run exports sequentially.
Never commit the private original reports or images.

For the first published artifact, run
[`qualify_export_image.py`](../../../scripts/vsphere/qualify_export_image.py)
as documented in the [helper instructions](../../../scripts/vsphere/README.md).
For later artifacts, use its three reference arguments to compare complete logical
contents against the first verified artifact. Verification work must finish before
the next timed export. This helper's initial full-decode source is retained in
`qualify-initial.py`, matching `tool-sha256.json`; the later reference-comparison
branch does not change the Rust transport executable.

The first decode took 52.340 seconds and allocated 3,770,687,488 bytes for the
30 GiB sparse RAW. That is a lab verification observation, not a native Rust
decoder benchmark. Offline full RAW hashing is also outside export CPU/elapsed.


## Performance interpretation

Encoded throughput uses actual encoded bytes divided by the receive/hash/write/
file-sync interval. Whole-operation time also includes discovery, lease lifecycle,
Logout and durable metadata/directory publication. Logical capacity is a separate
quantity. Process CPU excludes terminal entry and final report serialization;
process-lifetime peak RSS excludes filesystem page cache and kernel socket buffers.
QEMU conversion/comparison and private hashing are outside these timings.

These observations do not establish that the transfer is well tuned. Profile the
host NFC compression/storage path, guest network and client before choosing an
optimization; low client CPU alone does not locate the limiting component. Preserve
all samples and use controlled paired measurements for any future tuning claim.
PERF.0/R4.4, V0.3.1's adverse individual discovery pairs and the historical QEMU
partial second-extent discrepancy remain open.


## Validation and disposition

The production Rust source is unchanged from `4e57c87`, whose validation passed
584 unique workspace tests (one existing ignored) and Clippy with warnings denied.
This step reuses that tested binary rather than claiming a new Rust test run.
The native CLI release build succeeds; three offline oracle tests pass. The new
helper rejects an incomplete export before touching artifacts and is exercised
against one full decode and two complete logical comparisons. Plot input validation
and PNG visual inspection pass. Safe evidence hashes cover all generated artifacts.

**V0 is qualified for the bounded powered-off single-disk export workflow only.**
Combine this step's three complete exports and independent known-byte proof with
V0.3.2b's real cancellation/deadline cleanup. R6 production integration, process-loss
recovery and native compressed VMDK decoding remain open. Least-privilege roles,
active-license assignment/expiry, vCenter, online snapshots, CBT, restore and
vSphere 9 are not qualified by this proof. The existing certificate pin still has
its original TOFU provenance, not an independently authenticated trust claim.


Generate the plots from sanitized reports at the repository root:

```bash
target/benchmark-plots/bin/python scripts/vsphere/plot_live_export.py \
  docs/benchmark-results/2026-10-02-v032c \
  docs/benchmark-results/2026-10-02-v032c/run-{1,2,3}.json
```

[Validation audit](validation-summary.json) covers all eight lifecycle sessions
(preflight, three exports, final inspection and three final discovery sessions).
All 135 calls across the three exports report success. The audit also checks the
frozen binary hashes and confirms that public report redaction preserves every
numeric measurement and lifecycle field. Reference/helper hashes distinguish the
initial full-decode helper from the final helper with logical-comparison support.
