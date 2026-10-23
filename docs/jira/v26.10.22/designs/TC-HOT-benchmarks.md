# TC-HOT-01/02 benchmark harness design (design record)

Condensed from the TC-HOT spec lane. No bench infrastructure exists in-tree
(no benches/, no criterion) — build as `cargo bench` targets with
`harness = false` manual mains, plus hdrhistogram.

## TC-HOT-01 (H_query + invariant ≤0.25µs p99, 10M consecutive)
- Measure the evidentiary DELTA: `admitted_statement_hash` fold
  (canon bytes + `hash_param_buffer` over a mixed typed param slice) with
  `std::hint::black_box` on the digest; per-iteration `Instant` into a
  `hdrhistogram` Histogram (sig-figs 3) → p50/p95/p99/p99.9 vs 250ns.
- Refusal-invariant leg: measure `Refusal` construction + message format +
  dispatch (src/dialect.rs:152-200) — the typed-refusal path is what runs on
  hot-path misses pre-ring.
- 10M × ~150-200ns ≈ 1.5-2s timed work — cheap; do NOT shard (concurrency
  inflates p99; PRD says "consecutive").

## TC-HOT-02 (removed)
The parquet rows/sec and `eval_ast_query` p95 court timed DuckDB. `benches/tc_hot_02.rs` is deleted. Graphlaw benches time the receipt fold and the refusal render (`benches/tc_hot_01.rs`).

## Hardware disclosure (baked into every receipt)
ARCH, core count, machdep.cpu.brand_string, AC power required +
`pmset -g therm` (M-series thermal), blake3 NEON-SIMD note (0.25µs is a
NEON number; x86 differs — expected, not regression). Machine field in the
receipt so cross-silicon claims are falsifiable.

## Acceptance wiring
- Ladder rung (in-PR): smoke variants (100k iters / 500k rows) as
  `[[bin]]`s with `required-features = ["duckdb"]` — assert the harness
  produces a receipt, never the budget (timing asserts in CI are flake).
- Phase-2 exit: `scripts/tc_hot_receipt.sh` nightly-style job writing
  `receipts/tc_hot_01-<date>.json` (+ OCEL), exit code = budget verdict,
  machine disclosure embedded.

## Reconciliation flag
TC-HOT-01 is labeled both 0.25µs (PRD §5) and 12M-rows/sec (SQLRDF-18 row) —
reconcile the test-id mapping before receipts are filed. This design covers
both readings: TC-HOT-01 = latency court, TC-HOT-02 = throughput court.
