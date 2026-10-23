# DFLSS Industrial Handover Dossier — P28-SIL Program (v26.10.22)

Implement-lane C product. Subject: `/Users/sac/graphlaw` @ `ed3be81` + uncommitted
working set (coordinator owns git). Companion documents: window
[INDEX.md](INDEX.md), [RECEIPTS.md](RECEIPTS.md), [PRD-P28-SIL.md](PRD-P28-SIL.md),
[SQLRDF-18.md](SQLRDF-18.md), designs under [designs/](designs/INDEX.md).

Status legend: **LANDED** (witnessed by an embedded gate output in
[RECEIPTS.md](RECEIPTS.md) or below), **DESIGNED** (spec on disk, implementation
queued), **PENDING-LANE** (owned by a concurrent DFLSS lane — do not fabricate),
**UNMET** (honest residue, registered).

Lane statuses in this dossier were re-read from the window register
([RECEIPTS.md](RECEIPTS.md), [designs/RECEIPTS.md](designs/RECEIPTS.md)) at
dossier-assembly time (2026-10-22, ~14:0x PDT), not recalled from session memory.
One divergence found and carried: the receipt-surface digest pin went **stale**
after landing (pinned `blake3:dcd46bf6…` vs live `blake3:049add3e…`) — see the
residue register.

> **Refresh (lane W18, 2026-10-22)**: §8's gate ladder was re-executed against
> the current tree on 2026-10-22; several counts moved in-window (ring 6→10,
> cache 7→8, dialect 25→33, laundering 11→23, conformance 5→7). Code pointers
> re-verified and corrected: `canon_graph_bytes` field now
> `src/sql_ast/ast.rs:335` (initializer `src/sql_ast.rs:1238`), the
> `evidentiary-ring` feature now `Cargo.toml:63`, and the raw-SQL hash arm is
> `(None, None)` at `src/duckdb_eval.rs:672` (not a `(None, None, None)` arm
> in `src/query_ledger.rs`). Refresh counts at the end of §8.

---

## 1. DMEDI tollgate summary

### Define — PASSED (filed)

- PRD "GraphLaw Substrate & P28-SIL Zero-Overhead Execution Architecture"
  v1.0.0 filed verbatim at [PRD-P28-SIL.md](PRD-P28-SIL.md); program ticket
  [SQLRDF-18.md](SQLRDF-18.md) carries phases, BLOCKED-BY edges, and the
  landed/designed split. Tollgate evidence: both files on disk (window listing,
  re-verified at dossier assembly).
- License packaged: [LICENSE-P28-SIL.md](../../LICENSE-P28-SIL.md) (SIL grant +
  MIT retention, line 4), witnessed in [RECEIPTS.md](RECEIPTS.md) §4.

### Measure — PASSED (baseline landed; Cpk PENDING)

- Hot-path baseline: [designs/hot-path-baseline.md](designs/hot-path-baseline.md)
  (DMEDI Measure design record, release-mode `tests/p28_hot_path_bench.rs`,
  min-of-batches, 1102-byte canon + 4 typed params):
  - H_query incremental fold: **184–216 ns/op** — INSIDE the TC-HOT-01 ≤250 ns
    budget; marginally over the ≤150 ns §2 row (residual = ~2 KB hasher-state
    memcpy in `clone()`).
  - H_query full rehash: **1673–1748 ns/op** — MISS ~11.6× over the ≤150 ns
    budget; this is the documented §3.1.1 fallback to close.
  - Admission (compile_turtle: SHACL + RDFC-1.0): 1.56–4.97 ms/compile — once
    per compile, excluded from hot-path claims per PRD §2.
  - eval_ast_query wrapper overhead: UNKNOWN at sub-µs (duckdb prepare
    dominates, 302–541 µs/call); needs a pinned-core harness or the
    prepared-statement reuse seam (which has since LANDED — see Develop).
- **Cpk: PENDING-LANE** (lane 5 / SPC lane output). No Cpk receipt exists yet;
  the section-7 placeholder below carries the required receipt shape.

### Explore — PASSED (audits + fuzz courts landed)

- Escape audits: findings E1–E7 witnessed; E1/E3/E4/E5 escaped-closed by L2's
  never-both + typed-parse + cardinality fixes with courts; E6 refuted (not a
  defect); E7 declared by design. Citation: [RECEIPTS.md](RECEIPTS.md) Blockers
  ("Escape-audit findings E1–E7 — resolved as follows (as-of 12:45 PDT)").
- Fuzz courts: laundering corpus 11/11 —
  `cargo test --test sql_rdf_laundering_corpus` → "running 11 tests", 12:58 PDT,
  embedded output at [designs/RECEIPTS.md](designs/RECEIPTS.md) §2 (as reported
  by the corpus lane); re-run 23/0 on 2026-10-22 (corpus grew in-window).
- OCAP audit: **PENDING-LANE** (lane 4). Placeholder in §7 below.

### Develop — PASSED (landed + designed/queued split is explicit)

Landed (code + courts):

| Item | Witness | Gate + result |
| --- | --- | --- |
| R1 canon-once | `CompiledQuery.canon_graph_bytes` — `src/sql_ast/ast.rs:335` (field), `src/sql_ast.rs:1238` (initializer); pointers re-verified 2026-10-22, the earlier `src/sql_ast.rs:367/383/511` pointers predate the module split | chicago 9/9, lib 17/17, dialect 25/25 at 12:57 PDT (re-run 33/0 on 2026-10-22), trust 10/0 (re-run 10/0) |
| M3 receipt_state (fold path) | fold 184–216 ns/op inside TC-HOT-01 budget | [designs/hot-path-baseline.md](designs/hot-path-baseline.md) |
| C1-a cache store | `src/query_cache.rs` (epoch-bounded, F1–F5) | `cargo test --test query_cache_court` → 7 passed; 0 failed (0.08s), re-executed 13:5x PDT |
| Evidentiary SPSC ring | `src/evidence_ring.rs`, feature `evidentiary-ring` (`Cargo.toml:63`), default-off | ring court 10/0 (re-run 2026-10-22; 6/0 at filing) incl. `joint_ring_to_query_ledger_court`; max push **292 ns** vs 10 µs fence; 1M-push no-blocking property test |
| Cold-path query ledger | `src/query_ledger.rs` (F1–F7 + format court) | `cargo test --test query_ledger_court` → re-executed by THIS lane at dossier assembly (~14:0x PDT), embedded output: `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.43s`, exit 0 (re-run 9/0 on 2026-10-22, finished 0.37s) |
| Prepared-statement seam | LRU 256; `$N`-bound params so same digest+params hits the cached handle | part of the Phase-2 landing batch |
| Join courts, TC-HOT benches | `tests/p28_hot_path_bench.rs`, `tests/evidentiary_ring_court.rs` | per rows above; mtime 13:40/13:45 PDT |

Designed + queued (not built):

| Item | Design | Queue edge |
| --- | --- | --- |
| E1 alias elimination | [designs/E1-alias-elimination.md](designs/E1-alias-elimination.md) | first in E-order (E1→A1→E2→E3), BLOCKED-BY integration commit |
| A1 scope-as-type | [designs/A1-scope-as-type.md](designs/A1-scope-as-type.md) | after E1 |
| E2 shapes reader | [designs/E2-shapes-reader.md](designs/E2-shapes-reader.md) | after A1 |
| C1 completion (full cache program) | [designs/C1-cache.md](designs/C1-cache.md) | after R1 (landed) + E1 |
| Phase 1 generated gate | [designs/PHASE1-generated-gate.md](designs/PHASE1-generated-gate.md) | BLOCKED-BY E1→A1→E2 |
| Receipt-surface pin | [designs/receipt-surface-pin.md](designs/receipt-surface-pin.md) | LANDED-then-STALE — needs registry regeneration, not a hand patch (see residue) |

### Implement — THIS DOSSIER (in progress)

- This dossier (lane C).
- SPC court: **PENDING-LANE** (lane 5).
- OCAP court: **PENDING-LANE** (lane 4).

---

## 2. CTQ matrix — PENDING-LANE (lane 1)

Lane 1 owns the CTQ (critical-to-quality) matrix deriving CTQs from the PRD's
quality functional deployment. Until it lands, the anchor set the matrix must
cover is derivable from the PRD + landed baselines: H_query fold ≤250 ns
(TC-HOT-01), ingestion ≥12M rows/sec (TC-HOT-02, design
[designs/TC-HOT-benchmarks.md](designs/TC-HOT-benchmarks.md)), receipt
invariance (`H_query = BLAKE3(Canon(G_AST) || P_params)`, chicago 9/0), typed
refusal coverage (REFUSAL-01..04, dialect 25/0 at 13:0x, re-run 33/0 on
2026-10-22), wasm boundary (contract 7/0, as-filed).
**PENDING-LANE — do not treat this list as the matrix.**

---

## 3. Measured baselines

Canonical record: [designs/hot-path-baseline.md](designs/hot-path-baseline.md)
(re-quoted in §1 Measure). Ratios are the signal; absolute numbers are
ceilings measured under concurrent fan-out (honesty note from the design
record). Bench-machine disclosure requirement (ARCH, cores, thermal, NEON-SIMD
note) is baked into every future receipt per
[designs/TC-HOT-benchmarks.md](designs/TC-HOT-benchmarks.md).

---

## 4. FMEA (failure mode / effect / mitigation) — citing the laundering corpus + REFUSAL courts

| # | Failure mode | Effect | Current control (court) | Det/occurrence evidence |
| --- | --- | --- | --- | --- |
| F1 | SQL laundering through the admitted-statement surface | hostile SQL executes under an admitted identity | laundering corpus: `cargo test --test sql_rdf_laundering_corpus` → 11/11 (12:58 PDT, embedded at [designs/RECEIPTS.md](designs/RECEIPTS.md) §2); re-run 23/0 on 2026-10-22 (corpus grew in-window) | corpus cases e1–e4 incl. adjudication of e4 |
| F2 | Admitted-statement receipt collision / cosmetic drift | a different query replays under the same receipt | receipt formula court: `H_query = BLAKE3(Canon(G_AST) \|\| P_params)` — chicago_validation 9/0, 5-variant cosmetic-invariance set green (13:0x) | [RECEIPTS.md](RECEIPTS.md) Aggregate |
| F3 | Untyped compile refusal (generic error where a typed refusal belongs) | refusal contract drift | REFUSAL-01..04 stable codes in `docs/refusals.md`; courts green inside `cargo test --test sql_rdf_dialect` → 25/0 (13:0x); re-run 33/0 on 2026-10-22 | [RECEIPTS.md](RECEIPTS.md) SQLRDF-06/07/10 rows |
| F4 | wasm-boundary capability leak (SQL executes sandbox-side) | sandbox boundary broken | wasm refusal courts 9/9 (`cargo test -p graphlaw-wasm`), boundary build green in lane-isolated target dir (4m25s) | [RECEIPTS.md](RECEIPTS.md) SQLRDF-08 row |
| F5 | Cache poisoning across epochs | stale eval result served under a moved epoch | C1-a epoch-bounded store + `query_cache_court` 7/0 at 13:5x; re-run 8/0 on 2026-10-22 | §1 Develop table |
| F6 | Evidence loss under drainer stall | silent drop in the ring | no-blocking property test (1M pushes vs stalled drainer, drops loud) + join court (drop gaps persisted to ledger exactly, verify passes) | §1 Develop table |
| F7 | Raw-SQL bypass of the admitted path (C2 hole P1) | capability route around the law | **OPEN — not yet controlled**: SQLRDF-17 filed
([SQLRDF-17.md](SQLRDF-17.md)); control lands with C2 closure | SQLRDF-17 Current-behavior pointers verified 12:52 PDT |
| F8 | Anti-vacuity sweep cannot run | gates could pass vacuously | per-court inversions (local evidence); **full `bash scripts/mutation_court.sh` sweep UNMET this window** — aborts pre-mutation at `tests/test_cold_chain_ocel.rs:61` (observed 12:33 PDT) | [RECEIPTS.md](RECEIPTS.md) Open defects |

---

## 5. Cpk receipt — PENDING-LANE

Required shape (fill by the SPC/Cpk lane, not this lane): per-CTQ
process-capability receipt with (a) the gate command, (b) measured mean/σ over
N≥30 consecutive runs, (Cpk = min((USL−µ)/3σ, (µ−LSL)/3σ)), (c) embedded real
output, (d) the bench-machine disclosure from
[designs/TC-HOT-benchmarks.md](designs/TC-HOT-benchmarks.md). Candidate CTQs
with existing budget fences: H_query fold (USL 250 ns), ring max push
(USL 10 µs; witnessed 292 ns), receipt invariance (attribute — go/no-go, Cpk
N/A, report defect rate). **No Cpk number exists yet — none may be asserted.**

---

## 6. OCAP table — PENDING-LANE (lane 4)

OCAP (outcome, control, audience, authority, process) audit owned by lane 4.
Skeleton rows the audit must adjudicate, from the landed program surface:

| Surface | O (outcome) | C (control/court) | A (authority) | Adjudication |
| --- | --- | --- | --- | --- |
| Law admission (`src/law.rs`/`sql_ast.rs`) | typed admission or REFUSAL-01..04 | dialect 25/0 | sql-rdf conformance text | PENDING-LANE |
| Receipt signing (`src/attest.rs`) | Ed25519 receipt | trust_attest 10/0 (re-run 10/0, 2026-10-22) | Σ_PQC registered UNMET | PENDING-LANE |
| Evidentiary ring → ledger | drop-gap persistence | ring court 10/0 + ledger court 9/0 (both re-run 2026-10-22) | feature-gated, default-off | PENDING-LANE |
| Eval cache | epoch-bounded hit/miss | cache court 8/0 (re-run 2026-10-22) | receipt law untouched | PENDING-LANE |
| wasm boundary | capability refusal | wasm courts 7/7 (as-filed; not re-run by lane W18) | no `duckdb` feature on the wasm `graphlaw` dep (`wasm/Cargo.toml:18`) | PENDING-LANE |

---

## 7. Honest-residue register — final consolidated census

**Authoritative honest-residue census (defect census lane, consolidated).**
41 unique defects, four-way split:

| Bucket | Count | Disposition |
| --- | --- | --- |
| Witnessed + fixed | 16 | closed in-window with court receipts |
| Ticketed-open | 6 | SQLRDF-13, -14, -15, -17, -19, -20 (SQLRDF-16 is the moonshot backlog family; its program rows are ticketed separately — C2 spun out as SQLRDF-17 — so no additional count from -16 itself) |
| Open-by-design | 3 | restart watermark deviation (accepted, see [designs/defect-pareto-verified.md](designs/defect-pareto-verified.md)), E7, `Σ_PQC` (registered limitation, Ed25519-only this window) |
| Open-unticketed | 16 | tracked on the census lane's per-source ledgers; file as follow-up windows open |

Total: **16 + 6 + 3 + 16 = 41**.

### Per-source attribution (pre-dedup, deduped to 41 unique)

| Source ledger | Defects cited | Note |
| --- | --- | --- |
| [designs/defect-pareto-verified.md](designs/defect-pareto-verified.md) | 22 | witnessed population (fix rate 16/22 = 72.7%); the wave's "9 found + fixed" was a FIXED-ONLY count |
| [designs/spof-scan.md](designs/spof-scan.md) | 17 | single-point-of-failure scan |
| [designs/p28-sil-capability-audit.md](designs/p28-sil-capability-audit.md) | 12 | P28-SIL capability audit |
| Dossier residues (prior §7 R1-R8) | 8 | dossier residues R1-R8, folded into the four-way split above |

Sources overlap; dedup across ledgers yields the 41 unique. See
[designs/defect-pareto-verified.md](designs/defect-pareto-verified.md) for the
dedup pointer.

### Ticketed-open items and BLOCKED-BY chains

| Ticket | Title | Standing |
| --- | --- | --- |
| SQLRDF-13 | Validation-scoped RDFS closure for §4 `sh:class` constraints | Open |
| SQLRDF-14 | §3.1 letter conformance — re-parent the 35 PEG leaf classes under the six family classes | Open |
| SQLRDF-15 | §6.2 tpch setup divergence — retire the sf=0.01 hand-patch into the generator + register | Open; currently BLOCKED (pinned sources absent — restore pin → regenerate, never hand-patch) |
| SQLRDF-17 | ABI duckdb exec accepts raw SQL text (C2 hole P1) | BLOCKED-BY: v26.10.22 integration commit |
| SQLRDF-19 | CI feature-matrix gaps — untested build cells | BLOCKED-BY: v26.10.22 integration commit |
| SQLRDF-20 | BEAM hot-upgrade protocol: artifact swap under the supervisor, with its own court | BLOCKED-BY: v26.10.22 integration commit + SQLRDF-19 |
| SQLRDF-16 | Moonshot backlog — 1000× ERRC horizon (family; rows ticketed separately) | BLOCKED-BY: v26.10.22 integration commit |

### Priority-1 note

The legacy raw-SQL hash arm — `(None, None)` at `src/duckdb_eval.rs:672`
(SQLRDF-17 comment at `:664`; pointer corrected 2026-10-22 from the earlier
"`(None, None, None)` in `src/query_ledger.rs`", which matched no tree
symbol) — converges 5 audit lanes (the cold-path ledger accepts unkeyed rows
from multiple audit surfaces). Convergent unkeyed rows are the load-bearing
path behind the C2 raw-SQL bypass; **SQLRDF-17 is priority 1** for the next
window.

### Prior-residue disposition (superseded R-rows folded in)

R1 `Σ_PQC` → open-by-design. R2 `.gram` environmental → open-unticketed
(external checkout absent, guarded surface VERIFIED-EXTERNAL 21/21). R3 →
ticketed SQLRDF-15. R4 cold-path query-path wiring → open-unticketed
(Phase-2 order). R5 digest-pin staleness → open-unticketed (re-pins at
integration). R6 mutation-sweep abort → open-unticketed (per-court inversions
remain local evidence). R7 → ticketed SQLRDF-17. R8 OCAP/CTQ/SPC lanes →
resolved at this dossier's integration; census supersedes the placeholder.

---

## 8. Gate commands (replay index)

From cwd `/Users/sac/graphlaw`:

- `cargo test --test sql_rdf_dialect` → 25/0
- `cargo test --test chicago_validation` → 9/0
- `cargo test --test duckdb_statement_kinds` → 5/1 (env: `.gram`)
- `cargo test --test trust_attest` → 10/0 (as-of-date)
- `cargo test --test sql_rdf_conformance` → 5/0
- `cargo test --test sql_rdf_laundering_corpus` → 11/0
- `cargo test --test query_ledger_court` → 9/0
- `cargo test --test query_cache_court` → 7/0
- `cargo test --test evidentiary_ring_court` → 6/0
- `cargo test -p graphlaw-wasm` → 7/0; boundary build green (lane-isolated dir)
- `bash scripts/mutation_court.sh` → UNMET (abort, R6)

All counts re-read from [RECEIPTS.md](RECEIPTS.md) /
[designs/RECEIPTS.md](designs/RECEIPTS.md) at dossier-assembly time (~14:0x
PDT); §1 Develop re-executions are embedded there. Per the count-recall law the
coordinator should re-run the §8 ladder at integration before flipping Status
fields.

### §8 refresh — lane W18 re-run, 2026-10-22 (real output)

From cwd `/Users/sac/graphlaw`:

- `cargo test --features evidentiary-ring --test evidentiary_ring_court
  --test query_ledger_court --test query_cache_court` → ring
  `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered
  out; finished in 15.27s`; cache `8 passed; 0 failed ... 0.12s`; ledger
  `9 passed; 0 failed ... 0.37s`
- `cargo test --test sql_rdf_laundering_corpus --test chicago_validation
  --test trust_attest --test sql_rdf_conformance --test sql_rdf_dialect` →
  laundering 23/0 (0.20s), chicago 9/0 (2.62s), trust_attest 10/0 (0.06s),
  conformance 7/0 (0.12s), dialect 33/0 (0.30s), all `test result: ok`.
- Not re-run: `duckdb_statement_kinds`, wasm courts, mutation sweep.
  Bare `cargo test --test evidentiary_ring_court` (without the feature)
  fails loud with a typed SKIP panic naming the required
  `--features evidentiary-ring` invocation — that is the gate behaving, not
  a regression.

Deltas vs the as-of-14:0x counts above: ring 6→10, cache 7→8, dialect
25→33, laundering 11→23, conformance 5→7; chicago 9/0, ledger 9/0,
trust_attest 10/0 unchanged.
