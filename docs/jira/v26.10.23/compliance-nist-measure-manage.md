# NIST AI RMF Compliance Map — MEASURE + MANAGE

Milestone v26.10.23 lane. Repo: `/Users/sac/graphlaw` (read-only lane; this file is the lane's
single write). Scope: map repo evidence to NIST AI RMF v1.0 MEASURE and MANAGE functions.
All counts/status figures below were re-read from disk at write time (2026-10-23).

---

## MEASURE — quantified measurement and methodology

### M1. Performance baselines (Divan, real output embedded)

Receipt: `receipts/2026-10-23-engine-baseline.md` — `cargo bench --bench sql_rdf_engine`,
exit 0, 100 samples × 100 iters, timer precision 41 ns, verbatim Divan table embedded.
Receipt: `receipts/2026-10-23-egress-baseline.md` — `cargo bench --bench sql_rdf_egress`,
exit 0, same shape.

| Metric | Measured median | Receipt |
|---|---|---|
| C1 query-identity hit (`c1_hit_eval`) | 2.541 µs | receipts/2026-10-23-engine-baseline.md |
| Raw engine prepared (`raw_engine_prepared`) | 117.4 µs | same |
| Cold graph-backed eval (`cold_graph_eval`) | 23.13 ms | same (cited 238.5 ms was DISCREPANT; corrected here) |
| Fresh-connection eval | 16.42 ms | same (leg absent from the original cited triple) |
| N-Triples render (`ntriples_format`) | 9.246 ms | receipts/2026-10-23-egress-baseline.md |
| Raw materialization (`egress_result_triples`) | 1.354 ms | same |
| Columnar vs N-Triples egress tax | 6.83× (9.246/1.354) | same; supersedes prose 6.31× |

Correction trail: the prose figures (2.4 µs / 123 µs / 238.5 ms; 6.31×) were replaced with
measured medians; the cold figure was ~10× off and the egress ratio 8.2% off. Both receipts
name the machine (Apple M3 Max class, aarch64) and run condition, satisfying the "cite the
machine + sample condition" requirement the egress receipt itself states.

### M2. Mutation-court catch metrics

- `docs/jira/v26.10.22/designs/mutation-court-full-receipt.md:57-58` — the solo run was
  **refused before any mutant executed** (typed dirty-tree refusal): mutants tested/caught/missed
  = **0 / 0 / 0** with the refusal itself the witnessed outcome; gated surface total 124.
- `receipts/2026-10-23-v26.10.22-terminal.md:126-127` — full mutation verdict stays
  DISK-BLOCKED (`designs/t1-mutation-full-receipt.md`, DISK-BLOCKED) plus the dirty-tree typed
  refusal; the mutation ledger row stays IN PROGRESS with zero status flips.
- Also witnessed: 11+ missed mutants on the `abi.rs` response builders
  (`docs/jira/v26.10.23/ROADMAP.md` gap **G7**), driving Wave B court
  `tests/abi_response_builders_court.rs`.

### M3. Fuzz counts (wasm boundary)

Receipt: `receipts/gws-9-wasm-boundary-fuzz.md` — `cargo test --test wasm_boundary_fuzz`,
**2/2 passed, two consecutive green runs, deterministic** (same counts both runs, seeded
xorshift64* `0x9E37_79B9_7F4A_7C15`).

- **312 hostile calls, 0 panics/traps, 312/312 survival canary** (`capabilities` → `ok:true`
  after every call); every refusal `kind` in the documented vocabulary.
- Per-class coverage: depth 0/1/63/64/65/1000/10000 + random 0–2000; 16 MiB exactly-at/over;
  4 focused malformed-UTF-8 frames; 5 SQL-injection tokens × 3 surfaces (all refused typed
  `InvalidIdentifier`/`ShaclViolation`); NaN/Infinity/1e999 numeric fields; fixed + 40 random
  truncation cuts.
- Finding 2 (fail-closed two-stage 16 MiB boundary) and finding 3 (clean-AST positive-control
  gap) are recorded as gaps below.

### M4. SPC / Cpk methodology (P28-SIL program)

Gate: `scripts/spc_court.sh` (fail-closed, exit 1 on breach). Receipt:
`receipts/2026-10-23-spc-witness.md` — mode SMOKE, 1,000,000 iterations, USL=250 ns,
CPK_MIN=1.33.

- **Verdict FAIL is the honest measurement**: Cpk **0.0008** < 1.33 (p99 209 ns within USL,
  stddev 16282.667 ns from an outlier tail, max sample 7,483,391 ns). The gate exits 1;
  Shewhart rule-1 flagged 33 samples; variance self-check PASS (spread 0.00% vs 5% budget).
- Methodology shape: one-sided Cpk against a one-sided spec limit, Shewhart rules 1/2,
  variance self-check with a 5% spread budget, append-only JSON export
  (`receipts/spc-2026-10-23T152558.json`), variance self-check + positive control built in.
- Prior witness: `docs/jira/v26.22` charter REFUTED the dispatch-carried "Cpk 1.44" claim
  (`docs/jira/v26.10.22/CHARTER-DFLSS.md:90-106`) — the methodology refuses overclaimed
  capability numbers.

### M5. Test coverage courts

- **106 court targets** in `tests/*.rs` (counted on disk at write time; the "85-target"
  figure was not reproduced and is not asserted). Acceptance set named in `CLAUDE.md`
  Commands: `chicago_validation`, `sql_rdf_dialect`, `duckdb_statement_kinds`,
  `trust_attest`.
- Courts are Chicago-style: real frozen-duckdb connections, real readbacks, anti-vacuity
  twin per court (`bash scripts/mutation_court.sh`); witness receipts
  `receipts/chicago-validation-2026-10-22.md`, `receipts/duckdb-statement-kinds-2026-10-22.md`,
  `receipts/sql-rdf-dialect-2026-10-22.md`, `receipts/trust_attest` class in
  `receipts/2026-10-23-terminal-witnesses.md` §2.
- The 35-statement-kind court (`duckdb_statement_kinds.rs`) runs against real frozen-duckdb
  (receipt: `receipts/duckdb-statement-kinds-2026-10-22.md`).

---

## MANAGE — mitigation, escalation, incident response, change management

### MAN-1. Mitigation: typed refusals + fail-closed gates

- Register: `docs/refusals.md` — append-only, stable codes, every code triggered FOR REAL
  with its exact user-visible message captured by `cargo run --example refusal_probe`
  (12/12 exit-class MATCH, exit 0; re-audited lane T4 with every file anchor resolved:
  `src/sql_ast/refusals.rs:9-11`, `src/tier.rs:25`, `src/rules/builtins.rs:89`,
  `src/duckdb_eval.rs:102-104`, `src/attest.rs:38`).
- Fail-closed boundaries witnessed: GWS-9 fuzz — every one of 312 hostile frames answered
  typed, never panicked; 16 MiB boundary is fail-closed at TWO stages (guest `gl_alloc`
  null → typed `Unsupported`; depth 65/1000/10000 → `ResourceLimit`).
- A gate that cannot exit 1 is itself a defect: the eh-court skip path
  `tests/graphlaw_run_eh_court.rs:47-56` calling `std::process::exit(0)` mid-harness is
  filed as a KNOWN COURT GATE DEFECT (`receipts/2026-10-23-terminal-witnesses.md`,
  Witness 1).

### MAN-2. Escalation: the blocker-audit NO-GO → settle loop

- Board: `docs/jira/v26.10.22/designs/integration-gonogo.md` — "NO-GO today; GO gate is
  mechanical" (:70), "NO-GO until P1/P4 land or the operator accepts the wasmtime-host
  decision on P2" (:107). Pending items stay typed (BLOCKED/PENDING, loud OUT) — never
  absorbed into PASS ("Loud-skips honored: L110/L122/L125/L130 have no receipts on disk and
  are reported as OUT, not absorbed into PASS").
- Settle loop: flips only on a landed receipt (P3→READY flip cites a re-read at 19:18;
  P2→BLOCKED flip cites the wasmi-rejection receipt). Escalation is mechanical, not
  judgment-call: GO = P3 lands → 117/0 re-run → commit manifest → commit → P8/P9 sweep.
- Same discipline at register level: GWS register flips governed by "flip only on a landed
  receipt" (ROADMAP Wave A item 3).

### MAN-3. Incident response: staging-wipe incident + recovery

Receipt: `receipts/2026-10-23-staging-wipe-incident.md` (full lifecycle in one file:
detection → cause → impact → recovery → law).

- Detection: discovered 2026-10-23 01:47 (wipe mtime 01:34) of
  `~/.frozen-duckdb/cache/v2.0.0-dev-wasm32-wasip1/` (88,161,726 B libduckdb.a gone).
- Attribution honest: "no lane claimed it; no receipt exists for the deletion — by repo law
  a wiped staging without a receipt is an unreceipted destructive transition."
- Containment: wasm builds typed BLOCKED; survivor `*.legacy-eh.bak/` used as reference only.
- Recovery: staging rebuilt 01:52 via `build_wasi_duckdb.sh --force`, **VERIFY-OK exit 0**,
  byte-size match (88,161,726 B) against the W1 GREEN build; reversible option preserved
  (`libunwind.a.pre-candidate.bak`); canonical expectation re-pinned with a typed FAIL for
  the legacy variant so recurrence fails loudly.
- Post-incident law: "staging transitions must be receipted and coordinator-owned;
  concurrent lanes must never mutate `~/.frozen-duckdb`" — a permanent guard, not a note.
- Honest boundary: RESOLUTION covers staging + libunwind only; the dtor-runner trap remains
  OPEN (LTO matrix / dtor-dump in flight), and no ledger row was flipped on it.

### MAN-4. Change management: receipt-gated flips, append-only registers, pin-by-copy

- **Receipt-gated flips**: GWS/ledger rows flip only on a landed receipt (ROADMAP Wave A
  item 3; integration board MAN-2). Terminal receipt
  `receipts/2026-10-23-v26.10.22-terminal.md:156-171` records zero status flips where no
  receipt exists (mutation row IN PROGRESS).
- **Append-only registers**: `docs/refusals.md` (codes append-only, never renumbered),
  `docs/jira/v26.10.22/RECEIPTS.md` (append-only, `# historical` line-end marker convention
  honored by the staleness gate), `receipts/CHAIN.blake3` chaining.
- **Pin-by-copy flow**: `docs/jira/v26.10.23/ADR-release-lease.md:97-107` — pin-by-copy is
  being replaced by **derive-then-write**: the downstream step re-derives the artifact and
  writes only if its blake3 digest matches the attested digest; typed refusal on mismatch;
  rides `registry/ARTIFACTS.blake3` discipline. Current state: v26.10.22 final artifact
  `472d5557…` (58,121,500 B) pinned by copy at `priv/` + registry + npm 198/198
  (`docs/jira/v26.10.23/ROADMAP.md:8-11`).

---

## Honest gaps (7)

| # | Gap | Evidence pointer |
|---|---|---|
| 1 | **SPC solo rerun owed**: the Cpk FAIL (0.0008) is a fan-out-load-contended run; the solo-machine rerun that would separate load effect from real process incapability has not landed | `receipts/2026-10-23-spc-witness.md`; `docs/jira/v26.10.22/RECEIPTS.md:1471` ("SPC Cpk FAIL under load needs solo-machine rerun") |
| 2 | **Transcript-class witnesses pending durable receipts** (fuzz, memory-wall, exit-codes ×3, dialect census, banking, pm4pytest on `472d5557` lack `receipts/` files; Wave A item 4 in flight) | `docs/jira/v26.10.23/ROADMAP.md` gap D4 |
| 3 | **Full mutation-court verdict never landed** (solo run refused on dirty tree 0/0/0; full run DISK-BLOCKED; ledger row IN PROGRESS) | `designs/mutation-court-full-receipt.md:57-58`; `designs/t1-mutation-full-receipt.md`; `receipts/2026-10-23-v26.10.22-terminal.md:126` |
| 4 | **Court gate defect**: `tests/graphlaw_run_eh_court.rs:47-56` skip path exits 0 mid-harness — a gate that cannot exit 1 on its own skip | `receipts/2026-10-23-terminal-witnesses.md` Witness 1 |
| 5 | **Fuzz positive-control gap**: minimal clean AST does not conform to `ontologies/sql-ast.ttl` — shapes gate stricter than its own doc example | `receipts/gws-9-wasm-boundary-fuzz.md` finding 3 |
| 6 | **Egress baseline machine-condition provenance**: the superseded 6.31× prose figure's machine + sample condition was never identified; current 6.83× is single-machine, single-run | `receipts/2026-10-23-egress-baseline.md` "Audit-finding reconciliation" |
| 7 | **Coverage figure is a target count, not a coverage measurement**: 106 court targets counted on disk; no line/branch coverage tool receipt exists | counted `tests/*.rs` at write time; no coverage receipt in `receipts/` |

## Note on the dispatch's "85-target suite"

The dispatch named an "85-target suite"; a disk count at write time measured **106** `.rs`
targets under `tests/`. The 85 figure is not asserted here; the count was re-read from disk
per the count-staleness law.
