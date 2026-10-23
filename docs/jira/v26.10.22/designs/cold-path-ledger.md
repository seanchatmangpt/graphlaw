# Cold-path query-receipt ledger (design record)

Condensed from the cold-ledger spec lane (PRD §3.2.2; companion to C1-cache).
No SPSC ring exists in src/ yet — this fixes the descriptor format the ring
will carry plus the ledger contract behind it.

Module split on disk (2026-10-22): `src/query_ledger.rs` is the 29-line
facade. `src/query_ledger/batch.rs` (459 lines) holds the in-memory batch,
Blake3 parent chaining, and encode/parse. `src/query_ledger/storage.rs`
(564 lines) holds the atomic temp+rename writer, `.agg` sidecar, and gap
recovery. The operator brief's further cut — a `LedgerBatch::fold_entry`
with zero filesystem imports in `batch.rs` — was not enacted. `batch.rs`
is the pure side of the split only in the sense that the POSIX writer
lives in `storage.rs`.

## Descriptor (64B, hot path)
h_query[32] · rows_emitted u32 (saturating) · flags u8 (bit0 cached, bit1
cacheable-class) · pad · ts_unix_ms u64 · epoch u64. Deliberately excludes
sql_statement/result_triples (cold data; h_query is the content address —
C1 Layer-2 is where full receipts live).

## Format
Batched canonical-JSON batch files reusing receipt_store.rs conventions
verbatim: fixed key order, hand-rolled escaper (no serde), content-addressed
`<blake3-of-batch-bytes>.json`, subject `query-ledger-v1`, atomic temp+rename.
Batch chains batches (parent = previous batch digest; genesis = 64 zeros) —
C21 chains states, the ledger chains BATCHES. `first_seq` monotonic
sequence = drain-completeness check.

## Durability (disclosed bound, no WAL)
Flush: count 1024 OR age 2s, whichever first; drain-and-flush on Drop is an
obligation. Crash yields batch-exists-fully-valid or batch-absent (canonical
JSON + digest name makes torn batches unrepresentable). Loss on crash bounded
by the un-flushed partial batch, DISCLOSED via persisted gap markers —
aggregation carries `complete: false` over any gap period. Never backfilled.

## ASC 450 aggregation
Per-batch `.agg` sidecar (rows_emitted_sum by epoch, saturating-add WITH a
`saturated: true` flag — no overflow laundering) + a `CHAIN` head/tail index
file. Period query = O(batches in period), not O(receipts). Spot-re-fold of
at least one batch per query = the tamper check (the .agg is keyed to the
batch but not self-addressed — disclosed).

## Retention (7y ASC 450-10-50)
GC prunes chain TAIL only after folding into a terminal rollup batch
(receipts: [], rollup: {..., over: [pruned digests]}) — the chain never
forgets a digest, only a body. Below the $50M tier: `ledger: disabled` is the
aggregation answer, never a fabricated zero.

## Courts F1-F8
F1 byte-flip → DigestMismatch · F2 deleted middle batch → BrokenChain +
complete:false · F3 fork → refused · F4 double genesis → refused · F5
drain-completeness vs producer seq (gap = exactly the in-flight count) · F6
ring overflow = disclosed loss (drops observed, never absorbed) · F7
aggregation correctness with mandatory spot-re-fold · F8 .agg tamper via
spot-re-fold (or BLAKE3(agg) in CHAIN if tamper-evidence is promoted).

## Restart watermark reconciliation (landed)

As-of 2026-10-22, v26.10.22 window.

**Defect (fuzz court case 4)**: a crash-restart re-opened ring+ledger with
gaps=`[]` and `complete=true` — the fresh drainer silently renumbered its
records onto the persisted ledger head, laundering crash loss into a
complete ledger.

**Fix (landed)**: the drainer (`src/evidence_ring.rs:553`, `drain_loop`;
watermark read at 584–595) extracts a restart watermark from the resumed
NDJSON ledger's last line (`"seq":` coverage watermark) and opens the query
ledger via `QueryLedger::open_with_producer` (defined at
`src/query_ledger/storage.rs:65`, called at `src/evidence_ring.rs:619`) with
`restart_seq.max(producer_attempted)`. Crash loss now persists as gap
markers with `complete: false`; never backfilled.

**Deviation, receipt-cited**: the watermark is `restart_seq` (resumed NDJSON
coverage), NOT the live `seq` counter. The prescribed live-counter form
double-covered 911 buffered sequences — the joint court's gap coverage
exceeded `dropped_total` by exactly 911 when the live counter was used
(witness comment, `src/evidence_ring.rs:617`). Regression court:
`restart_mid_stream_persists_gap_not_silent_renumber`
(tests/evidentiary_ring_court.rs:329), receipt:

```
$ cargo test --features evidentiary-ring --test evidentiary_ring_court
(re-measured 2026-10-23: 10 passed; 0 failed)

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.74s
```

**Never-block fence conversion (landed)**: the single absolute
"max push <10µs" dev-profile bound is converted to a two-leg fence.

- Dev leg (the PROPERTY, load-independent): ratio fence — stalled max push
  ≤ K× unstalled same-binary baseline (K=10, baseline floored at 100ns so a
  sub-instruction baseline cannot make the ratio vacuous);
  `no_blocking_property_1m_pushes_vs_stalled_drainer`
  (tests/evidentiary_ring_court.rs:158).
- Release leg (the NUMBER, `--release` only): absolute stalled max <10µs;
  `no_blocking_release_absolute_bound`
  (tests/evidentiary_ring_court.rs:302), receipt:

```
$ CARGO_TARGET_DIR=target-bench cargo test --release --features evidentiary-ring \
    --test evidentiary_ring_court no_blocking_release_absolute_bound -- --nocapture
(re-measured 2026-10-23: full court 10 passed; 0 failed)

release absolute leg: stalled max push 1.584µs < 10µs
test result: ok. 1 passed; 0 failed; 8 filtered out; finished in 0.13s
```

Witnessed value is 1.584µs (single-shot, darwin 25.2.0) — well under the
10µs gate; no 250ns-class claim is made.
