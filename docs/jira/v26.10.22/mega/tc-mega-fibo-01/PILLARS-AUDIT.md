# PILLARS-AUDIT — TC-MEGA-FIBO-01 impossibility pillars vs witnessed state

Skeptic lane L40, 2026-10-22. Read-only audit of the operator's five
"impossibility pillars" against what this repo has actually witnessed.
Classification: SUPPORTED (witnessed receipt/court) / REFUTED (contradicting
receipt) / ARCHITECTURE-PRESENT (mechanism exists, performance unwitnessed) /
MISSING (no mechanism on disk). Every narrative claim without commit hash +
command + exit 0 is logged as UNVERIFIED in §8.

Court run this session (Lane L40, /Users/sac/graphlaw, quiet-compile window):

```
$ bash scripts/spc_court.sh
  "cpk": -0.0004,
  "p999_ns": 375,
  "max_ns": 10280959,
  "verdict": "FAIL",
  "reasons": ["cpk -0.0004 < 1.33", "p99 292ns > 250.0ns"]
spc_court: FAIL — process out of control (see receipt above)
```

The gate still writes the receipt; the verdict line is the point. This is the
freshest capability witness in the tree and it reads NOT-CAPABLE.

## P1 — 4.5B-triple intensional N3/Datalog closure, zero materialization

**MISSING.**

- No streaming/intensional-closure machinery anywhere: the only Datalog/
  OWL-RL route is `src/bin/dl_dualroute.rs` (in-memory `BTreeSet` closure,
  dl_dualroute.rs:194-202; goals evaluated against a fully materialized
  set, :227-229) — it materializes by construction. Zero incremental/
  semi-naive/streaming evaluation exists in `src/`.
- No 4.5B-scale anything: the largest witnessed corpus is the 25M-row
  parquet run (designs/pce-baseline.md:9-12). 4.5B triples = 180× that, on
  a claim with no fixture, generator, or court.
- **Traceability harness (operator-mandated: RSS during intensional
  expansion): MISSING.** No `maxrss`/RSS sampling exists in `src/` or
  `tests/` (grep witness: `RSS|maxrss|intensional|closure` over src/ and
  tests/ matches no expansion-memory harness; only dl_dualroute's
  in-memory closure). The benchmark would have to CREATE this harness
  first.

## P2 — 100k nettings > 12M rows/sec, zero runtime SQL text

**REFUTED at claimed scale (mechanism SUPPORTED).**

- Mechanism SUPPORTED: zero runtime SQL text is real — AST-compiled
  execution via `eval_ast_query` (src/duckdb_eval.rs), courted in
  `tests/sql_rdf_dialect.rs` and `tests/duckdb_statement_kinds.rs` (35 PEG
  statement kinds). Read-side scan throughput over real parquet is
  massively above budget (~1.92B rows/s scanned, pce-baseline.md:15-17).
- REFUTED at claimed scale: the charter itself already concedes
  **"PARTIAL, NOT-MET at claimed scale: 500k-row smoke only; 12M printed,
  never gated; the 25M TC_HOT_FULL=1 run is unexecuted"**
  (CHARTER-DFLSS.md:22-26). Bench baseline harness exists
  (`benches/tc_hot_02.rs:255-263` computes rows/sec over the real
  compiled path) but has not produced a ≥12M rows/sec gate at the
  netting scale. 100k nettings is a new fixture (no generator exists —
  `mega/tc-mega-fibo-01/generators/gen_corpus.py` generates the smoke
  corpus, not 100k nettings).
- Traceability harness: `benches/tc_hot_02.rs` — EXISTS, unwitnessed at
  target scale.

## P3 — regulatory refusal in 1-3 ns via pre-compiled bitmasks

**REFUTED (latency) + MISSING (mechanism).**

- Mechanism MISSING: no bitmask structure exists —
  `grep -rn "bitmask|BitMask|bit_mask" src/*.rs` returns zero hits. The
  refusal path is `Refusal` construction + dispatch (src/dialect.rs:152-200
  per TC-HOT-benchmarks.md:13-14); the zero-alloc `RefusalMessage::Static`
  fast path is explicitly **not landed** (CHARTER-DFLSS.md:17-20,
  designs/ctq-traceability.md row 1).
- Latency REFUTED: fresh court witness above — Cpk −0.0004 vs 1.33 gate,
  p99 292 ns vs 250 ns gate (scripts/spc_court.sh, receipt
  receipts/spc-2026-10-22.json: cpk 0.2486, reasons "cpk 0.2486 < 1.33").
  designs/cpk-correction.md:10-16: μ=278 ns, Cpk −0.05, p99=375 ns. The
  1-3 ns order of magnitude is ~2 decades below the witnessed 250-300 ns
  fold path; nothing in the tree operates at ns scale on the hot path.
- Traceability harness: refusal-latency leg of TC-HOT-01
  (TC-HOT-benchmarks.md:13-14) — EXISTS as design, latency UNGATED (no
  100 ns assert exists; charter line 18). No pre-compiled-bitmask leg
  exists to measure.

## P4 — H_query = Fold(h_ast, P_params) as cross-party non-repudiable proof replacing distributed ledgers

**ARCHITECTURE-PRESENT; "replacing distributed ledgers" MISSING.**

- Formula SUPPORTED as mechanism: `H_query = BLAKE3(h_ast || P_params)`
  (src/duckdb_eval.rs:345-368, :521, :549, :818 — two-node anchor-path
  integration, charter A03); law-stability comment at :268. Ed25519
  attestation courted in `tests/trust_attest.rs`.
- SPSC ring push witnessed: `src/evidence_ring.rs` — descriptor is exactly
  64 B `#[repr(C)]` with a compile-time size assert
  (evidence_ring.rs:46-70, `size_of == 64`); drainer + hash-chained ledger
  via `receipt_chain.rs`; joint ring→ledger court green
  (`tests/evidentiary_ring_court.rs` — EXISTS, green per CHARTER-DFLSS.md:29).
- BUT: push latency is the fold path at 250-300 ns (P3's refutation
  applies), and "cross-party non-repudiability" has been witnessed only
  single-party. designs/hot-path-baseline.md and cpk-correction.md:52 name
  the two-node migration as a *prerequisite still in flight* ("the
  two-node formula migration (hq2 lane) is the structural prerequisite").
  No cross-party replay court (two independent verifiers, one proving to
  the other) exists; "replacing distributed ledgers" is a narrative claim
  with no commit, command, or exit code — UNVERIFIED (§8).
- Traceability harness: `tests/evidentiary_ring_court.rs` (SPSC push) —
  EXISTS. Cross-party proof leg: MISSING.

## P5 — client-side WASM pre-admission, 64-byte wire payload

**MISSING (rests on an artifact that does not link).**

- The wasm32-wasip1 module does not link: `rust-lld: unable to find
  library -lduckdb`, pending Lane 1 (docs/wasm-duckdb-op15-design.md:8,
  docs/wasm-duckdb-integration.md:65, docs/WASM.md:31,74-76,
  SQLRDF-08.md:13 — "a link failure, not a capability refusal"). No
  `target/wasm32-wasip1/` exists on disk (ls witness, this session).
- `tests/wasm_memory_wall.rs` EXISTS as a court but is self-skipping when
  the artifact is absent: it resolves the real artifact with a bounded
  poll over the build state and panics with a typed skip naming the
  missing binary (wasm_memory_wall.rs:59-84) — a loud skip, correctly not
  a pass, and today it is exactly that skip.
- 64-byte payload: the 64 B quantity that exists on disk is the ring
  descriptor (evidence_ring.rs:46-70), a host-side evidentiary structure —
  not a client-side WASM wire payload. The wire claim conflates the two.
- Note for fairness: designs/stale-claims-audit.md §P3 records a
  *probable* green lane-isolated link (RECEIPTS.md:58, artifact digest
  match) but marks it "not firsthand-witnessed"; no `target/` artifact on
  disk this session, so MISSING stands until a witnessed `cargo build -p
  graphlaw-wasm --target wasm32-wasip1` exit 0 is filed.

## Traceability matrix (pillar → harness that must MAKE it true)

| Pillar | Harness by name | Status |
|---|---|---|
| P1 closure | RSS-during-intensional-expansion probe | **MISSING** — must be created |
| P1 scale | 4.5B-triple streaming fixture + generator | **MISSING** (largest witnessed corpus 25M rows, pce-baseline.md) |
| P2 throughput | `benches/tc_hot_02.rs` (rows/sec, :255-263) | EXISTS, **NOT-MET at 12M/100k-netting scale** (CHARTER-DFLSS.md:22-26) |
| P2 zero-SQL-text | `tests/sql_rdf_dialect.rs`, `tests/duckdb_statement_courts`→`duckdb_statement_kinds.rs` | EXISTS, green |
| P3 refusal latency | TC-HOT-01 refusal leg (TC-HOT-benchmarks.md:13-14) | EXISTS as design, **UNGATED**; fresh Cpk −0.0004 vs 1.33 (scripts/spc_court.sh) |
| P3 bitmask | pre-compiled bitmask structure + court | **MISSING** (zero grep hits in src/) |
| P4 SPSC push | `tests/evidentiary_ring_court.rs` | EXISTS, green (CHARTER-DFLSS.md:29) |
| P4 cross-party proof | two-verifier non-repudiation court | **MISSING** |
| P5 wasm boundary | `tests/wasm_memory_wall.rs` | EXISTS but **typed-skip today** (artifact does not link) |
| P5 wire payload | 64-byte wire court over a linking artifact | **MISSING** (64 B on disk = ring descriptor, not wire) |

## Log internal-arithmetic check

The operator log's numbers are **mutually incoherent by ~1946×**:

- 12,450,000 rows / 14,210,500 rows/sec ⇒ total wall time = **876 ms**,
  not "≤450 µs total". To do 12.45M rows in 450 µs total would require
  **27.7 G rows/sec**, ~1946× the claimed 14.2M rows/sec.
- The "≤450 µs total" and "214 ns p99" figures cannot both describe the
  same run at that row count under any reading: if 450 µs is the batch,
  per-row time is 36 ps; if 450 µs is per-op, the 214 ns p99 belongs to a
  different measured quantity with no stated mapping.
- 214 ns p99 also contradicts every witnessed fold-path figure: 250-300 ns
  intrinsic (cpk-correction.md:16), p99=375 ns (cpk-correction.md:11),
  292 ns (fresh scripts/spc_court.sh this session). 214 ns is *below the
  measured floor* — likely the `Instant::now()`-pair netting artifact
  cpk-correction.md:46-48 warns about, presented as a headline.

## Ledger: UNVERIFIED narrative claims (no commit + command + exit 0 filed)

| # | Claim | Source | Status |
|---|---|---|---|
| U1 | "4.5B-triple intensional closure" at any triple count | operator log | UNVERIFIED — no fixture, no harness, no receipt |
| U2 | "100k nettings" corpus | operator log | UNVERIFIED — no generator for nettings exists (gen_corpus.py is smoke-only) |
| U3 | "1-3 ns regulatory refusal via pre-compiled bitmasks" | operator log | UNVERIFIED — mechanism absent; witnessed numbers refute the magnitude |
| U4 | "H_query replaces distributed ledgers cross-party" | operator log | UNVERIFIED — no two-verifier court |
| U5 | "client-side WASM pre-admission, 64-byte wire" | operator log | UNVERIFIED — artifact does not link (rust-lld -lduckdb) |
| U6 | "12,450,000 rows / 14,210,500 rows/sec / ≤450 µs / 214 ns p99" | operator log | UNVERIFIED and internally incoherent (§7) |

## Honest scaffold — what the benchmark must MAKE TRUE

1. **Create** the missing harnesses: RSS-sampling intensional-expansion
   probe (P1), nettings corpus generator + 12M-rows/sec gate at netting
   scale (P2), pre-compiled bitmask refusal structure + a real latency
   gate (P3), two-verifier cross-party replay court (P4), a linking wasm
   artifact before any wire-payload claim (P5).
2. **Gate, don't print**: 12M rows/sec must become an asserting gate at
   the claimed scale (TC_HOT_FULL=1 executed, charter line 24 admits it
   never has been).
3. **Reconcile the log arithmetic** before any receipt carries these
   numbers: pick one metric per pillar; a batch wall-time and a rows/sec
   must be mutually consistent to first order.
4. **Move every UNVERIFIED row to SUPPORTED only via** commit hash +
   command + embedded exit-0 output. UNVERIFIED stays UNVERIFIED.
