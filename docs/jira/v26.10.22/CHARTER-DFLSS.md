# CHARTER-DFLSS.md

> **UNVERIFIED CHARTER TEXT**: The operator's charter ("Project Charter &
> Industrial Verification Dossier: The Invariant Data Substrate", DMEDI Track
> Phase 5) was dispatched to this milestone by summary only. No verbatim
> charter text is on disk in this repo. The dispatch-carried figures below are
> retained verbatim and each is adjudicated against on-disk DFLSS lane
> receipts. Do not cite the dispatch-carried numbers alone as receipts — cite
> the witness column.

Status: evidence-complete against on-disk receipts; verbatim charter text
still absent (NOT-MET — see checklist).

> **Count refresh (lane W18, 2026-10-22)**: all court counts below were
> re-executed against the current tree on 2026-10-22; current counts are
> embedded in the refresh block at the end of this file and supersede the
> earlier as-of-time counts where they differ.

## §1.2 CTQs (dispatch-carried summary, adjudicated)

**Refusal latency** — dispatch-carried 0.10 µs — **UNVERIFIED / NOT-MET**:
timed but UNGATED (no 100ns assert exists), the zero-alloc
`RefusalMessage::Static` fast path is not landed, and the 0.10µs figure
appears nowhere in code ([designs/ctq-traceability.md](designs/ctq-traceability.md)
row 1).

**Throughput** — dispatch-carried 12M rows/sec — **PARTIAL, NOT-MET at
claimed scale**: 500k-row smoke only; 12M printed, never gated; the 25M
`TC_HOT_FULL=1` run is unexecuted
([designs/ctq-traceability.md](designs/ctq-traceability.md) row 4,
[designs/TC-HOT-benchmarks.md](designs/TC-HOT-benchmarks.md)).

**Ledger write latency** — dispatch-carried 0.25 µs — **CONTRADICTED-BY-RECEIPT
(was MET-smoke)**: fold path 0.19–0.22 µs on the bench run, but the SPC court
over the same path verdicts FAIL (cpk −0.0018, p99 375 ns > 250 ns USL,
`receipts/spc-2026-10-22.json` + `-inject.json`) — the budget is UNMET as
gated. Ring max push 292 ns vs 10 µs fence; joint ring→ledger court green.
Caveats: 1M not 10M iters, contention ceiling,
eval wrapper UNKNOWN (prepare 302–541 µs/call)
([designs/hot-path-baseline.md](designs/hot-path-baseline.md),
[designs/ctq-traceability.md](designs/ctq-traceability.md) row 3,
[DFLSS-DOSSIER.md §1](DFLSS-DOSSIER.md) Develop table).

## §2 Pareto (dispatch-carried summary, adjudicated)

Dispatch-carried split 48.2 / 27.4 / 16.1 / 8.3 is **ASSERTED, not
witnessed**. The witnessed ledger (22 defects, each with an on-disk witness) is:

| Family | n | Witnessed % | Charter % | Verdict |
|---|---|---|---|---|
| Injection | 10 | 45.5% | 48.2% | NOT WITNESSED (injection #1; ~2.7 pts over) |
| Schema drift | 4 | 18.2% | 27.4% | REFUTED (~1.5× over; strict reading 9.1%) |
| Parser stalls | 3 | 13.6% | 16.1% | NOT WITNESSED (~2.5 pts inflated) |
| Audit | 1 | 4.5% | 8.3% | REFUTED (~1.8× over) |
| Other (test infra) | 4 | 18.2% | — | STRUCTURAL MISS — charter has no bucket for it |

Sensitivity: ±1–2 items (±5 pts) per bucketing, but injection stays <48% and
audit <8.3% under any defensible bucketing — the two REFUTED claims cannot be
rescued. Receipt: [designs/defect-pareto-verified.md](designs/defect-pareto-verified.md).
Consolidated census (dedup across 4 ledgers): 41 unique = 16 fixed + 6
ticketed-open + 3 open-by-design + 16 open-unticketed
([DFLSS-DOSSIER.md §7](DFLSS-DOSSIER.md)).

## §3 p-FMEA (dispatch-carried summary, adjudicated)

The RPN rows (420→10, 240→8, 270→9) are dispatch-carried only — no on-disk
p-FMEA receipt computes them. **UNVERIFIED**; they are not citable as
receipts. The on-disk FMEA that exists is the court-backed table at
[DFLSS-DOSSIER.md §4](DFLSS-DOSSIER.md) (F1–F8): F1 laundering corpus 11/0 at
12:58 PDT, re-run 23/0 on 2026-10-22 (corpus grew in-window — see refresh
block); F2 receipt-invariance chicago 9/0 (re-run 9/0, 2026-10-22); F3 typed
refusals dialect 25/0 at 13:0x, re-run 33/0 on 2026-10-22; F4 wasm 9/9
(as-filed; not re-run by lane W18); F5 cache epoch court 7/0 at 13:5x, re-run
8/0; F6 ring/ledger join — ring 6/0 at filing, re-run 10/0, ledger 9/0
(re-run 9/0); **F7 raw-SQL bypass OPEN** (SQLRDF-17, priority 1); F8 mutation
sweep abort (open-unticketed residue R6).

**Open residual found during verification (this session, verified on disk)**:
the sql_ast expression-depth channel (RecursionLimitExceeded(32), now at
`src/sql_ast.rs:150` and `src/sql_ast.rs:1762`; the serialize-side
`MAX_DEPTH(64)` pointer at `src/sql_ast.rs:573-574` no longer matches the
tree — no `MAX_DEPTH` symbol remains in `src/sql_ast.rs` as of 2026-10-22)
had NO flipping court — deleting the depth guard passed the entire suite.
Closed this session: `deep_nested_expression_refuses_recursion_limit` in
`tests/sql_rdf_laundering_corpus.rs` (40-deep nested sql:leftOperand chain →
typed RecursionLimitExceeded(32); benign 4-level twin flips).

## §5.2 Capability (dispatch-carried summary, adjudicated)

Dispatch-carried μ = 0.198 µs, σ = 0.012 µs, Cpk = 1.44 is **REFUTED by
measurement**. Witnessed (release-mode
`tests/p28_hot_path_bench.rs::p28_hot_path_capability`, n=1,000,000 per-sample
fold-path timings, true M3 path):
μ = 278.0 ns, σ = 202.1 ns, Cpk(Cpu) = **−0.05**, P(X > USL=250 ns) = 42.9%;
quantiles p50=250 / p90=333 / p99=375 / p99.9=458 ns; other runs μ=326/652/
340/370/347 ns, Cpk −0.09..0.00 under fan-out contention. Receipt:
[designs/cpk-correction.md](designs/cpk-correction.md).

SPC court receipt (honest-FAIL, both legs):
`receipts/spc-2026-10-22.json` + `-inject.json` (n=1,000,000, tc_hot_01
clone_path, SMOKE): cpk −0.0018 / −0.0025 < 1.33, p99 375 ns > 250 ns USL,
verdict FAIL both legs including the outlier-injected leg (degenerate flip
until perf closure). Receipt: [RECEIPTS.md, Open defects](RECEIPTS.md) (SPC
supplement, 14:1x PDT); gate `bash scripts/spc_court.sh`.

Verdict: **Cpk 1.44 NOT-MET — NOT-CAPABLE as measured** against USL=250 ns.
The 184–216 ns isolated baseline was contention-sensitive; intrinsic fold cost
is 250–300 ns (budget ADJACENT, not MET, under production load). Close
condition: quiet-machine re-measurement, no concurrent lanes.

## §6 OCAP (dispatch-carried summary, adjudicated)

OCAP control-plan rows: **NOT-LANDED — owned by lane 4; NOT-MET in this
file.** The
skeleton rows the audit must adjudicate are at
[DFLSS-DOSSIER.md §6](DFLSS-DOSSIER.md); lane 4's adjudication is the witness
when it lands. Per-surface courts that exist independent of the OCAP audit:
law admission dialect 25/0 at 13:0x, re-run 33/0 on 2026-10-22, receipt
signing trust_attest 10/0 (re-run 10/0, 2026-10-22), ring 6/0 at filing,
re-run 10/0 + ledger 9/0 (re-run 9/0), cache 7/0 at 13:5x, re-run 8/0, wasm
9/9 (as-filed; not re-run) — all re-read from
[DFLSS-DOSSIER.md §6/§8](DFLSS-DOSSIER.md) at this file's completion, with
the W18 re-execution receipt appended below.

## Count refresh — lane W18, 2026-10-22 (real re-run output)

Commands run from `/Users/sac/graphlaw`; embedded output is the receipt:

- `cargo test --features evidentiary-ring --test evidentiary_ring_court
  --test query_ledger_court --test query_cache_court` →
  `test result: ok. 10 passed; 0 failed` (ring), `8 passed; 0 failed`
  (cache), `9 passed; 0 failed` (ledger)
- `cargo test --test sql_rdf_laundering_corpus --test chicago_validation
  --test trust_attest --test sql_rdf_conformance --test sql_rdf_dialect` →
  23/0, 9/0, 10/0, 7/0, 33/0 respectively, all `ok`.
- Not re-run by this lane (kept as-of-date): `duckdb_statement_kinds`,
  wasm courts. Note: bare `cargo test --test evidentiary_ring_court` FAILS
  loud with a typed SKIP panic naming the required
  `--features evidentiary-ring` invocation — that failure is the gate, not
  a regression.

## Verbatim-landing checklist

- [ ] **NOT-MET** — verbatim charter text filed in this directory (still
      absent; dispatch summary only)
- [x] §1.2 CTQ values adjudicated — refusal latency NOT-MET (ungated);
      throughput NOT-MET at claimed scale; ledger latency MET (smoke) —
      citations above
- [x] §2 Pareto class labels + values re-verified — two charter claims
      REFUTED, one bucket STRUCTURAL MISS —
      [designs/defect-pareto-verified.md](designs/defect-pareto-verified.md)
- [x] §3 p-FMEA RPN rows marked UNVERIFIED (no on-disk witness); residual
      register reconciled with the `tests/sql_rdf_laundering_corpus.rs`
      depth court (closed, this session)
- [x] §5.2 capability stats re-verified — charter Cpk 1.44 REFUTED; measured
      Cpk ≤ 0 — [designs/cpk-correction.md](designs/cpk-correction.md) +
      `receipts/spc-2026-10-22.json`
- [ ] **NOT-MET** — §6 OCAP rows transcribed verbatim (lane 4 owns the
      adjudication; NOT-LANDED at this file's completion time)
