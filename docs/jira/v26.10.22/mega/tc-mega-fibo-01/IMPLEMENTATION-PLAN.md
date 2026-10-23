# TC-MEGA-FIBO-01 — Implementation Work-Order Plan

Lane L58 (design), 2026-10-22. Converts the setup scaffold (corpus,
ontology, twins, COURT-SPEC.md, PILLARS-AUDIT.md) into the dispatch-ready
work-order list for the implementation lane. Zero Rust written by this lane;
every row names the gate command that proves it. Statuses re-read from
COURT-SPEC.md §1 and PILLARS-AUDIT.md at plan time — re-verify at dispatch.

**Status refresh (2026-10-22 late session, lane L179)**: per-WO status lines
below were re-read from the receipts on disk; status vocabulary is
LANDED / IN-FLIGHT / BLOCKED / NOT-STARTED. An IN-FLIGHT row's receipt is its
dispatch; a completion receipt does not exist yet — no status reads LANDED on
the strength of a dispatch. Two environment changes touch multiple rows: the
TC-HOT corpus now lives at `var/tc-hot-25m` (durable relocation, L159, with
the `TC_HOT_CORPUS_PATH` knob re-landed by N22 —
`designs/tc-hot-full-receipt.md` "Durable corpus relocation (L159)" and
"Lane N22" sections), and the eyeron switchover has landed (facades + wired
split dirs, staging deleted; provenance gap documented in
`designs/commit-receipt-provenance-gap.md`) — file pointers below that name
pre-split `src/dialect.rs` paths are historical. The receipt-surface pin is
now 6-entry with `registry_artifacts` 22/0
(`designs/registry-regen-receipt.md`, P7 regen applied) — superseding the
2-entry pin named in `designs/receipt-surface-post-switchover.md` (N27).

Conventions:

- MECHANISM-EXISTS = the code path/harness exists in the repo; the row is
  wiring/court work. NEW-MECHANISM = no mechanism on disk today
  (PILLARS-AUDIT verdict); the row includes design + build + bench.
- Gates are command FORMS; test targets marked `(new)` are created by the
  row that owns them. Every gate must exit 0 on success and be capable of
  exit non-zero (a gate that cannot flip is vacuous — WO-12 owns the flip
  court for the phase-4 falsifier).
- Depends-on uses work-order ids; "Lane 1" = the wasm-link lane per
  PILLARS-AUDIT P5.

## Work orders

### WO-01 — Phase 1 (P1.3) — full-shape-corpus admission — serves M2

- Class: MECHANISM-EXISTS (SHACL admission path).
- Deliverable: admission run of the full 1,741-component / 367-shape
  corpus; count re-derived from admitted shape IRIs, not generator stdout.
- Files owned: `tests/tc_mega_shapes_admission.rs` (new).
- Gate: `cargo test --test tc_mega_shapes_admission`
- Depends-on: WO-11, WO-12.
- Status: IN-FLIGHT (2026-10-22) — admission-run lane dispatched; corpus on
  disk at `var/tc-hot-25m` (L159 relocation receipt,
  `designs/tc-hot-full-receipt.md`). Completion receipt pending.

### WO-02 — Phase 1 (P1.1, P1.2) — 100k-leg admission spine — serves M1, M8

- Class: MECHANISM-EXISTS (`src/sql_ast.rs` compile_turtle, `src/law.rs`).
- Deliverable: full-corpus compile + law admission; typed-refusal stdout
  scan (zero untyped error strings); batch-timed wall clock printed
  (M1 reported, not gated, until WITNESSED twice).
- Files owned: `tests/tc_mega_admission.rs` (new).
- Gate: `cargo test --test tc_mega_admission --release`
- Depends-on: WO-01.
- Status: NOT-STARTED (2026-10-22) — no dispatch or completion receipt on
  file; unchanged from plan time.

### WO-03 — P1 closure — RSS-during-intensional-expansion probe — serves M1

- Class: NEW-MECHANISM (audit: harness MISSING; only in-memory closure at
  `src/bin/dl_dualroute.rs:194-229` exists).
- Deliverable: maxrss sampling around the N3/Datalog closure run over the
  SA-CCR/EMIR stratified rule layers.
- Files owned: `tests/tc_mega_rss_closure.rs` (new).
- Gate: `cargo test --test tc_mega_rss_closure --release`
- Depends-on: WO-02.
- Status: IN-FLIGHT (2026-10-22) — RSS probe lane dispatched. Completion
  receipt pending.

### WO-04 — P2 (P2.2) — scale-gated throughput court — serves M3

- Class: MECHANISM-EXISTS (harness `benches/tc_hot_02.rs:255-263` exists;
  gate unwired — printed, never gated, CHARTER-DFLSS.md:22-26).
- Deliverable: wire `TC_HOT_FULL=1` into an ASSERTING >=12M rows/sec gate
  at netting scale.
- Files owned: `benches/tc_hot_02.rs` (gate wiring),
  `tests/tc_mega_throughput_gate.rs` (new).
- Gate: `TC_HOT_FULL=1 cargo test --test tc_mega_throughput_gate --release`
- Depends-on: WO-05.
- Status: IN-FLIGHT (2026-10-22) — corpus durable at `var/tc-hot-25m`
  (L159, `designs/tc-hot-full-receipt.md`); bench restored with the
  `TC_HOT_CORPUS_PATH` knob (N22, same receipt "Lane N22" section); the
  quiet-gate reattempt is queued post-L110. Completion receipt pending.

### WO-05 — Phase 2 (P2.1, P2.3) — eval readbacks — serves M3, M9

- Class: MECHANISM-EXISTS (`src/duckdb_eval.rs` prepared-statement path).
- Deliverable: prepared eval of the compression/tear-up query family over
  the 100k legs; exact row counts hardcoded from
  `generators/gen_corpus.py`; real frozen-duckdb readbacks (Chicago).
- Files owned: `tests/tc_mega_eval_readbacks.rs` (new).
- Gate: `cargo test --test tc_mega_eval_readbacks --release`
- Depends-on: WO-02.
- Status: NOT-STARTED (2026-10-22) — no dispatch or completion receipt on
  file.

### WO-06 — P3 refusal — pre-compiled bitmask fast path — serves M8

- Class: NEW-MECHANISM (audit: zero `bitmask` hits in `src/*.rs`;
  zero-alloc `RefusalMessage::Static` not landed). Honest scope: design +
  build + bench; the 1-3 ns operator figure stays UNVERIFIED until this
  row's own bench witnesses a number.
- Deliverable: bitmask refusal structure + court + latency bench.
- Files owned: `src/bitmask_refusal.rs` (new),
  `tests/bitmask_refusal_court.rs` (new).
- Gate: `cargo test --test bitmask_refusal_court --release`
- Depends-on: WO-12.
- Status: NOT-STARTED (2026-10-22) — still NEW-MECHANISM, no lane
  dispatched; unchanged from plan time.

### WO-07 — P3 latency — gated refusal-latency leg — serves M8

- Class: MECHANISM-EXISTS (TC-HOT-01 refusal leg designed, UNGATED — no
  100 ns assert exists today).
- Deliverable: typed-refusal timing ASSERT wired to the refusal leg.
- Files owned: `tests/tc_mega_refusal_latency.rs` (new).
- Gate: `cargo test --test tc_mega_refusal_latency --release`
- Depends-on: WO-06.
- Status: NOT-STARTED (2026-10-22) — no dispatch or completion receipt on
  file.

### WO-08 — P4 proof — two-verifier non-repudiation court — serves M6

- Class: NEW-MECHANISM (audit: cross-party witnessed single-party only;
  no two-verifier court exists).
- Deliverable: verifier A folds `H_query = BLAKE3(h_ast || P_params)`,
  signs (Ed25519); verifier B independently replays and verifies.
- Files owned: `tests/crossparty_nonrepudiation.rs` (new).
- Gate: `cargo test --test crossparty_nonrepudiation --release`
- Depends-on: WO-09.
- Status: IN-FLIGHT (2026-10-22) — two-verifier court lane dispatched.
  Completion receipt pending.

### WO-09 — Phase 3 (P3.1-P3.3) — ring/ledger over 100k legs — serves M4

- Class: MECHANISM-EXISTS (`src/evidence_ring.rs`,
  `src/receipt_chain.rs`, `tests/evidentiary_ring_court.rs`).
- Deliverable: 100k-leg ring fold + real-consumer drain (byte-identity per
  entry) + receipt-chain end-to-end verify; push cost printed vs M4
  (reported until WITNESSED twice).
- Files owned: `tests/tc_mega_ring_ledger.rs` (new).
- Gate: `cargo test --test tc_mega_ring_ledger --release`
- Depends-on: WO-02.
- Status: NOT-STARTED (2026-10-22) — no dispatch or completion receipt on
  file.

### WO-10 — P5 wasm — link-dependent client sim — serves M7

- Class: NEW-MECHANISM (audit: artifact does not link, `rust-lld -lduckdb`;
  `tests/wasm_memory_wall.rs` is a typed skip today).
- Deliverable: 64-byte wire payload pre-admission client sim over a
  LINKING wasm32-wasip1 artifact.
- Files owned: `tests/tc_mega_wasm_clientsim.rs` (new).
- Gate: `cargo build -p graphlaw-wasm --target wasm32-wasip1 --profile
  wasm && cargo test -p graphlaw-wasm && cargo test --test
  tc_mega_wasm_clientsim`
- Depends-on: Lane 1 (wasm link); until then a loud typed skip, never a
  silent pass.
- Status: SPLIT (2026-10-22). Lane 1 (wasm link) is GREEN
  (`RECEIPTS.md` "Successor wave 4": L1 GREEN; artifact links with a wasm
  Tag/EH section). The EH wall state changed: L110's census
  (`designs/sweep-post-link.md:56,133`) named the single test-host fix —
  `wasmtime::Config::wasm_exceptions(true)` in `tests/common/mod.rs:115`
  `Host::new()` — and lane N28 is dispatched to land it. The PRODUCTION EH
  route remains OPERATOR-DECISION (OD-1,
  `designs/wasmi-eh-research.md`: no wasmi release supports EH; wasmtime 48
  fails legacy-EH too; node/V8 runs it) — until that decision, the row stays
  BLOCKED for production runtimes and a loud typed skip for the host test
  path.

### WO-11 — Reconciliation 4 — operator unification — serves M2

- Class: MECHANISM-EXISTS (operator enum exists; the alias row is new).
- Deliverable: unify `math:lessThan` (N3) / `sql:LessThan` (axiom IRI) /
  `<` (`Operator::from_token`, `src/sql_ast.rs`) to one canonical form;
  record the mapping in the dialect pack.
- Files owned: `ggen/sql-dialect/ontology/statements.ttl`,
  `src/sql_ast.rs` (operator IRI map), `tests/sql_rdf_dialect.rs` (ext).
- Gate: `cargo test --test sql_rdf_dialect`
- Depends-on: none.
- Status: NOT-STARTED (2026-10-22) — no dispatch or completion receipt on
  file. Note: `src/dialect.rs` file pointers in WO definitions are
  historical — the switchover landed the split trees (facade + wired dirs,
  staging deleted; `designs/commit-receipt-provenance-gap.md`).

### WO-12 — Reconciliation 2 — decimal-scale enforcement — serves M8

- Class: NEW-MECHANISM (compiler has no decimal-scale check today; only
  SHACL `sh:pattern`, no sql-ast analogue).
- Deliverable: compiler-native typed refusal for xsd:decimal lexical
  precision; feeds the #41209 falsifier's byte-positioned refusal.
- Files owned: `src/sql_ast.rs` (literal check), `src/dialect.rs`
  (RefusalCode), `tests/tc_mega_precision_refusal.rs` (new).
- Gate: `cargo test --test tc_mega_precision_refusal`
- Depends-on: none.
- Status: NOT-STARTED (2026-10-22) — no dispatch or completion receipt on
  file; split-tree note under WO-11 applies to `src/sql_ast.rs` /
  `src/dialect.rs` pointers here too.

### WO-13 — Reconciliation 1 — `reg:` namespace registration — serves M2

- Class: MECHANISM-EXISTS (registry flow exists; the entry is new).
- Deliverable: register `http://graphlaw.org/ns/mega/fibo/reg#` in the
  dialect/registry surface so scenario shapes admit without silent
  renaming.
- Files owned: `ggen/sql-dialect/` pack, `src/registry.rs` re-pin,
  `tests/registry_dispatch.rs` (ext).
- Gate: `cargo test --test registry_dispatch`
- Depends-on: none.
- Status: IN-FLIGHT (2026-10-22) — `reg:` namespace reconciliation lane
  dispatched. Note: the receipt-surface pin is now 6-entry
  (`designs/registry-regen-receipt.md`, P7 regen applied,
  `registry_artifacts` 22/0), superseding the 2-entry analysis in
  `designs/receipt-surface-post-switchover.md` (N27); the registry re-pin
  (C9) already closed on the regen flow. Completion receipt for the `reg:`
  entry pending.

### WO-14 — Phase 4 (P4.1-P4.5) — anti-vacuity court — serves M8

- Class: MECHANISM-EXISTS (twins + `scripts/mutation_court.sh` protocol
  exist; the court is new).
- Deliverable: twins #41209 (benign admits, mutated emits a typed
  RefusalCode); BLAKE3 state-hash zero-mutation assert; single-shape
  constraint-removal flip proving the constraint causes the refusal.
- Files owned: `tests/tc_mega_antivacuity.rs` (new).
- Gate: `cargo test --test tc_mega_antivacuity --release && bash
  scripts/mutation_court.sh`
- Depends-on: WO-12.
- Status: IN-FLIGHT (2026-10-22) — twin court lane dispatched; may return
  BLOCKED-BY-WO-13 if `reg:` namespace blocks admission. Completion receipt
  pending. (Prior antivacuity corpus leg already on disk:
  `designs/antivacuity-leg-41209-malformed.ttl` witness,
  `corpus/antivacuity-leg-41209-malformed.ttl`.)

### WO-15 — M6 close condition — SPC gate re-run — serves M6, M5

- Class: MECHANISM-EXISTS (`scripts/spc_court.sh`).
- Deliverable: gate re-run per `designs/cpk-correction.md` close condition;
  receipt must postdate the correction record. Status stays
  `REFUTED (measured -0.05, run 3)` until then — no quiet flip.
- Files owned: none (gate run + receipt filing).
- Gate: `bash scripts/spc_court.sh`
- Depends-on: WO-07.
- Status: NOT-STARTED (2026-10-22) — REFUTED standing unchanged; no
  post-correction gate-run receipt on file.

### WO-16 — Phase 2 (P2.4) — hot-path p99 print — serves M5

- Class: MECHANISM-EXISTS (`tests/p28_hot_path_bench.rs`).
- Deliverable: p99 printed against M5 on the 100k corpus (reported, not
  gated — census note on M5).
- Files owned: `tests/p28_hot_path_bench.rs` (ext).
- Gate: `cargo test --test p28_hot_path_bench --release`
- Depends-on: WO-02.
- Status: NOT-STARTED (2026-10-22) — no dispatch or completion receipt on
  file.

### WO-17 — Suite driver — §0 disclosure + exit codes 0-7 — serves M9

- Class: NEW-MECHANISM (driver does not exist; phase order/predicates/exit
  codes are already normative in COURT-SPEC.md §0-§3 — zero design
  freedom).
- Deliverable: machine-disclosure block (debug build = exit 7) +
  phase-ordered driver; a missing phase-4 record = exit 6 — skipping the
  falsifier is a failure, never a pass.
- Files owned: `src/bin/tc_mega_suite.rs` (new),
  `tests/tc_mega_suite_driver.rs` (new).
- Gate: `cargo test --test tc_mega_suite_driver --release && cargo run
  --release --bin tc_mega_suite`
- Depends-on: WO-02, WO-05, WO-09, WO-14.
- Status: NOT-STARTED (2026-10-22) — no dispatch or completion receipt on
  file.

## Order of execution

1. WO-11, WO-12, WO-13 (reconciliation, no intra-lane deps).
2. WO-01 -> WO-02 (admission spine).
3. WO-03, WO-05, WO-09, WO-16 fan out from WO-02.
4. WO-04 (needs WO-05), WO-06 -> WO-07 -> WO-14 -> WO-15.
5. WO-08 (needs WO-09); WO-10 stays a loud typed skip until Lane 1 links
   the artifact.
6. WO-17 last — the driver is only honest once every phase court exists.

## CTQ coverage check

M1 WO-02, WO-03 · M2 WO-01, WO-11, WO-13 · M3 WO-04, WO-05 · M4 WO-09 ·
M5 WO-15, WO-16 · M6 WO-08, WO-15 · M7 WO-10 · M8 WO-02, WO-06, WO-07,
WO-12, WO-14 · M9 WO-05, WO-17. All nine covered; verify with:

```bash
grep -o 'M[1-9]' IMPLEMENTATION-PLAN.md | sort | uniq -c
```

## See Also

`COURT-SPEC.md` · `PILLARS-AUDIT.md` · `ontology/README.md` · `README.md`
(corpus scenario) · `docs/jira/v26.10.22/designs/cpk-correction.md` ·
`docs/refusals.md`
