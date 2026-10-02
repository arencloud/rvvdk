# ADR-0061: Own payload writes and make completion a conservative boundary

Status: Accepted for R6.1c.2b container-byte transfer/completion; production artifact
admission, conversion and publication remain open.

## Context

The durable probe has correct acquisition/abort intent ordering but no payload.
The legacy exporter can drop a download future and subsequently abort; blindly
reusing that lifecycle risks abandoned file writes or abort after an uncertain
completion request. A persisted state alone does not verify container bytes.

## Decision

Keep payload writes inside the existing blocking owner. Bound batches to 1 MiB,
serialize accepted writes with control commands, and retain the worker when a
payload reply waiter is dropped. Close the writer before recording abort intent.
Payload failure may still use a journaled abort; journal failure cannot authorize
another remote mutation.

After network/manifest checks, sync and close the writer, independently read length
and SHA-256/SHA-1 through a fresh checked descriptor, and recheck stage/member/marker
ownership. Only then record TransferComplete. Keep hash claims separate from VMDK
format validity and logical byte verification.

Enter the no-abort completion boundary before submitting CompleteIntent, since a
failed persistence call can already have renamed a new record. Revalidate source
and cancellation after slow barriers, complete once, and record the remote reply
separately from its durable acknowledgment. Uncertain outcomes remain pending;
there is no automatic completion retry or rollback to abort.

## Consequences

Accepted data writes finish before an abort intent or API return, including when
the receiver future is cancelled. Network and journal operations retain their
separate failure observations. Readback has bounded memory but reads the complete
container again and adds measurable CPU/I/O cost. Eight record commits and payload
sync add fixed local durability cost. Keep those barriers and retain adverse timing.

The stage remains private, metadata empty and no publisher or converter is invoked.
Next R6.1c.3 must persist the private artifact contract, admit native VMDK structure,
and define a fresh read-only capability without resurrecting remote Job ownership.
Conversion and real no-replace publication remain later integration packages.

[Contract](../durable-owned-transfer.md),
[tests and measurements](../benchmark-results/2026-10-02-r61c2b/README.md).
