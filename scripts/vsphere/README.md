# Read-only ESXi qualification probe

`probe.py` is a Python-standard-library discovery tool for V0.1, independent of
VMware SDKs. It is not the rvddk Rust transport or a disk reader. It only permits
RetrieveServiceContent, Login, RetrievePropertiesEx and Logout. It cannot shut
down VMs, create snapshots, obtain export leases or change licensing.

```bash
python3 scripts/vsphere/probe.py \
  --host ESXI_HOST \
  --certificate-sha256 EXPECTED_64_HEX_DIGIT_SHA256 \
  --user LAB_USER --repeats 3 > /path/outside/git/observations.json
python3 -m unittest discover -s scripts/vsphere -p test_probe.py -v
```

The password is read through a terminal without echo; no password argument,
environment variable, credential file or persisted cookie is used. Do not redirect
terminal input from a tracked file. The TLS peer certificate must match the
supplied SHA-256 pin **before any HTTP request**. Bootstrap the pin through an
approved out-of-band channel; a fingerprint collected from the same first network
connection only provides trust on first use, not independent host authentication.
Every request reconnects and checks the pin. There is no HTTP redirect or proxy
fallback, and only the selected host's `/sdk` endpoint is contacted.

The tool retains credentials/cookies only in process memory (Python does not
promise cryptographic memory erasure). JSON output selects version/build, license
edition metadata, generic VM/datastore labels, state, capacity, backing kind and
request measurements. Passwords, cookies, license keys, account/VM/datastore names,
host addresses, UUIDs, managed references and disk paths are never serialized.
Server fault text is suppressed; known fault type names remain. Review the
allowlisted output before staging evidence.

Each XML response is capped at 2 MiB, must be UTF-8, and cannot contain DTD/entity
declarations. Inventory is capped at 128 visited objects and queued references;
pagination/missing properties fail instead of silently producing a partial report.
The 15-second socket timeout is an inactivity bound, not a whole-operation deadline.
The tool attempts Logout on success and errors after receiving service content;
a lost connection or killed process cannot guarantee server session termination.
A failed logout makes an otherwise successful probe exit nonzero. Rust production
work must add explicit deadlines and report primary and cleanup failures separately.

`LicenseManager.licenses` describes available licenses; it is not a proof of the
active assignment or that an operation is licensed. `licensedEdition` is deprecated
and may be empty. `disabledMethod` reflects current state; an export listed there
for a powered-on VM does not independently establish a license restriction.
Resolve assignment/capability uncertainty before the export proof.

Timings include fresh TLS setup and bounded response parsing. Three discovery
runs are observations, not disk throughput, a matched implementation comparison,
or a tuned API benchmark. See [V0.1 design](../../docs/vmware-access-plan.md).

## Plot recorded discovery

`plot_probe.py INPUT_JSON OUTPUT_DIRECTORY` uses Matplotlib to show every request
and each session total. The checked-in V0.1 dataset has three sessions and fourteen
requests per session; this generator deliberately validates that dataset shape.
Run it with the existing `target/benchmark-plots/bin/python` environment. SVG/PNG
and computed provenance are deterministic with the recorded Python/Matplotlib
versions. [Evidence and reproduction](../../docs/benchmark-results/2026-10-01-v01/README.md).

## V0.2 Rust qualification

New VMware sessions use [rvvdk-vsphere](../../crates/rvvdk-vsphere/README.md).
The Python probe above remains V0.1 historical evidence.
`plot_rust_discovery.py INPUT_JSON OUTPUT_DIRECTORY` plots the six recorded Rust
fresh/reuse sessions, CPU and process high-water RSS, and computes each paired
elapsed change plus the >5% investigation trigger. It performs no VMware access.
[Results and reproduction](../../docs/benchmark-results/2026-10-01-v02/README.md).

## V0.3.1 export foundation

New export operations use the Rust `export` example in `rvvdk-vsphere`; the Python
probe remains historical. `plot_export_foundation.py INPUT_DIRECTORY OUTPUT_DIRECTORY`
validates and plots every baseline/candidate discovery session and computes the
aggregate, pair-median and individual >5% adverse triggers. Initial blocks contain
three sessions per arm; longer repeats contain six, in two three-session processes.
It does not access VMware. [Evidence](../../docs/benchmark-results/2026-10-01-v031/README.md).
