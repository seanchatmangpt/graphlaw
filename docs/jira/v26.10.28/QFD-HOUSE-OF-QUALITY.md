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
| 1.1 | ≤1000 reductions/call or pooled isolation | ALIVE | budget cliff court 10/10, wasm build exit 0 (WORK-ORDERS.md WO-1) |
| 1.2 | Cpk ≥ 1.67 heap-delta | PARTIAL | `spc_court.sh` exit 0, Cpk 6.0979 / 14.4149 on the receipt-fold latency leg (spc-2026-10-28T165319/T171756.json); a heap-delta-specific Cpk is not separately witnessed |
| 1.3 | p99 ≤ 1.2 ms | ALIVE | p99 125/118 ns embedded in the SPC receipts (hot-path fold leg only) |
| 2.1 | 0 false admissions | ALIVE | chicago_validation 12/12 @ 2223f894; mutation court 19 caught / 9 unviable / 0 missed (66fc0e2, `receipts/2026-10-29-mutation-court-v261028.md`) |
| 2.2 | verify per evaluation | ALIVE | proof_pipeline 5/5, `trust_attest` 12/12 exit 0 (WO-3) |
| 3.1 | ANSI SQL over RDF via embedded DuckDB | ALIVE | conformance 12/12, `sql_rdf_dialect` 36/36 (WO-3) |
| 3.2 | 35/35 statement kinds | ALIVE | `duckdb_statement_kinds` 8/8 covering 35 kinds; census 13/13 (WO-4) |
| 4.1 | 100% append-only receipts | ALIVE | capabilities digest 6/6 after repin; per-Edit transition receipts on disk (WO-5) |
| 4.2 | traceability to axioms + leases | ALIVE | abi_manifest_court 10/10; digest repin witnessed (WO-5) |
| 5.1 | `mix` green in-tree | ALIVE | graphlaw_ex suite 133 passed (RECEIPTS.md "GL28-1 legs (b)+(c)"); default `mix test` runs the engine courts (43ea53e) |
| 5.2 | 0 host imports | ALIVE | WASI audit 28/28 `wasi_snapshot_preview1`, zero host imports; packet-4 admit/tamper 2/2 (WO-6) |
| 5.3 | rev-pinned fork source | ALIVE | purrdf fork swap @ 2223f894: `cargo check` 0 errors |

Standings above refreshed 2026-11-09 from WORK-ORDERS.md / RECEIPTS.md (original 2026-10-28 write: UNVERIFIED/PARTIAL pre-closing-wave); it moves only on a
new command + exit. Full per-CTQ commands in [WORK-ORDERS.md](WORK-ORDERS.md).
