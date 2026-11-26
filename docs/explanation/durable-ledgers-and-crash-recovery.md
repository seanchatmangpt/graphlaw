# Durable Ledgers and Crash Recovery

How the three append-only stores keep their hash chains valid across crashes. Each names one
commit point and one recovery rule. Nothing here was executed in this lane; claims are read
from source.

## EvidenceRing (`src/evidence_ring.rs`)

- Producers push into a 4096-slot SPSC ring; a drainer thread writes an NDJSON ledger with a
  BLAKE3 parent-to-child chain. Writes are one `write_all` plus flush, with fsync every 16
  flushes (about a 1 s crash window) (`:1-12`, `:50-52`).
- Sequence minting and enqueue are one critical section under the producer mutex
  (`:497-502`). Minting before the lock let concurrent callers enqueue out of order and corrupt
  the chain.
- Overflow is drop-newest; the sequence gap is recorded as `DroppedEvidence` lines, so drop
  accounting needs no producer-side I/O (`:12-16`).
- After a restart the counter is seeded from `max(watermark, recovered.covered_seq)`
  (`:350-357`) so a new push never reuses a covered sequence.
- A torn tail is detected through the chain and never accepted (`:8-9`).

## QueryLedger (`src/query_ledger/storage.rs`)

- Commit point: `chain.json`, written atomically after batch and aggregate files and one
  directory fsync (`:125-150`). In-memory state advances only after it is durable; on failure
  the records return to the buffer.
- Recovery (`recover_orphans`, `:574-609`): batch or aggregate files not covered by
  `chain.json` that name its head as parent are moved to `quarantine/`, never deleted. Two
  files claiming one parent (fork) stay in place for `verify_dir` to refuse.

## ReceiptStore (`src/receipt_chain/store.rs`)

- `append` holds an exclusive advisory lock across recover, write and fsync (`:55-62`).
- Torn-tail law (`:76-83`): a final fragment with no newline that parses as a record gets its
  newline restored; otherwise it is truncated, only after all preceding lines verify. An
  unparsable newline-terminated line is never repaired and refuses with
  `RECEIPT-STORE-CORRUPT`.

## See Also

- `docs/refusals.md`
- `docs/explanation/receipt-fields.md`
- `docs/explanation/hook-refuse-semantics.md`
