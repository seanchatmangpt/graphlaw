# QFD-HOUSE-OF-QUALITY

House of Quality for v26.10.28. Rows = the CTQs from
[VOC-CTQ-TURNSTILE.md](VOC-CTQ-TURNSTILE.md). Columns = engineering
characteristics. The roof gives correlations; targets restate the CTQ
numbers; "current standing" is graded from this session's evidence only —
no rounding up.

## Engineering characteristics

| # | Characteristic | Unit | Measured by |
| --- | --- | --- | --- |
| E1 | wasm import count | count | wasm module inspect / build surface |
| E2 | reduction budget | reductions/call | preemption gate (pool or budget counter) |
| E3 | heap-delta bytes/call | bytes | P28-SIL measurement, Cpk from SPC court |
| E4 | p99 latency | ms | latency rung on the receipt fold path |
| E5 | refusal coverage | % | refusal corpus + mutation court flip |
| E6 | receipt emission rate | % of consequential acts | receipts index / census |
| E7 | statement-kind pass count | count of 35 | `duckdb_statement_kinds` court |
| E8 | lockfile fork-source count | count | `Cargo.lock` forked entries (target: 1, rev-pinned) |
| E9 | untracked-dep count | count | dependency-closure sweep |

## Relationship matrix (rows × columns)

Legend: `●` strong, `○` moderate, blank = none.

| CTQ | E1 | E2 | E3 | E4 | E5 | E6 | E7 | E8 | E9 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1.1 preemption-safe exec | ○ | ● | | ○ | | | | | |
| 1.2 zero heap growth | ○ | ○ | ● | ○ | | | | | |
| 1.3 p99 ≤ 1.2 ms | ○ | ○ | ○ | ● | | | | | |
| 2.1 fail-closed, 0 false admissions | | ○ | | ○ | ● | ○ | | | |
| 2.2 proof verify per evaluation | | | | ○ | ○ | ● | | | |
| 3.1 ANSI SQL over RDF | | | | | ○ | | ● | ○ | ○ |
| 3.2 parity 35 kinds | | | | | ○ | | ● | | ○ |
| 4.1 append-only receipts 100% | | | | | ○ | ● | | | |
| 4.2 traceability to axioms+leases | | | | | ○ | ● | | | |
| 5.1 graphlaw_ex builds in-tree | ● | | | | | | ○ | ○ | ● |
| 5.2 zero host imports | ● | | | | | | | | ○ |
| 5.3 rev-pinned canonical source | | | | | | | | ● | ○ |

## Roof (correlations among engineering characteristics)

| Pair | Correlation | Note |
| --- | --- | --- |
| E2 × E4 | + (reinforcing) | a hard reduction budget bounds tail latency |
| E1 × E4 | − (conflict) | fewer imports shrink surface but isolation/pooling adds call overhead |
| E3 × E4 | − (conflict) | per-call heap accounting costs measurement overhead |
| E5 × E6 | + (reinforcing) | typed refusals are themselves receipted acts |
| E8 × E9 | + (reinforcing) | one rev-pinned fork source shrinks the untracked-dep sweep |
| E1 × 5.1 | + (reinforcing) | zero host imports is the precondition for a clean BEAM embed |
| E7 × E9 | − (conflict) | freezing vendor/ for parity resists dep-closure churn |

## Targets and current standing

| CTQ | Target | Current standing | Evidence basis |
| --- | --- | --- | --- |
| 1.1 | ≤1000 reductions/call or pooled isolation | UNVERIFIED | wasm build not yet run this session; no reduction-budget witness |
| 1.2 | Cpk ≥ 1.67 heap-delta | UNVERIFIED | `spc_court.sh` not run this session (P28-SIL program exists) |
| 1.3 | p99 ≤ 1.2 ms | UNVERIFIED | no latency rung run this session |
| 2.1 | 0 false admissions | PARTIAL | chicago_validation 8/8 ALIVE @ 2223f894; mutation court not yet run — anti-vacuity unwitnessed |
| 2.2 | verify per evaluation | PARTIAL | proof_pipeline 12/12 + trust/attest surface ALIVE @ 2223f894; per-evaluation pinning not re-demonstrated |
| 3.1 | ANSI SQL over RDF via embedded DuckDB | PARTIAL | conformance 12/12 ALIVE @ 2223f894 |
| 3.2 | 35/35 statement kinds | PARTIAL | census 13/13 ALIVE @ 2223f894; `duckdb_statement_kinds` not re-run this session |
| 4.1 | 100% append-only receipts | UNVERIFIED | no receipt-emission census run this session |
| 4.2 | traceability to axioms + leases | UNVERIFIED | no traceability sweep run this session |
| 5.1 | `mix` green in-tree | UNVERIFIED | graphlaw_ex scaffold exists; `mix` never run |
| 5.2 | 0 host imports | UNVERIFIED | depends on a wasm build (not yet run) |
| 5.3 | rev-pinned fork source | ALIVE | purrdf fork swap @ 2223f894: `cargo check` 0 errors |

A standing above is as-of 2026-10-28 session receipts; it moves only on a
new command + exit. Full per-CTQ commands in [WORK-ORDERS.md](WORK-ORDERS.md).
