# Engine Hot/Cold Asymmetry Baseline — Measured Receipt

Date: 2026-10-23. Subject: audit finding that 2.4 µs / 123 µs / 238.5 ms in
`docs/whitepaper-v26.10.22.md` were prose-only. This receipt replaces prose with
real Divan output.

## Command

```
CARGO_TARGET_DIR=target-lane-eb cargo bench --bench sql_rdf_engine
```

Exit code: **0** (full log: /tmp/eb_bench.log, captured `EXIT=0`).
Build: `Finished bench profile [optimized] target(s) in 10m 23s`.
Binary: `target-lane-eb/release/deps/sql_rdf_engine-42ab34ce7a8a7139`.
Timer precision: 41 ns. Samples: 100 per metric.

## REAL Divan output (verbatim)

```
sql_rdf_engine            fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ c1_hit_eval            2.457 µs      │ 24.95 µs      │ 2.541 µs      │ 2.841 µs      │ 100     │ 100
├─ cold_graph_eval        15.77 ms      │ 103.4 ms      │ 23.13 ms      │ 27.71 ms      │ 100     │ 100
├─ fresh_connection_eval  14.63 ms      │ 27.72 ms      │ 16.42 ms      │ 16.79 ms      │ 100     │ 100
╰─ raw_engine_prepared    83.7 µs       │ 263.2 µs      │ 117.4 µs      │ 123.9 µs      │ 100     │ 100
```

## Measured medians vs cited figures

| Figure | Cited (prose) | Measured median | Verdict |
|---|---|---|---|
| c1_hit_eval (hot) | 2.4 µs | **2.541 µs** | consistent (cited figure ~fastest, median slightly higher) |
| raw_engine_prepared | 123 µs | **117.4 µs** | consistent (cited ~mean 123.9 µs, not median) |
| cold_graph_eval | 238.5 ms | **23.13 ms** | DISCREPANT — measured median is ~10x faster than the cited 238.5 ms |

## Finding for coordinator

The cited 238.5 ms cold figure does not reproduce on this machine/today's tree:
`cold_graph_eval` median is **23.13 ms** (max observed 103.4 ms across 100
samples; even the slowest sample is under half the cited figure). The
whitepaper's cold-path number should be corrected to the measured median
(23.13 ms) or re-derived with its original provenance identified. Note also the
bench emits a `fresh_connection_eval` (median 16.42 ms) not present in the
cited triple — if the whitepaper's "cold" meant fresh-connection rather than
cold-graph, the matching measured figure is 16.42 ms.

Hot (2.4 µs vs 2.541 µs) and prepared (123 µs vs 117.4 µs median / 123.9 µs
mean) figures are within run-to-run noise of the citations.

Lane build root: `target-lane-eb` (lease — coordinator deletes at integration).
