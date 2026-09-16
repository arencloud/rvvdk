# ADR-0011: Runtime Direct-I/O Alignment Discovery

## Status

Accepted

## Context

The first Linux direct-I/O implementation assumed a 4096-byte
alignment for:

- userspace buffer addresses
- file offsets
- transfer lengths

Linux direct-I/O requirements are not universally 4096 bytes.

Modern Linux kernels can expose direct-I/O alignment through
`statx()` using `STATX_DIOALIGN`.

The kernel reports separate memory and offset alignment requirements.

## Decision

The Linux local-file backend will discover direct-I/O alignment when a
direct file is opened.

The backend will represent alignment as two values:

```text
memory_alignment
offset_alignment
```

## Alignment provenance

The alignment representation records whether constraints came from:

```text
Statx
Fallback
```