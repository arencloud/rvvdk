# rvvdk

rvvdk is a high-performance virtual disk access and data movement
framework written in Rust.

The project aims to provide a modern, safe, extensible alternative for
virtual disk access and migration workloads, with an initial focus on
VMware environments.

rvvdk is designed around independent abstractions for:

- virtual disks
- block devices
- storage transports
- disk formats
- extent discovery
- changed block tracking
- high-performance data movement

The long-term goal is to support multiple virtualization and storage
platforms rather than coupling the data plane exclusively to VMware.

## Project status

rvvdk is currently under active development.

The current milestone establishes the Rust workspace and fundamental
disk types used by the rest of the project.

No production disk I/O is implemented yet.

## Architecture

The high-level architecture is:

```text
                rvvdk CLI / API
                       |
                       v
                 Data Mover
                       |
                       v
                 VirtualDisk
                       |
            +----------+----------+
            |          |          |
           RAW        VMDK       QCOW2
            |          |          |
            +----------+----------+
                       |
                       v
                 BlockDevice
                       |
          +------------+------------+
          |            |            |
       Local I/O      NBD          SAN

```

## Benchmarks

Run the DataMover benchmarks with:

```bash
cargo bench -p rvvdk-datamover --bench copy
```

For storage-backed benchmarks, select an explicit benchmark directory:

```bash
RVVDK_BENCH_DIR=/path/to/benchmark-storage \
cargo bench -p rvvdk-datamover --bench direct_io
```
